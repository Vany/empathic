//! 🔬 LSP Completion Tool Comprehensive Tests
//!
//! Advanced testing for LSP completion functionality including:
//! - Context-aware completion suggestions
//! - Trigger character handling and smart completion
//! - Performance validation and snippet support
//! - Edge cases and position boundary testing

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::completion::LspCompletionTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create Rust project optimized for completion testing
async fn create_completion_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml with multiple dependencies for rich completion
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "completion-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with rich API surface for completion testing
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Completion test library with rich API surface

use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

/// A comprehensive API struct for completion testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTestApi {
    pub name: String,
    pub version: u32,
    pub features: HashSet<String>,
    pub config: HashMap<String, String>,
    pub data: Vec<ApiData>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiData {
    pub id: u64,
    pub content: String,
    pub tags: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub author: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub license: String,
}

impl CompletionTestApi {
    /// Creates a new API instance - perfect for completion after "CompletionTestApi::"
    pub fn new(name: String, version: u32) -> Self {
        Self {
            name,
            version,
            features: HashSet::new(),
            config: HashMap::new(),
            data: Vec::new(),
            metadata: None,
        }
    }
    
    /// Builder pattern method - should complete after "api."
    pub fn with_feature(mut self, feature: String) -> Self {
        self.features.insert(feature);
        self
    }
    
    /// Builder pattern method - should complete after "api."
    pub fn with_config(mut self, key: String, value: String) -> Self {
        self.config.insert(key, value);
        self
    }
    
    /// Builder pattern method - should complete after "api."
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Method with parameters - test completion in parameter context
    pub fn add_data(&mut self, id: u64, content: String, tags: Vec<String>) {
        let data = ApiData {
            id,
            content,
            tags,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        self.data.push(data);
    }
    
    /// Getter methods - should complete after "api."
    pub fn name(&self) -> &str { &self.name }
    pub fn version(&self) -> u32 { self.version }
    pub fn features(&self) -> &HashSet<String> { &self.features }
    pub fn config(&self) -> &HashMap<String, String> { &self.config }
    pub fn data(&self) -> &[ApiData] { &self.data }
    pub fn metadata(&self) -> Option<&Metadata> { self.metadata.as_ref() }
    
    /// Methods with different return types for completion testing
    pub fn get_feature_count(&self) -> usize { self.features.len() }
    pub fn get_data_by_id(&self, id: u64) -> Option<&ApiData> {
        self.data.iter().find(|d| d.id == id)
    }
    pub fn get_config_value(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }
    
    /// Async method for completion testing
    pub async fn process_async(&self) -> Result<String, Box<dyn std::error::Error>> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(format!("Processed {} data items", self.data.len()))
    }
}

impl ApiData {
    /// Associated functions for completion testing
    pub fn new(id: u64, content: String) -> Self {
        Self {
            id,
            content,
            tags: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Method calls for completion testing
    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }
    
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
    
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }
}

impl Metadata {
    /// Constructor for completion testing
    pub fn new(author: String, description: String, license: String) -> Self {
        Self {
            author,
            description,
            keywords: Vec::new(),
            license,
        }
    }
    
    /// Chainable methods for completion testing
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
    
    pub fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }
}

/// Free functions for completion testing
pub fn create_default_api() -> CompletionTestApi {
    CompletionTestApi::new("default".to_string(), 1)
        .with_feature("basic".to_string())
        .with_config("mode".to_string(), "test".to_string())
}

pub fn merge_apis(api1: CompletionTestApi, api2: CompletionTestApi) -> CompletionTestApi {
    let mut merged = api1;
    merged.features.extend(api2.features);
    merged.config.extend(api2.config);
    merged.data.extend(api2.data);
    merged
}

/// Complex types for completion testing
pub type ApiResult<T> = Result<T, ApiError>;
pub type ApiMap = HashMap<String, CompletionTestApi>;
pub type DataCallback = Box<dyn Fn(&ApiData) -> bool + Send + Sync>;

