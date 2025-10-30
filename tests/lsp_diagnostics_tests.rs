//! 🩺 Tests for LSP Diagnostics Tool

use empathic::config::Config;
use empathic::tools::lsp::diagnostics::LspDiagnosticsTool;
use empathic::tools::Tool;
use empathic::lsp::ProjectDetector;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn test_diagnostics_schema() {
    let tool = LspDiagnosticsTool;
    let schema = tool.schema();
    
    assert!(schema["properties"]["file_path"]["type"].as_str() == Some("string"));
    assert!(schema["required"].as_array().unwrap().contains(&json!("file_path")));
}

#[tokio::test]
async fn test_diagnostics_file_validation() {
    let tool = LspDiagnosticsTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Test non-existent file
    let args = json!({"file_path": "nonexistent.rs"});
    let result = tool.execute(args, &config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_diagnostics_rust_file_only() {
    let tool = LspDiagnosticsTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a non-Rust file
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "hello world").unwrap();
    
    let args = json!({"file_path": file_path.to_string_lossy()});
    let result = tool.execute(args, &config).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("only supports Rust files"));
}

#[tokio::test]
async fn test_project_detector() {
    let temp_dir = tempdir().unwrap();
    
    // Create a proper Rust project structure
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    std::fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#).unwrap();
    
    // Test ProjectDetector directly
    let detector = ProjectDetector::new(temp_dir.path().to_path_buf());
    let projects = detector.find_rust_projects().unwrap();
    
    println!("🔍 ProjectDetector found {} projects", projects.len());
    for project in &projects {
        println!("  📦 {}: {}", 
            project.name.as_deref().unwrap_or("unnamed"), 
            project.root_path.display()
        );
    }
    
    assert!(!projects.is_empty(), "ProjectDetector should find the test project");
    assert_eq!(projects[0].name, Some("test-project".to_string()));
}

#[tokio::test]
async fn test_diagnostics_success() {
    let tool = LspDiagnosticsTool;
    let temp_dir = tempdir().unwrap();
    let config = Config::new(temp_dir.path().to_path_buf());
    
    // Create a proper Rust project structure
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    std::fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#).unwrap();
    
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    
    let main_rs = src_dir.join("main.rs");
    std::fs::write(&main_rs, "fn main() {\n    println!(\"Hello, world!\");\n}").unwrap();
    
    let args = json!({"file_path": main_rs.to_string_lossy()});
    let result = tool.execute(args, &config).await;
    
    assert!(result.is_ok(), "Expected success but got error: {:?}", result.err());
    
    let response = result.unwrap();
    assert!(response["content"][0]["text"].is_string());
}
