use std::path::PathBuf;
use std::sync::Arc;
use std::env;
use std::time::Duration;

use crate::error::{EmpathicError, EmpathicResult};
use crate::lsp::LspManager;

#[derive(Debug, Clone)]
pub struct Config {
    pub root_dir: PathBuf,
    pub add_path: Vec<PathBuf>,
    pub log_level: String,
    /// ⏱️ Hard timeout for all MCP tool requests (default: 55s, safe for Claude Desktop's 60s limit)
    pub request_timeout: Duration,
    /// 🧠 LSP manager for file synchronization with language servers
    pub lsp_manager: Option<Arc<LspManager>>,
}

impl Config {
    /// Create a new Config for testing purposes
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            add_path: Vec::new(),
            log_level: "warn".to_string(),
            request_timeout: Duration::from_secs(55),
            lsp_manager: None,
        }
    }

    /// Create a new Config with LSP manager for production use
    pub fn new_with_lsp(root_dir: PathBuf, lsp_manager: Arc<LspManager>) -> Self {
        Self {
            root_dir: root_dir.clone(),
            add_path: Vec::new(),
            log_level: "warn".to_string(),
            request_timeout: Duration::from_secs(55),
            lsp_manager: Some(lsp_manager),
        }
    }

    /// 🔧 Create Config from environment variables with proper validation
    pub fn from_env() -> EmpathicResult<Self> {
        // Get and validate ROOT_DIR
        let root_dir_str = env::var("ROOT_DIR")
            .map_err(|_| EmpathicError::MissingEnvVar { 
                name: "ROOT_DIR".to_string() 
            })?;
        
        let root_dir = PathBuf::from(&root_dir_str);
        
        // Validate that root directory exists
        if !root_dir.exists() {
            return Err(EmpathicError::RootDirectoryNotFound { 
                path: root_dir 
            });
        }
        
        if !root_dir.is_dir() {
            return Err(EmpathicError::InvalidConfigValue {
                field: "ROOT_DIR".to_string(),
                value: format!("{} (not a directory)", root_dir_str),
            });
        }
        
        // Parse ADD_PATH with validation
        let add_path = env::var("ADD_PATH")
            .unwrap_or_default()
            .split(':')
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        
        // Validate and normalize log level
        let log_level = env::var("LOGLEVEL")
            .unwrap_or_else(|_| "warn".to_string())
            .to_lowercase();
        
        // Validate log level
        match log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => return Err(EmpathicError::InvalidConfigValue {
                field: "LOGLEVEL".to_string(),
                value: log_level,
            }),
        }
        
        // ⏱️ Parse MCP_REQUEST_TIMEOUT (default: 55s to stay under Claude Desktop's 60s limit)
        let request_timeout = env::var("MCP_REQUEST_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(55));
        
        // Validate timeout is reasonable (1s - 300s)
        if request_timeout < Duration::from_secs(1) || request_timeout > Duration::from_secs(300) {
            return Err(EmpathicError::InvalidConfigValue {
                field: "MCP_REQUEST_TIMEOUT".to_string(),
                value: format!("{}s (must be 1-300)", request_timeout.as_secs()),
            });
        }
        
        let config = Config {
            root_dir,
            add_path,
            log_level,
            request_timeout,
            lsp_manager: None, // Will be set later by McpServer
        };
        
        // Perform final validation
        config.validate()?;
        
        Ok(config)
    }

    /// 🔍 Validate configuration integrity
    pub fn validate(&self) -> EmpathicResult<()> {
        // Validate root directory accessibility
        if !self.root_dir.exists() {
            return Err(EmpathicError::RootDirectoryNotFound { 
                path: self.root_dir.clone() 
            });
        }
        
        // Check if root directory is readable
        match std::fs::read_dir(&self.root_dir) {
            Ok(_) => {},
            Err(_e) => return Err(EmpathicError::FileAccessDenied { 
                path: self.root_dir.clone() 
            }),
        }
        
        // Validate ADD_PATH entries exist (warn but don't fail)
        for path in &self.add_path {
            if !path.exists() {
                log::warn!("📂 ADD_PATH entry does not exist: {}", path.display());
            }
        }
        
        Ok(())
    }

    /// Set the LSP manager (called by McpServer after creation)
    pub fn set_lsp_manager(&mut self, lsp_manager: Arc<LspManager>) {
        self.lsp_manager = Some(lsp_manager);
    }

    /// 📁 Get project path (legacy - for backward compatibility)
    pub fn project_path(&self, project: Option<&str>) -> PathBuf {
        match project {
            Some(project_name) => self.root_dir.join(project_name),
            None => self.root_dir.clone(),
        }
    }

    /// 📁 Get project path with validation (new typed error version)
    pub fn safe_project_path(&self, project: Option<&str>) -> EmpathicResult<PathBuf> {
        let path = match project {
            Some(project_name) => {
                // Validate project name doesn't contain path traversal
                if project_name.contains("..") || project_name.starts_with('/') {
                    return Err(EmpathicError::InvalidPath { 
                        path: PathBuf::from(project_name) 
                    });
                }
                self.root_dir.join(project_name)
            },
            None => self.root_dir.clone(),
        };
        
        // Ensure the path is within root_dir (security check)
        if let Ok(canonical_root) = self.root_dir.canonicalize()
            && let Ok(canonical_path) = path.canonicalize()
            && !canonical_path.starts_with(canonical_root)
        {
            return Err(EmpathicError::InvalidPath { 
                path: path.clone() 
            });
        }
        
        Ok(path)
    }

    /// Get LSP manager if available
    pub fn lsp_manager(&self) -> Option<&Arc<LspManager>> {
        self.lsp_manager.as_ref()
    }

    /// 📊 Get configuration summary for logging
    pub fn summary(&self) -> String {
        format!(
            "📁 Root: {}, 🔧 Paths: {}, 📝 Log: {}, ⏱️ Timeout: {}s, 🧠 LSP: {}",
            self.root_dir.display(),
            self.add_path.len(),
            self.log_level,
            self.request_timeout.as_secs(),
            if self.lsp_manager.is_some() { "enabled" } else { "disabled" }
        )
    }
}

