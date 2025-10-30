//! 📖 Read File Tool - Clean ToolBuilder implementation with custom text formatting

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::tools::{Tool, ToolBuilder, SchemaBuilder, format_text_response, default_fs_path};
use crate::config::Config;
use crate::fs::FileOps;
use crate::error::EmpathicResult;

/// 📖 Read File Tool using modern ToolBuilder pattern (with custom text output)
pub struct ReadFileTool;

#[derive(Deserialize)]
pub struct ReadFileArgs {
    path: Option<String>,
    #[serde(default)]
    line_offset: Option<usize>,
    line_length: Option<usize>,
    project: Option<String>,
}

pub type ReadFileOutput = String;

#[async_trait]
impl ToolBuilder for ReadFileTool {
    type Args = ReadFileArgs;
    type Output = String;

    fn name() -> &'static str {
        "read_file"
    }
    
    fn description() -> &'static str {
        "📖 Read file content with optional line-based chunking (auto-lists directories)"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .optional_string("path", "Path to the file to read (default: project root \".\"). If path is a directory, lists contents instead")
            .optional_integer("line_offset", "Starting line number (0-indexed)", Some(0))
            .optional_integer("line_length", "Number of lines to read", Some(1))
            .optional_string("project", "Project name for path resolution")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let path = default_fs_path(args.path, args.project.as_deref());
        let working_dir = config.project_path(args.project.as_deref());
        let file_path = working_dir.join(&path);
        
        // 🎯 AI Enhancement: Auto-detect directories and list contents instead of erroring
        if file_path.is_dir() {
            // List directory contents (non-recursive) when path is a directory
            let files = FileOps::list_files(&file_path, false, false, None).await?;
            
            // Format as readable directory listing
            let mut listing = format!("📁 Directory listing for: {}\n\n", file_path.display());
            
            if files.is_empty() {
                listing.push_str("  (empty directory)\n");
            } else {
                // Get count before consuming files in for loop
                let file_count = files.len();
                for file in files {
                    let file_type = if file.is_dir { "📁" } else { "📄" };
                    listing.push_str(&format!("  {} {}\n", file_type, file.name));
                }
                listing.push_str(&format!("\nTotal: {} items", file_count));
            }
            
            return Ok(listing);
        }
        
        // Original file reading logic
        let content = if let Some(offset) = args.line_offset {
            if offset > 0 || args.line_length.is_some() {
                FileOps::read_file_chunk(&file_path, offset, args.line_length).await?
            } else {
                FileOps::read_file(&file_path).await?
            }
        } else {
            FileOps::read_file(&file_path).await?
        };

        Ok(content)
    }
}

// 🎯 Custom Tool implementation for proper text formatting (not using macro)
#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        <ReadFileTool as ToolBuilder>::name()
    }
    
    fn description(&self) -> &'static str {
        <ReadFileTool as ToolBuilder>::description()
    }
    
    fn schema(&self) -> Value {
        <ReadFileTool as ToolBuilder>::schema()
    }
    
    async fn execute(&self, args: Value, config: &Config) -> EmpathicResult<Value> {
        let parsed_args = serde_json::from_value(args)
            .map_err(|e| crate::error::EmpathicError::McpParameterInvalid { 
                parameter: "args".to_string(), 
                value: format!("Invalid arguments for {}: {}", <ReadFileTool as ToolBuilder>::name(), e)
            })?;
        
        let content = Self::run(parsed_args, config).await?;
        
        // 📝 Use text formatting for raw file content (not JSON)
        Ok(format_text_response(&content))
    }
}
