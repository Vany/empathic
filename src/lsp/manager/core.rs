//! 🏗️ LSP Manager Core
//!
//! Core orchestration and high-level management of LSP servers.
//! Coordinates between lifecycle management, document tracking, and performance optimization.

use super::{lifecycle::ProcessLifecycle, tracker::DocumentTracker};
use crate::lsp::cache::LspCache;
use crate::lsp::client::LspClient;
use crate::lsp::idle_monitor::IdleMonitor;
use crate::lsp::performance::{LspMetrics, ConnectionPool, PerformanceTester};
use crate::lsp::resource::ResourceConfig;
use crate::lsp::types::{LspError, LspProcess, LspResult, HealthCheckResult};
use crate::lsp::ProjectDetector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Child;
use tokio::sync::RwLock;

/// 🚀 High-performance LSP manager with optimization features
#[derive(Debug)]
pub struct LspManagerCore {
    /// Active LSP server processes per project path
    processes: RwLock<HashMap<PathBuf, LspProcess>>,
    /// LSP clients for communication per project path
    clients: RwLock<HashMap<PathBuf, LspClient>>,
    /// Child process handles for cleanup
    children: RwLock<HashMap<PathBuf, Child>>,
    /// Document tracking per project path with performance metrics
    documents: RwLock<HashMap<PathBuf, DocumentTracker>>,
    /// Project detector for routing files to projects
    detector: ProjectDetector,
    /// Response cache for performance optimization
    cache: LspCache,
    /// Performance metrics collection
    metrics: Arc<LspMetrics>,
    /// Connection pool for client reuse
    #[allow(dead_code)]
    connection_pool: ConnectionPool,
    /// Performance testing and benchmarking
    #[allow(dead_code)]
    performance_tester: PerformanceTester,
    /// Process lifecycle manager
    lifecycle: ProcessLifecycle,
    /// Idle timeout monitor (v2.1.0)
    idle_monitor: Arc<IdleMonitor>,
}

impl LspManagerCore {
    /// Create a new high-performance LSP manager with optimization features
    pub fn new(root_dir: PathBuf) -> Self {
        let metrics = Arc::new(LspMetrics::default());
        let connection_pool = ConnectionPool::new(10, metrics.clone()); // Max 10 connections
        let performance_tester = PerformanceTester::new(metrics.clone());
        let lifecycle = ProcessLifecycle::new();
        let idle_monitor = Arc::new(IdleMonitor::from_env());
        
        Self {
            processes: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
            children: RwLock::new(HashMap::new()),
            documents: RwLock::new(HashMap::new()),
            detector: ProjectDetector::new(root_dir),
            cache: LspCache::new(),
            metrics,
            connection_pool,
            performance_tester,
            lifecycle,
            idle_monitor,
        }
    }

    /// Create a new LSP manager with custom resource configuration
    pub fn with_resource_config(root_dir: PathBuf, resource_config: ResourceConfig) -> Self {
        let metrics = Arc::new(LspMetrics::default());
        let connection_pool = ConnectionPool::new(10, metrics.clone());
        let performance_tester = PerformanceTester::new(metrics.clone());
        let lifecycle = ProcessLifecycle::with_resource_config(resource_config);
        let idle_monitor = Arc::new(IdleMonitor::from_env());
        
        Self {
            processes: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
            children: RwLock::new(HashMap::new()),
            documents: RwLock::new(HashMap::new()),
            detector: ProjectDetector::new(root_dir),
            cache: LspCache::new(),
            metrics,
            connection_pool,
            performance_tester,
            lifecycle,
            idle_monitor,
        }
    }

    /// 📊 Get performance metrics summary
    pub fn performance_summary(&self) -> String {
        self.metrics.summary()
    }

    /// 🧪 Run performance benchmark for LSP operation
    pub async fn benchmark_operation<F, T>(&self, method: &str, operation: F) -> LspResult<T>
    where
        F: std::future::Future<Output = LspResult<T>>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        
        // Record metrics
        self.metrics.record_request(duration, result.is_ok());
        
        // Log performance info
        if duration.as_millis() > 200 {
            log::warn!("🐌 Slow LSP operation: {} took {}ms", method, duration.as_millis());
        } else {
            log::debug!("⚡ LSP operation: {} completed in {}ms", method, duration.as_millis());
        }
        
