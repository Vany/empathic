//! 📁 Project Detector - Multi-language project discovery logic
//!
//! Scans the filesystem for project markers (Cargo.toml, pom.xml, pyproject.toml, etc.)
//! to identify projects of different languages within the configured ROOT_DIR boundary.

use crate::lsp::server_config::ServerConfig;
use crate::lsp::types::{LspError, LspResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 📦 Generic project information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Language of the project (rust, java, python)
    pub language: String,
    /// Project root directory (containing project marker file)
    pub root_path: PathBuf,
    /// Project name (if available)
    pub name: Option<String>,
    /// Marker file that identified this project (e.g., "Cargo.toml", "pom.xml")
    pub marker_file: String,
}

impl Project {
    /// Create a new Project
    pub fn new(language: String, root_path: PathBuf, marker_file: String) -> Self {
        Self {
            language,
            root_path,
            name: None,
            marker_file,
        }
    }
}

/// 🦀 Rust project information (backward compatibility)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustProject {
    /// Project root directory (containing Cargo.toml)
    pub root_path: PathBuf,
    /// Project name from Cargo.toml
    pub name: Option<String>,
    /// Whether this is a workspace root
    pub is_workspace: bool,
}

impl RustProject {
    /// Create a new RustProject
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            name: None,
            is_workspace: false,
        }
    }

    /// Get the Cargo.toml path for this project
    pub fn cargo_toml_path(&self) -> PathBuf {
        self.root_path.join("Cargo.toml")
    }
}

/// 🔍 Project detection and management
#[derive(Debug)]
pub struct ProjectDetector {
    root_dir: PathBuf,
    /// Language server configurations
    server_configs: HashMap<String, ServerConfig>,
}

