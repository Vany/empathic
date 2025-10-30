//! 🎯 LSP Completion Tool - Intelligent autocompletion for Rust code
//!
//! Provides context-aware completion suggestions using rust-analyzer

use super::base::{BaseLspTool, LspInput, LspOutput, get_lsp_manager};
use crate::config::Config;
use crate::error::{EmpathicResult, EmpathicError};
use async_trait::async_trait;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use url::Url;

/// 🎯 LSP Completion Tool implementation
pub struct LspCompletionTool;

/// Input parameters for lsp_completion tool
#[derive(Debug, Deserialize)]
pub struct CompletionInput {
    file_path: String,
    project: String,
    line: u32,
    character: u32,
}

impl LspInput for CompletionInput {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn project(&self) -> &str {
        &self.project
    }
}

/// Output format for completion suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionOutput {
    pub file_path: String,
    pub project: String,
    pub position: PositionInfo,
    pub completions: Vec<CompletionItem>,
    pub context: CompletionContext,
}

impl LspOutput for CompletionOutput {
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

/// Completion context information
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionContext {
    pub trigger_kind: String,
    pub current_word: String,
    pub context_line: String,
}

/// Individual completion item
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub filter_text: Option<String>,
    pub sort_text: Option<String>,
}

impl CompletionItem {
    /// Convert from LSP CompletionItem to our internal format
    fn from_lsp_completion_item(item: &lsp_types::CompletionItem) -> Self {
        Self {
            label: item.label.clone(),
            kind: format!("{:?}", item.kind.unwrap_or(CompletionItemKind::TEXT)),
            detail: item.detail.clone(),
            documentation: item.documentation.as_ref().map(|doc| match doc {
                Documentation::String(s) => s.clone(),
                Documentation::MarkupContent(markup) => markup.value.clone(),
            }),
            insert_text: item.insert_text.clone(),
            filter_text: item.filter_text.clone(),
            sort_text: item.sort_text.clone(),
        }
    }
}

#[async_trait]
impl BaseLspTool for LspCompletionTool {
    type Input = CompletionInput;
    type Output = CompletionOutput;

    fn name() -> &'static str {
        "lsp_completion"
    }

    fn description() -> &'static str {
        "🎯 Get intelligent autocompletion suggestions for Rust code using rust-analyzer"
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
                "lsp_completion",
                format!("Failed to sync document {}: {}", file_path.display(), e)
            ))?;

        // Get LSP client
        let client = lsp_manager.get_client(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_completion",
                format!("Failed to get LSP client for {}: {}", file_path.display(), e)
            ))?;

        log::info!("🎯 Completion at {}:{}:{}", file_path.display(), input.line, input.character);

        // Read file to get context
        let file_content = tokio::fs::read_to_string(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_completion",
                format!("Failed to read file {}: {}", file_path.display(), e)
            ))?;

        let lines: Vec<&str> = file_content.lines().collect();
        let context_line = lines.get(input.line as usize)
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Extract current word being typed
        let current_word = if let Some(line) = lines.get(input.line as usize) {
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

            chars[start..end].iter().collect()
        } else {
            String::new()
        };

        // Build LSP completion request
        let uri = Url::from_file_path(&file_path)
            .map_err(|_| EmpathicError::InvalidPath { path: file_path.clone() })?;

        let params = CompletionParams {
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
            context: Some(lsp_types::CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
        };

        // Send completion request
        let completion_result = client.completion(params).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_completion",
                format!("Completion request failed for {}:{}:{}: {}",
                    file_path.display(), input.line, input.character, e)
            ))?;

        // Convert LSP response to our format
        let completions = match completion_result {
            Some(CompletionResponse::Array(items)) => {
                items.iter()
                    .map(CompletionItem::from_lsp_completion_item)
                    .collect()
            }
            Some(CompletionResponse::List(list)) => {
                list.items.iter()
                    .map(CompletionItem::from_lsp_completion_item)
                    .collect()
            }
            None => Vec::new(),
        };

        Ok(CompletionOutput {
            file_path: String::new(), // Set by base trait
            project: String::new(),   // Set by base trait
            position: PositionInfo {
                line: input.line,
                character: input.character,
            },
            completions,
            context: CompletionContext {
                trigger_kind: "invoked".to_string(),
                current_word,
                context_line,
            },
        })
    }
}
