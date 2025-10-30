//! 📁 List Files Tool - Modern ToolBuilder implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::fs::FileOps;
use crate::error::EmpathicResult;

/// 📁 List Files Tool using modern ToolBuilder pattern
pub struct ListFilesTool;

#[derive(Deserialize)]
pub struct ListFilesArgs {
    #[serde(default = "default_path")]
    path: String,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    show_metadata: bool,
    pattern: Option<String>,
    project: Option<String>,
}

#[derive(Serialize)]
pub struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<String>,
}

#[derive(Serialize)]
pub struct ListFilesOutput {
    files: Vec<FileEntry>,
    path: String,
    recursive: bool,
    show_metadata: bool,
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
}

fn default_path() -> String {
    ".".to_string()
}

#[async_trait]
impl ToolBuilder for ListFilesTool {
    type Args = ListFilesArgs;
    type Output = ListFilesOutput;

    fn name() -> &'static str {
        "list_files"
    }
    
    fn description() -> &'static str {
        "📁 List directory contents with optional metadata and recursion"
    }
    
    fn schema() -> Value {
        SchemaBuilder::new()
            .optional_string("path", "Directory path to list (default: '.')")
            .optional_bool("recursive", "List files recursively, will use .gitignore rules", Some(false))
            .optional_bool("show_metadata", "Show file metadata (size, permissions, dates)", Some(false))
            .optional_string("pattern", "Glob pattern to search files by name (implies recursive=true, will use .gitignore)")
            .optional_string("project", "Project name for path resolution")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        // If pattern is specified, force recursive to true
        let recursive = if args.pattern.is_some() { true } else { args.recursive };
        
        let working_dir = config.project_path(args.project.as_deref());
        let list_path = working_dir.join(&args.path);
        
        let files = FileOps::list_files(&list_path, recursive, args.show_metadata, args.pattern.as_deref()).await?;
        
        let file_entries: Vec<FileEntry> = files.into_iter()
            .map(|file| {
                let mut entry = FileEntry {
                    name: file.name,
                    path: file.path.to_string_lossy().to_string(),
                    is_dir: file.is_dir,
                    size: None,
                    modified: None,
                    permissions: None,
                };
                
                if args.show_metadata {
                    entry.size = file.size;
                    entry.modified = file.modified.and_then(|m| {
                        m.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs())
                    });
                    entry.permissions = file.permissions;
                }
                
                entry
            })
            .collect();
        
        Ok(ListFilesOutput {
            path: list_path.to_string_lossy().to_string(),
            recursive,
            show_metadata: args.show_metadata,
            count: file_entries.len(),
            pattern: args.pattern,
            files: file_entries,
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(ListFilesTool);
