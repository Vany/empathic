//! 🦀 Tests for LSP Completion Tool

use empathic::config::Config;
use empathic::tools::lsp::completion::{LspCompletionTool, CompletionOutput};
use empathic::tools::Tool;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn test_completion_schema() {
    let tool = LspCompletionTool;
    let schema = tool.schema();
    
    assert!(schema["properties"]["file_path"]["type"].as_str() == Some("string"));
    assert!(schema["properties"]["line"]["type"].as_str() == Some("integer"));
    assert!(schema["properties"]["character"]["type"].as_str() == Some("integer"));
    assert!(schema["required"].as_array().unwrap().contains(&json!("file_path")));
    assert!(schema["required"].as_array().unwrap().contains(&json!("line")));
    assert!(schema["required"].as_array().unwrap().contains(&json!("character")));
}

#[tokio::test]
async fn test_completion_file_validation() {
    let tool = LspCompletionTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Test non-existent file
    let args = json!({"file_path": "nonexistent.rs", "line": 0, "character": 0, "project": "test"});
    let result = tool.execute(args, &config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_completion_rust_file_only() {
    let tool = LspCompletionTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a non-Rust file within the project directory  
    let file_path = "test.txt";
    let full_path = temp_dir.path().join(file_path);
    std::fs::write(&full_path, "hello world").unwrap();
    
    let args = json!({"file_path": file_path, "line": 0, "character": 0, "project": "test"});
    let result = tool.execute(args, &config).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("file type") || error_msg.contains("expecting .rs file"));
}

#[tokio::test]
async fn test_completion_mock_success() {
    let tool = LspCompletionTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a simple Rust file within the project directory
    let file_path = "main.rs";
    let full_path = temp_dir.path().join(file_path);
    std::fs::write(&full_path, "fn main() {\n    pri\n}").unwrap();
    
    // Test completion for 'pri' (should suggest println!)
    let args = json!({"file_path": file_path, "line": 1, "character": 7, "project": "test"});
    let result = tool.execute(args, &config).await;
    
    // Handle both success and LSP-related failures gracefully
    match result {
        Ok(response) => {
            // If LSP works, verify the response format
            assert!(response["content"][0]["text"].is_string());
            
            let text = response["content"][0]["text"].as_str().unwrap();
            let output: CompletionOutput = serde_json::from_str(text).unwrap();
            
            assert_eq!(output.position.line, 1);
            assert_eq!(output.position.character, 7);
            assert_eq!(output.context.current_word, "pri");
            assert!(!output.completions.is_empty());
            
            // Should have println! in completions
            let has_println = output.completions.iter().any(|c| c.label.contains("println"));
            assert!(has_println, "Expected println! in completions");
        }
        Err(e) => {
            // If LSP fails (e.g., rust-analyzer not available), that's okay for this test
            // Just ensure it's an LSP-related error, not a fundamental issue
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
