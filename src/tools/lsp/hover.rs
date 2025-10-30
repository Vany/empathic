//! 🔍 LSP Hover Tool - Get type information and documentation at cursor position
//!
//! Retrieves hover information from rust-analyzer for Rust files at specific positions

use super::base::{BaseLspTool, LspInput, LspOutput, get_lsp_manager};
use crate::config::Config;
use crate::error::{EmpathicError, EmpathicResult};
use async_trait::async_trait;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use url::Url;

/// 🔍 LSP Hover Tool implementation
pub struct LspHoverTool;

/// Input parameters for lsp_hover tool
#[derive(Debug, Deserialize)]
pub struct HoverInput {
    file_path: String,
    project: String,
    line: u32,
    character: u32,
}

impl LspInput for HoverInput {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn project(&self) -> &str {
        &self.project
    }
}

/// Output format for hover information
#[derive(Debug, Serialize, Deserialize)]
pub struct HoverOutput {
    pub file_path: String,
    pub project: String,
    pub position: PositionInfo,
    pub hover_info: Option<HoverInfo>,
}

impl LspOutput for HoverOutput {
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

/// Hover information content
#[derive(Debug, Serialize, Deserialize)]
pub struct HoverInfo {
    pub contents: Vec<String>,
    pub documentation: Option<String>,
    pub range: Option<RangeInfo>,
}

/// Range information for hover
#[derive(Debug, Serialize, Deserialize)]
pub struct RangeInfo {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

impl RangeInfo {
    fn from_lsp_range(range: &Range) -> Self {
        Self {
            start_line: range.start.line,
            start_character: range.start.character,
            end_line: range.end.line,
            end_character: range.end.character,
        }
    }
}

impl HoverInfo {
    /// Convert from LSP Hover type to our internal format
    fn from_lsp_hover(hover: &Hover) -> Self {
        let mut contents = Vec::new();
        let mut documentation = None;

        match &hover.contents {
            HoverContents::Scalar(marked_string) => {
                contents.push(Self::extract_content_from_marked_string(marked_string));
            }
            HoverContents::Array(marked_strings) => {
                for marked_string in marked_strings {
                    contents.push(Self::extract_content_from_marked_string(marked_string));
                }
            }
            HoverContents::Markup(markup) => {
                match markup.kind {
                    MarkupKind::PlainText => {
                        contents.push(markup.value.clone());
                    }
                    MarkupKind::Markdown => {
                        // Split markdown content into code blocks and documentation
                        let lines: Vec<&str> = markup.value.lines().collect();
                        let mut current_content = String::new();

                        for line in lines {
                            if line.starts_with("```") || current_content.is_empty() {
                                current_content.push_str(line);
                                current_content.push('\n');
                            } else if !line.trim().is_empty() {
                                // This might be documentation
                                if documentation.is_none() {
                                    documentation = Some(line.to_string());
                                } else {
                                    documentation = Some(format!("{}\n{}", 
                                        documentation.as_ref().unwrap(), line));
                                }
                            }
                        }

                        if !current_content.is_empty() {
                            contents.push(current_content);
                        }
                    }
                }
            }
        }

        Self {
            contents,
            documentation,
            range: hover.range.as_ref().map(RangeInfo::from_lsp_range),
        }
    }

    fn extract_content_from_marked_string(marked_string: &MarkedString) -> String {
        match marked_string {
            MarkedString::String(s) => s.clone(),
            MarkedString::LanguageString(lang_string) => {
                format!("```{}\n{}\n```", lang_string.language, lang_string.value)
            }
        }
    }
}

#[async_trait]
impl BaseLspTool for LspHoverTool {
    type Input = HoverInput;
    type Output = HoverOutput;

    fn name() -> &'static str {
        "lsp_hover"
    }

    fn description() -> &'static str {
        "🔍 Get type information and documentation at cursor position for Rust files using rust-analyzer"
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

        // Ensure document is open/synced with LSP server
        lsp_manager.ensure_document_open(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_hover",
                format!("Failed to sync document {}: {}", file_path.display(), e)
            ))?;

        // Get LSP client for this file's project
        let client = lsp_manager.get_client(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_hover",
                format!("Failed to get LSP client for {}: {}", file_path.display(), e)
            ))?;

        log::info!("🔍 Hover at {}:{}:{}", file_path.display(), input.line, input.character);

        // Build LSP hover request parameters
        let uri = Url::from_file_path(&file_path)
            .map_err(|_| EmpathicError::InvalidPath { path: file_path.clone() })?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { 
                    uri: uri.to_string().parse().unwrap() 
                },
                position: Position {
                    line: input.line,
                    character: input.character,
                },
            },
            work_done_progress_params: Default::default(),
        };

        // Send hover request to rust-analyzer
        let hover_result = client.hover(params).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_hover",
                format!("Hover request failed for {}:{}:{}: {}", 
                    file_path.display(), input.line, input.character, e)
            ))?;

        // Convert LSP response to our format
        let hover_info = hover_result.map(|h| HoverInfo::from_lsp_hover(&h));

        Ok(HoverOutput {
            file_path: String::new(), // Set by base trait
            project: String::new(),   // Set by base trait
            position: PositionInfo {
                line: input.line,
                character: input.character,
            },
            hover_info,
        })
    }
}
