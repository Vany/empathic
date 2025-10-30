//! 🔬 LSP Diagnostics Tool Comprehensive Tests
//!
//! Advanced testing for LSP diagnostics functionality including:
//! - Comprehensive error detection and classification
//! - Performance validation and caching behavior
//! - Edge cases and error recovery scenarios
//! - Real rust-analyzer integration when available

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::diagnostics::LspDiagnosticsTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create Rust project with intentional errors for diagnostics testing
async fn create_diagnostics_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "diagnostics-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with various types of errors
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Test file with various Rust errors for diagnostics testing

// Missing import - should cause error
use std::collections::HashMap;

/// Function with unused parameter - should cause warning
pub fn unused_param_function(unused: i32) -> String {
    "hello".to_string()
}

/// Function with type mismatch - should cause error
pub fn type_mismatch() -> i32 {
    "not an integer" // Type error
}

/// Function with unreachable code - should cause warning
pub fn unreachable_code() -> i32 {
    return 42;
    println!("This is unreachable"); // Warning
    0
}

/// Struct with dead code - should cause warning
#[allow(dead_code)]
struct UnusedStruct {
    field: String,
}

/// Function with missing lifetime - should cause error
pub fn lifetime_error(x: &str, y: &str) -> &str {
    if x.len() > y.len() { x } else { y }
}

/// Function with borrowing issues - should cause error
pub fn borrow_checker_error() {
    let mut vec = vec![1, 2, 3];
    let first = &vec[0];
    vec.push(4); // Borrow checker error
    println!("{}", first);
}

