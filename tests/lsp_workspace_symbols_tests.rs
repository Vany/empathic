//! 🔍 LSP Workspace Symbols Tool Tests
//! Comprehensive testing for workspace symbol search functionality

use empathic::tools::lsp::LspWorkspaceSymbolsTool;
use empathic::tools::Tool;
use serde_json::{json, Value};

mod common;
use common::setup::TestEnv;

/// ✅ Test workspace symbols tool schema validation
#[tokio::test]
async fn test_workspace_symbols_schema() {
    let tool = LspWorkspaceSymbolsTool;
    let schema = tool.schema();
    
    // Verify required fields
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["required"], json!(["query"]));
    
    // Verify query parameter schema
    let query_prop = &schema["properties"]["query"];
    assert_eq!(query_prop["type"], "string");
    assert!(query_prop["description"].is_string());
}

/// ✅ Test workspace symbols with valid Rust project search
#[tokio::test]
async fn test_workspace_symbols_rust_project() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;

    // Create a mock Rust project structure with various symbols
    let src_dir = env.temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create main.rs with functions and structs
    let main_content = r#"
use std::collections::HashMap;

/// Main application entry point
fn main() {
    println!("Hello, world!");
    let config = AppConfig::new();
    config.initialize();
}

/// Application configuration structure
pub struct AppConfig {
    pub settings: HashMap<String, String>,
    pub debug_mode: bool,
}

impl AppConfig {
    /// Create new application configuration
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
            debug_mode: false,
        }
    }
    
    /// Initialize the configuration
    pub async fn initialize(&self) {
        // Setup code here
    }
}

/// User data structure
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

/// User management trait
pub trait UserManager {
    fn create_user(&self, name: &str, email: &str) -> User;
    fn find_user(&self, id: u64) -> Option<User>;
}

/// Application constants
pub const MAX_USERS: usize = 1000;
pub const DEFAULT_TIMEOUT: u64 = 30;

/// Error types for the application
#[derive(Debug)]
pub enum AppError {
    ValidationError(String),
    DatabaseError(String),
    NetworkError,
}
"#;
    tokio::fs::write(src_dir.join("main.rs"), main_content).await?;

    // Create lib.rs with additional symbols
    let lib_content = r#"
pub mod database;
pub mod network;

/// Library initialization function
pub fn initialize_lib() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// Configuration trait for modules
pub trait Configurable {
    fn configure(&mut self, options: &ConfigOptions);
}

/// Configuration options structure
pub struct ConfigOptions {
    pub connection_timeout: u64,
    pub retry_count: u32,
}

/// Utility functions
pub mod utils {
    /// Helper function for string processing
    pub fn process_string(input: &str) -> String {
        input.trim().to_lowercase()
    }
    
    /// Format user display name
    pub fn format_display_name(first: &str, last: &str) -> String {
        format!("{} {}", first, last)
    }
}
"#;
    tokio::fs::write(src_dir.join("lib.rs"), lib_content).await?;

    // Create database.rs module
    let database_content = r#"
use std::collections::HashMap;

/// Database connection manager
pub struct DatabaseManager {
    connections: HashMap<String, Connection>,
}

impl DatabaseManager {
    /// Create new database manager
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
    
    /// Connect to database
    pub async fn connect(&mut self, name: &str, url: &str) -> Result<(), DbError> {
        // Connection logic here
        Ok(())
    }
}

/// Database connection structure
pub struct Connection {
    pub url: String,
    pub is_active: bool,
}