impl ProjectDetector {
    /// Create a new ProjectDetector with the given root directory
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            server_configs: ServerConfig::create_registry(),
        }
    }

    /// 🌍 Find all projects (all languages) within ROOT_DIR
    pub fn find_all_projects(&self) -> LspResult<Vec<Project>> {
        let mut projects = Vec::new();

        let walker = WalkDir::new(&self.root_dir)
            .follow_links(false)
            .max_depth(10) // Reasonable depth limit
            .into_iter()
            .filter_entry(|e| {
                // Skip common hidden directories (like .git, .cache), but allow temp directories
                if let Some(name) = e.file_name().to_str() {
                    let should_skip = name.starts_with('.') && 
                        !name.starts_with(".tmp") &&  // Allow tempfile directories
                        (name == ".git" || name == ".cache" || name == ".vscode" || name == ".idea" || name == ".DS_Store");
                    !should_skip
                } else {
                    true
                }
            });

        for entry in walker {
            let entry = entry.map_err(|e| LspError::ProjectDetectionError {
                message: format!("Failed to traverse directory: {e}"),
            })?;

            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                
                // Check if this file is a project marker for any language
                if let Some(language) = ServerConfig::detect_language_from_marker(&file_name, &self.server_configs) {
                    let project_root = entry
                        .path()
                        .parent()
                        .ok_or_else(|| LspError::ProjectDetectionError {
                            message: format!("{} has no parent directory", file_name),
                        })?
                        .to_path_buf();

                    // Ensure the project is within our root_dir
                    if project_root.starts_with(&self.root_dir) {
                        let mut project = Project::new(language, project_root, file_name.clone());

                        // Try to parse project name
                        if let Ok(name) = self.parse_project_name(&project) {
                            project.name = name;
                        }

                        projects.push(project);
                    }
                }
            }
        }

        // Sort projects by path for consistent ordering
        projects.sort_by(|a, b| a.root_path.cmp(&b.root_path));

        Ok(projects)
    }

    /// 🎯 Find all Rust projects within ROOT_DIR (backward compatibility)
    pub fn find_rust_projects(&self) -> LspResult<Vec<RustProject>> {
        let all_projects = self.find_all_projects()?;
        
        let mut rust_projects = Vec::new();
        for project in all_projects {
            if project.language == "rust" {
                let mut rust_project = RustProject::new(project.root_path.clone());
                rust_project.name = project.name;
                
                // Try to parse workspace info
                if let Ok(cargo_info) = self.parse_cargo_toml(&rust_project.cargo_toml_path()) {
                    rust_project.is_workspace = cargo_info.is_workspace;
                }
                
                rust_projects.push(rust_project);
            }
        }

        Ok(rust_projects)
    }

    /// 🎯 Find the project containing a specific file
    pub fn find_project_for_file(&self, file_path: &Path) -> LspResult<Option<Project>> {
        let projects = self.find_all_projects()?;

        // Find the project that contains this file (most specific match)
        let mut best_match = None;
        let mut best_depth = usize::MAX;

        for project in projects {
            if file_path.starts_with(&project.root_path) {
                let depth = file_path
                    .strip_prefix(&project.root_path)
                    .unwrap()
                    .components()
                    .count();

                if depth < best_depth {
                    best_match = Some(project);
                    best_depth = depth;
                }
            }
        }

        Ok(best_match)
    }

    /// 🔍 Detect language from file extension
    pub fn detect_language_from_file(&self, file_path: &Path) -> Option<String> {
        if let Some(extension) = file_path.extension() {
            let ext_with_dot = format!(".{}", extension.to_string_lossy());
            ServerConfig::detect_language_from_extension(&ext_with_dot, &self.server_configs)
        } else {
            None
        }
    }

    /// Parse project name from project file
    fn parse_project_name(&self, project: &Project) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match project.language.as_str() {
            "rust" => {
                let cargo_path = project.root_path.join(&project.marker_file);
                let cargo_info = self.parse_cargo_toml(&cargo_path)?;
                Ok(cargo_info.name)
            }
            "java" => {
                // For Java, try to parse from pom.xml or build.gradle
                if project.marker_file == "pom.xml" {
                    let pom_path = project.root_path.join(&project.marker_file);
                    self.parse_pom_xml(&pom_path)
                } else {
                    // For Gradle, we could parse build.gradle, but it's complex
                    // Just use directory name for now
                    Ok(project.root_path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string()))
                }
            }
            "python" => {
                // For Python, try to parse from pyproject.toml or setup.py
                if project.marker_file == "pyproject.toml" {
                    let pyproject_path = project.root_path.join(&project.marker_file);
                    self.parse_pyproject_toml(&pyproject_path)
                } else {
                    // For other Python projects, use directory name
                    Ok(project.root_path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string()))
                }
            }
            _ => Ok(None),
        }
    }

    /// Parse basic information from Cargo.toml
    fn parse_cargo_toml(&self, cargo_path: &Path) -> Result<CargoInfo, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(cargo_path)?;

        let mut name = None;
        let mut is_workspace = false;

        for line in content.lines() {
            let line = line.trim();

            // Look for package name
            if line.starts_with("name") && line.contains('=')
                && let Some(value_part) = line.split('=').nth(1)
            {
                let value = value_part.trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    name = Some(value.to_string());
                }
            }

            // Check for workspace
            if line == "[workspace]" || line.starts_with("[workspace.") {
                is_workspace = true;
            }
        }

        Ok(CargoInfo { name, is_workspace })
    }

    /// Parse project name from pom.xml
    fn parse_pom_xml(&self, pom_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(pom_path)?;
        
        // Very simple XML parsing - look for <artifactId>...</artifactId>
        if let Some(start) = content.find("<artifactId>")
            && let Some(end) = content[start..].find("</artifactId>")
        {
            let name_start = start + "<artifactId>".len();
            let name_end = start + end;
            let name = content[name_start..name_end].trim().to_string();
            return Ok(Some(name));
        }
        
        Ok(None)
    }

    /// Parse project name from pyproject.toml
    fn parse_pyproject_toml(&self, pyproject_path: &Path) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(pyproject_path)?;
        
        // Simple TOML parsing for name
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") && line.contains('=')
                && let Some(value_part) = line.split('=').nth(1)
            {
                let value = value_part.trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Ok(Some(value.to_string()));
                }
            }
        }
        
        Ok(None)
    }
}

/// Simple Cargo.toml information
#[derive(Debug)]
struct CargoInfo {
    name: Option<String>,
    is_workspace: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new(
            "rust".to_string(),
            PathBuf::from("/test/project"),
            "Cargo.toml".to_string(),
        );
        assert_eq!(project.language, "rust");
        assert_eq!(project.root_path, PathBuf::from("/test/project"));
        assert_eq!(project.marker_file, "Cargo.toml");
        assert_eq!(project.name, None);
    }

    #[test]
    fn test_rust_project_backward_compatibility() {
        let rust_project = RustProject::new(PathBuf::from("/test/rust_project"));
        assert_eq!(rust_project.root_path, PathBuf::from("/test/rust_project"));
        assert_eq!(rust_project.cargo_toml_path(), PathBuf::from("/test/rust_project/Cargo.toml"));
        assert!(!rust_project.is_workspace);
    }
}
