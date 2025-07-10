use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;

pub struct SymlinkTool;

#[async_trait]
impl Tool for SymlinkTool {
    fn name(&self) -> &'static str {
        "symlink"
    }
    
    fn description(&self) -> &'static str {
        "🔗 Create symbolic links"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target path (what the symlink points to)"
                },
                "link": {
                    "type": "string", 
                    "description": "Symlink path (where to create the link)"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            },
            "required": ["target", "link"]
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let target_str = args.get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required 'target' field"))?;
            
        let link_str = args.get("link")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required 'link' field"))?;
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        
        let target_path = working_dir.join(target_str);
        let link_path = working_dir.join(link_str);
        
        // Create parent directory for the symlink if needed
        if let Some(parent) = link_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Create the symbolic link
        #[cfg(unix)]
        {
            tokio::fs::symlink(&target_path, &link_path).await?;
        }
        
        #[cfg(windows)]
        {
            // Windows requires different calls for files vs directories
            let metadata = tokio::fs::metadata(&target_path).await;
            if metadata.map(|m| m.is_dir()).unwrap_or(false) {
                tokio::fs::symlink_dir(&target_path, &link_path).await?;
            } else {
                tokio::fs::symlink_file(&target_path, &link_path).await?;
            }
        }
        
        Ok(crate::tools::format_json_response(&json!({
            "success": true,
            "target": target_path.to_string_lossy(),
            "link": link_path.to_string_lossy(),
            "working_dir": working_dir.to_string_lossy()
        }))?)
    }
}
