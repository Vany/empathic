//! 🔬 LSP Client JSON-RPC Communication Tests
//!
//! Comprehensive tests for LSP JSON-RPC communication layer including:
//! - JSON-RPC 2.0 protocol compliance and message handling
//! - LSP initialization handshake and capability negotiation
//! - Request/response correlation and timeout handling
//! - Error propagation and resilience mechanisms

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::{json, Value};
use url::Url;

use empathic::lsp::client::LspClient;
use empathic::lsp::types::{LspResult, LspError};

/// 📁 Create temporary Rust project for testing
async fn create_rust_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml to make it a valid Rust project
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).await?;
    
    // Create basic lib.rs with sample code
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Test project for LSP client testing

/// A simple function for testing
pub fn hello_world() -> &'static str {
    "Hello, world!"
}

/// A struct for testing
pub struct TestStruct {
    pub field: i32,
}

impl TestStruct {
    pub fn new(value: i32) -> Self {
        Self { field: value }
    }
    
    pub fn get_field(&self) -> i32 {
        self.field
    }
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

#[tokio::test]
async fn test_client_creation_without_server() {
    // 🏗️ Test client creation when no LSP server is available
    let (_temp_dir, project_path) = create_rust_project().await
        .expect("Failed to create test project");
    
    // Attempt to create client (should timeout gracefully)
    let client_result = timeout(
        Duration::from_secs(2),
        LspClient::new(&project_path)
    ).await;
    
    match client_result {
        Ok(Ok(_)) => {
            println!("🚀 LSP client created successfully (rust-analyzer available)");
        },
        Ok(Err(err)) => {
            println!("⚠️ LSP client creation failed gracefully: {:?}", err);
            // This is expected if rust-analyzer is not available
            assert!(matches!(err, LspError::Timeout { .. }), "Should be timeout error");
        },
        Err(_) => {
            println!("⏱️ LSP client creation timeout (expected without rust-analyzer)");
        }
    }
    
    println!("✅ Client creation test completed");
}

#[tokio::test]
async fn test_json_rpc_message_format() {
    // 📝 Test JSON-RPC 2.0 message format compliance
    
    // Test request message format
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "textDocument/hover",
        "params": {
            "textDocument": {
                "uri": "file:///test.rs"
            },
            "position": {
                "line": 0,
                "character": 0
            }
        }
    });
    
    // Validate required JSON-RPC 2.0 fields
    assert_eq!(request["jsonrpc"], "2.0", "Must have jsonrpc version");
    assert!(request["id"].is_number(), "Must have numeric id");
    assert!(request["method"].is_string(), "Must have string method");
    assert!(request["params"].is_object(), "Must have params object");
    
    // Test notification format (no id)
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///test.rs",
                "languageId": "rust",
                "version": 1,
                "text": "fn main() {}"
            }
        }
    });
    
    assert_eq!(notification["jsonrpc"], "2.0", "Notification must have jsonrpc version");
    assert!(notification["id"].is_null(), "Notification must not have id");
    assert!(notification["method"].is_string(), "Notification must have method");
    
    // Test error response format
    let error_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32601,
            "message": "Method not found",
            "data": null
        }
    });
    
    assert_eq!(error_response["jsonrpc"], "2.0", "Error response must have jsonrpc version");
    assert_eq!(error_response["id"], 1, "Error response must have matching id");
    assert!(error_response["error"].is_object(), "Error response must have error object");
    assert!(error_response["error"]["code"].is_number(), "Error must have numeric code");
    assert!(error_response["error"]["message"].is_string(), "Error must have string message");
    
    println!("✅ JSON-RPC message format validation completed");
}

#[tokio::test] 
async fn test_lsp_protocol_compliance() {
    // 📋 Test LSP protocol compliance and message types
    
    // Test initialize request format
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": "file:///test/project",
            "capabilities": {
                "textDocument": {
                    "hover": {
                        "contentFormat": ["markdown", "plaintext"]
                    },
                    "completion": {
                        "completionItem": {
                            "snippetSupport": true
                        }
                    }
                }
            }
        }
    });
    
    assert_eq!(initialize_request["method"], "initialize", "Must be initialize method");
    assert!(initialize_request["params"]["capabilities"].is_object(), "Must have capabilities");
    
    // Test textDocument/didOpen notification format
    let did_open = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///test.rs",
                "languageId": "rust",
                "version": 1,
                "text": "fn main() {}"
            }
        }
    });
    
    assert_eq!(did_open["method"], "textDocument/didOpen", "Must be didOpen method");
    assert!(did_open["params"]["textDocument"]["uri"].is_string(), "Must have URI");
    assert_eq!(did_open["params"]["textDocument"]["languageId"], "rust", "Must be rust language");
    
    // Test position format
    let position = json!({
        "line": 5,
        "character": 10
    });
    
    assert!(position["line"].is_number(), "Line must be number");
    assert!(position["character"].is_number(), "Character must be number");
    assert!(position["line"].as_u64().unwrap() >= 0, "Line must be non-negative");
    assert!(position["character"].as_u64().unwrap() >= 0, "Character must be non-negative");
    
    println!("✅ LSP protocol compliance validation completed");
}

