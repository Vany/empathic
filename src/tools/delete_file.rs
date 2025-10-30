//! 🗑️ Delete File Tool - Modern ToolBuilder implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::fs::FileOps;
use crate::error::{EmpathicResult, EmpathicError};

/// 🗑️ Delete File Tool using modern ToolBuilder pattern
pub struct DeleteFileTool;

#[derive(Deserialize)]
pub struct DeleteFileArgs {
    path: Option<String>,
    #[serde(default)]
    recursive: bool,
    project: Option<String>,
}

#[derive(Serialize)]
pub struct DeleteFileOutput {
    success: bool,
    path: String,
    was_directory: bool,
    recursive: bool,
    lsp_closed: bool,
}

#[async_trait]
impl ToolBuilder for DeleteFileTool {
    type Args = DeleteFileArgs;
    type Output = DeleteFileOutput;

    fn name() -> &'static str {
        "delete_file"
    }
    
    fn description() -> &'static str {
        "🗑️ Delete file or directory with optional recursive deletion"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_string("path", "Path to the file or directory to delete")
            .optional_bool("recursive", "Delete directories recursively", Some(false))
            .optional_string("project", "Project name for path resolution")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let working_dir = config.project_path(args.project.as_deref());
        let file_path = working_dir.join(
            args.path
                .as_ref()
                .ok_or_else(|| EmpathicError::MissingRequiredParameter { parameter: "path".to_string() })?
        );
        
        // Check if path exists and get its type
        let metadata = tokio::fs::metadata(&file_path).await
            .map_err(|_| EmpathicError::FileNotFound { path: file_path.clone() })?;
        let is_dir = metadata.is_dir();
        
        // 🚀 No LSP sync needed - rust-analyzer detects file deletions automatically
        let lsp_closed = false;
        
        FileOps::delete_file(&file_path, args.recursive).await
            .map_err(|e| EmpathicError::FileOperationFailed {
                operation: "delete".to_string(),
                path: file_path.clone(),
                reason: e.to_string(),
            })?;
        
        Ok(DeleteFileOutput {
            success: true,
            path: file_path.to_string_lossy().to_string(),
            was_directory: is_dir,
            recursive: args.recursive,
            lsp_closed,
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(DeleteFileTool);
