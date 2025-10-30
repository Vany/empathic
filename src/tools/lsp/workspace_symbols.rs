//! 🔍 LSP Workspace Symbols Tool - Search for symbols across the entire project
//!
//! Provides project-wide symbol search capabilities for Rust workspaces

use crate::error::EmpathicResult;
use async_trait::async_trait;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

/// 🔍 LSP Workspace Symbols Tool implementation
pub struct LspWorkspaceSymbolsTool;

/// Input parameters for lsp_workspace_symbols tool
#[derive(Debug, Deserialize)]
struct WorkspaceSymbolsInput {
    query: String,
    project: String,
}

/// Output format for workspace symbols
#[derive(Debug, Serialize)]
struct WorkspaceSymbolsOutput {
    query: String,
    project: String,
    symbols: Vec<WorkspaceSymbolInfo>,
    summary: WorkspaceSymbolsSummary,
}

/// Simplified workspace symbol information for MCP output
#[derive(Debug, Serialize)]
struct WorkspaceSymbolInfo {
    name: String,
    kind: String,
    location: LocationInfo,
    container_name: Option<String>,
    detail: Option<String>,
}

/// Location information for symbols
#[derive(Debug, Serialize)]
struct LocationInfo {
    file_path: String,
    line: u32,
    character: u32,
    end_line: u32,
    end_character: u32,
}

/// Summary statistics for workspace symbols
#[derive(Debug, Serialize)]
struct WorkspaceSymbolsSummary {
    total_symbols: usize,
    files_searched: usize,
    query_length: usize,
    symbol_types: std::collections::HashMap<String, usize>,
}

impl WorkspaceSymbolInfo {
    fn from_symbol_information(symbol: &SymbolInformation) -> Self {
        Self {
            name: symbol.name.clone(),
            kind: format!("{:?}", symbol.kind),
            location: LocationInfo {
                file_path: Url::parse(symbol.location.uri.as_str()).ok().and_then(|u| u.to_file_path().ok())
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                line: symbol.location.range.start.line,
                character: symbol.location.range.start.character,
                end_line: symbol.location.range.end.line,
                end_character: symbol.location.range.end.character,
            },
            container_name: symbol.container_name.clone(),
            detail: None,
        }
    }


}

impl WorkspaceSymbolsSummary {
    fn from_symbols(symbols: &[WorkspaceSymbolInfo], query: &str, files_searched: usize) -> Self {
        let mut symbol_types = std::collections::HashMap::new();
        
        for symbol in symbols {
            *symbol_types.entry(symbol.kind.clone()).or_insert(0) += 1;
        }

        Self {
            total_symbols: symbols.len(),
            files_searched,
            query_length: query.len(),
            symbol_types,
        }
    }
}

#[async_trait]
impl crate::tools::Tool for LspWorkspaceSymbolsTool {
    fn name(&self) -> &'static str {
        "lsp_workspace_symbols"
    }

    fn description(&self) -> &'static str {
        "🔍 Search for symbols across the entire Rust workspace using rust-analyzer"
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Symbol search pattern (e.g., function name, struct name, etc.)"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for path resolution"
                }
            },
            "required": ["query", "project"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: serde_json::Value, config: &crate::config::Config) -> EmpathicResult<serde_json::Value> {
        let input: WorkspaceSymbolsInput = serde_json::from_value(args)?;
        
        // 🎯 Resolve project directory
        let working_dir = config.project_path(Some(&input.project));

        // Validate project directory exists
        if !working_dir.exists() {
            return Err(crate::error::EmpathicError::FileNotFound { 
                path: working_dir.clone() 
            });
        }

        // Ensure we have a Rust project (check for Cargo.toml)
        let cargo_toml = working_dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(crate::error::EmpathicError::LspInitializationFailed { 
                reason: format!("Not a Rust project - Cargo.toml not found in: {}", working_dir.display()) 
            });
        }

        log::info!("🔍 Searching workspace symbols for query: '{}' in project: {}", 
            input.query, working_dir.display());

        // Get LSP manager and client
        let lsp_manager = config.lsp_manager()
            .ok_or_else(|| crate::error::EmpathicError::LspInitializationFailed {
                reason: "LSP manager not available".to_string(),
            })?;

        let client = lsp_manager.get_client(&working_dir).await?;

        // Create WorkspaceSymbolParams
        let params = WorkspaceSymbolParams {
            query: input.query.clone(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        // Call LSP server
        let response = client.workspace_symbols(params).await?;

        // Convert response to our format
        let symbols: Vec<WorkspaceSymbolInfo> = match response {
            Some(symbol_info_vec) => {
                symbol_info_vec.iter()
                    .map(WorkspaceSymbolInfo::from_symbol_information)
                    .collect()
            }
            None => Vec::new(),
        };

        let summary = WorkspaceSymbolsSummary::from_symbols(&symbols, &input.query, symbols.len());

        let output = WorkspaceSymbolsOutput {
            query: input.query.clone(),
            project: input.project.clone(),
            symbols,
            summary,
        };

        crate::tools::format_json_response(&output)
    }
}

