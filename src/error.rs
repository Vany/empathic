//! 🚨 Unified Error Handling - Consolidated error types for empathic MCP server
//!
//! This module provides a unified error system that consolidates various error types
//! while maintaining compatibility with existing anyhow usage patterns.

use std::path::PathBuf;
use thiserror::Error;

/// 🎯 Primary result type for empathic operations
pub type EmpathicResult<T> = Result<T, EmpathicError>;

/// 🚨 Unified error type for empathic MCP server operations
#[derive(Debug, Error)]
pub enum EmpathicError {
    // === 📁 File System Errors ===
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("File access denied: {path}")]
    FileAccessDenied { path: PathBuf },

    #[error("Invalid file path: {path}")]
    InvalidPath { path: PathBuf },

    #[error("Directory creation failed: {path} - {reason}")]
    DirectoryCreationFailed { path: PathBuf, reason: String },

    #[error("File operation failed: {operation} on {path} - {reason}")]
    FileOperationFailed {
        operation: String,
        path: PathBuf,
        reason: String,
    },

    // === ⚙️ Configuration Errors ===
    #[error("Configuration validation failed: {message}")]
    ConfigValidation { message: String },

    #[error("Missing required environment variable: {name}")]
    MissingEnvVar { name: String },

    #[error("Invalid configuration value: {field} = {value}")]
    InvalidConfigValue { field: String, value: String },

    #[error("Root directory not found: {path}")]
    RootDirectoryNotFound { path: PathBuf },

    // === 🧠 LSP Server Errors ===
    #[error("LSP server not found: {server_name}")]
    LspServerNotFound { server_name: String },

    #[error("LSP server spawn failed: {message}")]
    LspSpawnFailed { message: String },

    #[error("LSP server crashed: {project_path}")]
    LspServerCrashed { project_path: PathBuf },

    #[error("LSP JSON-RPC error: {message}")]
    LspJsonRpcError { message: String },

    #[error("LSP request timeout: {timeout_secs}s")]
    LspTimeout { timeout_secs: u64 },

    #[error("No LSP server available for: {file_path}")]
    LspNoServerAvailable { file_path: PathBuf },

    #[error("LSP initialization failed: {reason}")]
    LspInitializationFailed { reason: String },

    #[error("LSP workspace sync failed: {reason}")]
    LspWorkspaceSyncFailed { reason: String },

    // === 🔧 Tool Execution Errors ===
    #[error("Tool execution failed: {tool_name} - {message}")]
    ToolExecutionFailed { tool_name: String, message: String },

    #[error("Command execution failed: {command} - exit code {exit_code}")]
    CommandFailed { command: String, exit_code: i32 },

    #[error("Missing required parameter: {parameter}")]
    MissingRequiredParameter { parameter: String },

    #[error("Invalid argument: {arg} - {reason}")]
    InvalidArgument { arg: String, reason: String },

    #[error("Task join error: {message}")]
    TaskJoinError { message: String },

    #[error("Command not found: {command}")]
    CommandNotFound { command: String },

    #[error("Tool timeout: {tool_name} exceeded {timeout_secs}s")]
    ToolTimeout { tool_name: String, timeout_secs: u64 },

    // === 📡 MCP Protocol Errors ===
    #[error("Invalid MCP request: {message}")]
    InvalidMcpRequest { message: String },

    #[error("MCP parameter missing: {parameter}")]
    McpParameterMissing { parameter: String },

    #[error("MCP parameter invalid: {parameter} = {value}")]
    McpParameterInvalid { parameter: String, value: String },

    #[error("JSON-RPC protocol error: {message}")]
    JsonRpcProtocol { message: String },

    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    // === 🔍 Search & Replace Errors ===
    #[error("Search pattern not found: {pattern} in {file}")]
    SearchPatternNotFound { pattern: String, file: PathBuf },

    #[error("Invalid regex pattern: {pattern} - {reason}")]
    InvalidRegexPattern { pattern: String, reason: String },

    #[error("Replace operation failed: {operation} - {reason}")]
    ReplaceOperationFailed { operation: String, reason: String },

    #[error("String not found in file: '{search_str}' does not appear in {path}")]
    StrReplaceNotFound { path: String, search_str: String },

    #[error("String appears {count} times in file: '{search_str}' must appear exactly once in {path}")]
    StrReplaceMultipleMatches {
        path: String,
        search_str: String,
        count: usize,
    },

    // === 🧬 Unicode & Text Processing Errors ===
    #[error("Unicode processing error: {message}")]
    UnicodeError { message: String },

    #[error("Text encoding error: expected {expected}, found {found}")]
    TextEncodingError { expected: String, found: String },

    #[error("Line range invalid: {start}-{end} in file with {total_lines} lines")]
    InvalidLineRange {
        start: usize,
        end: usize,
        total_lines: usize,
    },

    // === 🔗 External Dependencies ===
    #[error("External command error: {source}")]
    ExternalCommand {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON processing error: {source}")]
    JsonProcessing {
        #[from]
        source: serde_json::Error,
    },

    #[error("Path processing error: {message}")]
    PathProcessing { message: String },

    // === 🔄 Generic & Compatibility ===
    #[error("Operation failed: {message}")]
    Generic { message: String },

    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },

    #[error("Operation not supported: {operation} on {platform}")]
    NotSupported { operation: String, platform: String },

    /// Bridge for anyhow errors - provides compatibility
    #[error("Legacy error: {source}")]
    Anyhow {
        #[from]
        source: anyhow::Error,
    },
}

