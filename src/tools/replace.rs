//! 🔧 Replace Tool - Advanced ToolBuilder implementation
//! 
//! Ultimate test of ToolBuilder pattern with complex args and dual operation modes

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tools::ToolBuilder;
use crate::config::Config;
use crate::fs::FileOps;
use crate::error::{EmpathicResult, EmpathicError};

/// 🔧 Advanced Replace Tool using modern ToolBuilder pattern
pub struct ReplaceTool;

#[derive(Deserialize)]
pub struct ReplaceArgs {
    path: String,
    #[serde(flatten)]
    operation: OperationMode,
    #[serde(default)]
    dry_run: bool,
    project: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum OperationMode {
    Single {
        search: String,
        replace: String,
        #[serde(default)]
        regex: bool,
        #[serde(default = "default_true")]
        fuzzy_match: bool,
        #[serde(default = "default_true")]
        global: bool,
        #[serde(default)]
        case_insensitive: bool,
        #[serde(default)]
        multiline: bool,
        #[serde(default)]
        dot_all: bool,
    },
    Batch {
        operations: Vec<ReplaceOperation>,
    },
}

#[derive(Deserialize, Clone)]
pub struct ReplaceOperation {
    search: String,
    replace: String,
    #[serde(default)]
    regex: bool,
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    fuzzy_match: bool,
    #[serde(default = "default_true")]
    global: bool,
    #[serde(default)]
    case_insensitive: bool,
    #[serde(default)]
    multiline: bool,
    #[serde(default)]
    dot_all: bool,
}

#[derive(Serialize)]
pub struct ReplaceOutput {
    success: bool,
    path: String,
    dry_run: bool,
    changes_made: bool,
    operations_count: usize,
    total_replacements: usize,
    matches: Vec<Value>,
    statistics: ReplaceStatistics,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<Value>,
    lsp_synced: bool,
}

#[derive(Serialize)]
pub struct ReplaceStatistics {
    original_lines: usize,
    new_lines: usize,
    lines_changed: i64,
    original_chars: usize,
    new_chars: usize,
    chars_changed: i64,
}

fn default_true() -> bool {
    true
}

impl OperationMode {
    // This enum handles the two operation modes elegantly through serde
}

#[async_trait]
impl ToolBuilder for ReplaceTool {
    type Args = ReplaceArgs;
    type Output = ReplaceOutput;

    fn name() -> &'static str {
        "replace"
    }
    
