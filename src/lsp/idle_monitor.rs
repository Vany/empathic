//! ⏱️ LSP Server Idle Monitoring and Automatic Cleanup
//!
//! Monitors LSP server activity and automatically shuts down servers that have been
//! idle for longer than the configured timeout period. This helps manage system
//! resources by cleaning up servers that are no longer being used.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Key identifying a unique LSP server instance: (project_path, language)
pub type ServerKey = (PathBuf, String);

/// ⏱️ Configuration for idle monitoring behavior
#[derive(Debug, Clone)]
pub struct IdleMonitorConfig {
    /// How long a server can be idle before shutdown
    pub idle_timeout: Duration,
    
    /// How often to check for idle servers
    pub check_interval: Duration,
    
    /// Enable/disable idle monitoring
    pub enabled: bool,
}

impl Default for IdleMonitorConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(10 * 60), // 10 minutes
            check_interval: Duration::from_secs(60),     // 1 minute
            enabled: true,
        }
    }
}

impl IdleMonitorConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let idle_timeout = std::env::var("LSP_IDLE_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(10 * 60));
        
        let check_interval = std::env::var("LSP_CHECK_INTERVAL")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(60));
        
        let enabled = std::env::var("LSP_ENABLE_IDLE_MONITOR")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(true);
        
        Self {
            idle_timeout,
            check_interval,
            enabled,
        }
    }
}

/// ⏱️ Monitors LSP server idle time
///
/// This component tracks the last time each LSP server was used.
/// External tasks can query for idle servers and handle shutdowns.
#[derive(Debug)]
pub struct IdleMonitor {
    /// Last request time per server
    last_used: Arc<RwLock<HashMap<ServerKey, Instant>>>,
    
    /// Configuration
    config: IdleMonitorConfig,
}

