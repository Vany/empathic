use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;

pub struct MkdirTool;

#[async_trait]
impl Tool for MkdirTool {
    fn name(&self) -> &'static str {
        "mkdir"
    }
    
    fn description(&self) -> &'static str {
        "📁 Create directories with parent creation"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to create"
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
            .ok_or_else(|| anyhow::anyhow!("Missing required 'path' field"))?;
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        let create_path = working_dir.join(path_str);
        
        tokio::fs::create_dir_all(&create_path).await?;
        
        Ok(crate::tools::format_json_response(&json!({
            "success": true,
            "path": create_path.to_string_lossy(),
            "working_dir": working_dir.to_string_lossy()
        }))?)
    }
}
