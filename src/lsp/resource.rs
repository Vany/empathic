//! 📊 Resource monitoring and management for LSP processes
//!
//! Provides memory usage tracking, automatic restart capabilities, and graceful
//! shutdown management for LSP server processes to ensure system stability.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

/// 💾 Memory usage information for a process
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Process ID
    pub pid: u32,
    /// Resident Set Size in bytes (physical memory currently used)
    pub rss_bytes: u64,
    /// Virtual Memory Size in bytes (total virtual memory used)
    pub vms_bytes: u64,
    /// Memory usage as percentage of system total
    pub memory_percent: f64,
    /// Timestamp of measurement
    pub timestamp: Instant,
}

impl MemoryUsage {
    /// Create new memory usage measurement
    pub fn new(pid: u32, rss_bytes: u64, vms_bytes: u64, memory_percent: f64) -> Self {
        Self {
            pid,
            rss_bytes,
            vms_bytes,
            memory_percent,
            timestamp: Instant::now(),
        }
    }
    
    /// Get RSS in megabytes for human-readable display
    pub fn rss_mb(&self) -> f64 {
        self.rss_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// Get VMS in megabytes for human-readable display
    pub fn vms_mb(&self) -> f64 {
        self.vms_bytes as f64 / (1024.0 * 1024.0)
    }
    
    /// Check if memory usage exceeds given thresholds
    pub fn exceeds_limits(&self, max_rss_mb: f64, max_memory_percent: f64) -> bool {
        self.rss_mb() > max_rss_mb || self.memory_percent > max_memory_percent
    }
}

/// ⚙️ Resource monitoring configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// Maximum RSS memory per process in MB (default: 1GB)
    pub max_rss_mb: f64,
    /// Maximum memory percentage per process (default: 10%)
    pub max_memory_percent: f64,
    /// Monitoring interval in seconds (default: 30s)
    pub monitor_interval_secs: u64,
    /// Grace period before restart in seconds (default: 60s)
    pub restart_grace_secs: u64,
    /// Maximum restart attempts before giving up (default: 3)
    pub max_restart_attempts: u32,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_rss_mb: 1024.0,        // 1GB
            max_memory_percent: 10.0,   // 10%
            monitor_interval_secs: 30,  // 30 seconds
            restart_grace_secs: 60,     // 1 minute
            max_restart_attempts: 3,    // 3 attempts
        }
    }
}

/// 📈 Resource monitoring statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Total monitored processes
    pub total_processes: usize,
    /// Processes exceeding memory limits
    pub over_limit_processes: usize,
    /// Total restarts performed
    pub total_restarts: u64,
    /// Failed restart attempts
    pub failed_restarts: u64,
    /// Average memory usage across all processes (MB)
    pub avg_memory_mb: f64,
    /// Peak memory usage observed (MB)
    pub peak_memory_mb: f64,
    /// Last monitoring update
    pub last_update: Instant,
}