impl IdleMonitor {
    /// Create a new idle monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(IdleMonitorConfig::default())
    }
    
    /// Create a new idle monitor with custom configuration
    pub fn with_config(config: IdleMonitorConfig) -> Self {
        Self {
            last_used: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create from environment variables
    pub fn from_env() -> Self {
        Self::with_config(IdleMonitorConfig::from_env())
    }
    
    /// Mark a server as recently used
    pub async fn mark_used(&self, project_path: &Path, language: &str) {
        let key = (project_path.to_path_buf(), language.to_string());
        let mut last_used = self.last_used.write().await;
        last_used.insert(key, Instant::now());
        
        log::trace!(
            "📊 Marked server as used: {} ({})",
            project_path.display(),
            language
        );
    }
    
    /// Remove a server from tracking (called when server is shut down)
    pub async fn remove_server(&self, project_path: &Path, language: &str) {
        let key = (project_path.to_path_buf(), language.to_string());
        let mut last_used = self.last_used.write().await;
        last_used.remove(&key);
        
        log::debug!(
            "🗑️ Removed server from idle tracking: {} ({})",
            project_path.display(),
            language
        );
    }
    
    /// Get list of servers that have exceeded idle timeout
    pub async fn get_idle_servers(&self) -> Vec<ServerKey> {
        let now = Instant::now();
        let last_used = self.last_used.read().await;
        
        last_used
            .iter()
            .filter(|(_, last_time)| {
                now.duration_since(**last_time) > self.config.idle_timeout
            })
            .map(|(key, _)| key.clone())
            .collect()
    }
    
    /// Get time since last use for a specific server
    pub async fn time_since_last_use(&self, project_path: &Path, language: &str) -> Option<Duration> {
        let key = (project_path.to_path_buf(), language.to_string());
        let last_used = self.last_used.read().await;
        
        last_used.get(&key).map(|&last_time| {
            Instant::now().duration_since(last_time)
        })
    }
    
    /// Get current monitoring statistics
    pub async fn get_stats(&self) -> IdleMonitorStats {
        let last_used = self.last_used.read().await;
        let now = Instant::now();
        
        let total_tracked = last_used.len();
        let idle_servers = last_used
            .iter()
            .filter(|(_, last_time)| {
                now.duration_since(**last_time) > self.config.idle_timeout
            })
            .count();
        
        let oldest_idle_time = last_used
            .values()
            .map(|last_time| now.duration_since(*last_time))
            .max();
        
        IdleMonitorStats {
            total_tracked,
            idle_servers,
            oldest_idle_time,
            config: self.config.clone(),
        }
    }
}

impl Default for IdleMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about idle monitoring
#[derive(Debug, Clone)]
pub struct IdleMonitorStats {
    /// Total number of servers being tracked
    pub total_tracked: usize,
    
    /// Number of servers currently idle (exceeding timeout)
    pub idle_servers: usize,
    
    /// Time since last use for the oldest idle server
    pub oldest_idle_time: Option<Duration>,
    
    /// Current configuration
    pub config: IdleMonitorConfig,
}

impl IdleMonitorStats {
    /// Format as human-readable string
    pub fn summary(&self) -> String {
        let oldest = self.oldest_idle_time
            .map(|d| format!("{:.1}m", d.as_secs_f64() / 60.0))
            .unwrap_or_else(|| "N/A".to_string());
        
        format!(
            "📊 Tracking {} servers, {} idle (oldest: {}), timeout: {:.1}m",
            self.total_tracked,
            self.idle_servers,
            oldest,
            self.config.idle_timeout.as_secs_f64() / 60.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_idle_monitor_creation() {
        let monitor = IdleMonitor::new();
        let stats = monitor.get_stats().await;
        
        assert_eq!(stats.total_tracked, 0);
        assert_eq!(stats.idle_servers, 0);
    }
    
    #[tokio::test]
    async fn test_mark_used() {
        let monitor = IdleMonitor::new();
        let project = PathBuf::from("/test/project");
        
        monitor.mark_used(&project, "rust").await;
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_tracked, 1);
    }
    
    #[tokio::test]
    async fn test_remove_server() {
        let monitor = IdleMonitor::new();
        let project = PathBuf::from("/test/project");
        
        monitor.mark_used(&project, "rust").await;
        assert_eq!(monitor.get_stats().await.total_tracked, 1);
        
        monitor.remove_server(&project, "rust").await;
        assert_eq!(monitor.get_stats().await.total_tracked, 0);
    }
    
    #[tokio::test]
    async fn test_idle_detection() {
        let config = IdleMonitorConfig {
            idle_timeout: Duration::from_millis(100),
            check_interval: Duration::from_millis(50),
            enabled: true,
        };
        
        let monitor = IdleMonitor::with_config(config);
        let project = PathBuf::from("/test/project");
        
        // Mark as used
        monitor.mark_used(&project, "rust").await;
        
        // Should not be idle immediately
        let idle = monitor.get_idle_servers().await;
        assert_eq!(idle.len(), 0);
        
        // Wait for idle timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should now be idle
        let idle = monitor.get_idle_servers().await;
        assert_eq!(idle.len(), 1);
        assert_eq!(idle[0].0, project);
        assert_eq!(idle[0].1, "rust");
    }
    
    #[tokio::test]
    async fn test_time_since_last_use() {
        let monitor = IdleMonitor::new();
        let project = PathBuf::from("/test/project");
        
        monitor.mark_used(&project, "rust").await;
        
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let time_since = monitor.time_since_last_use(&project, "rust").await;
        assert!(time_since.is_some());
        assert!(time_since.unwrap() >= Duration::from_millis(50));
    }
    
    #[tokio::test]
    async fn test_stats_summary() {
        let monitor = IdleMonitor::new();
        let project = PathBuf::from("/test/project");
        
        monitor.mark_used(&project, "rust").await;
        
        let stats = monitor.get_stats().await;
        let summary = stats.summary();
        
        assert!(summary.contains("Tracking 1 servers"));
    }
}
