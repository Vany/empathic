//! 🔍 LSP Find References Tool - Find all references to a symbol
//!
//! Provides comprehensive reference finding using rust-analyzer

use super::base::{BaseLspTool, LspInput, LspOutput, get_lsp_manager};
use crate::config::Config;
use crate::error::{EmpathicError, EmpathicResult};
use async_trait::async_trait;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use url::Url;

/// 🔍 LSP Find References Tool implementation
pub struct LspFindReferencesTool;

/// Input parameters for lsp_find_references tool
#[derive(Debug, Deserialize)]
pub struct FindReferencesInput {
    file_path: String,
    project: String,
    line: u32,
    character: u32,
    include_declaration: Option<bool>,
}

impl LspInput for FindReferencesInput {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn project(&self) -> &str {
        &self.project
    }
}

/// Output format for reference results
#[derive(Debug, Serialize, Deserialize)]
pub struct FindReferencesOutput {
    file_path: String,
    project: String,
    position: PositionInfo,
    symbol_info: Option<SymbolInfo>,
    references: Vec<ReferenceLocation>,
    summary: ReferenceSummary,
}

impl LspOutput for FindReferencesOutput {
    fn set_file_path(&mut self, path: String) {
        self.file_path = path;
    }

    fn set_project(&mut self, project: String) {
        self.project = project;
    }
}

/// Position information
#[derive(Debug, Serialize, Deserialize)]
pub struct PositionInfo {
    pub line: u32,
    pub character: u32,
}

/// Symbol information
#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub detail: Option<String>,
}

/// Reference location with context
#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceLocation {
    pub file_path: String,
    pub line: u32,
    pub character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub context: String,
    pub reference_kind: String,
}

/// Summary of reference search results
#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceSummary {
    pub total_references: usize,
    pub files_with_references: usize,
    pub include_declaration: bool,
}

impl ReferenceLocation {
    fn from_lsp_location(location: &Location, reference_kind: &str, context: &str) -> Self {
        let file_path = Url::parse(location.uri.as_str())
            .ok()
            .and_then(|u| u.to_file_path().ok())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            file_path,
            line: location.range.start.line,
            character: location.range.start.character,
            end_line: location.range.end.line,
            end_character: location.range.end.character,
            context: context.to_string(),
            reference_kind: reference_kind.to_string(),
        }
    }
}

#[async_trait]
impl BaseLspTool for LspFindReferencesTool {
    type Input = FindReferencesInput;
    type Output = FindReferencesOutput;

    fn name() -> &'static str {
        "lsp_find_references"
    }

    fn description() -> &'static str {
        "🔍 Find all references to a symbol in Rust code using rust-analyzer"
    }

    fn additional_schema() -> serde_json::Value {
        json!({
            "line": {
                "type": "integer",
                "minimum": 0,
                "description": "Line number (0-indexed)"
            },
            "character": {
                "type": "integer",
                "minimum": 0,
                "description": "Character position (0-indexed)"
            },
            "include_declaration": {
                "type": "boolean",
                "description": "Whether to include the symbol declaration in results (default: true)"
            }
        })
    }

    fn additional_required() -> Vec<&'static str> {
        vec!["line", "character"]
    }

    async fn execute_lsp(
        &self,
        input: Self::Input,
        file_path: PathBuf,
        config: &Config,
    ) -> EmpathicResult<Self::Output> {
        let lsp_manager = get_lsp_manager(config)?;

        // Ensure document is open/synced
        lsp_manager.ensure_document_open(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_find_references",
                format!("Failed to sync document {}: {}", file_path.display(), e)
            ))?;

        // Get LSP client
        let client = lsp_manager.get_client(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_find_references",
                format!("Failed to get LSP client for {}: {}", file_path.display(), e)
            ))?;

        let include_declaration = input.include_declaration.unwrap_or(true);

        log::info!("🔍 Finding references at {}:{}:{} (include_declaration: {})",
            file_path.display(), input.line, input.character, include_declaration);

        // Build LSP find references request
        let uri = Url::from_file_path(&file_path)
            .map_err(|_| EmpathicError::InvalidPath { path: file_path.clone() })?;

        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: uri.to_string().parse().unwrap()
                },
                position: Position {
                    line: input.line,
                    character: input.character,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: ReferenceContext {
                include_declaration,
            },
        };

        // Send find references request
        let references_result = client.find_references(params).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_find_references",
                format!("Find references request failed for {}:{}:{}: {}",
                    file_path.display(), input.line, input.character, e)
            ))?;

        // Read file contents for context extraction
        let file_contents = tokio::fs::read_to_string(&file_path).await
            .unwrap_or_default();
        let lines: Vec<&str> = file_contents.lines().collect();

        // Get symbol info from current position
        let symbol_info = if let Some(line) = lines.get(input.line as usize) {
            let chars: Vec<char> = line.chars().collect();
            let mut start = input.character as usize;
            let mut end = input.character as usize;

            // Find word boundaries
            while start > 0 && chars.get(start - 1).map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
                start -= 1;
            }
            while end < chars.len() && chars.get(end).map(|c| c.is_alphanumeric() || *c == '_').unwrap_or(false) {
                end += 1;
            }

            let symbol_name: String = chars[start..end].iter().collect();
            if !symbol_name.is_empty() {
                Some(SymbolInfo {
                    name: symbol_name,
                    kind: "symbol".to_string(),
                    detail: Some(line.trim().to_string()),
                })
            } else {
                None
            }
        } else {
            None
        };

        // Convert LSP response to our format
        let references: Vec<ReferenceLocation> = if let Some(locations) = references_result {
            let mut refs = Vec::new();
            let mut unique_files = std::collections::HashSet::new();

            for location in locations {
                // Extract context from the reference location
                let ref_path = Url::parse(location.uri.as_str())
                    .ok()
                    .and_then(|u| u.to_file_path().ok());

                let context = if let Some(ref_path) = &ref_path {
                    if let Ok(content) = tokio::fs::read_to_string(ref_path).await {
                        let ref_lines: Vec<&str> = content.lines().collect();
                        ref_lines.get(location.range.start.line as usize)
                            .map(|s| s.trim().to_string())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let reference_kind = if location.range.start.line == input.line && 
                    location.range.start.character == input.character {
                    "declaration"
                } else {
                    "reference"
                };

                if let Some(path) = ref_path {
                    unique_files.insert(path);
                }

                refs.push(ReferenceLocation::from_lsp_location(&location, reference_kind, &context));
            }

            refs
        } else {
            Vec::new()
        };

        let files_with_references = references.iter()
            .map(|r| r.file_path.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        // Calculate length before moving references
        let total_references = references.len();

        Ok(FindReferencesOutput {
            file_path: String::new(), // Set by base trait
            project: String::new(),   // Set by base trait
            position: PositionInfo {
                line: input.line,
                character: input.character,
            },
            symbol_info,
            references,
            summary: ReferenceSummary {
                total_references,
                files_with_references,
                include_declaration,
            },
        })
    }
}
