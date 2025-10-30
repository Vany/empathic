//! ✍️ Write File Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::{ToolBuilder, SchemaBuilder, default_fs_path};
use crate::config::Config;
use crate::fs::FileOps;
use crate::error::EmpathicResult;

/// ✍️ Write File Tool using modern ToolBuilder pattern
pub struct WriteFileTool;

#[derive(Deserialize)]
pub struct WriteFileArgs {
    path: Option<String>,
    content: String,
    start: Option<usize>,
    end: Option<usize>,
    project: Option<String>,
}

#[derive(Serialize)]
pub struct WriteFileOutput {
    success: bool,
    path: String,
    bytes_written: usize,
    start: Option<usize>,
    end: Option<usize>,
    lsp_synced: bool,
}

#[async_trait]
impl ToolBuilder for WriteFileTool {
    type Args = WriteFileArgs;
    type Output = WriteFileOutput;

    fn name() -> &'static str {
        "write_file"
    }
    
    fn description() -> &'static str {
        "✍️ Write file content with optional line-based replacement"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .optional_string("path", "Path to the file to write (default: project root \".\" when project is set)")
            .required_string("content", "Content to write to the file")
            .optional_integer("start", "Starting line number (0-indexed) for replacement", Some(0))
            .optional_integer("end", "Ending line number (exclusive) for replacement", Some(0))
            .optional_string("project", "Project name for path resolution")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let path = default_fs_path(args.path, args.project.as_deref());
        let working_dir = config.project_path(args.project.as_deref());
        let file_path = working_dir.join(&path);
        
        // Write the file
        if let Some(start_line) = args.start {
            FileOps::write_file_range(&file_path, &args.content, start_line, args.end).await?;
        } else {
            FileOps::write_file(&file_path, &args.content).await?;
        }
        
        // 🚀 No LSP sync - let rust-analyzer detect changes via file watchers
        
        Ok(WriteFileOutput {
            success: true,
            path: file_path.to_string_lossy().to_string(),
            bytes_written: args.content.len(),
            start: args.start,
            end: args.end,
            lsp_synced: false, // 🚀 LSP sync removed for performance
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(WriteFileTool);