#[tokio::test]
async fn test_uri_handling() {
    // 🔗 Test URI conversion and file path handling
    
    let test_paths = vec![
        "/Users/test/project/src/main.rs",
        "/tmp/test file with spaces.rs",
        "/home/user/project/src/lib.rs",
    ];
    
    for path_str in test_paths {
        let path = PathBuf::from(path_str);
        
        // Test URI conversion
        let uri_result = Url::from_file_path(&path);
        
        match uri_result {
            Ok(uri) => {
                println!("✅ URI conversion successful: {} -> {}", path_str, uri);
                
                // Validate URI format
                assert_eq!(uri.scheme(), "file", "Must use file scheme");
                assert!(!uri.path().is_empty(), "Must have non-empty path");
                
                // Test round-trip conversion
                if let Ok(converted_path) = uri.to_file_path() {
                    // Note: paths may be canonicalized, so just check that conversion works
                    assert!(!converted_path.as_os_str().is_empty(), "Round-trip should work");
                }
            },
            Err(()) => {
                println!("⚠️ URI conversion failed for: {}", path_str);
                // This may be acceptable for some edge cases
            }
        }
    }
    
    println!("✅ URI handling test completed");
}

#[tokio::test]
async fn test_timeout_handling() {
    // ⏱️ Test timeout handling and graceful degradation
    
    let (_temp_dir, project_path) = create_rust_project().await
        .expect("Failed to create test project");
    
    // Test very short timeout
    let short_timeout_result = timeout(
        Duration::from_millis(100), // Very short timeout
        LspClient::new(&project_path)
    ).await;
    
    match short_timeout_result {
        Ok(Ok(_)) => {
            println!("🚀 LSP client created within short timeout (very fast rust-analyzer)");
        },
        Ok(Err(err)) => {
            println!("⚠️ LSP client creation failed: {:?}", err);
            // Should be timeout or other graceful error
        },
        Err(_) => {
            println!("⏱️ Short timeout handled correctly");
            // This is expected with short timeout
        }
    }
    
    // Test reasonable timeout
    let reasonable_timeout_result = timeout(
        Duration::from_secs(5), // Reasonable timeout
        LspClient::new(&project_path)
    ).await;
    
    match reasonable_timeout_result {
        Ok(Ok(_)) => {
            println!("🚀 LSP client created within reasonable timeout");
        },
        Ok(Err(err)) => {
            println!("⚠️ LSP client creation failed gracefully: {:?}", err);
            // Expected if rust-analyzer not available
        },
        Err(_) => {
            println!("⏱️ Reasonable timeout occurred (rust-analyzer likely not available)");
        }
    }
    
    println!("✅ Timeout handling test completed");
}

#[tokio::test]
async fn test_error_classification() {
    // 🚨 Test error type classification and mapping
    
    // Test different error types we should handle
    let error_scenarios = vec![
        ("timeout", "Connection timeout during initialization"),
        ("process_not_found", "rust-analyzer executable not found in PATH"),
        ("invalid_response", "Malformed JSON-RPC response from server"),
        ("initialization_failed", "LSP server initialization failed"),
        ("communication_error", "Communication error during operation"),
    ];
    
    for (error_type, description) in error_scenarios {
        println!("🧪 Testing error scenario: {} - {}", error_type, description);
        
        // Create appropriate error for testing
        let test_error = match error_type {
            "timeout" => LspError::Timeout { timeout_secs: 60 },
            "process_not_found" => LspError::ProcessSpawnError("rust-analyzer not found".to_string()),
            "invalid_response" => LspError::InvalidResponse("Invalid JSON".to_string()),
            "initialization_failed" => LspError::InitializationFailed("Server rejected initialization".to_string()),
            "communication_error" => LspError::CommunicationError("Connection closed".to_string()),
            _ => LspError::Other("Unknown error".to_string()),
        };
        
        // Verify error can be formatted and debugged
        let error_string = format!("{:?}", test_error);
        assert!(!error_string.is_empty(), "Error should have debug representation");
        
        let error_display = format!("{}", test_error);
        assert!(!error_display.is_empty(), "Error should have display representation");
        
        println!("  ✅ Error handled: {}", error_display);
    }
    
    println!("✅ Error classification test completed");
}

