//! 🧭 LSP Go To Definition Tool - Navigate to symbol definitions
//!
//! Provides navigation to symbol definitions using rust-analyzer

use super::base::{BaseLspTool, LspInput, LspOutput, get_lsp_manager};
use crate::config::Config;
use crate::error::{EmpathicError, EmpathicResult};
use async_trait::async_trait;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use url::Url;

/// 🧭 LSP Go To Definition Tool implementation
pub struct LspGotoDefinitionTool;

/// Input parameters for lsp_goto_definition tool
#[derive(Debug, Deserialize)]
pub struct GotoDefinitionInput {
    file_path: String,
    project: String,
    line: u32,
    character: u32,
}

impl LspInput for GotoDefinitionInput {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn project(&self) -> &str {
        &self.project
    }
}

/// Output format for definition locations
#[derive(Debug, Serialize, Deserialize)]
pub struct GotoDefinitionOutput {
    file_path: String,
    project: String,
    position: PositionInfo,
    definitions: Vec<DefinitionLocation>,
    symbol_info: Option<SymbolInfo>,
}

impl LspOutput for GotoDefinitionOutput {
    fn set_file_path(&mut self, path: String) {
        self.file_path = path;
    }

    fn set_project(&mut self, project: String) {
        self.project = project;
    }
}

/// Position information
#[derive(Debug, Serialize, Deserialize)]
struct PositionInfo {
    line: u32,
    character: u32,
}

/// Definition location information
#[derive(Debug, Serialize, Deserialize)]
struct DefinitionLocation {
    file_path: String,
    line: u32,
    character: u32,
    end_line: u32,
    end_character: u32,
    context: Option<String>,
}

/// Symbol information at current position
#[derive(Debug, Serialize, Deserialize)]
struct SymbolInfo {
    name: String,
    kind: String,
    detail: Option<String>,
}

impl DefinitionLocation {
    /// Convert from LSP Location to our format
    fn from_lsp_location(location: &Location, file_path_context: Option<&str>) -> EmpathicResult<Self> {
        let uri = Url::parse(location.uri.as_str())
            .map_err(|e| EmpathicError::tool_failed("lsp_goto_definition", format!("Invalid URI: {}", e)))?;
        
        let file_path = uri.to_file_path()
            .map_err(|_| EmpathicError::tool_failed("lsp_goto_definition", "Failed to convert URI to file path"))?
            .to_string_lossy()
            .to_string();
        
        Ok(Self {
            file_path,
            line: location.range.start.line,
            character: location.range.start.character,
            end_line: location.range.end.line,
            end_character: location.range.end.character,
            context: file_path_context.map(String::from),
        })
    }
}

#[async_trait]
impl BaseLspTool for LspGotoDefinitionTool {
    type Input = GotoDefinitionInput;
    type Output = GotoDefinitionOutput;

    fn name() -> &'static str {
        "lsp_goto_definition"
    }

    fn description() -> &'static str {
        "🧭 Navigate to symbol definition for Rust code using rust-analyzer"
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
                "lsp_goto_definition",
                format!("Failed to sync document {}: {}", file_path.display(), e)
            ))?;

        // Get LSP client for this file's project
        let client = lsp_manager.get_client(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_goto_definition",
                format!("Failed to get LSP client for {}: {}", file_path.display(), e)
            ))?;

        log::info!("🧭 Finding definition at {}:{}:{}", 
            file_path.display(), input.line, input.character);

        // Build LSP goto definition request parameters
        let uri = Url::from_file_path(&file_path)
            .map_err(|_| EmpathicError::InvalidPath { path: file_path.clone() })?;

        let params = GotoDefinitionParams {
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
            partial_result_params: Default::default(),
        };

        // Send goto definition request
        let definition_result = client.goto_definition(params).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_goto_definition",
                format!("Goto definition failed for {}:{}:{}: {}", 
                    file_path.display(), input.line, input.character, e)
            ))?;

        // Convert LSP response to our format
        let mut definitions = Vec::new();
        
        if let Some(response) = definition_result {
            match response {
                GotoDefinitionResponse::Scalar(location) => {
                    definitions.push(DefinitionLocation::from_lsp_location(&location, None)?);
                }
                GotoDefinitionResponse::Array(locations) => {
                    for location in locations {
                        definitions.push(DefinitionLocation::from_lsp_location(&location, None)?);
                    }
                }
                GotoDefinitionResponse::Link(location_links) => {
                    // Location links provide more detail but we can extract basic Location from them
                    for link in location_links {
                        let location = Location {
                            uri: link.target_uri.clone(),
                            range: link.target_selection_range,
                        };
                        definitions.push(DefinitionLocation::from_lsp_location(&location, None)?);
                    }
                }
            }
        }

        // Create symbol info (optional, could extract from hover if needed)
        let symbol_info = if definitions.is_empty() {
            None
        } else {
            Some(SymbolInfo {
                name: format!("Symbol at {}:{}", input.line, input.character),
                kind: "definition".to_string(),
                detail: Some(format!("{} definition(s) found", definitions.len())),
            })
        };

        Ok(GotoDefinitionOutput {
            file_path: String::new(), // Will be set by base trait
            project: String::new(),   // Will be set by base trait
            position: PositionInfo {
                line: input.line,
                character: input.character,
            },
            definitions,
            symbol_info,
        })
    }
}
