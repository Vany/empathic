//! ⚡ LSP Process Lifecycle Management
//!
//! Handles spawning, monitoring, and shutting down LSP server processes.
//! Includes resource management, health monitoring, and automatic restart capabilities.

use crate::lsp::client::LspClient;
use crate::lsp::resource::{ResourceMonitor, ResourceConfig, ResourceStats};
use crate::lsp::types::{LspError, LspProcess, LspResult, HealthCheckResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::{Child, Command};
use std::process::Stdio;

/// ⚡ LSP Process Lifecycle Manager
#[derive(Debug)]
pub struct ProcessLifecycle {
    /// Resource monitoring and automatic restart
    resource_monitor: ResourceMonitor,
}

impl ProcessLifecycle {
    pub fn new() -> Self {
        Self {
            resource_monitor: ResourceMonitor::with_defaults(),
        }
    }

    pub fn with_resource_config(resource_config: ResourceConfig) -> Self {
        Self {
            resource_monitor: ResourceMonitor::new(resource_config),
        }
    }

    /// 🦀 Spawn a new rust-analyzer process for the given project
    pub async fn spawn_rust_analyzer(&self, project_path: &Path) -> LspResult<(LspProcess, LspClient, Child)> {
        // Find rust-analyzer binary
        let rust_analyzer_path = self.find_rust_analyzer().await?;

        // Spawn the process
        let mut command = Command::new(&rust_analyzer_path);
        command
            .current_dir(project_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|e| LspError::SpawnError {
            message: format!("Failed to spawn rust-analyzer: {e}"),
        })?;

        let process_id = child
            .id()
            .ok_or_else(|| LspError::SpawnError {
                message: "Failed to get process ID".to_string(),
            })?;

        // Create LSP client for communication
        let stdin = child.stdin.take().ok_or_else(|| LspError::SpawnError {
            message: "Failed to get stdin handle".to_string(),
        })?;
        let stdout = child.stdout.take().ok_or_else(|| LspError::SpawnError {
            message: "Failed to get stdout handle".to_string(),
        })?;

        let client = LspClient::new(stdin, stdout, project_path.to_path_buf()).await?;

        // Initialize the LSP server
        let init_result = client.initialize().await?;
        log::info!("🚀 LSP server initialized with capabilities: {:?}", init_result.capabilities);

        // Send initialized notification
        client.send_notification("initialized", None).await?;

        let lsp_process = LspProcess {
            project_path: project_path.to_path_buf(),
            server_name: "rust-analyzer".to_string(),
            process_id,
            capabilities: Some(init_result.capabilities),
            initialized: true,
        };

        log::info!(
            "🦀 Spawned rust-analyzer for project: {} (PID: {})",
            project_path.display(),
            process_id
        );

        Ok((lsp_process, client, child))
    }

    /// 🔍 Find rust-analyzer binary in PATH
    async fn find_rust_analyzer(&self) -> LspResult<PathBuf> {
        // Try common names for rust-analyzer
        let candidates = ["rust-analyzer", "rust-analyzer.exe"];

        for candidate in &candidates {
            // Try 'which' first (Unix-like systems)
            if let Ok(output) = Command::new("which").arg(candidate).output().await
                && output.status.success()
            {
                let path_output = String::from_utf8_lossy(&output.stdout);
                let path_str = path_output.trim();
                if !path_str.is_empty() {
                    return Ok(PathBuf::from(path_str));
                }
            }

            // Try 'where' for Windows
            if let Ok(output) = Command::new("where").arg(candidate).output().await
                && output.status.success()
            {
                let path_output = String::from_utf8_lossy(&output.stdout);
                let path_str = path_output.lines().next().unwrap_or("").trim();
                if !path_str.is_empty() {
                    return Ok(PathBuf::from(path_str));
                }
            }
        }

        Err(LspError::NoServerAvailable {
            file_path: PathBuf::from("rust-analyzer not found in PATH"),
        })
    }

    /// 🛑 Shutdown an LSP server for a specific project
    pub async fn shutdown_server(
        &self,
        project_path: &Path,
        processes: &mut HashMap<PathBuf, LspProcess>,
        clients: &mut HashMap<PathBuf, LspClient>,
        children: &mut HashMap<PathBuf, Child>,
    ) -> LspResult<()> {
        // Send shutdown request if client exists
        if let Some(client) = clients.get(project_path)
            && let Err(e) = client.shutdown().await
        {
            log::warn!("❌ Failed to send shutdown request: {}", e);
        }

        // Clean up child process
        if let Some(mut child) = children.remove(project_path) {
            // Try graceful termination first
            match child.kill().await {
                Ok(_) => log::debug!("🛑 Terminated process for {}", project_path.display()),
                Err(e) => log::warn!("❌ Failed to terminate process: {}", e),
            }
        }

        // Remove from tracking
        processes.remove(project_path);
        clients.remove(project_path);

        log::info!("🛑 Shut down LSP server for {}", project_path.display());
        Ok(())
    }

    /// 🛑 Shutdown all LSP servers
    pub async fn shutdown_all(
        &self,
        processes: &mut HashMap<PathBuf, LspProcess>,
        clients: &mut HashMap<PathBuf, LspClient>,
        children: &mut HashMap<PathBuf, Child>,
    ) -> LspResult<()> {
        log::info!("🛑 Starting graceful shutdown of all LSP servers...");

        // Stop resource monitoring first
        self.stop_resource_monitoring().await;

        // Get list of all projects
        let project_paths: Vec<PathBuf> = processes.keys().cloned().collect();

        let mut shutdown_results = Vec::new();

        // Shutdown each server
        for project_path in project_paths {
            log::info!("🛑 Shutting down LSP server for {}", project_path.display());

            match self.shutdown_server(&project_path, processes, clients, children).await {
                Ok(_) => {
                    log::info!("✅ Successfully shut down server for {}", project_path.display());
                    shutdown_results.push(Ok(()));
                }
                Err(e) => {
                    log::error!("❌ Failed to shutdown server for {}: {}", project_path.display(), e);
                    shutdown_results.push(Err(e));
                }
            }
        }

        // Report results
        let successful = shutdown_results.iter().filter(|r| r.is_ok()).count();
        let failed = shutdown_results.len() - successful;

        if failed == 0 {
            log::info!("✅ All {} LSP servers shut down successfully", successful);
            Ok(())
        } else {
            log::error!("❌ Failed to shut down {} out of {} LSP servers", failed, successful + failed);
            // Return the first error encountered
            shutdown_results.into_iter().find(|r| r.is_err()).unwrap()
        }
    }

    /// 🏥 Perform health check on all running processes
    pub async fn health_check(&self, children: &HashMap<PathBuf, Child>) -> LspResult<Vec<(PathBuf, bool)>> {
        let mut results = Vec::new();

        for (project_path, child) in children.iter() {
            // Simple health check - see if the process is still running
            let is_healthy = child.id().is_some();
            results.push((project_path.clone(), is_healthy));
        }

        Ok(results)
    }

    // === 📊 Resource Management Methods ===

    /// Start resource monitoring for all LSP processes
    pub async fn start_resource_monitoring(&self) -> LspResult<()> {
        self.resource_monitor.start_monitoring().await
            .map_err(|e| LspError::InvalidRequest { message: e })?;
        
        log::info!("📊 Resource monitoring started");
        Ok(())
    }

    /// Stop resource monitoring
    pub async fn stop_resource_monitoring(&self) {
        self.resource_monitor.stop_monitoring().await;
        log::info!("📊 Resource monitoring stopped");
    }

    /// Get resource monitoring statistics
    pub async fn get_resource_stats(&self) -> ResourceStats {
        self.resource_monitor.get_stats().await
    }

    /// Get resource monitoring summary
    pub async fn get_resource_summary(&self) -> String {
        self.resource_monitor.get_summary().await
    }

    /// Check if a specific process exceeds resource limits
    pub async fn check_process_limits(&self, pid: u32) -> Option<bool> {
        self.resource_monitor.check_process_limits(pid).await
    }

    /// Comprehensive health check including resource monitoring
    pub async fn comprehensive_health_check(&self, children: &HashMap<PathBuf, Child>) -> LspResult<HealthCheckResult> {
        // Basic health check
        let process_health = self.health_check(children).await?;
        
        // Resource statistics
        let resource_stats = self.get_resource_stats().await;
        
        // Separate healthy and unhealthy processes
        let mut healthy_processes = Vec::new();
        let mut unhealthy_processes = Vec::new();
        
        for (project_path, is_healthy) in process_health {
            if is_healthy {
                healthy_processes.push(project_path);
            } else {
                unhealthy_processes.push(project_path);
            }
        }
        
        // Create performance summary
        let performance_summary = format!(
            "📊 {}/{} processes healthy, {} monitored", 
            healthy_processes.len(), 
            healthy_processes.len() + unhealthy_processes.len(),
            resource_stats.total_processes
        );
        
        Ok(HealthCheckResult {
            healthy_processes,
            unhealthy_processes,
            resource_stats,
            performance_summary,
            over_limit_count: 0, // TODO: Get actual over-limit count from resource monitor
            timestamp: std::time::Instant::now(),
        })
    }
}

impl Default for ProcessLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
