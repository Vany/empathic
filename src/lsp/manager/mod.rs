//! 🚀 LSP Manager - Modular LSP server management
//!
//! This module provides unified LSP server management through specialized submodules:
//! - `core`: Main orchestration and performance optimization
//! - `lifecycle`: Process spawning, shutdown, and health monitoring  
//! - `tracker`: Document synchronization and state tracking

pub mod core;
pub mod lifecycle;
pub mod tracker;

use self::core::LspManagerCore;
use crate::lsp::resource::ResourceConfig;
use crate::lsp::types::{LspProcess, LspResult, HealthCheckResult};
use crate::lsp::ProjectDetector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 🚀 High-level LSP Manager facade
/// 
/// Provides a clean interface that delegates to specialized manager modules.
/// This design reduces cognitive load while maintaining all functionality.
#[derive(Debug)]
pub struct LspManager {
    /// Core manager handling orchestration and performance
    core: LspManagerCore,
}

impl LspManager {
    /// Create a new high-performance LSP manager with optimization features
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            core: LspManagerCore::new(root_dir),
        }
    }

    /// Create a new LSP manager with custom resource configuration
    pub fn with_resource_config(root_dir: PathBuf, resource_config: ResourceConfig) -> Self {
        Self {
            core: LspManagerCore::with_resource_config(root_dir, resource_config),
        }
    }

    // === 🎯 Core Server Management ===

    /// Get or spawn an LSP server for the given file
    pub async fn get_or_spawn_server(&self, file_path: &Path) -> LspResult<LspProcess> {
        self.core.get_or_spawn_server(file_path).await
    }

    /// Get LSP client for the given file path
    pub async fn get_client(&self, file_path: &Path) -> LspResult<crate::lsp::client::LspClient> {
        self.core.get_client(file_path).await
    }

    /// Gracefully shutdown a specific LSP server
    pub async fn shutdown_server(&self, project_path: &Path) -> LspResult<()> {
        self.core.shutdown_server(project_path).await
    }

    /// Shutdown all LSP servers
    pub async fn shutdown_all(&self) -> LspResult<()> {
        self.core.shutdown_all().await
    }

    /// Get status of all running LSP servers
    pub async fn get_server_status(&self) -> Vec<LspProcess> {
        self.core.get_server_status().await
    }

    // === 📂 Document Management ===

    /// Ensure a document is open in the LSP server (sends didOpen if needed)
    pub async fn ensure_document_open(&self, file_path: &Path) -> LspResult<()> {
        self.core.ensure_document_open(file_path).await
    }

    /// Open a document in the LSP server (textDocument/didOpen)
    pub async fn open_document(&self, file_path: &Path) -> LspResult<()> {
        // For now, delegate to core - full document operations integration pending
        log::debug!("📂 Opening document: {}", file_path.display());
        
        // Ensure we have a running server first
        let _server = self.get_or_spawn_server(file_path).await?;
        
        // TODO: Integrate with tracker::DocumentOperations properly
        // This is a simplified implementation for now
        Ok(())
    }

    /// Update document content in the LSP server (textDocument/didChange)
    pub async fn update_document(&self, file_path: &Path, new_content: &str) -> LspResult<()> {
        log::debug!("📝 Updating document: {} ({} chars)", file_path.display(), new_content.len());
        
        // Ensure we have a running server first
        let _server = self.get_or_spawn_server(file_path).await?;
        
        // TODO: Integrate with tracker::DocumentOperations properly
        Ok(())
    }

    /// Close a document in the LSP server (textDocument/didClose)
    pub async fn close_document(&self, file_path: &Path) -> LspResult<()> {
        log::debug!("📄 Closing document: {}", file_path.display());
        
        // TODO: Integrate with tracker::DocumentOperations properly
        Ok(())
    }

    // === 🏥 Health & Monitoring ===

    /// Health check for LSP servers
    pub async fn health_check(&self) -> LspResult<Vec<(PathBuf, bool)>> {
        self.core.health_check().await
    }

    /// Perform comprehensive health check with resource monitoring
    pub async fn comprehensive_health_check(&self) -> LspResult<HealthCheckResult> {
        self.core.comprehensive_health_check().await
    }

    // === 📊 Performance & Metrics ===

    /// Get performance metrics summary
    pub fn performance_summary(&self) -> String {
        self.core.performance_summary()
    }

    /// Run performance benchmark for LSP operation
    pub async fn benchmark_operation<F, T>(&self, method: &str, operation: F) -> LspResult<T>
    where
        F: std::future::Future<Output = LspResult<T>>,
    {
        self.core.benchmark_operation(method, operation).await
    }

    /// Get document statistics
    pub async fn get_document_stats(&self) -> HashMap<PathBuf, usize> {
        self.core.get_document_stats().await
    }

    // === 📊 Resource Management ===

    /// Start resource monitoring for all LSP processes
    pub async fn start_resource_monitoring(&self) -> LspResult<()> {
        self.core.start_resource_monitoring().await
    }

    /// Stop resource monitoring
    pub async fn stop_resource_monitoring(&self) {
        self.core.stop_resource_monitoring().await;
    }

    /// Get resource monitoring statistics
    pub async fn get_resource_stats(&self) -> crate::lsp::resource::ResourceStats {
        self.core.get_resource_stats().await
    }

    /// Get resource monitoring summary
    pub async fn get_resource_summary(&self) -> String {
        self.core.get_resource_summary().await
    }

    /// Check if a specific process exceeds resource limits
    pub async fn check_process_limits(&self, pid: u32) -> Option<bool> {
        self.core.check_process_limits(pid).await
    }

    /// Gracefully shutdown all LSP servers (alias for compatibility)
    pub async fn graceful_shutdown_all(&self) -> LspResult<()> {
        self.shutdown_all().await
    }

    // === 🗄️ Cache Management ===

    /// Get access to the LSP response cache
    pub fn cache(&self) -> &crate::lsp::cache::LspCache {
        self.core.cache()
    }

    /// Invalidate cache for a specific file
    pub async fn invalidate_file_cache(&self, file_path: &Path) {
        self.core.invalidate_file_cache(file_path).await;
    }

    /// Invalidate cache for a specific project
    pub async fn invalidate_project_cache(&self, project_path: &Path) {
        self.core.invalidate_project_cache(project_path).await;
    }

    // === 🔍 Utilities ===

    /// Get the project detector
    pub fn detector(&self) -> &ProjectDetector {
        self.core.detector()
    }

    /// 🔄 Restart a crashed LSP server (convenience method)
    pub async fn restart_server(&self, project_path: &Path) -> LspResult<LspProcess> {
        log::warn!("🔄 Restarting crashed LSP server for project: {}", project_path.display());
        
        // Shutdown the old server
        self.shutdown_server(project_path).await?;
        
        // Create a dummy file path within the project to trigger spawn
        let dummy_file = project_path.join("src").join("lib.rs");
        
        // Spawn new server
        self.get_or_spawn_server(&dummy_file).await
    }

    // === ⏱️ Idle Monitoring (v2.1.0) ===

    /// Get idle monitoring statistics
    pub async fn get_idle_stats(&self) -> crate::lsp::idle_monitor::IdleMonitorStats {
        self.core.get_idle_stats().await
    }

    /// Manually trigger shutdown of idle servers
    ///
    /// This checks for idle servers and shuts them down immediately.
    /// For automatic monitoring, call this method periodically from your own task.
    ///
    /// Example:
    /// ```ignore
    /// let manager = Arc::new(LspManager::new(root));
    /// let manager_clone = Arc::clone(&manager);
    /// tokio::spawn(async move {
    ///     loop {
    ///         tokio::time::sleep(Duration::from_secs(60)).await;
    ///         let _ = manager_clone.shutdown_idle_servers().await;
    ///     }
    /// });
    /// ```
    pub async fn shutdown_idle_servers(&self) -> LspResult<Vec<PathBuf>> {
        self.core.shutdown_idle_servers().await
    }
}

impl Drop for LspManager {
    fn drop(&mut self) {
        log::info!("🗑️ LSP Manager facade dropped, core manager will clean up");
    }
}