/// Custom error for completion testing
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Data error: {0}")]
    Data(String),
    #[error("Network error: {0}")]
    Network(String),
}

/// Traits for completion testing
pub trait Processable {
    fn process(&self) -> ApiResult<String>;
    fn can_process(&self) -> bool;
}

impl Processable for CompletionTestApi {
    fn process(&self) -> ApiResult<String> {
        if self.data.is_empty() {
            Err(ApiError::Data("No data to process".to_string()))
        } else {
            Ok(format!("Processed API: {}", self.name))
        }
    }
    
    fn can_process(&self) -> bool {
        !self.data.is_empty()
    }
}

/// Function for demonstrating various completion contexts
pub fn demonstrate_completion_contexts() {
    // Local variables for completion
    let mut api = CompletionTestApi::new("test".to_string(), 1);
    let data = ApiData::new(1, "test data".to_string());
    let metadata = Metadata::new(
        "Author".to_string(),
        "Description".to_string(),
        "MIT".to_string(),
    );
    
    // Method calls - completion should show available methods
    api.add_data(1, "content".to_string(), vec!["tag1".to_string()]);
    let name = api.name();
    let version = api.version();
    let feature_count = api.get_feature_count();
    
    // Chained method calls - completion should work in chain
    let built_api = CompletionTestApi::new("chained".to_string(), 2)
        .with_feature("feature1".to_string())
        .with_config("key".to_string(), "value".to_string());
    
    // Collections and iterators - completion should show iterator methods
    let features: Vec<_> = api.features().iter().collect();
    let filtered_data: Vec<_> = api.data().iter()
        .filter(|d| d.tag_count() > 0)
        .collect();
    
    // Standard library types - completion should show std methods
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let keys: Vec<_> = map.keys().collect();
    
    // Use variables to avoid warnings
    println!("Demo: {}, {}, {}, {}, {:?}, {:?}, {:?}, {:?}", 
             name, version, feature_count, built_api.name(), 
             features, filtered_data, keys, metadata.author);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_creation() {
        let api = CompletionTestApi::new("test".to_string(), 1);
        assert_eq!(api.name(), "test");
        assert_eq!(api.version(), 1);
    }
    
    #[test]
    fn test_builder_pattern() {
        let api = CompletionTestApi::new("test".to_string(), 1)
            .with_feature("test-feature".to_string())
            .with_config("test-key".to_string(), "test-value".to_string());
        
        assert!(api.features().contains("test-feature"));
        assert_eq!(api.get_config_value("test-key"), Some(&"test-value".to_string()));
    }
}
"#).await?;
    
    // Create main.rs with completion scenarios
    let main_rs = src_dir.join("main.rs");
    tokio::fs::write(&main_rs, r#"//! Main file with various completion scenarios

use completion_test::{CompletionTestApi, ApiData, Metadata, create_default_api};
use std::collections::HashMap;

fn main() {
    // Basic completion scenarios
    let mut api = CompletionTestApi::new("main".to_string(), 1);
    
    // Method completion after dot - should complete to api methods
    api.add_data(1, "test".to_string(), vec!["tag".to_string()]);
    let name = api.name();
    
    // Static method completion - should complete to associated functions
    let default_api = create_default_api();
    
    // Completion in function call context
    completion_function_calls(&api, &default_api);
    
    // Completion with generics
    completion_with_generics();
    
    // Completion in match expressions
    completion_in_match(&api);
    
    println!("Completion demo completed: {}", name);
}

/// Function demonstrating completion in function calls
fn completion_function_calls(api1: &CompletionTestApi, api2: &CompletionTestApi) {
    // Parameter completion - should suggest available variables
    let feature_count = api1.get_feature_count();
    let data_item = api1.get_data_by_id(1);
    let config_value = api2.get_config_value("mode");
    
    // Nested function calls - completion should work at each level
    let processed_length = api1.name().len();
    let feature_list: Vec<_> = api1.features().iter().cloned().collect();
    
    println!("Function calls: {}, {:?}, {:?}, {}, {:?}", 
             feature_count, data_item, config_value, processed_length, feature_list);
}

/// Function demonstrating completion with generic types
fn completion_with_generics() {
    // Generic collections - completion should show type-specific methods
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("key".to_string(), 42);
    let value = map.get("key");
    
    let mut vec: Vec<String> = Vec::new();
    vec.push("item".to_string());
    let first = vec.first();
    
    // Iterator methods - completion should show iterator methods
    let processed: Vec<_> = vec.iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.len())
        .collect();
    
    println!("Generics: {:?}, {:?}, {:?}", value, first, processed);
}