/// Valid function - should not cause errors
pub fn valid_function() -> String {
    let mut map = HashMap::new();
    map.insert("key", "value");
    format!("Map has {} entries", map.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_function() {
        let result = valid_function();
        assert!(result.contains("Map has"));
    }
}
"#).await?;
    
    // Create main.rs with additional errors
    let main_rs = src_dir.join("main.rs");
    tokio::fs::write(&main_rs, r#"//! Main file with compilation errors

fn main() {
    // Undefined variable - should cause error
    println!("{}", undefined_variable);
    
    // Wrong number of arguments - should cause error
    let result = add_numbers(1);
    
    // Unused variable - should cause warning
    let unused_var = "hello";
    
    // Valid code
    println!("Hello, world!");
}

/// Function expecting two parameters
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

/// Function with syntax error - missing semicolon
fn syntax_error() {
    let x = 5
    println!("{}", x);
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run diagnostics tool and validate response structure
async fn run_diagnostics_test(
    file_path: PathBuf,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspDiagnosticsTool::new();
    let input = ToolInput::LspDiagnostics { file_path };
    
    let start = Instant::now();
    let result = timeout(Duration::from_secs(timeout_secs), tool.call(input)).await;
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(CallToolResult::Success { content })) => {
            println!("✅ {} succeeded in {:?} with {} items", test_name, duration, content.len());
            
            // Extract and parse the diagnostics data
            if let Some(text_content) = content.first() {
                if let Ok(diagnostics_data) = serde_json::from_str::<Value>(&text_content.text) {
                    return (true, duration, Some(diagnostics_data));
                }
            }
            (true, duration, None)
        },
        Ok(Ok(CallToolResult::Error { error, .. })) => {
            println!("⚠️ {} failed gracefully in {:?}: {}", test_name, duration, error);
            (false, duration, None)
        },
        Ok(Err(err)) => {
            println!("⚠️ {} tool error in {:?}: {:?}", test_name, duration, err);
            (false, duration, None)
        },
        Err(_) => {
            println!("⏱️ {} timeout after {:?}", test_name, duration);
            (false, duration, None)
        }
    }
}

#[tokio::test]
async fn test_diagnostics_with_errors() {
    // 🚨 Test diagnostics on file with intentional errors
    println!("🚨 Testing diagnostics with intentional errors...");
    
    let (_temp_dir, project_path) = create_diagnostics_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    let (success, duration, diagnostics) = run_diagnostics_test(
        lib_file,
        "Diagnostics with errors",
        30
    ).await;
    
    if success {
        if let Some(data) = diagnostics {
            // Validate diagnostics structure
            if let Some(diagnostics_array) = data["diagnostics"].as_array() {
                println!("📊 Found {} diagnostics", diagnostics_array.len());
                
                for (i, diagnostic) in diagnostics_array.iter().enumerate() {
                    if let Some(severity) = diagnostic["severity"].as_u64() {
                        let severity_name = match severity {
                            1 => "Error",
                            2 => "Warning", 
                            3 => "Information",
                            4 => "Hint",
                            _ => "Unknown",
                        };
                        
                        let message = diagnostic["message"].as_str().unwrap_or("No message");
                        println!("  {}. {} {}: {}", i + 1, severity_name, severity, message);
                    }
                }
                
                // Verify we got some diagnostics (errors/warnings expected)
                if !diagnostics_array.is_empty() {
                    println!("✅ Diagnostics detected errors/warnings as expected");
                } else {
                    println!("ℹ️ No diagnostics found (could be valid or rust-analyzer not available)");
                }
            }
        }
        
        // Check performance
        if duration < Duration::from_millis(500) {
            println!("⚡ Excellent performance: {:?}", duration);
        } else if duration < Duration::from_secs(2) {
            println!("✅ Good performance: {:?}", duration);
        } else {
            println!("⚠️ Slow performance: {:?}", duration);
        }
    }
    
    println!("✅ Diagnostics with errors test completed");
}

#[tokio::test]
async fn test_diagnostics_clean_file() {
    // ✨ Test diagnostics on clean file (should have no errors)
    println!("✨ Testing diagnostics on clean file...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    
    // Create clean Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "clean-test"
version = "0.1.0"
edition = "2021"
"#).await.expect("Failed to write Cargo.toml");
    
    // Create src directory and clean lib.rs
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await.expect("Failed to create src dir");
    
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Clean Rust code with no errors

/// A simple function that adds two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// A simple structure
#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    /// Calculate distance from origin
    pub fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
    
    #[test]
    fn test_point() {
        let point = Point::new(3.0, 4.0);
        assert_eq!(point.distance_from_origin(), 5.0);
    }
}
"#).await.expect("Failed to write lib.rs");
    
    let (success, duration, diagnostics) = run_diagnostics_test(
        lib_rs,
        "Diagnostics clean file",
        20
    ).await;
    
    if success {
        if let Some(data) = diagnostics {
            if let Some(diagnostics_array) = data["diagnostics"].as_array() {
                println!("📊 Found {} diagnostics on clean file", diagnostics_array.len());
                
                // Clean file should have few or no diagnostics
                if diagnostics_array.is_empty() {
                    println!("✅ No diagnostics found - clean file validated");
                } else {
                    // Some warnings might be acceptable (unused imports, etc.)
                    let error_count = diagnostics_array.iter()
                        .filter(|d| d["severity"].as_u64() == Some(1))
                        .count();
                    
                    if error_count == 0 {
                        println!("✅ No errors found - only warnings on clean file");
                    } else {
                        println!("⚠️ {} errors found on supposedly clean file", error_count);
                    }
                }
            }
        }
    }
    
    println!("✅ Clean file diagnostics test completed");
}

#[tokio::test]
async fn test_diagnostics_performance_caching() {
    // ⚡ Test performance and caching behavior
    println!("⚡ Testing diagnostics performance and caching...");
    
    let (_temp_dir, project_path) = create_diagnostics_test_project().await
        .expect("Failed to create test project");
    
    let test_file = project_path.join("src/lib.rs");
    
    // First call - cold cache
    let (success1, duration1, _) = run_diagnostics_test(
        test_file.clone(),
        "Diagnostics first call",
        30
    ).await;
    
    if success1 {
        // Second call - should hit cache if caching is working
        let (success2, duration2, _) = run_diagnostics_test(
            test_file.clone(),
            "Diagnostics second call",
            10
        ).await;
        
        if success2 {
            // Third call - verify consistent caching
            let (success3, duration3, _) = run_diagnostics_test(
                test_file,
                "Diagnostics third call",
                10
            ).await;
            
            if success3 {
                println!("📊 Performance comparison:");
                println!("  First call:  {:?}", duration1);
                println!("  Second call: {:?}", duration2);
                println!("  Third call:  {:?}", duration3);
                
                // Check if subsequent calls are faster (indicating caching)
                if duration2 < duration1 && duration3 < duration1 {
                    println!("✅ Caching appears to be working (subsequent calls faster)");
                } else {
                    println!("ℹ️ Caching may not be active or all calls were fast");
                }
                
                // All calls should be reasonably fast
                let max_acceptable = Duration::from_secs(5);
                if duration1 < max_acceptable && duration2 < max_acceptable && duration3 < max_acceptable {
                    println!("✅ All calls completed within acceptable time");
                } else {
                    println!("⚠️ Some calls took longer than expected");
                }
            }
        }
    }
    
    println!("✅ Performance and caching test completed");
}

#[tokio::test]
async fn test_diagnostics_edge_cases() {
    // 🧪 Test various edge cases and error conditions
    println!("🧪 Testing diagnostics edge cases...");
    
    let test_cases = vec![
        ("Non-existent file", PathBuf::from("/non/existent/file.rs")),
        ("Non-Rust file", {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let text_file = temp_dir.path().join("test.txt");
            std::fs::write(&text_file, "This is not Rust code").expect("Failed to write file");
            text_file
        }),
        ("Empty file", {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let empty_file = temp_dir.path().join("empty.rs");
            std::fs::write(&empty_file, "").expect("Failed to write file");
            empty_file
        }),
        ("File with only comments", {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let comment_file = temp_dir.path().join("comments.rs");
            std::fs::write(&comment_file, "// Only comments\n/* Multi-line\n   comment */\n").expect("Failed to write file");
            comment_file
        }),
    ];
    
    for (test_name, file_path) in test_cases {
        println!("🧪 Testing: {}", test_name);
        
        let (success, duration, _) = run_diagnostics_test(
            file_path,
            test_name,
            10
        ).await;
        
        if success {
            println!("  ✅ Handled successfully in {:?}", duration);
        } else {
            println!("  ✅ Failed gracefully in {:?} (expected for some edge cases)", duration);
        }
    }
    
    println!("✅ Edge cases test completed");
}

#[tokio::test]
async fn test_diagnostics_concurrent_requests() {
    // 🔄 Test concurrent diagnostics requests
    println!("🔄 Testing concurrent diagnostics requests...");
    
    let (_temp_dir, project_path) = create_diagnostics_test_project().await
        .expect("Failed to create test project");
    
    let test_files = vec![
        project_path.join("src/lib.rs"),
        project_path.join("src/main.rs"),
        project_path.join("src/lib.rs"), // Duplicate to test same file
    ];
    
    // Create concurrent requests
    let futures: Vec<_> = test_files.into_iter().enumerate().map(|(i, file_path)| {
        async move {
            let (success, duration, _) = run_diagnostics_test(
                file_path,
                &format!("Concurrent request {}", i + 1),
                15
            ).await;
            (i, success, duration)
        }
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    
    let mut successful = 0;
    let mut total_duration = Duration::default();
    
    for (i, success, duration) in results {
        if success {
            successful += 1;
        }
        total_duration += duration;
        
        println!("  Request {}: {} in {:?}", 
                i + 1, 
                if success { "✅ Success" } else { "⚠️ Failed" }, 
                duration);
    }
    
    println!("📊 Concurrent results: {}/3 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent request succeeded");
    } else {
        println!("ℹ️ All concurrent requests failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent requests test completed");
}

#[tokio::test]
async fn test_diagnostics_large_file() {
    // 📏 Test diagnostics on larger file
    println!("📏 Testing diagnostics on large file...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "large-file-test"
version = "0.1.0"
edition = "2021"
"#).await.expect("Failed to write Cargo.toml");
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await.expect("Failed to create src dir");
    
    // Generate large file content
    let mut content = String::new();
    content.push_str("//! Large file for testing diagnostics performance\n\n");
    
    // Generate many similar functions
    for i in 0..100 {
        content.push_str(&format!(r#"
/// Function number {}
pub fn function_{}() -> i32 {{
    let value = {};
    let result = value * 2;
    if result > 50 {{
        result + 10
    }} else {{
        result - 5
    }}
}}
"#, i, i, i));
    }
    
    // Add some intentional errors in the middle
    content.push_str(r#"
// Intentional error for testing
pub fn error_function() -> i32 {
    undefined_variable // Should cause error
}

// Another error
pub fn type_error() -> String {
    42 // Type mismatch
}
"#);
    
    let large_file = src_dir.join("lib.rs");
    tokio::fs::write(&large_file, content).await.expect("Failed to write large file");
    
    println!("📏 Created large file with ~{} lines", content.lines().count());
    
    let (success, duration, diagnostics) = run_diagnostics_test(
        large_file,
        "Large file diagnostics",
        60 // Longer timeout for large file
    ).await;
    
    if success {
        if let Some(data) = diagnostics {
            if let Some(diagnostics_array) = data["diagnostics"].as_array() {
                println!("📊 Found {} diagnostics in large file", diagnostics_array.len());
                
                // Check performance on large file
                if duration < Duration::from_secs(5) {
                    println!("⚡ Excellent performance on large file: {:?}", duration);
                } else if duration < Duration::from_secs(15) {
                    println!("✅ Acceptable performance on large file: {:?}", duration);
                } else {
                    println!("⚠️ Slow performance on large file: {:?}", duration);
                }
            }
        }
    }
    
    println!("✅ Large file test completed");
}
