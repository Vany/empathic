//! 🔗 Symlink Tool - Modern ToolBuilder implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::error::{EmpathicResult, EmpathicError};

/// 🔗 Create Symbolic Link Tool using modern ToolBuilder pattern
pub struct SymlinkTool;

#[derive(Deserialize)]
pub struct SymlinkArgs {
    target: String,
    link: String,
    project: Option<String>,
}

#[derive(Serialize)]
pub struct SymlinkOutput {
    success: bool,
    target: String,
    link: String,
    working_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_type: Option<String>,
}

#[async_trait]
impl ToolBuilder for SymlinkTool {
    type Args = SymlinkArgs;
    type Output = SymlinkOutput;

    fn name() -> &'static str {
        "symlink"
    }
    
    fn description() -> &'static str {
        "🔗 Create symbolic links"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_string("target", "Target path (what the symlink points to)")
            .required_string("link", "Symlink path (where to create the link)")
            .optional_string("project", "Project name for path resolution")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let working_dir = config.project_path(args.project.as_deref());
        let target_path = working_dir.join(&args.target);
        let link_path = working_dir.join(&args.link);
        
        // Create parent directory for the symlink if needed
        if let Some(parent) = link_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| EmpathicError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    reason: e.to_string(),
                })?;
        }
        
        // Create the symbolic link with platform-specific logic
        let link_type = create_symlink(&target_path, &link_path).await?;
        
        Ok(SymlinkOutput {
            success: true,
            target: target_path.to_string_lossy().to_string(),
            link: link_path.to_string_lossy().to_string(),
            working_dir: working_dir.to_string_lossy().to_string(),
            link_type,
        })
    }
}

/// Cross-platform symbolic link creation
async fn create_symlink(target: &std::path::Path, link: &std::path::Path) -> EmpathicResult<Option<String>> {
    #[cfg(unix)]
    {
        tokio::fs::symlink(target, link).await
            .map_err(|e| EmpathicError::FileOperationFailed {
                operation: "create symlink".to_string(),
                path: link.to_path_buf(),
                reason: e.to_string(),
            })?;
        Ok(Some("unix".to_string()))
    }
    
    #[cfg(windows)]
    {
        // Windows requires different calls for files vs directories
        let metadata = tokio::fs::metadata(target).await;
        if metadata.map(|m| m.is_dir()).unwrap_or(false) {
            tokio::fs::symlink_dir(target, link).await
                .map_err(|e| EmpathicError::FileOperationFailed {
                    operation: "create symlink dir".to_string(),
                    path: link.to_path_buf(),
                    reason: e.to_string(),
                })?;
            Ok(Some("windows_dir".to_string()))
        } else {
            tokio::fs::symlink_file(target, link).await
                .map_err(|e| EmpathicError::FileOperationFailed {
                    operation: "create symlink file".to_string(),
                    path: link.to_path_buf(),
                    reason: e.to_string(),
                })?;
            Ok(Some("windows_file".to_string()))
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        Err(EmpathicError::NotSupported {
            operation: "symbolic links".to_string(),
            platform: std::env::consts::OS.to_string(),
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(SymlinkTool);