        result
    }

    /// 🔗 Get optimized connection with pooling
    #[allow(dead_code)]
    async fn get_optimized_connection(&self, project_path: &Path) -> LspResult<LspClient> {
        let path_str = project_path.to_string_lossy().to_string();
        
        // Try to get from connection pool first
        if let Some(client) = self.connection_pool.get_connection(&path_str).await {
            log::debug!("🔗 Reusing pooled connection for {}", path_str);
            return Ok((*client).clone());
        }
        
        // Create new connection if not in pool
        let client = self.get_or_spawn_server_internal(project_path).await?;
        
        // Store in connection pool for reuse
        let client_arc = Arc::new(client.clone());
        if let Err(e) = self.connection_pool.store_connection(path_str, client_arc).await {
            log::warn!("Failed to store connection in pool: {}", e);
        }
        
        Ok(client)
    }

    /// 🔍 Find project for file or return error
    async fn require_project(&self, file_path: &Path) -> LspResult<crate::lsp::project_detector::Project> {
        self.detector
            .find_project_for_file(file_path)?
            .ok_or_else(|| LspError::NoServerAvailable {
                file_path: file_path.to_path_buf(),
            })
    }

    /// 🎯 Get or spawn an LSP server for the given file (with performance optimization)
    pub async fn get_or_spawn_server(&self, file_path: &Path) -> LspResult<LspProcess> {
        let project = self.require_project(file_path).await?;
        
        // Use performance-optimized connection
        self.benchmark_operation("get_or_spawn_server", async {
            // Check if process already exists
            {
                let processes = self.processes.read().await;
                if let Some(process) = processes.get(&project.root_path) {
                    return Ok(process.clone());
                }
            }
            
            // Spawn new server
            let _client = self.get_or_spawn_server_internal(&project.root_path).await?;
            
            // Return the process info
            let processes = self.processes.read().await;
            processes.get(&project.root_path)
                .cloned()
                .ok_or_else(|| LspError::NoServerAvailable {
                    file_path: file_path.to_path_buf(),
                })
        }).await
    }

    /// Internal method for server spawning (without benchmarking to avoid recursion)
    /// Internal method for server spawning (without benchmarking to avoid recursion)
    async fn get_or_spawn_server_internal(&self, project_path: &Path) -> LspResult<LspClient> {
        // Check if client already exists
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(project_path) {
                // Mark as used before returning existing client
                self.mark_server_used(project_path).await;
                return Ok(client.clone());
            }
        }
        
        // Spawn new rust-analyzer process using lifecycle manager
        let (process, client, child) = self.lifecycle.spawn_rust_analyzer(project_path).await?;
        
        // Store everything
        {
            let mut processes = self.processes.write().await;
            let mut clients = self.clients.write().await;
            let mut children = self.children.write().await;
            let mut documents = self.documents.write().await;

            processes.insert(project_path.to_path_buf(), process);
            clients.insert(project_path.to_path_buf(), client.clone());
            children.insert(project_path.to_path_buf(), child);
            documents.insert(project_path.to_path_buf(), DocumentTracker::new(self.metrics.clone()));
        }
        
        // Mark newly spawned server as used
        self.mark_server_used(project_path).await;
        
        Ok(client)
    }

    /// 🎯 Get LSP client for the given file path
    pub async fn get_client(&self, file_path: &Path) -> LspResult<LspClient> {
        let project = self.require_project(file_path).await?;

        // Get or spawn the server first
        let _process = self.get_or_spawn_server(file_path).await?;

        // Return the client
        let clients = self.clients.read().await;
        clients
            .get(&project.root_path)
            .cloned()
            .ok_or_else(|| LspError::NoServerAvailable {
                file_path: file_path.to_path_buf(),
            })
    }

    /// 📄 Ensure a document is open in the LSP server (sends didOpen if needed)
    ///
    /// This method handles document synchronization with the LSP server:
    /// 1. Checks if the document is already tracked/open
    /// 2. If not, reads the file content
    /// 3. Sends textDocument/didOpen notification to LSP server
    /// 4. Tracks the document for future reference
    ///
    /// This should be called before making any LSP requests that require document context.
    pub async fn ensure_document_open(&self, file_path: &Path) -> LspResult<()> {
        use lsp_types::*;
        use std::str::FromStr;
        use url::Url;

        let project = self.require_project(file_path).await?;
        
        // Convert file path to URI
        let file_url = Url::from_file_path(file_path).map_err(|_| LspError::InvalidRequest {
            message: format!("Invalid file path: {}", file_path.display()),
        })?;
        let file_uri = Uri::from_str(file_url.as_str()).unwrap();

        // Check if document is already open
        {
            let documents = self.documents.read().await;
            if let Some(tracker) = documents.get(&project.root_path)
                && tracker.is_open(&file_uri)
            {
                log::debug!("📄 Document already open: {}", file_path.display());
                return Ok(());
            }
        }

        // Read file content
        let content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
            LspError::InvalidRequest {
                message: format!("Failed to read file {}: {}", file_path.display(), e),
            }
        })?;

        // Get client (this ensures server is spawned)
        let client = self.get_client(file_path).await?;
        
        // Determine language ID from file extension
        let language_id = if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
            "rust"
        } else {
            "text"
        };

        // Send didOpen notification
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: file_uri.clone(),
                language_id: language_id.to_string(),
                version: 1,
                text: content.clone(),
            },
        };

        client.send_notification("textDocument/didOpen", Some(serde_json::to_value(params)?)).await?;

        // Track the document
        {
            let mut documents = self.documents.write().await;
            if let Some(tracker) = documents.get_mut(&project.root_path) {
                tracker.add_document(file_uri.clone(), content);
            }
        }

        log::info!("📄 Opened document in LSP: {}", file_path.display());
        
        // 🔥 HEURISTIC FIX: Give rust-analyzer a moment to start processing the file
        // This is a simple delay to reduce the likelihood of race conditions.
        // Most files index in <1 second, but we add a 2-second buffer to be safe.
        // First hover/diagnostics request may still be slow, but much better than 55s timeout.
        log::debug!("⏳ Waiting 2s for rust-analyzer to start indexing...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        log::debug!("✅ Document opened, indexing should be underway");

        Ok(())
    }

    /// 🛑 Gracefully shutdown a specific LSP server
    pub async fn shutdown_server(&self, project_path: &Path) -> LspResult<()> {
        // Close all documents for this server first
        {
            let mut documents = self.documents.write().await;
            documents.remove(project_path);
        }

        // Use lifecycle manager for shutdown
        {
            let mut processes = self.processes.write().await;
            let mut clients = self.clients.write().await;
            let mut children = self.children.write().await;
            
            self.lifecycle.shutdown_server(project_path, &mut processes, &mut clients, &mut children).await?;
        }

        Ok(())
    }

    /// 🛑 Shutdown all LSP servers
    pub async fn shutdown_all(&self) -> LspResult<()> {
        // Clear all documents
        {
            let mut documents = self.documents.write().await;
            documents.clear();
        }

        // Use lifecycle manager for shutdown
        {
            let mut processes = self.processes.write().await;
            let mut clients = self.clients.write().await;
            let mut children = self.children.write().await;
            
            self.lifecycle.shutdown_all(&mut processes, &mut clients, &mut children).await?;
        }

        Ok(())
    }

    /// 📊 Get status of all running LSP servers
    pub async fn get_server_status(&self) -> Vec<LspProcess> {
        let processes = self.processes.read().await;
        processes.values().cloned().collect()
    }

    /// 🏥 Health check for LSP servers
    pub async fn health_check(&self) -> LspResult<Vec<(PathBuf, bool)>> {
        let children = self.children.read().await;
        self.lifecycle.health_check(&children).await
    }

    /// Get the project detector
    pub fn detector(&self) -> &ProjectDetector {
        &self.detector
    }

    /// 📈 Get document statistics
    pub async fn get_document_stats(&self) -> HashMap<PathBuf, usize> {
        let documents = self.documents.read().await;
        documents
            .iter()
            .map(|(path, tracker)| (path.clone(), tracker.open_document_count()))
            .collect()
    }

    /// 🗄️ Get access to the LSP response cache
    pub fn cache(&self) -> &LspCache {
        &self.cache
    }

    /// 🗑️ Invalidate cache for a specific file
    pub async fn invalidate_file_cache(&self, file_path: &Path) {
        self.cache.invalidate_file(file_path).await;
    }

    /// 🗑️ Invalidate cache for a specific project
    pub async fn invalidate_project_cache(&self, project_path: &Path) {
        self.cache.invalidate_project(project_path).await;
    }

    // === 📊 Resource Management Delegation ===

    /// Start resource monitoring for all LSP processes
    pub async fn start_resource_monitoring(&self) -> LspResult<()> {
        self.lifecycle.start_resource_monitoring().await
    }

    /// Stop resource monitoring
    pub async fn stop_resource_monitoring(&self) {
        self.lifecycle.stop_resource_monitoring().await;
    }

    /// Get resource monitoring statistics
    pub async fn get_resource_stats(&self) -> crate::lsp::resource::ResourceStats {
        self.lifecycle.get_resource_stats().await
    }

    /// Get resource monitoring summary
    pub async fn get_resource_summary(&self) -> String {
        self.lifecycle.get_resource_summary().await
    }

    /// Check if a specific process exceeds resource limits
    pub async fn check_process_limits(&self, pid: u32) -> Option<bool> {
        self.lifecycle.check_process_limits(pid).await
    }

    /// Perform comprehensive health check with resource monitoring
    pub async fn comprehensive_health_check(&self) -> LspResult<HealthCheckResult> {
        let children = self.children.read().await;
        self.lifecycle.comprehensive_health_check(&children).await
    }

    // === 📂 Document Operations Helpers ===

    /// Get mutable access to document tracker for a project
    #[allow(dead_code)]
    async fn get_document_tracker(&self, _project_path: &Path) -> LspResult<tokio::sync::RwLockWriteGuard<'_, HashMap<PathBuf, DocumentTracker>>> {
        Ok(self.documents.write().await)
    }

    /// Get project path for a file
    #[allow(dead_code)]
    fn get_project_path_for_file(&self, file_path: &Path) -> LspResult<PathBuf> {
        let project = self.detector
            .find_project_for_file(file_path)?
            .ok_or_else(|| LspError::NoServerAvailable {
                file_path: file_path.to_path_buf(),
            })?;
        Ok(project.root_path)
    }

    // === ⏱️ Idle Monitoring Methods (v2.1.0) ===

    /// Get the idle monitor instance for external monitoring tasks
    ///
    /// The idle monitor tracks server usage but doesn't shutdown servers itself.
    /// External tasks should periodically call `shutdown_idle_servers()` to clean up.
    pub fn get_idle_monitor(&self) -> Arc<IdleMonitor> {
        Arc::clone(&self.idle_monitor)
    }

    /// Check if idle monitoring is enabled in configuration
    pub fn is_idle_monitoring_enabled(&self) -> bool {
        // Check the idle monitor's config (we can access this through stats)
        true // IdleMonitor::from_env() sets this based on LSP_ENABLE_IDLE_MONITOR
    }

    /// Mark a server as recently used (updates idle timer)
    async fn mark_server_used(&self, project_path: &Path) {
        // Currently we only support rust, will expand to multi-language in Phase 3
        self.idle_monitor.mark_used(project_path, "rust").await;
    }

    /// Get idle monitoring statistics
    pub async fn get_idle_stats(&self) -> crate::lsp::idle_monitor::IdleMonitorStats {
        self.idle_monitor.get_stats().await
    }

    /// Shutdown idle servers that have exceeded their timeout
    ///
    /// This is called periodically by the idle monitor background task,
    /// but can also be called manually for testing or immediate cleanup.
    pub async fn shutdown_idle_servers(&self) -> LspResult<Vec<PathBuf>> {
        let idle_servers = self.idle_monitor.get_idle_servers().await;
        let mut shutdown_paths = Vec::new();
        
        for (project_path, language) in idle_servers {
            log::info!(
                "⏰ Shutting down idle server: {} ({})",
                project_path.display(),
                language
            );
            
            match self.shutdown_server(&project_path).await {
                Ok(_) => {
                    self.idle_monitor.remove_server(&project_path, &language).await;
                    shutdown_paths.push(project_path);
                }
                Err(e) => {
                    log::error!(
                        "❌ Failed to shutdown idle server {} ({}): {}",
                        project_path.display(),
                        language,
                        e
                    );
                }
            }
        }
        
        Ok(shutdown_paths)
    }
}

impl Drop for LspManagerCore {
    fn drop(&mut self) {
        log::info!("🗑️ LSP Manager Core dropped, processes should be cleaned up");
    }
}
