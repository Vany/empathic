//! 🔧 LSP Types - Error wrappers and empathic-specific LSP types

use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// 🎯 LSP operation result type
pub type LspResult<T> = Result<T, LspError>;

/// 🚨 LSP-specific error types for empathic
#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("LSP server not found in PATH: {server_name}")]
    ServerNotFound { server_name: String },

    #[error("Failed to spawn LSP server: {message}")]
    SpawnError { message: String },

    #[error("LSP server crashed for project: {project_path}")]
    ServerCrashed { project_path: PathBuf },

    #[error("JSON-RPC communication error: {message}")]
    JsonRpcError { message: String },

    #[error("LSP request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    #[error("No LSP server available for file: {file_path}")]
    NoServerAvailable { file_path: PathBuf },

    #[error("Project detection failed: {message}")]
    ProjectDetectionError { message: String },

    #[error("LSP server initialization failed: {message}")]
    InitializationError { message: String },

    #[error("Invalid LSP response: {message}")]
    InvalidResponse { message: String },

    #[error("Invalid LSP request: {message}")]
    InvalidRequest { message: String },

    #[error("IO error in LSP communication: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
}

/// 🏗️ LSP server process information
#[derive(Debug, Clone)]
pub struct LspProcess {
    pub project_path: PathBuf,
    pub server_name: String,
    pub process_id: u32,
    pub capabilities: Option<ServerCapabilities>,
    pub initialized: bool,
}

/// 📍 Position wrapper with file path context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePosition {
    pub file_path: PathBuf,
    pub position: Position,
}

impl FilePosition {
    pub fn new(file_path: PathBuf, line: u32, character: u32) -> Self {
        Self {
            file_path,
            position: Position::new(line, character),
        }
    }
}

/// 🎯 Diagnostic result with file context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub file_path: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
}

/// ⚡ Caching configuration for different LSP operations
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub diagnostics_ttl_secs: u64,
    pub completion_ttl_secs: u64,
    pub symbols_ttl_secs: u64,
    pub hover_ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            diagnostics_ttl_secs: 300,  // 5 minutes
            completion_ttl_secs: 30,    // 30 seconds
            symbols_ttl_secs: 600,      // 10 minutes
            hover_ttl_secs: 60,         // 1 minute
        }
    }
}

/// 🏥 Comprehensive health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// List of healthy project paths
    pub healthy_processes: Vec<PathBuf>,
    /// List of unhealthy project paths
    pub unhealthy_processes: Vec<PathBuf>,
    /// Resource monitoring statistics
    pub resource_stats: crate::lsp::resource::ResourceStats,
    /// Performance summary string
    pub performance_summary: String,
    /// Number of processes over resource limits
    pub over_limit_count: usize,
    /// Timestamp of health check
    pub timestamp: Instant,
}

impl HealthCheckResult {
    /// Get total number of processes
    pub fn total_processes(&self) -> usize {
        self.healthy_processes.len() + self.unhealthy_processes.len()
    }
    
    /// Get health percentage (0-100)
    pub fn health_percentage(&self) -> f64 {
        let total = self.total_processes();
        if total == 0 {
            100.0
        } else {
            (self.healthy_processes.len() as f64 / total as f64) * 100.0
        }
    }
    
    /// Check if system is healthy (all processes running and within limits)
    pub fn is_healthy(&self) -> bool {
        self.unhealthy_processes.is_empty() && self.over_limit_count == 0
    }
    
    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "🏥 Health: {:.1}% ({}/{} healthy), {} over limits, {} restarts",
            self.health_percentage(),
            self.healthy_processes.len(),
            self.total_processes(),
            self.over_limit_count,
            self.resource_stats.total_restarts
        )
    }
}