impl Default for ResourceStats {
    fn default() -> Self {
        Self {
            total_processes: 0,
            over_limit_processes: 0,
            total_restarts: 0,
            failed_restarts: 0,
            avg_memory_mb: 0.0,
            peak_memory_mb: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// 🔄 Process restart information
#[derive(Debug, Clone)]
pub struct RestartInfo {
    /// Number of restart attempts for this process
    pub attempts: u32,
    /// Last restart timestamp
    pub last_restart: Instant,
    /// Reason for restart
    pub reason: String,
}

/// 📊 LSP process resource monitor
pub struct ResourceMonitor {
    /// Monitoring configuration
    config: ResourceConfig,
    /// Current memory usage per process
    memory_usage: Arc<RwLock<HashMap<u32, MemoryUsage>>>,
    /// Restart tracking per project path
    restart_info: Arc<RwLock<HashMap<PathBuf, RestartInfo>>>,
    /// Resource monitoring statistics
    stats: Arc<RwLock<ResourceStats>>,
    /// Whether monitoring is active
    monitoring_active: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for ResourceMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceMonitor")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ResourceMonitor {
    /// Create new resource monitor with configuration
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            config,
            memory_usage: Arc::new(RwLock::new(HashMap::new())),
            restart_info: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ResourceStats::default())),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Create monitor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ResourceConfig::default())
    }
    
    /// Start background monitoring task
    pub async fn start_monitoring(&self) -> Result<(), String> {
        let mut active = self.monitoring_active.write().await;
        if *active {
            return Err("Monitoring already active".to_string());
        }
        *active = true;
        
        let config = self.config.clone();
        let memory_usage = self.memory_usage.clone();
        let stats = self.stats.clone();
        let monitoring_active = self.monitoring_active.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.monitor_interval_secs));
            
            while *monitoring_active.read().await {
                interval.tick().await;
                
                if let Err(e) = Self::update_memory_usage(&memory_usage, &stats).await {
                    log::warn!("📊 Failed to update memory usage: {}", e);
                }
            }
            
            log::info!("📊 Resource monitoring stopped");
        });
        
        log::info!("📊 Resource monitoring started (interval: {}s)", self.config.monitor_interval_secs);
        Ok(())
    }
    
    /// Stop background monitoring
    pub async fn stop_monitoring(&self) {
        let mut active = self.monitoring_active.write().await;
        *active = false;
        log::info!("📊 Resource monitoring stopping...");
    }
    
    /// Get current memory usage for a specific process
    pub async fn get_memory_usage(&self, pid: u32) -> Option<MemoryUsage> {
        let usage = self.memory_usage.read().await;
        usage.get(&pid).cloned()
    }
    
    /// Get memory usage for all monitored processes
    pub async fn get_all_memory_usage(&self) -> HashMap<u32, MemoryUsage> {
        let usage = self.memory_usage.read().await;
        usage.clone()
    }
    
    /// Check if a process exceeds resource limits
    pub async fn check_process_limits(&self, pid: u32) -> Option<bool> {
        let usage = self.memory_usage.read().await;
        usage.get(&pid).map(|mem| {
            mem.exceeds_limits(self.config.max_rss_mb, self.config.max_memory_percent)
        })
    }
    
    /// Get processes that exceed resource limits
    pub async fn get_over_limit_processes(&self) -> Vec<(u32, MemoryUsage)> {
        let usage = self.memory_usage.read().await;
        usage
            .iter()
            .filter(|(_, mem)| {
                mem.exceeds_limits(self.config.max_rss_mb, self.config.max_memory_percent)
            })
            .map(|(pid, mem)| (*pid, mem.clone()))
            .collect()
    }
    
    /// Record a process restart
    pub async fn record_restart(&self, project_path: PathBuf, reason: String) {
        let mut restarts = self.restart_info.write().await;
        let mut stats = self.stats.write().await;
        
        let restart_info = restarts.entry(project_path).or_insert(RestartInfo {
            attempts: 0,
            last_restart: Instant::now(),
            reason: reason.clone(),
        });
        
        restart_info.attempts += 1;
        restart_info.last_restart = Instant::now();
        restart_info.reason = reason;
        
        stats.total_restarts += 1;
        
        log::warn!("🔄 Recorded process restart: attempts={}, reason='{}'", 
                  restart_info.attempts, restart_info.reason);
    }
    
    /// Check if a process can be restarted (hasn't exceeded max attempts)
    pub async fn can_restart(&self, project_path: &PathBuf) -> bool {
        let restarts = self.restart_info.read().await;
        match restarts.get(project_path) {
            Some(info) => info.attempts < self.config.max_restart_attempts,
            None => true, // First restart
        }
    }
    
    /// Get current resource monitoring statistics
    pub async fn get_stats(&self) -> ResourceStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Remove monitoring for a process (when it's shut down)
    pub async fn remove_process(&self, pid: u32) {
        let mut usage = self.memory_usage.write().await;
        if usage.remove(&pid).is_some() {
            log::debug!("📊 Removed process {} from monitoring", pid);
        }
    }
    
    /// Get human-readable monitoring summary
    pub async fn get_summary(&self) -> String {
        let stats = self.get_stats().await;
        let usage = self.get_all_memory_usage().await;
        
        let total_memory: f64 = usage.values().map(|u| u.rss_mb()).sum();
        let over_limit = self.get_over_limit_processes().await;
        
        format!(
            "📊 Resource Monitor: {} processes, {:.1}MB total, {} over limits, {} restarts",
            usage.len(),
            total_memory,
            over_limit.len(),
            stats.total_restarts
        )
    }
    
    /// Internal method to update memory usage for all processes
    async fn update_memory_usage(
        memory_usage: &Arc<RwLock<HashMap<u32, MemoryUsage>>>,
        stats: &Arc<RwLock<ResourceStats>>,
    ) -> Result<(), String> {
        let current_usage = Self::get_system_memory_usage().await?;
        
        // Update memory usage map
        {
            let mut usage = memory_usage.write().await;
            usage.clear();
            for mem in &current_usage {
                usage.insert(mem.pid, mem.clone());
            }
        }
        
        // Update statistics
        {
            let mut stats = stats.write().await;
            stats.total_processes = current_usage.len();
            stats.over_limit_processes = current_usage
                .iter()
                .filter(|mem| mem.exceeds_limits(1024.0, 10.0)) // Use default limits
                .count();
            
            if !current_usage.is_empty() {
                let total_memory: f64 = current_usage.iter().map(|u| u.rss_mb()).sum();
                stats.avg_memory_mb = total_memory / current_usage.len() as f64;
                
                let max_memory = current_usage
                    .iter()
                    .map(|u| u.rss_mb())
                    .fold(0.0, f64::max);
                
                if max_memory > stats.peak_memory_mb {
                    stats.peak_memory_mb = max_memory;
                }
            }
            
            stats.last_update = Instant::now();
        }
        
        Ok(())
    }
    
    /// Get memory usage for all rust-analyzer processes
    async fn get_system_memory_usage() -> Result<Vec<MemoryUsage>, String> {
        // Try different approaches based on OS
        #[cfg(target_os = "macos")]
        return Self::get_memory_usage_macos().await;
        
        #[cfg(target_os = "linux")]
        return Self::get_memory_usage_linux().await;
        
        #[cfg(target_os = "windows")]
        return Self::get_memory_usage_windows().await;
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            log::warn!("📊 Memory monitoring not supported on this platform");
            Ok(Vec::new())
        }
    }
    
    #[cfg(target_os = "macos")]
    async fn get_memory_usage_macos() -> Result<Vec<MemoryUsage>, String> {
        let output = Command::new("ps")
            .args(["-ax", "-o", "pid,rss,vsz,%mem,comm"])
            .output()
            .map_err(|e| format!("Failed to run ps command: {}", e))?;
        
        if !output.status.success() {
            return Err("ps command failed".to_string());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();
        
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && parts[4].contains("rust-analyzer")
                && let (Ok(pid), Ok(rss_kb), Ok(vsz_kb), Ok(mem_percent)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<u64>(),
                    parts[2].parse::<u64>(),
                    parts[3].parse::<f64>(),
                ) {
                processes.push(MemoryUsage::new(
                    pid,
                    rss_kb * 1024, // Convert KB to bytes
                    vsz_kb * 1024, // Convert KB to bytes
                    mem_percent,
                ));
            }
        }
        
        Ok(processes)
    }
    
    #[cfg(target_os = "linux")]
    async fn get_memory_usage_linux() -> Result<Vec<MemoryUsage>, String> {
        let output = Command::new("ps")
            .args(["-ax", "-o", "pid,rss,vsz,%mem,comm"])
            .output()
            .map_err(|e| format!("Failed to run ps command: {}", e))?;
        
        if !output.status.success() {
            return Err("ps command failed".to_string());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();
        
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && parts[4].contains("rust-analyzer") {
                if let (Ok(pid), Ok(rss_kb), Ok(vsz_kb), Ok(mem_percent)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<u64>(),
                    parts[2].parse::<u64>(),
                    parts[3].parse::<f64>(),
                ) {
                    processes.push(MemoryUsage::new(
                        pid,
                        rss_kb * 1024, // Convert KB to bytes
                        vsz_kb * 1024, // Convert KB to bytes
                        mem_percent,
                    ));
                }
            }
        }
        
        Ok(processes)
    }
    
    #[cfg(target_os = "windows")]
    async fn get_memory_usage_windows() -> Result<Vec<MemoryUsage>, String> {
        let output = Command::new("tasklist")
            .args(["/FO", "CSV", "/FI", "IMAGENAME eq rust-analyzer*"])
            .output()
            .map_err(|e| format!("Failed to run tasklist command: {}", e))?;
        
        if !output.status.success() {
            return Err("tasklist command failed".to_string());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();
        
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                // Parse CSV format from tasklist
                if let (Ok(pid), Ok(memory_str)) = (
                    parts[1].trim_matches('"').parse::<u32>(),
                    parts[4].trim_matches('"'),
                ) {
                    // Parse memory string like "1,234 K" to bytes
                    let memory_kb = memory_str
                        .replace(",", "")
                        .replace(" K", "")
                        .parse::<u64>()
                        .unwrap_or(0);
                    
                    processes.push(MemoryUsage::new(
                        pid,
                        memory_kb * 1024, // Convert KB to bytes
                        memory_kb * 1024, // VSZ approximation
                        0.0, // Memory percentage not available from tasklist
                    ));
                }
            }
        }
        
        Ok(processes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_usage_creation() {
        let mem = MemoryUsage::new(1234, 1024 * 1024 * 100, 1024 * 1024 * 200, 5.5);
        
        assert_eq!(mem.pid, 1234);
        assert_eq!(mem.rss_mb(), 100.0);
        assert_eq!(mem.vms_mb(), 200.0);
        assert_eq!(mem.memory_percent, 5.5);
    }
    
    #[test]
    fn test_memory_usage_limits() {
        let mem = MemoryUsage::new(1234, 1024 * 1024 * 2048, 0, 15.0); // 2GB, 15%
        
        assert!(mem.exceeds_limits(1024.0, 10.0)); // Exceeds both RSS and %
        assert!(!mem.exceeds_limits(4096.0, 20.0)); // Within both limits
        assert!(mem.exceeds_limits(1024.0, 20.0)); // Exceeds RSS only
        assert!(mem.exceeds_limits(4096.0, 10.0)); // Exceeds % only
    }
    
    #[test]
    fn test_resource_config_defaults() {
        let config = ResourceConfig::default();
        
        assert_eq!(config.max_rss_mb, 1024.0);
        assert_eq!(config.max_memory_percent, 10.0);
        assert_eq!(config.monitor_interval_secs, 30);
        assert_eq!(config.restart_grace_secs, 60);
        assert_eq!(config.max_restart_attempts, 3);
    }
    
    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::with_defaults();
        let stats = monitor.get_stats().await;
        
        assert_eq!(stats.total_processes, 0);
        assert_eq!(stats.total_restarts, 0);
    }
    
    #[tokio::test]
    async fn test_restart_tracking() {
        let monitor = ResourceMonitor::with_defaults();
        let project_path = PathBuf::from("/test/project");
        
        assert!(monitor.can_restart(&project_path).await);
        
        monitor.record_restart(project_path.clone(), "Memory limit exceeded".to_string()).await;
        assert!(monitor.can_restart(&project_path).await);
        
        monitor.record_restart(project_path.clone(), "Crash detected".to_string()).await;
        assert!(monitor.can_restart(&project_path).await);
        
        monitor.record_restart(project_path.clone(), "Third failure".to_string()).await;
        assert!(!monitor.can_restart(&project_path).await); // Exceeded max attempts
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_restarts, 3);
    }
    
    #[tokio::test]
    async fn test_process_removal() {
        let monitor = ResourceMonitor::with_defaults();
        
        // Simulate adding process to monitoring
        {
            let mut usage = monitor.memory_usage.write().await;
            usage.insert(1234, MemoryUsage::new(1234, 1024, 2048, 1.0));
        }
        
        assert!(monitor.get_memory_usage(1234).await.is_some());
        
        monitor.remove_process(1234).await;
        assert!(monitor.get_memory_usage(1234).await.is_none());
    }
}