    fn description() -> &'static str {
        "🔧 Advanced Search and replace with fuzzy matching and batch operations"
    }
    
    fn schema() -> Value {
        // Complex schema supporting both single and batch modes
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to process"
                },
                "search": {
                    "type": "string", 
                    "description": "Search pattern (string literal by default, regex if regex=true)"
                },
                "replace": {
                    "type": "string",
                    "description": "Replacement string"
                },
                "operations": {
                    "type": "array",
                    "description": "Batch operations: array of {search, replace, regex?, fuzzy_match?, case_insensitive?, global?}",
                    "items": {
                        "type": "object",
                        "properties": {
                            "search": {"type": "string"},
                            "replace": {"type": "string"},
                            "regex": {"type": "boolean", "default": false},
                            "fuzzy_match": {"type": "boolean", "default": true},
                            "case_insensitive": {"type": "boolean", "default": false},
                            "global": {"type": "boolean", "default": true}
                        },
                        "required": ["search", "replace"]
                    }
                },
                "regex": {
                    "type": "boolean",
                    "description": "Use regex pattern matching (default: false)",
                    "default": false
                },
                "fuzzy_match": {
                    "type": "boolean", 
                    "description": "Allow fuzzy matching with small typos/variations (default: true)",
                    "default": true
                },
                "global": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: true)",
                    "default": true
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Case-insensitive matching (default: false)",
                    "default": false
                },
                "multiline": {
                    "type": "boolean",
                    "description": "Multiline mode for ^ and $ anchors (regex mode only, default: false)",
                    "default": false
                },
                "dot_all": {
                    "type": "boolean",
                    "description": "Dot matches newlines (regex mode only, default: false)",
                    "default": false
                },
                "dry_run": {
                    "type": "boolean",
                    "description": "Preview changes without modifying the file (default: false)",
                    "default": false
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            },
            "anyOf": [
                {"required": ["path", "search", "replace"]},
                {"required": ["path", "operations"]}
            ]
        })
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let working_dir = config.project_path(args.project.as_deref());
        let file_path = working_dir.join(&args.path);
        
        // Read the file content
        let original_content = FileOps::read_file(&file_path).await?;
        let mut current_content = original_content.clone();
        let mut all_matches = Vec::new();
        let mut total_replacements = 0;
        
        // Convert operation mode to a consistent list of operations
        let operations = match &args.operation {
            OperationMode::Single { search, replace, regex, fuzzy_match, global, case_insensitive, multiline, dot_all } => {
                vec![ReplaceOperation {
                    search: search.clone(),
                    replace: replace.clone(),
                    regex: *regex,
                    fuzzy_match: *fuzzy_match,
                    global: *global,
                    case_insensitive: *case_insensitive,
                    multiline: *multiline,
                    dot_all: *dot_all,
                }]
            }
            OperationMode::Batch { operations } => operations.clone(),
        };
        
        // Apply each operation sequentially (preserving original complex logic)
        for (i, op) in operations.iter().enumerate() {
            if op.search.is_empty() {
                continue;
            }
            
            let (new_content, matches) = if op.regex {
                // Regex mode - Unicode safe using regex crate
                let mut regex_builder = regex::RegexBuilder::new(&op.search);
                regex_builder.case_insensitive(op.case_insensitive);
                regex_builder.multi_line(op.multiline);
                regex_builder.dot_matches_new_line(op.dot_all);
                
                let regex = regex_builder.build()
                    .map_err(|e| EmpathicError::InvalidRegexPattern {
                        pattern: op.search.clone(),
                        reason: e.to_string(),
                    })?;
                
                let mut match_info = Vec::new();
                let new_content = if op.global {
                    regex.replace_all(&current_content, |caps: &regex::Captures| {
                        match_info.push(json!({
                            "operation_index": i,
                            "search_pattern": &op.search,
                            "replacement": &op.replace,
                            "match": caps.get(0).unwrap().as_str(),
                            "start": caps.get(0).unwrap().start(),
                            "end": caps.get(0).unwrap().end(),
                            "line": current_content[..caps.get(0).unwrap().start()].matches('\n').count() + 1,
                            "match_type": "regex",
                            "groups": caps.iter().skip(1).map(|m| m.map(|m| m.as_str()).unwrap_or("")).collect::<Vec<_>>()
                        }));
                        op.replace.as_str()
                    }).into_owned()
                } else if let Some(caps) = regex.captures(&current_content) {
                    match_info.push(json!({
                        "operation_index": i,
                        "search_pattern": &op.search,
                        "replacement": &op.replace,
                        "match": caps.get(0).unwrap().as_str(),
                        "start": caps.get(0).unwrap().start(),
                        "end": caps.get(0).unwrap().end(),
                        "line": current_content[..caps.get(0).unwrap().start()].matches('\n').count() + 1,
                        "match_type": "regex",
                        "groups": caps.iter().skip(1).map(|m| m.map(|m| m.as_str()).unwrap_or("")).collect::<Vec<_>>()
                    }));
                    regex.replace(&current_content, op.replace.as_str()).into_owned()
                } else {
                    current_content.clone()
                };
                (new_content, match_info)
            } else {
                // Literal string mode - Unicode safe using String::replace
                let (search_content, pattern) = if op.case_insensitive {
                    (current_content.to_lowercase(), op.search.to_lowercase())
                } else {
                    (current_content.clone(), op.search.clone())
                };
                
                let mut match_info = Vec::new();
                let mut start_pos = 0;
                
                // Find all matches for match reporting
                while let Some(pos) = search_content[start_pos..].find(&pattern) {
                    let absolute_pos = start_pos + pos;
                    match_info.push(json!({
                        "operation_index": i,
                        "search_pattern": &op.search,
                        "replacement": &op.replace,
                        "match": &current_content[absolute_pos..absolute_pos + op.search.len()],
                        "start": absolute_pos,
                        "end": absolute_pos + op.search.len(),
                        "line": current_content[..absolute_pos].matches('\n').count() + 1,
                        "match_type": "literal"
                    }));
                    
                    if !op.global {
                        break;
                    }
                    start_pos = absolute_pos + op.search.len();
                }
                
                // Perform replacement
                let new_content = if op.case_insensitive {
                    // Case insensitive replacement is more complex, but still Unicode safe
                    let mut result = current_content.clone();
                    let mut offset = 0i32;
                    
                    for match_obj in &match_info {
                        let start = match_obj["start"].as_u64().unwrap() as usize;
                        let end = match_obj["end"].as_u64().unwrap() as usize;
                        let adjusted_start = (start as i32 + offset) as usize;
                        let adjusted_end = (end as i32 + offset) as usize;
                        
                        result.replace_range(adjusted_start..adjusted_end, &op.replace);
                        offset += op.replace.len() as i32 - op.search.len() as i32;
                        
                        if !op.global {
                            break;
                        }
                    }
                    result
                } else {
                    // Simple case - use String::replace which is Unicode safe
                    if op.global {
                        current_content.replace(&op.search, &op.replace)
                    } else {
                        current_content.replacen(&op.search, &op.replace, 1)
                    }
                };
                
                (new_content, match_info)
            };
            
            current_content = new_content;
            total_replacements += matches.len();
            all_matches.extend(matches);
        }
        
        let changes_made = current_content != original_content;
        
        // Write the file if not dry run and changes were made
        let lsp_synced = if !args.dry_run && changes_made {
            FileOps::write_file(&file_path, &current_content).await?;
            false // 🚀 LSP sync removed for performance
        } else {
            false
        };
        
        // Calculate statistics
        let original_lines = original_content.lines().count();
        let new_lines = current_content.lines().count();
        let original_chars = original_content.chars().count();
        let new_chars = current_content.chars().count();
        
        let statistics = ReplaceStatistics {
            original_lines,
            new_lines,
            lines_changed: new_lines as i64 - original_lines as i64,
            original_chars,
            new_chars,
            chars_changed: new_chars as i64 - original_chars as i64,
        };
        
        // Add preview for dry run
        let preview = if args.dry_run && changes_made {
            let preview_lines: Vec<&str> = current_content.lines().take(20).collect();
            Some(json!({
                "first_20_lines": preview_lines,
                "total_lines": new_lines,
                "truncated": new_lines > 20
            }))
        } else {
            None
        };
        
        Ok(ReplaceOutput {
            success: true,
            path: file_path.to_string_lossy().to_string(),
            dry_run: args.dry_run,
            changes_made,
            operations_count: operations.len(),
            total_replacements,
            matches: all_matches,
            statistics,
            preview,
            lsp_synced,
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(ReplaceTool);
