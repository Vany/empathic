//! 🧪 Common test utilities and helpers for empathic MCP tools
//! 
//! This module provides shared functionality to reduce boilerplate
//! and ensure consistent testing patterns across all MCP tools.

#![allow(dead_code)] // Test utilities may not all be used in every test
#![allow(unused_imports)] // Re-exports for convenience

pub mod setup;
pub mod mcp;
pub mod content;
pub mod fs_helpers;

pub use setup::*;
pub use mcp::*;
pub use content::*;
pub use fs_helpers::*;
