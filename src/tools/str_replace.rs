//! ✂️ String Replace Tool - Simple, safe, surgical edits
//! 
//! Replaces text that appears EXACTLY ONCE in a file.
//! Errors if the old string appears 0 or >1 times for safety.
//! 
//! This is the tool you reach for when you know exactly what to change
//! and want the operation to fail if your assumption is wrong.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tools::ToolBuilder;
use crate::config::Config;
use crate::fs::FileOps;
use crate::error::{EmpathicResult, EmpathicError};

/// ✂️ Simple String Replace - surgical edits with safety
pub struct StrReplaceTool;

#[derive(Deserialize)]
pub struct StrReplaceArgs {
    /// Path to the file to edit
    path: String,
    /// Old string that must appear exactly once
    old_str: String,
    /// New string to replace with (empty string deletes)
    #[serde(default)]
    new_str: String,
    /// Optional project name for path resolution
    project: Option<String>,
}

#[derive(Serialize)]
pub struct StrReplaceOutput {
    success: bool,
    path: String,
    old_str: String,
    new_str: String,
    replaced: bool,
    line_number: usize,
    context_before: String,
    context_after: String,
}

#[async_trait]
impl ToolBuilder for StrReplaceTool {
    type Args = StrReplaceArgs;
    type Output = StrReplaceOutput;

    fn name() -> &'static str {
        "str_replace"
    }
    
    fn description() -> &'static str {
        "✂️ Replace a unique string in a file with another string. The string to replace must appear exactly once in the file."
    }
    
    fn schema() -> Value {
        json!({
            "type": "object",
            "required": ["path", "old_str"],
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_str": {
                    "type": "string",
                    "description": "String to replace (must be unique in file)"
                },
                "new_str": {
                    "type": "string",
                    "description": "String to replace with (empty to delete)",
                    "default": ""
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            }
        })
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        // Validate old_str is not empty
        if args.old_str.is_empty() {
            return Err(EmpathicError::InvalidArgument {
                arg: "old_str".to_string(),
                reason: "Cannot replace empty string".to_string(),
            });
        }
        
        // Resolve file path
        let working_dir = config.project_path(args.project.as_deref());
        let file_path = working_dir.join(&args.path);
        
        // Read file content
        let original_content = FileOps::read_file(&file_path).await?;
        
        // Count occurrences of old_str
        let occurrences: Vec<usize> = original_content
            .match_indices(&args.old_str)
            .map(|(idx, _)| idx)
            .collect();
        
        // Safety check: must appear exactly once
        match occurrences.len() {
            0 => {
                return Err(EmpathicError::StrReplaceNotFound {
                    path: file_path.to_string_lossy().to_string(),
                    search_str: args.old_str.clone(),
                });
            }
            1 => {
                // Perfect! Proceed with replacement
            }
            n => {
                return Err(EmpathicError::StrReplaceMultipleMatches {
                    path: file_path.to_string_lossy().to_string(),
                    search_str: args.old_str.clone(),
                    count: n,
                });
            }
        }
        
        // Perform the replacement (we know it's unique)
        let new_content = original_content.replacen(&args.old_str, &args.new_str, 1);
        
        // Calculate line number and context
        let match_pos = occurrences[0];
        let before_match = &original_content[..match_pos];
        let line_number = before_match.matches('\n').count() + 1;
        
        // Extract context (up to 2 lines before and after)
        let lines: Vec<&str> = original_content.lines().collect();
        let match_line_idx = line_number.saturating_sub(1);
        
        let context_start = match_line_idx.saturating_sub(2);
        let context_before = lines[context_start..match_line_idx].join("\n");
        
        let new_lines: Vec<&str> = new_content.lines().collect();
        let new_context_end = (match_line_idx + 3).min(new_lines.len());
        let context_after = new_lines[match_line_idx..new_context_end].join("\n");
        
        // Write the modified content back to file
        FileOps::write_file(&file_path, &new_content).await?;
        
        Ok(StrReplaceOutput {
            success: true,
            path: file_path.to_string_lossy().to_string(),
            old_str: args.old_str,
            new_str: args.new_str,
            replaced: true,
            line_number,
            context_before,
            context_after,
        })
    }
}

// ✂️ Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(StrReplaceTool);