// === 🔄 Error Conversion Implementations ===

impl From<std::env::VarError> for EmpathicError {
    fn from(err: std::env::VarError) -> Self {
        match err {
            std::env::VarError::NotPresent => EmpathicError::Generic {
                message: "Environment variable not present".to_string(),
            },
            std::env::VarError::NotUnicode(_) => EmpathicError::UnicodeError {
                message: "Environment variable contains invalid Unicode".to_string(),
            },
        }
    }
}

impl From<regex::Error> for EmpathicError {
    fn from(err: regex::Error) -> Self {
        EmpathicError::InvalidRegexPattern {
            pattern: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<walkdir::Error> for EmpathicError {
    fn from(err: walkdir::Error) -> Self {
        let path = err.path().unwrap_or_else(|| std::path::Path::new("unknown"));
        EmpathicError::FileOperationFailed {
            operation: "directory traversal".to_string(),
            path: path.to_path_buf(),
            reason: err.to_string(),
        }
    }
}

impl From<tokio::task::JoinError> for EmpathicError {
    fn from(err: tokio::task::JoinError) -> Self {
        EmpathicError::TaskJoinError {
            message: err.to_string(),
        }
    }
}

impl From<ignore::Error> for EmpathicError {
    fn from(err: ignore::Error) -> Self {
        // ignore::Error doesn't always have a path, so use a generic approach
        EmpathicError::FileOperationFailed {
            operation: "file traversal".to_string(),
            path: std::path::PathBuf::from("unknown"),
            reason: err.to_string(),
        }
    }
}

impl From<crate::lsp::types::LspError> for EmpathicError {
    fn from(err: crate::lsp::types::LspError) -> Self {
        use crate::lsp::types::LspError;
        match err {
            LspError::ServerNotFound { server_name } => {
                EmpathicError::LspServerNotFound { server_name }
            }
            LspError::SpawnError { message } => EmpathicError::LspSpawnFailed { message },
            LspError::ServerCrashed { project_path } => {
                EmpathicError::LspServerCrashed { project_path }
            }
            LspError::JsonRpcError { message } => EmpathicError::LspJsonRpcError { message },
            LspError::Timeout { timeout_secs } => EmpathicError::LspTimeout { timeout_secs },
            LspError::NoServerAvailable { file_path } => {
                EmpathicError::LspNoServerAvailable { file_path }
            }
            LspError::ProjectDetectionError { message } => EmpathicError::LspInitializationFailed {
                reason: format!("Project detection failed: {}", message),
            },
            LspError::InitializationError { message } => EmpathicError::LspInitializationFailed {
                reason: format!("LSP server initialization failed: {}", message),
            },
            LspError::InvalidResponse { message } => EmpathicError::LspJsonRpcError { message },
            LspError::InvalidRequest { message } => EmpathicError::LspJsonRpcError { message },
            LspError::IoError { source } => EmpathicError::ExternalCommand { source },
            LspError::SerializationError { source } => EmpathicError::JsonProcessing { source },
        }
    }
}

// === 🛠️ Error Helper Functions ===

impl EmpathicError {
    /// Create a file not found error with context
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a tool execution error with context
    pub fn tool_failed(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ToolExecutionFailed {
            tool_name: tool_name.into(),
            message: message.into(),
        }
    }

    /// Create a generic error from any displayable type
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create a configuration validation error
    pub fn config_validation(message: impl Into<String>) -> Self {
        Self::ConfigValidation {
            message: message.into(),
        }
    }

    /// Create an MCP parameter error
    pub fn mcp_parameter(parameter: impl Into<String>, value: impl Into<String>) -> Self {
        Self::McpParameterInvalid {
            parameter: parameter.into(),
            value: value.into(),
        }
    }

    /// Create an LSP server error
    pub fn lsp_server_failed(server_name: impl Into<String>) -> Self {
        Self::LspServerNotFound {
            server_name: server_name.into(),
        }
    }

    /// Create an LSP timeout error
    pub fn lsp_timeout(timeout_secs: u64) -> Self {
        Self::LspTimeout { timeout_secs }
    }

    /// Create an LSP initialization error
    pub fn lsp_init_failed(reason: impl Into<String>) -> Self {
        Self::LspInitializationFailed {
            reason: reason.into(),
        }
    }

    /// Check if this error indicates a missing file
    pub fn is_file_not_found(&self) -> bool {
        matches!(self, EmpathicError::FileNotFound { .. })
    }

    /// Check if this error is a configuration issue
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            EmpathicError::ConfigValidation { .. }
                | EmpathicError::MissingEnvVar { .. }
                | EmpathicError::InvalidConfigValue { .. }
                | EmpathicError::RootDirectoryNotFound { .. }
        )
    }

    /// Check if this error is recoverable (user can fix it)
    pub fn is_recoverable(&self) -> bool {
        match self {
            EmpathicError::FileNotFound { .. }
            | EmpathicError::FileAccessDenied { .. }
            | EmpathicError::ConfigValidation { .. }
            | EmpathicError::MissingEnvVar { .. }
            | EmpathicError::CommandNotFound { .. }
            | EmpathicError::InvalidMcpRequest { .. }
            | EmpathicError::McpParameterMissing { .. }
            | EmpathicError::McpParameterInvalid { .. } => true,
            EmpathicError::NotImplemented { .. } | EmpathicError::NotSupported { .. } => false,
            _ => true, // Most errors are recoverable by fixing user input
        }
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            EmpathicError::FileNotFound { .. }
            | EmpathicError::FileAccessDenied { .. }
            | EmpathicError::InvalidPath { .. }
            | EmpathicError::DirectoryCreationFailed { .. }
            | EmpathicError::FileOperationFailed { .. } => "filesystem",

            EmpathicError::ConfigValidation { .. }
            | EmpathicError::MissingEnvVar { .. }
            | EmpathicError::InvalidConfigValue { .. }
            | EmpathicError::RootDirectoryNotFound { .. } => "configuration",

            EmpathicError::ToolExecutionFailed { .. }
            | EmpathicError::CommandFailed { .. }
            | EmpathicError::CommandNotFound { .. }
            | EmpathicError::ToolTimeout { .. }
            | EmpathicError::InvalidArgument { .. } => "execution",

            EmpathicError::LspServerNotFound { .. }
            | EmpathicError::LspSpawnFailed { .. }
            | EmpathicError::LspServerCrashed { .. }
            | EmpathicError::LspJsonRpcError { .. }
            | EmpathicError::LspTimeout { .. }
            | EmpathicError::LspNoServerAvailable { .. }
            | EmpathicError::LspInitializationFailed { .. }
            | EmpathicError::LspWorkspaceSyncFailed { .. } => "lsp",

            EmpathicError::InvalidMcpRequest { .. }
            | EmpathicError::McpParameterMissing { .. }
            | EmpathicError::McpParameterInvalid { .. }
            | EmpathicError::JsonRpcProtocol { .. }
            | EmpathicError::ToolNotFound { .. } => "protocol",

            EmpathicError::SearchPatternNotFound { .. }
            | EmpathicError::InvalidRegexPattern { .. }
            | EmpathicError::ReplaceOperationFailed { .. }
            | EmpathicError::StrReplaceNotFound { .. }
            | EmpathicError::StrReplaceMultipleMatches { .. } => "search_replace",

            EmpathicError::UnicodeError { .. }
            | EmpathicError::TextEncodingError { .. }
            | EmpathicError::InvalidLineRange { .. } => "text_processing",

            EmpathicError::ExternalCommand { .. }
            | EmpathicError::JsonProcessing { .. }
            | EmpathicError::PathProcessing { .. } => "external",

            EmpathicError::Generic { .. }
            | EmpathicError::NotImplemented { .. }
            | EmpathicError::NotSupported { .. }
            | EmpathicError::Anyhow { .. }
            | EmpathicError::MissingRequiredParameter { .. }
            | EmpathicError::TaskJoinError { .. } => "general",
        }
    }
}

