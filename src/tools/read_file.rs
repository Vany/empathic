use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;
use crate::fs::FileOps;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }
    
    fn description(&self) -> &'static str {
        "📖 Read file content with optional line-based chunking"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "line_offset": {
                    "type": "integer",
                    "description": "Starting line number (0-indexed)",
                    "minimum": 0
                },
                "line_length": {
                    "type": "integer",
                    "description": "Number of lines to read",
                    "minimum": 1
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
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        let file_path = working_dir.join(path_str);
        
        let line_offset = args.get("line_offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        
        let line_length = args.get("line_length")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        
        let content = if line_offset > 0 || line_length.is_some() {
            FileOps::read_file_chunk(&file_path, line_offset, line_length).await?
        } else {
            FileOps::read_file(&file_path).await?
        };
        
        // Format as MCP-compliant text content
        Ok(crate::tools::format_text_response(&content))
    }
}
