//! 🛠️ Test setup utilities

use anyhow::Result;
use empathic::Config;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;

/// 🎯 Test environment for MCP tool testing
#[derive(Debug)]
pub struct TestEnv {
    pub temp_dir: TempDir,
    pub config: Config,
    pub root_path: PathBuf,
}

impl TestEnv {
    /// Create a new test environment with temporary directory
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path().to_path_buf();
        
        let config = Config::new(root_path.clone());

        Ok(Self {
            temp_dir,
            config,
            root_path,
        })
    }

    /// Get the root directory path
    pub fn root_dir(&self) -> &PathBuf {
        &self.root_path
    }

    /// Create a new Rust project with Cargo.toml and basic structure
    pub async fn create_rust_project(&self, name: &str) -> Result<PathBuf> {
        let project_path = self.root_path.join(name);
        
        // Create project directory structure
        tokio::fs::create_dir_all(&project_path).await?;
        tokio::fs::create_dir_all(project_path.join("src")).await?;
        
        // Create Cargo.toml
        let cargo_toml_content = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, name);
        
        tokio::fs::write(project_path.join("Cargo.toml"), cargo_toml_content).await?;
        
        // Create basic src/lib.rs
        let lib_content = format!(r#"
//! {} library

/// Example function
pub fn hello() {{
    println!("Hello from {}!");
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_hello() {{
        hello();
    }}
}}
"#, name, name);
        
        tokio::fs::write(project_path.join("src/lib.rs"), lib_content).await?;
        
        Ok(project_path)
    }

    /// Write content to a file, creating parent directories if needed
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Create a basic project suitable for testing
    pub async fn create_project(&self, name: &str) -> Result<PathBuf> {
        let project_path = self.root_path.join(name);
        tokio::fs::create_dir_all(&project_path).await?;
        Ok(project_path)
    }

    /// Get relative path from root
    pub fn relative_path(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root_path)
            .unwrap_or(path)
            .to_path_buf()
    }

    /// Create multiple files at once
    pub async fn create_files(&self, files: &[(&str, &str)]) -> Result<Vec<PathBuf>> {
        let mut created_files = Vec::new();
        
        for (file_path, content) in files {
            let full_path = self.root_path.join(file_path);
            self.write_file(&full_path, content).await?;
            created_files.push(full_path);
        }
        
        Ok(created_files)
    }

    /// Create a single file with content (legacy method for compatibility)
    pub async fn create_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let full_path = self.root_path.join(relative_path);
        self.write_file(&full_path, content).await?;
        Ok(full_path)
    }
}

/// 🧹 Quick test setup helper
pub async fn quick_test_setup() -> Result<(Config, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path().to_path_buf();
    let config = Config::new(root_path.clone());
    Ok((config, root_path))
}

/// 🗑️ Cleanup test directory
pub async fn cleanup_test_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_creates_correctly() {
        let env = TestEnv::new().unwrap();
        assert!(env.root_path.exists());
        assert_eq!(env.config.root_dir, env.root_path);
    }

    #[tokio::test] 
    async fn test_file_creation() {
        let env = TestEnv::new().unwrap();
        let path = env.create_file("test.txt", "hello").await.unwrap();
        assert!(path.exists());
        
        let content = fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "hello");
    }
}