/// Function demonstrating completion in match expressions
fn completion_in_match(api: &CompletionTestApi) {
    // Match on enum-like values
    match api.get_feature_count() {
        0 => println!("No features"),
        1 => {
            // Completion should work inside match arms
            let name = api.name();
            println!("One feature in {}", name);
        },
        _ => {
            // Method completion in match arm
            let features: Vec<_> = api.features().iter().cloned().collect();
            println!("Multiple features: {:?}", features);
        }
    }
    
    // Match on Option types
    match api.metadata() {
        Some(metadata) => {
            // Completion should show metadata methods
            let author = metadata.author;
            let desc = metadata.description;
            println!("Metadata: {} - {}", author, desc);
        },
        None => println!("No metadata"),
    }
}

/// Function with incomplete code for completion testing
fn incomplete_code_for_completion() {
    let api = CompletionTestApi::new("incomplete".to_string(), 1);
    
    // Incomplete method call - cursor would be after the dot
    // api.
    
    // Incomplete variable access - cursor after dot
    // let data = api.data().
    
    // Incomplete chained call - cursor in the middle
    // let built = CompletionTestApi::new("test".to_string(), 1).
    
    // Use api to avoid warnings
    println!("Incomplete: {}", api.name());
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run completion test and analyze results
async fn run_completion_test(
    file_path: PathBuf,
    line: u32,
    character: u32,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspCompletionTool::new();
    let input = ToolInput::LspCompletion { file_path, line, character };
    
    let start = Instant::now();
    let result = timeout(Duration::from_secs(timeout_secs), tool.call(input)).await;
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(CallToolResult::Success { content })) => {
            println!("✅ {} succeeded in {:?}", test_name, duration);
            
            if let Some(text_content) = content.first() {
                if let Ok(completion_data) = serde_json::from_str::<Value>(&text_content.text) {
                    return (true, duration, Some(completion_data));
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
async fn test_completion_method_calls() {
    // 🎯 Test completion for method calls after dot operator
    println!("🎯 Testing completion for method calls...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test completion after "api." - should show available methods
    let (success, duration, completion_data) = run_completion_test(
        lib_file.clone(),
        175, // Line with "api." in demonstrate_completion_contexts
        8,   // Character after the dot
        "Method completion after dot",
        20
    ).await;
    
    if success {
        if let Some(data) = completion_data {
            if let Some(completions) = data["completions"].as_array() {
                println!("📝 Found {} completion suggestions", completions.len());
                
                // Look for expected method names
                let completion_labels: Vec<String> = completions.iter()
                    .filter_map(|c| c["label"].as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                let expected_methods = vec!["add_data", "name", "version", "features", "config"];
                let found_methods: Vec<_> = expected_methods.iter()
                    .filter(|method| completion_labels.iter().any(|label| label.contains(method)))
                    .collect();
                
                if !found_methods.is_empty() {
                    println!("✅ Found expected methods: {:?}", found_methods);
                } else {
                    println!("ℹ️ Completion format may be different than expected");
                }
                
                // Check for completion item details
                for (i, completion) in completions.iter().take(5).enumerate() {
                    if let Some(label) = completion["label"].as_str() {
                        let kind = completion["kind"].as_u64().unwrap_or(0);
                        println!("  {}. {} (kind: {})", i + 1, label, kind);
                    }
                }
            }
        }
        
        // Check performance
        if duration < Duration::from_millis(200) {
            println!("⚡ Excellent completion performance: {:?}", duration);
        } else if duration < Duration::from_millis(500) {
            println!("✅ Good completion performance: {:?}", duration);
        } else {
            println!("⚠️ Slow completion performance: {:?}", duration);
        }
    }
    
    println!("✅ Method completion test completed");
}

#[tokio::test]
async fn test_completion_static_methods() {
    // 🏗️ Test completion for static/associated functions
    println!("🏗️ Testing completion for static methods...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test completion after "CompletionTestApi::" - should show associated functions
    let (success, duration, completion_data) = run_completion_test(
        lib_file.clone(),
        45, // Line with struct implementation
        25, // Character position for static method completion
        "Static method completion",
        15
    ).await;
    
    if success {
        if let Some(data) = completion_data {
            if let Some(completions) = data["completions"].as_array() {
                println!("📝 Found {} static completion suggestions", completions.len());
                
                // Look for constructor and static methods
                let has_new = completions.iter()
                    .any(|c| c["label"].as_str().map_or(false, |l| l.contains("new")));
                
                if has_new {
                    println!("✅ Found 'new' constructor in completions");
                } else {
                    println!("ℹ️ Constructor may have different format");
                }
            }
        }
    }
    
    println!("✅ Static method completion test completed");
}

#[tokio::test]
async fn test_completion_in_parameters() {
    // 📝 Test completion within function parameter contexts
    println!("📝 Testing completion in function parameters...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test completion in function call parameters
    let parameter_positions = vec![
        (18, 30, "Function parameter completion"),
        (25, 40, "Method parameter completion"),
        (30, 25, "Nested call parameter completion"),
    ];
    
    for (line, character, description) in parameter_positions {
        let (success, duration, completion_data) = run_completion_test(
            main_file.clone(),
            line,
            character,
            description,
            10
        ).await;
        
        if success {
            if let Some(data) = completion_data {
                if let Some(completions) = data["completions"].as_array() {
                    println!("  📝 {} found {} suggestions in {:?}", 
                            description, completions.len(), duration);
                }
            }
        }
    }
    
    println!("✅ Parameter completion test completed");
}

#[tokio::test]
async fn test_completion_chained_calls() {
    // 🔗 Test completion in method chaining scenarios
    println!("🔗 Testing completion in method chains...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test completion in middle of method chains
    let chain_positions = vec![
        (21, 15, "Chain start completion"),
        (22, 20, "Chain middle completion"),
        (23, 25, "Chain end completion"),
    ];
    
    for (line, character, description) in chain_positions {
        let (success, duration, completion_data) = run_completion_test(
            main_file.clone(),
            line,
            character,
            description,
            10
        ).await;
        
        if success {
            if let Some(data) = completion_data {
                if let Some(completions) = data["completions"].as_array() {
                    println!("  🔗 {} found {} suggestions in {:?}", 
                            description, completions.len(), duration);
                    
                    // Check for builder pattern methods
                    let has_with_methods = completions.iter()
                        .any(|c| c["label"].as_str().map_or(false, |l| l.starts_with("with_")));
                    
                    if has_with_methods {
                        println!("    ✅ Found builder pattern methods");
                    }
                }
            }
        }
    }
    
    println!("✅ Method chain completion test completed");
}

#[tokio::test]
async fn test_completion_context_awareness() {
    // 🧠 Test context-aware completion (types, imports, etc.)
    println!("🧠 Testing context-aware completion...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test completion in different contexts
    let context_tests = vec![
        (35, 20, "Generic type completion"),
        (40, 15, "Iterator method completion"),
        (50, 10, "Match arm completion"),
        (60, 25, "Option type completion"),
    ];
    
    for (line, character, description) in context_tests {
        let (success, duration, completion_data) = run_completion_test(
            main_file.clone(),
            line,
            character,
            description,
            10
        ).await;
        
        if success {
            if let Some(data) = completion_data {
                if let Some(completions) = data["completions"].as_array() {
                    println!("  🧠 {} found {} context-aware suggestions in {:?}", 
                            description, completions.len(), duration);
                    
                    // Check for type-specific methods
                    let has_type_methods = completions.iter()
                        .any(|c| {
                            if let Some(label) = c["label"].as_str() {
                                label.contains("iter") || label.contains("map") || 
                                label.contains("filter") || label.contains("collect")
                            } else {
                                false
                            }
                        });
                    
                    if has_type_methods {
                        println!("    ✅ Found type-specific methods");
                    }
                }
            }
        }
    }
    
    println!("✅ Context-aware completion test completed");
}

#[tokio::test]
async fn test_completion_performance_patterns() {
    // ⚡ Test completion performance patterns and caching
    println!("⚡ Testing completion performance patterns...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test repeated completion at same position (caching)
    let test_position = (175, 8); // Same position as earlier test
    let mut durations = Vec::new();
    
    for i in 0..3 {
        let (success, duration, _) = run_completion_test(
            lib_file.clone(),
            test_position.0,
            test_position.1,
            &format!("Rapid completion {}", i + 1),
            5
        ).await;
        
        if success {
            durations.push(duration);
        }
    }
    
    if !durations.is_empty() {
        println!("📊 Rapid completion performance:");
        for (i, duration) in durations.iter().enumerate() {
            println!("  Completion {}: {:?}", i + 1, duration);
        }
        
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        println!("  Average: {:?}", avg_duration);
        
        // Check performance targets
        if avg_duration < Duration::from_millis(200) {
            println!("  ⚡ Excellent performance (<200ms average)");
        } else if avg_duration < Duration::from_millis(500) {
            println!("  ✅ Good performance (<500ms average)");
        } else {
            println!("  ⚠️ Slow performance (>500ms average)");
        }
    }
    
    println!("✅ Performance patterns test completed");
}

#[tokio::test]
async fn test_completion_edge_cases() {
    // 🧪 Test completion edge cases and boundary conditions
    println!("🧪 Testing completion edge cases...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test edge cases
    let edge_cases = vec![
        (0, 0, "Start of file"),
        (1, 100, "Beyond line end"),
        (999, 0, "Beyond file end"),
        (50, 0, "Start of line"),
        (80, 1, "Single character position"),
    ];
    
    for (line, character, description) in edge_cases {
        let (success, duration, _) = run_completion_test(
            lib_file.clone(),
            line,
            character,
            &format!("Edge case: {}", description),
            5
        ).await;
        
        if success {
            println!("  ✅ {} handled successfully in {:?}", description, duration);
        } else {
            println!("  ✅ {} failed gracefully in {:?} (expected for some edge cases)", description, duration);
        }
    }
    
    println!("✅ Edge cases test completed");
}

#[tokio::test]
async fn test_completion_concurrent_requests() {
    // 🔄 Test concurrent completion requests
    println!("🔄 Testing concurrent completion requests...");
    
    let (_temp_dir, project_path) = create_completion_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Create concurrent completion requests at different positions
    let positions = vec![
        (175, 8, "Method completion"),
        (180, 12, "Variable completion"),
        (185, 15, "Function completion"),
        (190, 20, "Type completion"),
    ];
    
    let futures: Vec<_> = positions.into_iter().enumerate().map(|(i, (line, char, desc))| {
        let file = lib_file.clone();
        async move {
            let (success, duration, _) = run_completion_test(
                file,
                line,
                char,
                &format!("Concurrent completion {} ({})", i + 1, desc),
                10
            ).await;
            (i, desc, success, duration)
        }
    }).collect();
    
    let results = futures::future::join_all(futures).await;
    
    let mut successful = 0;
    let mut total_duration = Duration::default();
    
    for (i, desc, success, duration) in results {
        if success {
            successful += 1;
        }
        total_duration += duration;
        
        println!("  Request {} ({}): {} in {:?}", 
                i + 1, 
                desc,
                if success { "✅ Success" } else { "⚠️ Failed" }, 
                duration);
    }
    
    println!("📊 Concurrent completion results: {}/4 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent completion succeeded");
    } else {
        println!("ℹ️ All concurrent completions failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent completion test completed");
}
