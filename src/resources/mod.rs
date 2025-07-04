//! 📚 Resources module - MCP resource management
//! 
//! This module provides resource management functionality for the MCP server,
//! handling file access, directory listing, and resource discovery.

pub mod manager;

pub use manager::{
    get_base_dir, get_allowed_dir, validate_path,
    get_resources, read_resource
};
