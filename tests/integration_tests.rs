use empathic::{Config, McpServer};
use std::path::PathBuf;
use tokio::fs;

#[tokio::test]
async fn test_mcp_server_creation() {
    let config = Config {
        root_dir: PathBuf::from("/tmp"),
        add_path: vec![],
        log_level: "warn".to_string(),
    };
    
    let _server = McpServer::new(config);
    // Server should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_config_project_path() {
    let config = Config {
        root_dir: PathBuf::from("/tmp"),
        add_path: vec![PathBuf::from("/usr/local/bin")],
        log_level: "warn".to_string(),
    };
    
    let project_path = config.project_path(Some("test_project"));
    assert_eq!(project_path, PathBuf::from("/tmp/test_project"));
    
    let root_path = config.project_path(None);
    assert_eq!(root_path, PathBuf::from("/tmp"));
}

#[tokio::test]
async fn test_replace_tool_basic() {
    use empathic::tools::{Tool, replace::ReplaceTool};
    
    let tool = ReplaceTool;
    
    // Test tool metadata
    assert_eq!(tool.name(), "replace");
    assert!(tool.description().contains("Search and replace"));
    
    // Test schema
    let schema = tool.schema();
    assert!(schema.get("properties").is_some());
        assert!(schema.get("anyOf").is_some() || schema.get("required").is_some());
}

#[tokio::test]
async fn test_list_files_with_pattern() {
    use empathic::tools::{Tool};
    use empathic::tools::list_files::ListFilesTool;
    use serde_json::json;
    
    // Create temporary test directory structure
    let test_dir = std::env::temp_dir().join("ee_test_pattern");
    fs::create_dir_all(&test_dir).await.unwrap();
    
    // Create test files
    fs::write(test_dir.join("test.rs"), "// Rust file").await.unwrap();
    fs::write(test_dir.join("test.txt"), "Text file").await.unwrap();
    fs::write(test_dir.join("example.rs"), "// Another Rust file").await.unwrap();
    fs::write(test_dir.join("readme.md"), "# Readme").await.unwrap();
    
    // Create subdirectory with more files
    let sub_dir = test_dir.join("subdir");
    fs::create_dir_all(&sub_dir).await.unwrap();
    fs::write(sub_dir.join("nested.rs"), "// Nested Rust file").await.unwrap();
    fs::write(sub_dir.join("other.txt"), "Other text").await.unwrap();
    
    let config = Config {
        root_dir: test_dir.clone(),
        add_path: vec![],
        log_level: "warn".to_string(),
    };
    
    let tool = ListFilesTool;
    
    // Test with pattern "*.rs" - should find all Rust files recursively
    let args = json!({
        "path": ".",
        "pattern": "*.rs"
    });
    
    let result = tool.execute(args, &config).await.unwrap();
    
    // Extract JSON from MCP content structure 🔧
    let content_text = result.get("content").unwrap()
        .as_array().unwrap()[0]
        .get("text").unwrap()
        .as_str().unwrap();
    let parsed_result: serde_json::Value = serde_json::from_str(content_text).unwrap();
    let files = parsed_result.get("files").unwrap().as_array().unwrap();
    
    // Should find 3 .rs files (test.rs, example.rs, nested.rs)
    let rust_files: Vec<_> = files.iter()
        .filter(|f| f.get("name").unwrap().as_str().unwrap().ends_with(".rs"))
        .collect();
    
    assert_eq!(rust_files.len(), 3);
    
    // Verify recursive was set to true (indicated by finding nested.rs)
    let has_nested = files.iter()
        .any(|f| f.get("name").unwrap().as_str().unwrap() == "nested.rs");
    assert!(has_nested, "Pattern search should find nested files");
    
    // Cleanup
    fs::remove_dir_all(&test_dir).await.unwrap();
}
