//! 📝 Prompts module - MCP prompt templates and management
//! 
//! This module provides comprehensive prompt templates for common development tasks,
//! including code review, optimization, debugging, architecture design, and more.

pub mod templates;

pub use templates::{get_prompts_schema, get_prompt};
