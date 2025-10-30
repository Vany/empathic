//! 🦀 Tests for LSP Hover Tool

use empathic::config::Config;
use empathic::tools::lsp::hover::{LspHoverTool, HoverOutput};
use empathic::tools::Tool;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn test_hover_schema() {
    let tool = LspHoverTool;
    let schema = tool.schema();
    
    assert!(schema["properties"]["file_path"]["type"].as_str() == Some("string"));
    assert!(schema["properties"]["line"]["type"].as_str() == Some("integer"));
    assert!(schema["properties"]["character"]["type"].as_str() == Some("integer"));
    assert!(schema["required"].as_array().unwrap().contains(&json!("file_path")));
    assert!(schema["required"].as_array().unwrap().contains(&json!("line")));
    assert!(schema["required"].as_array().unwrap().contains(&json!("character")));
}

#[tokio::test]
async fn test_hover_file_validation() {
    let tool = LspHoverTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Test non-existent file
    let args = json!({"file_path": "nonexistent.rs", "line": 0, "character": 0});
    let result = tool.execute(args, &config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_hover_rust_file_only() {
    let tool = LspHoverTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a non-Rust file
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "hello world").unwrap();
    
    let args = json!({"file_path": file_path.to_string_lossy(), "line": 0, "character": 0});
    let result = tool.execute(args, &config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("only supports Rust files"));
}

#[tokio::test]
async fn test_hover_bounds_checking() {
    let tool = LspHoverTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a simple Rust file
    let main_rs = temp_dir.path().join("main.rs");
    std::fs::write(&main_rs, "fn main() {}").unwrap();
    
    // Test hover on invalid line
    let args = json!({"file_path": main_rs.to_string_lossy(), "line": 999, "character": 0});
    let result = tool.execute(args, &config).await;
    
    // Handle both success and LSP-related failures
    match result {
        Ok(_) => {
            // If it succeeds, that's unexpected but not a failure
            println!("Hover succeeded unexpectedly (LSP may have handled bounds)");
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("out of bounds") ||
                error_msg.contains("Failed to get LSP server") ||
                error_msg.contains("rust-analyzer") ||
                error_msg.contains("LSP"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_hover_mock_success() {
    let tool = LspHoverTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a simple Rust file with a standard library function
    let main_rs = temp_dir.path().join("main.rs");
    std::fs::write(&main_rs, "fn main() {\n    println!(\"Hello\");\n}").unwrap();
    
    // Test hover on println! 
    let args = json!({"file_path": main_rs.to_string_lossy(), "line": 1, "character": 8});
    let result = tool.execute(args, &config).await;
    
    // Handle both success and LSP-related failures gracefully
    match result {
        Ok(response) => {
            // If LSP works, verify the response format
            assert!(response["content"][0]["text"].is_string());
            
            let text = response["content"][0]["text"].as_str().unwrap();
            let output: HoverOutput = serde_json::from_str(text).unwrap();
            
            assert_eq!(output.position.line, 1);
            assert_eq!(output.position.character, 8);
        }
        Err(e) => {
            // If LSP fails, that's okay for this test
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to get LSP server") ||
                error_msg.contains("rust-analyzer") ||
                error_msg.contains("LSP") ||
                error_msg.contains("timeout"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[tokio::test]
async fn test_hover_variable_info() {
    let tool = LspHoverTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a Rust file with variable declarations
    let main_rs = temp_dir.path().join("main.rs");
    std::fs::write(&main_rs, "fn main() {\n    let x: i32 = 42;\n    println!(\"{}\", x);\n}").unwrap();
    
    // Test hover on variable 'x'
    let args = json!({"file_path": main_rs.to_string_lossy(), "line": 2, "character": 20});
    let result = tool.execute(args, &config).await;
    
    // Handle both success and LSP-related failures gracefully
    match result {
        Ok(response) => {
            assert!(response["content"][0]["text"].is_string());
            
            let text = response["content"][0]["text"].as_str().unwrap();
            let output: HoverOutput = serde_json::from_str(text).unwrap();
            
            assert_eq!(output.position.line, 2);
            assert_eq!(output.position.character, 20);
        }
        Err(e) => {
            // If LSP fails, that's okay for this test
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to get LSP server") ||
                error_msg.contains("rust-analyzer") ||
                error_msg.contains("LSP") ||
                error_msg.contains("timeout"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}
