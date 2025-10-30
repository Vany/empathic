//! 🩺 LSP Diagnostics Tool - Get errors, warnings, and hints for files
//!
//! Retrieves semantic diagnostics from rust-analyzer for Rust files

use super::base::{BaseLspTool, LspInput, LspOutput, get_lsp_manager};
use crate::config::Config;
use async_trait::async_trait;
use lsp_types::DiagnosticSeverity;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{EmpathicResult, EmpathicError};

/// 🩺 LSP Diagnostics Tool implementation
pub struct LspDiagnosticsTool;

/// Input parameters for lsp_diagnostics tool
#[derive(Debug, Deserialize)]
pub struct DiagnosticsInput {
    file_path: String,
    project: String,
}

impl LspInput for DiagnosticsInput {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn project(&self) -> &str {
        &self.project
    }
}

/// Output format for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsOutput {
    file_path: String,
    project: String,
    diagnostics: Vec<DiagnosticInfo>,
    summary: DiagnosticSummary,
}

impl LspOutput for DiagnosticsOutput {
    fn set_file_path(&mut self, path: String) {
        self.file_path = path;
    }

    fn set_project(&mut self, project: String) {
        self.project = project;
    }
}

/// Simplified diagnostic information for MCP output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiagnosticInfo {
    message: String,
    severity: String,
    line: u32,
    character: u32,
    end_line: Option<u32>,
    end_character: Option<u32>,
    source: Option<String>,
    code: Option<String>,
}

impl DiagnosticInfo {
    /// Convert from LSP Diagnostic to our format
    fn from_lsp_diagnostic(diagnostic: &lsp_types::Diagnostic) -> Self {
        use lsp_types::NumberOrString;
        
        Self {
            message: diagnostic.message.clone(),
            severity: Self::severity_to_string(diagnostic.severity),
            line: diagnostic.range.start.line,
            character: diagnostic.range.start.character,
            end_line: Some(diagnostic.range.end.line),
            end_character: Some(diagnostic.range.end.character),
            source: diagnostic.source.clone(),
            code: diagnostic.code.as_ref().map(|c| match c {
                NumberOrString::Number(n) => n.to_string(),
                NumberOrString::String(s) => s.clone(),
            }),
        }
    }
    
    /// Convert LSP severity to string
    fn severity_to_string(severity: Option<DiagnosticSeverity>) -> String {
        match severity {
            Some(DiagnosticSeverity::ERROR) => "error".to_string(),
            Some(DiagnosticSeverity::WARNING) => "warning".to_string(),
            Some(DiagnosticSeverity::INFORMATION) => "information".to_string(),
            Some(DiagnosticSeverity::HINT) => "hint".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

/// Diagnostic summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiagnosticSummary {
    total: usize,
    errors: usize,
    warnings: usize,
    information: usize,
    hints: usize,
}

impl DiagnosticSummary {
    fn from_diagnostics(diagnostics: &[DiagnosticInfo]) -> Self {
        let mut errors = 0;
        let mut warnings = 0;
        let mut information = 0;
        let mut hints = 0;

        for diagnostic in diagnostics {
            match diagnostic.severity.as_str() {
                "error" => errors += 1,
                "warning" => warnings += 1,
                "information" => information += 1,
                "hint" => hints += 1,
                _ => {} // Unknown severity
            }
        }

        Self {
            total: diagnostics.len(),
            errors,
            warnings,
            information,
            hints,
        }
    }
}

#[async_trait]
impl BaseLspTool for LspDiagnosticsTool {
    type Input = DiagnosticsInput;
    type Output = DiagnosticsOutput;

    fn name() -> &'static str {
        "lsp_diagnostics"
    }

    fn description() -> &'static str {
        "🩺 Get semantic diagnostics (errors, warnings, hints) for Rust files using rust-analyzer"
    }