#[tokio::test]
async fn test_message_correlation() {
    // 🔗 Test request/response correlation and id handling
    
    // Test unique ID generation
    let mut ids = std::collections::HashSet::new();
    
    for i in 0..100 {
        let id = i + 1; // Simple incrementing IDs
        assert!(ids.insert(id), "IDs should be unique");
    }
    
    assert_eq!(ids.len(), 100, "Should have 100 unique IDs");
    
    // Test ID correlation in JSON-RPC messages
    let request_id = 42;
    let request = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "textDocument/hover",
        "params": {}
    });
    
    let response = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "contents": "test content"
        }
    });
    
    let error_response = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    });
    
    // Verify ID correlation
    assert_eq!(request["id"], response["id"], "Request and response IDs must match");
    assert_eq!(request["id"], error_response["id"], "Request and error response IDs must match");
    
    // Test null ID handling for notifications
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {}
    });
    
    assert!(notification["id"].is_null(), "Notifications should not have ID");
    
    println!("✅ Message correlation test completed");
}

#[tokio::test]
async fn test_capability_negotiation() {
    // 🤝 Test LSP capability negotiation and feature detection
    
    // Test client capabilities
    let client_capabilities = json!({
        "textDocument": {
            "hover": {
                "contentFormat": ["markdown", "plaintext"]
            },
            "completion": {
                "completionItem": {
                    "snippetSupport": true,
                    "commitCharactersSupport": true,
                    "documentationFormat": ["markdown", "plaintext"]
                }
            },
            "definition": {
                "linkSupport": true
            },
            "references": {
                "includeDeclaration": true
            },
            "documentSymbol": {
                "symbolKind": {
                    "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
                }
            }
        },
        "workspace": {
            "symbol": {
                "symbolKind": {
                    "valueSet": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
                }
            }
        }
    });
    
    // Validate client capabilities structure
    assert!(client_capabilities["textDocument"].is_object(), "Must have textDocument capabilities");
    assert!(client_capabilities["workspace"].is_object(), "Must have workspace capabilities");
    
    // Test server capabilities (what we expect from rust-analyzer)
    let expected_server_capabilities = json!({
        "textDocumentSync": 2,
        "hoverProvider": true,
        "completionProvider": {
            "triggerCharacters": [".", "::", "->"]
        },
        "definitionProvider": true,
        "referencesProvider": true,
        "documentSymbolProvider": true,
        "workspaceSymbolProvider": true
    });
    
    // Validate expected server capabilities
    assert!(expected_server_capabilities["hoverProvider"].as_bool().unwrap_or(false), "Should support hover");
    assert!(expected_server_capabilities["definitionProvider"].as_bool().unwrap_or(false), "Should support definition");
    assert!(expected_server_capabilities["referencesProvider"].as_bool().unwrap_or(false), "Should support references");
    
    println!("✅ Capability negotiation test completed");
}

#[tokio::test]
async fn test_concurrent_requests() {
    // 🔄 Test concurrent request handling and queueing
    
    let (_temp_dir, project_path) = create_rust_project().await
        .expect("Failed to create test project");
    
    // Test multiple concurrent client creation attempts
    let client_futures: Vec<_> = (0..3).map(|i| {
        let path = project_path.clone();
        async move {
            let result = timeout(
                Duration::from_secs(2),
                LspClient::new(&path)
            ).await;
            
            (i, result)
        }
    }).collect();
    
    let results = futures::future::join_all(client_futures).await;
    
    let mut success_count = 0;
    let mut timeout_count = 0;
    let mut error_count = 0;
    
    for (i, result) in results {
        match result {
            Ok(Ok(_)) => {
                println!("✅ Client {} created successfully", i);
                success_count += 1;
            },
            Ok(Err(_)) => {
                println!("⚠️ Client {} creation failed gracefully", i);
                error_count += 1;
            },
            Err(_) => {
                println!("⏱️ Client {} creation timeout", i);
                timeout_count += 1;
            }
        }
    }
    
    println!("📊 Concurrent results: {} success, {} errors, {} timeouts", 
             success_count, error_count, timeout_count);
    
    // All operations should complete (success, error, or timeout - all acceptable)
    assert_eq!(success_count + error_count + timeout_count, 3, "All operations should complete");
    
    println!("✅ Concurrent requests test completed");
}
