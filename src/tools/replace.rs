use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

use crate::config::Config;
use crate::tools::Tool;
use crate::fs::FileOps;

pub struct ReplaceTool;

#[async_trait]
impl Tool for ReplaceTool {
    fn name(&self) -> &'static str {
        "replace"
    }
    
    fn description(&self) -> &'static str {
        "🔧 Advanced Search and replace with fuzzy matching and batch operations"
    }
    
    fn schema(&self) -> Value {
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
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("path is required"))?;
        
        let dry_run = args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        let file_path = working_dir.join(path_str);
        
        // Read the file content
        let original_content = FileOps::read_file(&file_path).await?;
        let mut current_content = original_content.clone();
        let mut all_matches = Vec::new();
        let mut total_replacements = 0;
        
        // Determine if we're doing batch operations or single operation
        let operations = if let Some(ops_array) = args.get("operations").and_then(|v| v.as_array()) {
            // Batch mode
            ops_array.iter().map(|op| {
                (
                    op.get("search").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    op.get("replace").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    op.get("regex").and_then(|v| v.as_bool()).unwrap_or(false),
                    op.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false),
                    op.get("global").and_then(|v| v.as_bool()).unwrap_or(true),
                    op.get("multiline").and_then(|v| v.as_bool()).unwrap_or(false),
                    op.get("dot_all").and_then(|v| v.as_bool()).unwrap_or(false),
                )
            }).collect()
        } else {
            // Single operation mode
            let search = args.get("search").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let replace = args.get("replace").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let regex = args.get("regex").and_then(|v| v.as_bool()).unwrap_or(false);
            let case_insensitive = args.get("case_insensitive").and_then(|v| v.as_bool()).unwrap_or(false);
            let global = args.get("global").and_then(|v| v.as_bool()).unwrap_or(true);
            let multiline = args.get("multiline").and_then(|v| v.as_bool()).unwrap_or(false);
            let dot_all = args.get("dot_all").and_then(|v| v.as_bool()).unwrap_or(false);
            
            vec![(search, replace, regex, case_insensitive, global, multiline, dot_all)]
        };
        
        // Apply each operation sequentially
        for (i, (search_pattern, replace_str, is_regex, case_insensitive, global, multiline, dot_all)) in operations.iter().enumerate() {
            if search_pattern.is_empty() {
                continue;
            }
            
            let (new_content, matches) = if *is_regex {
                // Regex mode - Unicode safe using regex crate
                let mut regex_builder = regex::RegexBuilder::new(search_pattern);
                regex_builder.case_insensitive(*case_insensitive);
                regex_builder.multi_line(*multiline);
                regex_builder.dot_matches_new_line(*dot_all);
                
                let regex = regex_builder.build()
                    .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;
                
                let mut match_info = Vec::new();
                let new_content = if *global {
                    regex.replace_all(&current_content, |caps: &regex::Captures| {
                        match_info.push(json!({
                            "operation_index": i,
                            "search_pattern": search_pattern,
                            "replacement": replace_str,
                            "match": caps.get(0).unwrap().as_str(),
                            "start": caps.get(0).unwrap().start(),
                            "end": caps.get(0).unwrap().end(),
                            "line": current_content[..caps.get(0).unwrap().start()].matches('\n').count() + 1,
                            "match_type": "regex",
                            "groups": caps.iter().skip(1).map(|m| m.map(|m| m.as_str()).unwrap_or("")).collect::<Vec<_>>()
                        }));
                        replace_str.as_str()
                    }).into_owned()
                } else {
                    if let Some(caps) = regex.captures(&current_content) {
                        match_info.push(json!({
                            "operation_index": i,
                            "search_pattern": search_pattern,
                            "replacement": replace_str,
                            "match": caps.get(0).unwrap().as_str(),
                            "start": caps.get(0).unwrap().start(),
                            "end": caps.get(0).unwrap().end(),
                            "line": current_content[..caps.get(0).unwrap().start()].matches('\n').count() + 1,
                            "match_type": "regex",
                            "groups": caps.iter().skip(1).map(|m| m.map(|m| m.as_str()).unwrap_or("")).collect::<Vec<_>>()
                        }));
                        regex.replace(&current_content, replace_str.as_str()).into_owned()
                    } else {
                        current_content.clone()
                    }
                };
                (new_content, match_info)
            } else {
                // Literal string mode - Unicode safe using String::replace
                let (search_content, pattern) = if *case_insensitive {
                    (current_content.to_lowercase(), search_pattern.to_lowercase())
                } else {
                    (current_content.clone(), search_pattern.clone())
                };
                
                let mut match_info = Vec::new();
                let mut start_pos = 0;
                
                // Find all matches for match reporting
                while let Some(pos) = search_content[start_pos..].find(&pattern) {
                    let absolute_pos = start_pos + pos;
                    match_info.push(json!({
                        "operation_index": i,
                        "search_pattern": search_pattern,
                        "replacement": replace_str,
                        "match": &current_content[absolute_pos..absolute_pos + search_pattern.len()],
                        "start": absolute_pos,
                        "end": absolute_pos + search_pattern.len(),
                        "line": current_content[..absolute_pos].matches('\n').count() + 1,
                        "match_type": "literal"
                    }));
                    
                    if !*global {
                        break;
                    }
                    start_pos = absolute_pos + search_pattern.len();
                }
                
                // Perform replacement
                let new_content = if *case_insensitive {
                    // Case insensitive replacement is more complex, but still Unicode safe
                    let mut result = current_content.clone();
                    let mut offset = 0i32;
                    
                    for match_obj in &match_info {
                        let start = match_obj["start"].as_u64().unwrap() as usize;
                        let end = match_obj["end"].as_u64().unwrap() as usize;
                        let adjusted_start = (start as i32 + offset) as usize;
                        let adjusted_end = (end as i32 + offset) as usize;
                        
                        result.replace_range(adjusted_start..adjusted_end, replace_str);
                        offset += replace_str.len() as i32 - search_pattern.len() as i32;
                        
                        if !*global {
                            break;
                        }
                    }
                    result
                } else {
                    // Simple case - use String::replace which is Unicode safe
                    if *global {
                        current_content.replace(search_pattern, replace_str)
                    } else {
                        current_content.replacen(search_pattern, replace_str, 1)
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
        if !dry_run && changes_made {
            FileOps::write_file(&file_path, &current_content).await?;
        }
        
        // Calculate statistics
        let original_lines = original_content.lines().count();
        let new_lines = current_content.lines().count();
        let original_chars = original_content.chars().count();
        let new_chars = current_content.chars().count();
        
        let mut result = json!({
            "success": true,
            "path": file_path.to_string_lossy(),
            "dry_run": dry_run,
            "changes_made": changes_made,
            "operations_count": operations.len(),
            "total_replacements": total_replacements,
            "matches": all_matches,
            "statistics": {
                "original_lines": original_lines,
                "new_lines": new_lines,
                "lines_changed": new_lines as i64 - original_lines as i64,
                "original_chars": original_chars,
                "new_chars": new_chars,
                "chars_changed": new_chars as i64 - original_chars as i64
            }
        });
        
        // Add preview for dry run
        if dry_run && changes_made {
            let preview_lines: Vec<&str> = current_content.lines().take(20).collect();
            if let Some(obj) = result.as_object_mut() {
                obj.insert("preview".to_string(), json!({
                    "first_20_lines": preview_lines,
                    "total_lines": new_lines,
                    "truncated": new_lines > 20
                }));
            }
        }
        
        // Always use MCP format wrapper - no early returns!
        Ok(crate::tools::format_json_response(&result)?)
    }
}
