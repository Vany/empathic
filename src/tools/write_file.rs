use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;
use crate::fs::FileOps;

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }
    
    fn description(&self) -> &'static str {
        "✍️ Write file content with optional line-based replacement"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                },
                "start": {
                    "type": "integer",
                    "description": "Starting line number (0-indexed) for replacement",
                    "minimum": 0
                },
                "end": {
                    "type": "integer",
                    "description": "Ending line number (exclusive) for replacement",
                    "minimum": 0
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            },
            "required": ["path", "content"]
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("path is required"))?;
        
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("content is required"))?;
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        let file_path = working_dir.join(path_str);
        
        let start = args.get("start")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        
        let end = args.get("end")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        
        if let Some(start_line) = start {
            FileOps::write_file_range(&file_path, content, start_line, end).await?;
        } else {
            FileOps::write_file(&file_path, content).await?;
        }
        
        // Return success message with file info
        let response = json!({
            "success": true,
            "path": file_path.to_string_lossy(),
            "bytes_written": content.len(),
            "start": start,
            "end": end
        });
        
        Ok(crate::tools::format_json_response(&response)?)
    }
}
