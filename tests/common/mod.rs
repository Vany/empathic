//! 🧪 Common test utilities and helpers for empathic MCP tools
//! 
//! This module provides shared functionality to reduce boilerplate
//! and ensure consistent testing patterns across all MCP tools.

pub mod setup;
pub mod mcp;
pub mod content;
pub mod fs_helpers;

pub use setup::*;
pub use mcp::*;
pub use content::*;
pub use fs_helpers::*;
