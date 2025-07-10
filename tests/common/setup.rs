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
        
        let config = Config {
            root_dir: root_path.clone(),
            add_path: vec![],
            log_level: "warn".to_string(),
        };

        Ok(Self {
            temp_dir,
            config,
            root_path,
        })
    }

    /// Create a project subdirectory within the test environment
    pub async fn create_project(&self, name: &str) -> Result<PathBuf> {
        let project_path = self.root_path.join(name);
        fs::create_dir_all(&project_path).await?;
        Ok(project_path)
    }

    /// Get path relative to root for tool args
    pub fn relative_path(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root_path)
            .unwrap_or(path)
            .to_path_buf()
    }

    /// Create test file with content
    pub async fn create_file(&self, path: &str, content: &str) -> Result<PathBuf> {
        let file_path = self.root_path.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    /// Create multiple test files in subdirectory
    pub async fn create_files(&self, files: &[(&str, &str)]) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        for (path, content) in files {
            paths.push(self.create_file(path, content).await?);
        }
        Ok(paths)
    }
}

/// 🏗️ Quick setup for simple test scenarios
pub async fn quick_test_setup() -> Result<(Config, PathBuf)> {
    let test_dir = std::env::temp_dir().join(format!("ee_test_{}", rand::random::<u32>()));
    fs::create_dir_all(&test_dir).await?;
    
    let config = Config {
        root_dir: test_dir.clone(),
        add_path: vec![],
        log_level: "warn".to_string(),
    };

    Ok((config, test_dir))
}

/// 🧹 Cleanup helper for manual cleanup scenarios
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