// === 🔄 Compatibility Functions ===

/// Convert any anyhow::Result to EmpathicResult for gradual migration
pub fn from_anyhow<T>(result: anyhow::Result<T>) -> EmpathicResult<T> {
    result.map_err(EmpathicError::from)
}

/// Convert EmpathicResult to anyhow::Result for backward compatibility
pub fn to_anyhow<T>(result: EmpathicResult<T>) -> anyhow::Result<T> {
    result.map_err(|e| anyhow::anyhow!(e))
}

/// Create a typed error context helper macro
#[macro_export]
macro_rules! empathic_context {
    ($result:expr, $error_type:ident { $($field:ident: $value:expr),* }) => {
        $result.map_err(|_| $crate::error::EmpathicError::$error_type {
            $($field: $value.into()),*
        })
    };
}

/// Helper for creating file operation errors
#[macro_export]
macro_rules! file_error {
    ($op:expr, $path:expr, $err:expr) => {
        $crate::error::EmpathicError::FileOperationFailed {
            operation: $op.into(),
            path: $path.into(),
            reason: $err.to_string(),
        }
    };
}

/// Helper for creating tool execution errors  
#[macro_export]
macro_rules! tool_error {
    ($tool:expr, $msg:expr) => {
        $crate::error::EmpathicError::ToolExecutionFailed {
            tool_name: $tool.into(),
            message: $msg.into(),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        assert_eq!(
            EmpathicError::FileNotFound {
                path: "/test".into()
            }
            .category(),
            "filesystem"
        );

        assert_eq!(
            EmpathicError::ConfigValidation {
                message: "test".into()
            }
            .category(),
            "configuration"
        );

        assert_eq!(
            EmpathicError::ToolExecutionFailed {
                tool_name: "test".into(),
                message: "failed".into()
            }
            .category(),
            "execution"
        );
    }

    #[test]
    fn test_error_recovery() {
        assert!(EmpathicError::FileNotFound {
            path: "/test".into()
        }
        .is_recoverable());

        assert!(!EmpathicError::NotImplemented {
            feature: "test".into()
        }
        .is_recoverable());
    }

    #[test]
    fn test_error_helpers() {
        let err = EmpathicError::file_not_found("/test/file.txt");
        assert!(err.is_file_not_found());

        let err = EmpathicError::tool_failed("git", "command failed");
        assert_eq!(err.category(), "execution");
    }

    #[test]
    fn test_anyhow_conversion() {
        let anyhow_err: anyhow::Result<()> = Err(anyhow::anyhow!("test error"));
        let empathic_result = from_anyhow(anyhow_err);
        assert!(empathic_result.is_err());

        let empathic_err: EmpathicResult<()> = Err(EmpathicError::generic("test"));
        let anyhow_result = to_anyhow(empathic_err);
        assert!(anyhow_result.is_err());
    }
}