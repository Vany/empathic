use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;
use crate::fs::FileOps;

pub struct DeleteFileTool;

#[async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &'static str {
        "delete_file"
    }
    
    fn description(&self) -> &'static str {
        "🗑️ Delete file or directory with optional recursive deletion"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory to delete"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Delete directories recursively",
                    "default": false
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("path is required"))?;
        
        let recursive = args.get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        let file_path = working_dir.join(path_str);
        
        // Check if path exists and get its type
        let metadata = tokio::fs::metadata(&file_path).await?;
        let is_dir = metadata.is_dir();
        
        FileOps::delete_file(&file_path, recursive).await?;
        
        Ok(crate::tools::format_json_response(&json!({
            "success": true,
            "path": file_path.to_string_lossy(),
            "was_directory": is_dir,
            "recursive": recursive
        }))?)
    }
}
