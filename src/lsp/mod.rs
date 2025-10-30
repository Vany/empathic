//! 🧠 LSP Integration Module - Language Server Protocol support for empathic
//!
//! This module provides LSP integration capabilities, enabling empathic to communicate
//! with external Language Server Protocol servers (like rust-analyzer) to provide
//! rich semantic code analysis tools.
//!
//! ## Architecture
//!
//! - **manager**: LSP process lifecycle management
//! - **client**: JSON-RPC communication layer  
//! - **project_detector**: Multi-language project detection logic
//! - **server_config**: Language server configuration registry
//! - **types**: LSP error wrappers and empathic-specific types
//! - **cache**: Response caching for performance optimization
//! - **performance**: Request queuing, metrics, and optimization
//! - **resource**: Memory monitoring and process management
//! - **idle_monitor**: Automatic idle timeout and cleanup

pub mod cache;
pub mod client;
pub mod idle_monitor;
pub mod manager;
pub mod performance;
pub mod project_detector;
pub mod resource;
pub mod server_config;
pub mod types;

pub use cache::LspCache;
pub use client::LspClient;
pub use idle_monitor::{IdleMonitor, IdleMonitorConfig, IdleMonitorStats, ServerKey};
pub use manager::LspManager;
pub use performance::{LspMetrics, RequestQueue, ConnectionPool, PerformanceTester, RequestPriority};
pub use project_detector::{Project, ProjectDetector, RustProject};
pub use resource::{ResourceMonitor, ResourceConfig, MemoryUsage, ResourceStats};
pub use server_config::ServerConfig;
pub use types::{LspError, LspResult, HealthCheckResult};