// === 🎯 Compatibility Layer ===
// For gradual migration - provides anyhow::Result version

impl Config {
    /// Legacy from_env for backward compatibility
    /// 
    /// **Deprecated**: Use `from_env()` which returns `EmpathicResult<Config>` instead
    #[deprecated(note = "Use from_env() which returns EmpathicResult instead")]
    pub fn from_env_legacy() -> anyhow::Result<Self> {
        crate::error::to_anyhow(Self::from_env())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_creation() {
        let config = Config::new("/tmp".into());
        assert_eq!(config.root_dir, PathBuf::from("/tmp"));
        assert_eq!(config.log_level, "warn");
        assert!(config.lsp_manager.is_none());
    }

    #[test]
    fn test_project_path_validation() {
        let config = Config::new("/tmp".into());
        
        // Valid project name (old API)
        let result = config.project_path(Some("myproject"));
        assert_eq!(result, PathBuf::from("/tmp/myproject"));
        
        // Valid project name (new safe API)
        let result = config.safe_project_path(Some("myproject"));
        assert!(result.is_ok());
        
        // Invalid project names (safe API catches these)
        assert!(config.safe_project_path(Some("../etc")).is_err());
        assert!(config.safe_project_path(Some("/etc/passwd")).is_err());
        assert!(config.safe_project_path(Some("project/../../../etc")).is_err());
    }

    #[test]
    fn test_log_level_validation() {
        // Test valid log levels by setting env var temporarily
        let original = env::var("LOGLEVEL").ok();
        
        unsafe {
            env::set_var("ROOT_DIR", "/tmp");
            env::set_var("LOGLEVEL", "debug");
        }
        
        // This should work if /tmp exists
        if std::path::Path::new("/tmp").exists() {
            let result = Config::from_env();
            assert!(result.is_ok());
            if let Ok(config) = result {
                assert_eq!(config.log_level, "debug");
            }
        }
        
        // Test invalid log level
        unsafe {
            env::set_var("LOGLEVEL", "invalid_level");
            env::set_var("ROOT_DIR", "/tmp");
        }
        let result = Config::from_env();
        assert!(result.is_err());
        
        // Restore original env var
        unsafe {
            match original {
                Some(val) => env::set_var("LOGLEVEL", val),
                None => env::remove_var("LOGLEVEL"),
            }
            env::remove_var("ROOT_DIR");
        }
    }

    #[test]
    fn test_config_summary() {
        let config = Config::new("/tmp".into());
        let summary = config.summary();
        assert!(summary.contains("/tmp"));
        assert!(summary.contains("warn"));
        assert!(summary.contains("disabled"));
    }

    #[test] 
    fn test_missing_root_dir() {
        unsafe {
            env::remove_var("ROOT_DIR");
        }
        let result = Config::from_env();
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(matches!(e, EmpathicError::MissingEnvVar { .. }));
        }
    }
}
