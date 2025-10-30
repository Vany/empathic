//! 📄 LSP Document Symbols Tool - Get file structure outline
//!
//! Provides a hierarchical view of symbols in a Rust file (functions, structs, enums, etc.)

use super::base::{BaseLspTool, LspInput, LspOutput};
use crate::error::EmpathicResult;
use async_trait::async_trait;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 📄 LSP Document Symbols Tool implementation
pub struct LspDocumentSymbolsTool;

/// Input parameters for lsp_document_symbols tool
#[derive(Debug, Deserialize)]
pub struct DocumentSymbolsInput {
    file_path: String,
    project: String,
}

impl LspInput for DocumentSymbolsInput {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn project(&self) -> &str {
        &self.project
    }
}

/// Output format for document symbols
#[derive(Debug, Serialize)]
pub struct DocumentSymbolsOutput {
    file_path: String,
    project: String,
    symbols: Vec<SymbolInfo>,
    summary: SymbolsSummary,
}

impl LspOutput for DocumentSymbolsOutput {
    fn set_file_path(&mut self, path: String) {
        self.file_path = path;
    }

    fn set_project(&mut self, project: String) {
        self.project = project;
    }
}

/// Simplified symbol information for MCP output
#[derive(Debug, Serialize)]
struct SymbolInfo {
    name: String,
    kind: String,
    detail: Option<String>,
    line: u32,
    character: u32,
    end_line: u32,
    end_character: u32,
    children: Vec<SymbolInfo>,
}

/// Summary statistics for document symbols
#[derive(Debug, Serialize)]
struct SymbolsSummary {
    total_symbols: usize,
    functions: usize,
    structs: usize,
    enums: usize,
    traits: usize,
    impl_blocks: usize,
    constants: usize,
    modules: usize,
}

impl SymbolInfo {
    /// Convert from LSP DocumentSymbol to our format
    fn from_document_symbol(symbol: &DocumentSymbol) -> Self {
        let children = symbol.children.as_ref()
            .map(|children| children.iter().map(Self::from_document_symbol).collect())
            .unwrap_or_default();

        Self {
            name: symbol.name.clone(),
            kind: format!("{:?}", symbol.kind),
            detail: symbol.detail.clone(),
            line: symbol.range.start.line,
            character: symbol.range.start.character,
            end_line: symbol.range.end.line,
            end_character: symbol.range.end.character,
            children,
        }
    }

    fn from_symbol_information(symbol: &SymbolInformation) -> Self {
        Self {
            name: symbol.name.clone(),
            kind: format!("{:?}", symbol.kind),
            detail: None,
            line: symbol.location.range.start.line,
            character: symbol.location.range.start.character,
            end_line: symbol.location.range.end.line,
            end_character: symbol.location.range.end.character,
            children: Vec::new(),
        }
    }
}

impl SymbolsSummary {
    fn from_symbols(symbols: &[SymbolInfo]) -> Self {
        fn count_symbols(symbols: &[SymbolInfo], summary: &mut (usize, usize, usize, usize, usize, usize, usize)) {
            for symbol in symbols {
                match symbol.kind.as_str() {
                    "Function" => summary.0 += 1,
                    "Struct" => summary.1 += 1,
                    "Enum" => summary.2 += 1,
                    "Interface" => summary.3 += 1, // Traits are represented as Interface in LSP
                    "Class" => summary.4 += 1,     // Impl blocks might be represented as Class
                    "Constant" => summary.5 += 1,
                    "Module" => summary.6 += 1,
                    _ => {}
                }
                count_symbols(&symbol.children, summary);
            }
        }

        let mut counts = (0, 0, 0, 0, 0, 0, 0);
        count_symbols(symbols, &mut counts);

        let (functions, structs, enums, traits, impl_blocks, constants, modules) = counts;

        Self {
            total_symbols: symbols.len(),
            functions,
            structs,
            enums,
            traits,
            impl_blocks,
            constants,
            modules,
        }
    }
}

#[async_trait]
impl BaseLspTool for LspDocumentSymbolsTool {
    type Input = DocumentSymbolsInput;
    type Output = DocumentSymbolsOutput;

    fn name() -> &'static str where Self: Sized {
        "lsp_document_symbols"
    }

    fn description() -> &'static str where Self: Sized {
        "📄 Get document structure outline (functions, structs, enums) for Rust files using rust-analyzer"
    }

    async fn execute_lsp(
        &self,
        _input: Self::Input,
        file_path: PathBuf,
        config: &crate::config::Config,
    ) -> EmpathicResult<Self::Output> {
        log::info!("📄 Getting document symbols for: {}", file_path.display());

        // Get LSP manager and client
        let lsp_manager = config.lsp_manager()
            .ok_or_else(|| crate::error::EmpathicError::LspInitializationFailed {
                reason: "LSP manager not available".to_string(),
            })?;

        let project_root = config.project_path(Some(&_input.project));
        let client = lsp_manager.get_client(&project_root).await?;

        // Convert file path to URI
        let uri = url::Url::from_file_path(&file_path)
            .map_err(|_| crate::error::EmpathicError::InvalidPath {
                path: file_path.clone(),
            })?;

        // Ensure document is opened in LSP
        lsp_manager.ensure_document_open(&file_path).await?;

        // Create DocumentSymbolParams
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { 
                uri: uri.to_string().parse().unwrap()
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        // Call LSP server
        let response = client.document_symbols(params).await?;

        // Convert response to our format
        let symbols: Vec<SymbolInfo> = match response {
            Some(DocumentSymbolResponse::Flat(symbol_info)) => {
                symbol_info.iter()
                    .map(SymbolInfo::from_symbol_information)
                    .collect()
            }
            Some(DocumentSymbolResponse::Nested(document_symbols)) => {
                document_symbols.iter()
                    .map(SymbolInfo::from_document_symbol)
                    .collect()
            }
            None => Vec::new(),
        };

        let summary = SymbolsSummary::from_symbols(&symbols);

        Ok(DocumentSymbolsOutput {
            file_path: String::new(), // Will be set by BaseLspTool
            project: String::new(),    // Will be set by BaseLspTool
            symbols,
            summary,
        })
    }
}