    async fn execute_lsp(
        &self,
        _input: Self::Input,
        file_path: PathBuf,
        config: &Config,
    ) -> EmpathicResult<Self::Output> {
        use lsp_types::*;
        use std::time::Duration;

        // 🧠 Get LSP manager (shared instance that persists across calls)
        let lsp_manager = get_lsp_manager(config)?;

        log::info!("🩺 Getting diagnostics for: {}", file_path.display());

        // 🚀 Ensure document is open/synced with LSP server
        lsp_manager.ensure_document_open(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_diagnostics",
                format!("Failed to sync document {}: {}", file_path.display(), e)
            ))?;

        // 📡 Get LSP client
        let client = lsp_manager.get_client(&file_path).await
            .map_err(|e| EmpathicError::tool_failed(
                "lsp_diagnostics",
                format!("Failed to get LSP client for {}: {}", file_path.display(), e)
            ))?;

        // 🎯 Strategy: Try to get diagnostics from publishDiagnostics notification
        // LSP servers send diagnostics as notifications after analyzing a file
        // Note: Error-free files might not send diagnostics immediately
        
        // Subscribe to notifications before waiting
        let file_uri = url::Url::from_file_path(&file_path)
            .map_err(|_| EmpathicError::InvalidPath { path: file_path.clone() })?;
        
        // Wait for publishDiagnostics notification (with short timeout for error-free files)
        let notification_result = client.wait_for_notification(
            "textDocument/publishDiagnostics",
            Duration::from_secs(3) // Short timeout - don't block forever on clean files
        ).await;

        let diagnostics = match notification_result {
            Ok(notification) => {
                // Parse publishDiagnostics params
                if let Some(params) = notification.params {
                    let publish_params: PublishDiagnosticsParams = serde_json::from_value(params)
                        .map_err(|e| EmpathicError::tool_failed(
                            "lsp_diagnostics",
                            format!("Failed to parse diagnostics: {}", e)
                        ))?;
                    
                    // Verify this is for our file
                    if publish_params.uri.to_string() == file_uri.to_string() {
                        log::debug!("📊 Received {} diagnostics from rust-analyzer", 
                            publish_params.diagnostics.len());
                        
                        // Convert LSP diagnostics to our format
                        publish_params.diagnostics.iter()
                            .map(DiagnosticInfo::from_lsp_diagnostic)
                            .collect()
                    } else {
                        // Diagnostics for different file, treat as no diagnostics
                        log::debug!("📊 Received diagnostics for different file, treating as clean");
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Err(_) => {
                // Timeout or error - likely a clean file with no diagnostics
                log::debug!("📊 No diagnostics received (likely clean file)");
                Vec::new()
            }
        };

        let summary = DiagnosticSummary::from_diagnostics(&diagnostics);

        Ok(DiagnosticsOutput {
            file_path: String::new(), // Will be set by base trait
            project: String::new(),   // Will be set by base trait
            diagnostics,
            summary,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_summary() {
        let diagnostics = vec![
            DiagnosticInfo {
                message: "Error".to_string(),
                severity: "error".to_string(),
                line: 1,
                character: 0,
                end_line: None,
                end_character: None,
                source: None,
                code: None,
            },
            DiagnosticInfo {
                message: "Warning".to_string(),
                severity: "warning".to_string(),
                line: 2,
                character: 0,
                end_line: None,
                end_character: None,
                source: None,
                code: None,
            },
        ];

        let summary = DiagnosticSummary::from_diagnostics(&diagnostics);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.information, 0);
        assert_eq!(summary.hints, 0);
    }

    #[test]
    fn test_severity_conversion() {
        use lsp_types::DiagnosticSeverity;
        
        assert_eq!(DiagnosticInfo::severity_to_string(Some(DiagnosticSeverity::ERROR)), "error");
        assert_eq!(DiagnosticInfo::severity_to_string(Some(DiagnosticSeverity::WARNING)), "warning");
        assert_eq!(DiagnosticInfo::severity_to_string(Some(DiagnosticSeverity::INFORMATION)), "information");
        assert_eq!(DiagnosticInfo::severity_to_string(Some(DiagnosticSeverity::HINT)), "hint");
        assert_eq!(DiagnosticInfo::severity_to_string(None), "unknown");
    }
}
