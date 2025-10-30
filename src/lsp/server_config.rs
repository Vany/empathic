//! 🔧 LSP Server Configuration Registry
//!
//! Defines how to spawn and initialize different language server types.
//! Supports rust-analyzer, jdtls, and pylsp with extensible architecture.

use serde_json::{json, Value};
use std::collections::HashMap;

/// 🔧 Language Server Configuration
///
/// Defines everything needed to spawn and initialize an LSP server for a specific language.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Language identifier (rust, java, python)
    pub language: String,
    
    /// Server binary name or path
    pub server_command: String,
    
    /// Command-line arguments
    pub args: Vec<String>,
    
    /// File patterns for project detection (e.g., "Cargo.toml", "pom.xml")
    pub project_markers: Vec<String>,
    
    /// File extensions this server handles (e.g., ".rs", ".java")
    pub file_extensions: Vec<String>,
    
    /// LSP initialization options (language-specific settings)
    pub init_options: Option<Value>,
}

impl ServerConfig {
    /// 🦀 rust-analyzer configuration
    pub fn rust_analyzer() -> Self {
        Self {
            language: "rust".to_string(),
            server_command: "rust-analyzer".to_string(),
            args: vec![],
            project_markers: vec!["Cargo.toml".to_string()],
            file_extensions: vec![".rs".to_string()],
            init_options: None,
        }
    }

    /// ☕ jdtls (Eclipse JDT Language Server) configuration
    pub fn jdtls() -> Self {
        Self {
            language: "java".to_string(),
            server_command: "jdtls".to_string(),
            args: vec![],
            project_markers: vec![
                "pom.xml".to_string(),
                "build.gradle".to_string(),
                "build.gradle.kts".to_string(),
            ],
            file_extensions: vec![".java".to_string()],
            // jdtls requires workspace and data directory in init options
            init_options: Some(json!({
                "settings": {
                    "java": {
                        "home": null, // Use JAVA_HOME environment variable
                        "format": {
                            "enabled": true
                        }
                    }
                }
            })),
        }
    }

    /// 🐍 pylsp (Python Language Server) configuration
    pub fn pylsp() -> Self {
        Self {
            language: "python".to_string(),
            server_command: "pylsp".to_string(),
            args: vec![],
            project_markers: vec![
                "pyproject.toml".to_string(),
                "setup.py".to_string(),
                "requirements.txt".to_string(),
            ],
            file_extensions: vec![".py".to_string()],
            init_options: Some(json!({
                "pylsp": {
                    "plugins": {
                        "pycodestyle": { "enabled": true },
                        "pyflakes": { "enabled": true },
                        "pylint": { "enabled": false }
                    }
                }
            })),
        }
    }

    /// 📚 Create registry with all built-in server configurations
    pub fn create_registry() -> HashMap<String, ServerConfig> {
        let mut registry = HashMap::new();
        
        let rust_config = Self::rust_analyzer();
        registry.insert(rust_config.language.clone(), rust_config);
        
        let java_config = Self::jdtls();
        registry.insert(java_config.language.clone(), java_config);
        
        let python_config = Self::pylsp();
        registry.insert(python_config.language.clone(), python_config);
        
        registry
    }

    /// 🔍 Detect language from file extension
    pub fn detect_language_from_extension(file_extension: &str, registry: &HashMap<String, ServerConfig>) -> Option<String> {
        for (lang, config) in registry.iter() {
            if config.file_extensions.iter().any(|ext| ext == file_extension) {
                return Some(lang.clone());
            }
        }
        None
    }

    /// 🔍 Detect language from project marker file
    pub fn detect_language_from_marker(marker_file: &str, registry: &HashMap<String, ServerConfig>) -> Option<String> {
        for (lang, config) in registry.iter() {
            if config.project_markers.iter().any(|marker| marker == marker_file) {
                return Some(lang.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_analyzer_config() {
        let config = ServerConfig::rust_analyzer();
        assert_eq!(config.language, "rust");
        assert_eq!(config.server_command, "rust-analyzer");
        assert!(config.project_markers.contains(&"Cargo.toml".to_string()));
        assert!(config.file_extensions.contains(&".rs".to_string()));
    }

    #[test]
    fn test_jdtls_config() {
        let config = ServerConfig::jdtls();
        assert_eq!(config.language, "java");
        assert_eq!(config.server_command, "jdtls");
        assert!(config.project_markers.contains(&"pom.xml".to_string()));
        assert!(config.project_markers.contains(&"build.gradle".to_string()));
        assert!(config.file_extensions.contains(&".java".to_string()));
        assert!(config.init_options.is_some());
    }

    #[test]
    fn test_pylsp_config() {
        let config = ServerConfig::pylsp();
        assert_eq!(config.language, "python");
        assert_eq!(config.server_command, "pylsp");
        assert!(config.project_markers.contains(&"pyproject.toml".to_string()));
        assert!(config.file_extensions.contains(&".py".to_string()));
        assert!(config.init_options.is_some());
    }

    #[test]
    fn test_registry_creation() {
        let registry = ServerConfig::create_registry();
        assert_eq!(registry.len(), 3);
        assert!(registry.contains_key("rust"));
        assert!(registry.contains_key("java"));
        assert!(registry.contains_key("python"));
    }

    #[test]
    fn test_detect_language_from_extension() {
        let registry = ServerConfig::create_registry();
        
        assert_eq!(ServerConfig::detect_language_from_extension(".rs", &registry), Some("rust".to_string()));
        assert_eq!(ServerConfig::detect_language_from_extension(".java", &registry), Some("java".to_string()));
        assert_eq!(ServerConfig::detect_language_from_extension(".py", &registry), Some("python".to_string()));
        assert_eq!(ServerConfig::detect_language_from_extension(".unknown", &registry), None);
    }

    #[test]
    fn test_detect_language_from_marker() {
        let registry = ServerConfig::create_registry();
        
        assert_eq!(ServerConfig::detect_language_from_marker("Cargo.toml", &registry), Some("rust".to_string()));
        assert_eq!(ServerConfig::detect_language_from_marker("pom.xml", &registry), Some("java".to_string()));
        assert_eq!(ServerConfig::detect_language_from_marker("build.gradle", &registry), Some("java".to_string()));
        assert_eq!(ServerConfig::detect_language_from_marker("pyproject.toml", &registry), Some("python".to_string()));
        assert_eq!(ServerConfig::detect_language_from_marker("unknown.txt", &registry), None);
    }
}
