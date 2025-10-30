//! 🚀 Model Context Protocol (MCP) Server Implementation
//! 
//! This module provides a complete MCP server implementation with:
//! - 📨 JSON-RPC 2.0 protocol compliance
//! - 🔧 Tool registration and execution
//! - 🧠 LSP integration for semantic analysis
//! - 📊 Structured request/response handling

pub mod protocol;
pub mod handlers;
pub mod server;

// Re-export main types for convenience
pub use server::McpServer;
pub use protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