/// Database error types
#[derive(Debug)]
pub enum DbError {
    ConnectionFailed,
    QueryFailed(String),
    Timeout,
}
"#;
    tokio::fs::write(src_dir.join("database.rs"), database_content).await?;

    let tool = LspWorkspaceSymbolsTool;

    // Test 1: Search for "User" - should find User struct and UserManager trait
    let args = json!({
        "query": "User"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    assert_eq!(parsed["query"], "User");
    assert!(parsed["symbols"].is_array());
    
    let symbols = parsed["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty(), "Should find User-related symbols");
    
    // Verify we found both User struct and UserManager trait
    let symbol_names: Vec<&str> = symbols.iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    
    assert!(symbol_names.contains(&"User"), "Should find User struct");
    assert!(symbol_names.contains(&"UserManager"), "Should find UserManager trait");

    // Test 2: Search for "Config" - should find multiple config-related symbols
    let args = json!({
        "query": "Config"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    let symbols = parsed["symbols"].as_array().unwrap();
    let symbol_names: Vec<&str> = symbols.iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    
    // Should find AppConfig, Configurable, ConfigOptions
    assert!(symbol_names.iter().any(|&name| name.contains("Config")));

    // Test 3: Search for "connect" - should find connect function
    let args = json!({
        "query": "connect"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    let symbols = parsed["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty(), "Should find connect function");

    // Verify summary information
    let summary = &parsed["summary"];
    assert!(summary["total_symbols"].as_u64().unwrap() > 0);
    assert!(summary["files_searched"].as_u64().unwrap() >= 3); // main.rs, lib.rs, database.rs
    assert_eq!(summary["search_query"], "connect");
    
    // Verify symbol kind distribution
    let by_kind = &summary["by_kind"];
    assert!(by_kind.is_object());

    Ok(())
}

/// ✅ Test workspace symbols with empty query validation
#[tokio::test]
async fn test_workspace_symbols_empty_query() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;
    let tool = LspWorkspaceSymbolsTool;

    // Test empty string query
    let args = json!({
        "query": ""
    });
    
    let result = tool.execute(args, config).await;
    assert!(result.is_err(), "Empty query should be rejected");
    assert!(result.unwrap_err().to_string().contains("empty"));

    // Test whitespace-only query
    let args = json!({
        "query": "   "
    });
    
    let result = tool.execute(args, config).await;
    assert!(result.is_err(), "Whitespace-only query should be rejected");

    Ok(())
}

/// ✅ Test workspace symbols with missing query parameter
#[tokio::test]
async fn test_workspace_symbols_missing_query() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;
    let tool = LspWorkspaceSymbolsTool;

    // Test missing query parameter
    let args = json!({});
    
    let result = tool.execute(args, config).await;
    assert!(result.is_err(), "Missing query parameter should be rejected");

    Ok(())
}

/// ✅ Test workspace symbols with case-insensitive search
#[tokio::test]
async fn test_workspace_symbols_case_insensitive() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;

    // Create a simple Rust file with mixed case symbols
    let src_dir = env.temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    let content = r#"
pub struct MyStruct {
    field: String,
}

impl MyStruct {
    pub fn new() -> Self {
        Self { field: String::new() }
    }
}

pub fn my_function() {
    // function body
}
"#;
    tokio::fs::write(src_dir.join("main.rs"), content).await?;

    let tool = LspWorkspaceSymbolsTool;

    // Test lowercase search for uppercase symbol
    let args = json!({
        "query": "mystruct"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    let symbols = parsed["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty(), "Case-insensitive search should find MyStruct");
    
    let found_mystruct = symbols.iter().any(|s| 
        s["name"].as_str().unwrap_or("") == "MyStruct"
    );
    assert!(found_mystruct, "Should find MyStruct with lowercase search");

    // Test uppercase search for lowercase function
    let args = json!({
        "query": "MY_FUNCTION"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    let symbols = parsed["symbols"].as_array().unwrap();
    let found_function = symbols.iter().any(|s| 
        s["name"].as_str().unwrap_or("") == "my_function"
    );
    assert!(found_function, "Should find my_function with uppercase search");

    Ok(())
}

/// ✅ Test workspace symbols with complex project structure
#[tokio::test]
async fn test_workspace_symbols_complex_project() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;

    // Create a more complex project structure
    let src_dir = env.temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create nested modules
    let handlers_dir = src_dir.join("handlers");
    tokio::fs::create_dir_all(&handlers_dir).await?;
    
    let models_dir = src_dir.join("models");
    tokio::fs::create_dir_all(&models_dir).await?;

    // handlers/user.rs
    let user_handler = r#"
use crate::models::User;

pub struct UserHandler {
    db: Database,
}

impl UserHandler {
    pub fn create_user(&self, name: &str) -> Result<User, HandlerError> {
        // Implementation
        Ok(User { id: 1, name: name.to_string() })
    }
    
    pub async fn find_user_by_id(&self, id: u64) -> Option<User> {
        // Implementation
        None
    }
}

#[derive(Debug)]
pub enum HandlerError {
    ValidationError,
    DatabaseError,
}
"#;
    tokio::fs::write(handlers_dir.join("user.rs"), user_handler).await?;

    // models/user.rs
    let user_model = r#"
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
}

impl User {
    pub fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }
}

pub trait UserValidation {
    fn validate_name(&self, name: &str) -> bool;
}

impl UserValidation for User {
    fn validate_name(&self, name: &str) -> bool {
        !name.is_empty() && name.len() >= 2
    }
}
"#;
    tokio::fs::write(models_dir.join("user.rs"), user_model).await?;

    // main.rs that ties it together
    let main_content = r#"
mod handlers;
mod models;

use handlers::user::UserHandler;
use models::user::User;

fn main() {
    let handler = UserHandler::new();
    handler.find_user_by_id(1);
}
"#;
    tokio::fs::write(src_dir.join("main.rs"), main_content).await?;

    let tool = LspWorkspaceSymbolsTool;

    // Search for "User" across the complex structure
    let args = json!({
        "query": "User"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    let symbols = parsed["symbols"].as_array().unwrap();
    assert!(!symbols.is_empty(), "Should find User symbols across files");
    
    // Should find User struct, UserHandler, UserValidation, etc.
    let symbol_names: Vec<&str> = symbols.iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    
    // Verify we found symbols from different files
    assert!(symbol_names.contains(&"User"), "Should find User struct");
    assert!(symbol_names.contains(&"UserHandler"), "Should find UserHandler struct");
    assert!(symbol_names.contains(&"UserValidation"), "Should find UserValidation trait");
    
    // Verify file paths are relative and correct
    let file_paths: Vec<&str> = symbols.iter()
        .filter_map(|s| s["file_path"].as_str())
        .collect();
    
    assert!(file_paths.iter().any(|&path| path.contains("models")));
    assert!(file_paths.iter().any(|&path| path.contains("handlers")));

    // Verify summary shows multiple files searched
    let summary = &parsed["summary"];
    assert!(summary["files_searched"].as_u64().unwrap() >= 3);

    Ok(())
}

/// ✅ Test workspace symbols response format
#[tokio::test]
async fn test_workspace_symbols_response_format() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;

    // Create a simple test file
    let src_dir = env.temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    tokio::fs::write(src_dir.join("main.rs"), "fn test() {}").await?;

    let tool = LspWorkspaceSymbolsTool;
    let args = json!({
        "query": "test"
    });
    
    let result = tool.execute(args, config).await?;
    
    // Verify MCP response format
    assert!(result["content"].is_array());
    assert_eq!(result["content"].as_array().unwrap().len(), 1);
    assert_eq!(result["content"][0]["type"], "text");
    
    let response_text = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response_text)?;
    
    // Verify workspace symbols response structure
    assert!(parsed["query"].is_string());
    assert!(parsed["symbols"].is_array());
    assert!(parsed["summary"].is_object());
    
    // Verify summary structure
    let summary = &parsed["summary"];
    assert!(summary["total_symbols"].is_number());
    assert!(summary["query_matches"].is_number());
    assert!(summary["files_searched"].is_number());
    assert!(summary["search_query"].is_string());
    assert!(summary["by_kind"].is_object());
    
    // Verify symbol structure (if any symbols found)
    if let Some(symbols) = parsed["symbols"].as_array()
        && !symbols.is_empty()
    {
        let symbol = &symbols[0];
        assert!(symbol["name"].is_string());
        assert!(symbol["kind"].is_string());
        assert!(symbol["file_path"].is_string());
        assert!(symbol["line"].is_number());
        assert!(symbol["character"].is_number());
        assert!(symbol["end_line"].is_number());
        assert!(symbol["end_character"].is_number());
    }

    Ok(())
}

/// ✅ Test workspace symbols with no matches
#[tokio::test]
async fn test_workspace_symbols_no_matches() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let config = &env.config;

    // Create a simple test file
    let src_dir = env.temp_dir.path().join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    tokio::fs::write(src_dir.join("main.rs"), "fn hello() {}").await?;

    let tool = LspWorkspaceSymbolsTool;
    
    // Search for something that doesn't exist
    let args = json!({
        "query": "nonexistent_symbol_xyz"
    });
    
    let result = tool.execute(args, config).await?;
    let response = result["content"][0]["text"].as_str().unwrap();
    let parsed: Value = serde_json::from_str(response)?;
    
    // Should return empty results but valid structure
    assert_eq!(parsed["query"], "nonexistent_symbol_xyz");
    assert!(parsed["symbols"].is_array());
    assert_eq!(parsed["symbols"].as_array().unwrap().len(), 0);
    
    let summary = &parsed["summary"];
    assert_eq!(summary["total_symbols"].as_u64().unwrap(), 0);
    assert_eq!(summary["query_matches"].as_u64().unwrap(), 0);
    assert!(summary["files_searched"].as_u64().unwrap() >= 1);

    Ok(())
}
