use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;
use crate::fs::FileOps;

pub struct ListFilesTool;

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &'static str {
        "list_files"
    }
    
    fn description(&self) -> &'static str {
        "📁 List directory contents with optional metadata and recursion"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list",
                    "default": "."
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List files recursively, will use .gitignore rules",
                    "default": false
                },
                "show_metadata": {
                    "type": "boolean",
                    "description": "Show file metadata (size, permissions, dates)",
                    "default": false
                },
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to search files by name (implies recursive=true, will use .gitignore)"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        
        let recursive = args.get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let pattern = args.get("pattern")
            .and_then(|v| v.as_str());
        
        // If pattern is specified, force recursive to true
        let recursive = if pattern.is_some() { true } else { recursive };
        
        let show_metadata = args.get("show_metadata")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        let list_path = working_dir.join(path_str);
        
        let files = FileOps::list_files(&list_path, recursive, show_metadata, pattern).await?;
        
        let file_list: Vec<Value> = files.into_iter()
            .map(|file| {
                let mut file_json = json!({
                    "name": file.name,
                    "path": file.path.to_string_lossy(),
                    "is_dir": file.is_dir
                });
                
                if show_metadata {
                    if let Some(size) = file.size {
                        file_json["size"] = json!(size);
                    }
                    if let Some(modified) = file.modified {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            file_json["modified"] = json!(duration.as_secs());
                        }
                    }
                    if let Some(permissions) = file.permissions {
                        file_json["permissions"] = json!(permissions);
                    }
                }
                
                file_json
            })
            .collect();
        
        let response = json!({
            "files": file_list,
            "path": list_path.to_string_lossy(),
            "recursive": recursive,
            "show_metadata": show_metadata,
            "count": file_list.len()
        });
        
        Ok(crate::tools::format_json_response(&response)?)
    }
}
