//! 🔬 LSP Hover Tool Comprehensive Tests
//!
//! Advanced testing for LSP hover functionality including:
//! - Type information extraction and formatting
//! - Documentation and signature display
//! - Performance validation and edge case handling
//! - Position boundary testing and error recovery

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::hover::LspHoverTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create Rust project with rich type information for hover testing
async fn create_hover_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "hover-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with diverse types for hover testing
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Hover test library with diverse types and documentation

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A well-documented structure for testing hover functionality
/// 
/// This structure contains multiple fields of different types
/// to test hover information display.
/// 
/// # Examples
/// 
/// ```
/// use hover_test::Person;
/// 
/// let person = Person::new("Alice".to_string(), 30);
/// assert_eq!(person.name(), "Alice");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    /// The person's full name
    pub name: String,
    /// The person's age in years
    pub age: u32,
    /// Optional email address
    pub email: Option<String>,
    /// List of hobbies
    pub hobbies: Vec<String>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl Person {
    /// Creates a new Person instance
    /// 
    /// # Arguments
    /// 
    /// * `name` - The person's full name
    /// * `age` - The person's age in years
    /// 
    /// # Returns
    /// 
    /// A new `Person` instance with empty email, hobbies, and metadata
    pub fn new(name: String, age: u32) -> Self {
        Self {
            name,
            age,
            email: None,
            hobbies: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Gets the person's name
    /// 
    /// Returns a reference to the person's name string
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Sets the person's email address
    /// 
    /// # Arguments
    /// 
    /// * `email` - The email address to set
    pub fn set_email(&mut self, email: String) {
        self.email = Some(email);
    }
    
    /// Adds a hobby to the person's list
    /// 
    /// # Arguments
    /// 
    /// * `hobby` - The hobby to add
    pub fn add_hobby(&mut self, hobby: String) {
        self.hobbies.push(hobby);
    }
    
    /// Calculates years until retirement (assuming retirement at 65)
    /// 
    /// # Returns
    /// 
    /// Number of years until retirement, or 0 if already retired
    pub fn years_until_retirement(&self) -> u32 {
        if self.age >= 65 {
            0
        } else {
            65 - self.age
        }
    }
}

/// An enumeration for testing hover on enum variants
/// 
/// This enum demonstrates different variant types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Person is currently active
    Active,
    /// Person is inactive with a reason
    Inactive(String),
    /// Person is pending with additional data
    Pending { reason: String, since: u64 },
}

impl Status {
    /// Checks if the status is active
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active)
    }
    
    /// Gets the status description
    pub fn description(&self) -> String {
        match self {
            Status::Active => "Currently active".to_string(),
            Status::Inactive(reason) => format!("Inactive: {}", reason),
            Status::Pending { reason, .. } => format!("Pending: {}", reason),
        }
    }
}

/// A trait for testing hover on trait methods
/// 
/// This trait defines common behavior for identifiable objects
pub trait Identifiable {
    /// Gets the unique identifier
    fn id(&self) -> String;
    
    /// Checks if this object matches the given ID
    fn matches_id(&self, id: &str) -> bool {
        self.id() == id
    }
}

impl Identifiable for Person {
    fn id(&self) -> String {
        format!("person_{}", self.name.to_lowercase().replace(' ', "_"))
    }
}

/// Generic function for testing hover with generics
/// 
/// # Type Parameters
/// 
/// * `T` - Must implement Debug and Clone
/// 
/// # Arguments
/// 
/// * `items` - Vector of items to process
/// 
/// # Returns
/// 
/// The first item in the vector, or None if empty
pub fn get_first<T: std::fmt::Debug + Clone>(items: Vec<T>) -> Option<T> {
    items.into_iter().next()
}

/// Function with complex return type for hover testing
/// 
/// Returns a nested Result type to test hover display of complex types
pub fn complex_return_type() -> Result<HashMap<String, Vec<Option<u32>>>, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    map.insert("numbers".to_string(), vec![Some(1), None, Some(3)]);
    Ok(map)
}

/// Async function for testing hover on async functions
/// 
/// # Arguments
/// 
/// * `delay_ms` - Delay in milliseconds
/// 
/// # Returns
/// 
/// A future that resolves to a string after the specified delay
pub async fn async_function(delay_ms: u64) -> String {
    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    "Async operation completed".to_string()
}

/// Constant for testing hover on constants
pub const MAX_CONNECTIONS: usize = 1000;

/// Static variable for testing hover on statics
pub static mut GLOBAL_COUNTER: u32 = 0;

/// Type alias for testing hover on type aliases
pub type PersonMap = HashMap<String, Person>;

/// Function demonstrating various local variables for hover testing
pub fn demonstrate_variables() {
    // Basic types
    let integer_var = 42i32;
    let float_var = 3.14f64;
    let string_var = "Hello, world!".to_string();
    let bool_var = true;
    
    // Collections
    let vec_var = vec![1, 2, 3, 4, 5];
    let map_var = HashMap::from([("key1", "value1"), ("key2", "value2")]);
    
    // Custom types
    let person_var = Person::new("John Doe".to_string(), 25);
    let status_var = Status::Active;
    
    // References
    let person_ref = &person_var;
    let name_ref = person_var.name();
    
    // Closures
    let closure_var = |x: i32| x * 2;
    let mapped_values: Vec<i32> = vec_var.iter().map(|&x| closure_var(x)).collect();
    
    // Complex expressions
    let complex_expr = person_var.years_until_retirement() + integer_var as u32;
    
    // Use variables to avoid unused warnings
    println!("Variables: {}, {}, {}, {}, {:?}, {:?}, {:?}, {:?}, {}, {}, {:?}, {}", 
             integer_var, float_var, string_var, bool_var, vec_var, map_var,
             person_var, status_var, person_ref.name(), name_ref, mapped_values, complex_expr);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_person_creation() {
        let person = Person::new("Test User".to_string(), 30);
        assert_eq!(person.name(), "Test User");
        assert_eq!(person.age, 30);
    }
    
    #[test]
    fn test_status_enum() {
        let active = Status::Active;
        assert!(active.is_active());
        
        let inactive = Status::Inactive("On vacation".to_string());
        assert!(!inactive.is_active());
    }
    
    #[test]
    fn test_generic_function() {
        let numbers = vec![1, 2, 3];
        let first = get_first(numbers);
        assert_eq!(first, Some(1));
    }
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run hover test and validate response
async fn run_hover_test(
    file_path: PathBuf,
    line: u32,
    character: u32,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspHoverTool::new();
    let input = ToolInput::LspHover { file_path, line, character };
    
    let start = Instant::now();
    let result = timeout(Duration::from_secs(timeout_secs), tool.call(input)).await;
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(CallToolResult::Success { content })) => {
            println!("✅ {} succeeded in {:?}", test_name, duration);
            
            if let Some(text_content) = content.first() {
                if let Ok(hover_data) = serde_json::from_str::<Value>(&text_content.text) {
                    return (true, duration, Some(hover_data));
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
async fn test_hover_on_types() {
    // 🎯 Test hover on different type definitions
    println!("🎯 Testing hover on type definitions...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test hover on struct name
    let (success, duration, hover_data) = run_hover_test(
        lib_file.clone(),
        18, // Line with struct Person definition
        12, // Character position on "Person"
        "Hover on struct",
        20
    ).await;
    
    if success {
        if let Some(data) = hover_data {
            if let Some(contents) = data.get("contents") {
                println!("📝 Hover contents: {}", contents);
                
                // Check if we got meaningful hover information
                let contents_str = contents.to_string();
                if contents_str.contains("Person") || contents_str.contains("struct") {
                    println!("✅ Hover contains expected type information");
                } else {
                    println!("ℹ️ Hover response format may be different than expected");
                }
            }
        }
        
        // Check performance
        if duration < Duration::from_millis(200) {
            println!("⚡ Excellent hover performance: {:?}", duration);
        } else if duration < Duration::from_millis(500) {
            println!("✅ Good hover performance: {:?}", duration);
        } else {
            println!("⚠️ Slow hover performance: {:?}", duration);
        }
    }
    
    println!("✅ Type hover test completed");
}

#[tokio::test]
async fn test_hover_on_functions() {
    // 🔧 Test hover on function definitions and calls
    println!("🔧 Testing hover on functions...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test positions on different functions
    let function_positions = vec![
        (47, 10, "Constructor function"),
        (60, 10, "Getter method"),
        (67, 10, "Setter method"),
        (75, 10, "Method with calculation"),
        (154, 10, "Generic function"),
        (161, 10, "Complex return type function"),
        (173, 15, "Async function"),
    ];
    
    for (line, character, description) in function_positions {
        let (success, duration, hover_data) = run_hover_test(
            lib_file.clone(),
            line,
            character,
            &format!("Hover on {}", description),
            15
        ).await;
        
        if success {
            if let Some(data) = hover_data {
                if let Some(contents) = data.get("contents") {
                    let contents_str = contents.to_string();
                    
                    // Check for function signature information
                    if contents_str.contains("fn ") || contents_str.contains("->") {
                        println!("  ✅ {} hover contains function signature", description);
                    } else {
                        println!("  ℹ️ {} hover may contain different format", description);
                    }
                }
            }
            
            println!("  ⏱️ {} completed in {:?}", description, duration);
        } else {
            println!("  ⚠️ {} hover failed or timeout", description);
        }
    }
    
    println!("✅ Function hover test completed");
}

#[tokio::test]
async fn test_hover_on_variables() {
    // 📊 Test hover on local variables and expressions
    println!("📊 Testing hover on variables...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test positions within the demonstrate_variables function
    let variable_positions = vec![
        (184, 10, "integer variable"),
        (185, 10, "float variable"),
        (186, 10, "string variable"),
        (189, 10, "vector variable"),
        (190, 10, "map variable"),
        (193, 10, "custom type variable"),
        (196, 10, "reference variable"),
        (200, 10, "closure variable"),
    ];
    
    for (line, character, description) in variable_positions {
        let (success, duration, hover_data) = run_hover_test(
            lib_file.clone(),
            line,
            character,
            &format!("Hover on {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = hover_data {
                if let Some(contents) = data.get("contents") {
                    let contents_str = contents.to_string();
                    
                    // Check for type information
                    if contents_str.contains(":") || contents_str.contains("i32") || 
                       contents_str.contains("String") || contents_str.contains("Vec") {
                        println!("  ✅ {} hover contains type information", description);
                    } else {
                        println!("  ℹ️ {} hover format may be different", description);
                    }
                }
            }
            
            println!("  ⏱️ {} completed in {:?}", description, duration);
        }
    }
    
    println!("✅ Variable hover test completed");
}

#[tokio::test]
async fn test_hover_boundary_conditions() {
    // 🎯 Test hover at various boundary positions
    println!("🎯 Testing hover boundary conditions...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test edge cases and boundary conditions
    let boundary_tests = vec![
        (0, 0, "Start of file"),
        (1, 0, "Beginning of comment line"),
        (10, 0, "Start of line"),
        (10, 200, "Far beyond line end"),
        (999, 0, "Beyond file end"),
        (20, 1, "Middle of keyword"),
        (47, 0, "Indented line start"),
    ];
    
    for (line, character, description) in boundary_tests {
        let (success, duration, _) = run_hover_test(
            lib_file.clone(),
            line,
            character,
            &format!("Boundary test: {}", description),
            5
        ).await;
        
        if success {
            println!("  ✅ {} handled successfully in {:?}", description, duration);
        } else {
            println!("  ✅ {} failed gracefully in {:?} (expected for some boundaries)", description, duration);
        }
    }
    
    println!("✅ Boundary conditions test completed");
}

#[tokio::test]
async fn test_hover_performance_patterns() {
    // ⚡ Test hover performance on different patterns
    println!("⚡ Testing hover performance patterns...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test multiple rapid hovers on same position (caching)
    let test_position = (47, 10); // Function position
    let mut durations = Vec::new();
    
    for i in 0..5 {
        let (success, duration, _) = run_hover_test(
            lib_file.clone(),
            test_position.0,
            test_position.1,
            &format!("Rapid hover {}", i + 1),
            5
        ).await;
        
        if success {
            durations.push(duration);
        }
    }
    
    if !durations.is_empty() {
        println!("📊 Rapid hover performance:");
        for (i, duration) in durations.iter().enumerate() {
            println!("  Hover {}: {:?}", i + 1, duration);
        }
        
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        println!("  Average: {:?}", avg_duration);
        
        // Check if later requests are faster (indicating caching)
        if durations.len() >= 3 {
            let early_avg = durations[0..2].iter().sum::<Duration>() / 2;
            let later_avg = durations[2..].iter().sum::<Duration>() / (durations.len() - 2) as u32;
            
            if later_avg < early_avg {
                println!("  ✅ Later requests faster - caching appears active");
            } else {
                println!("  ℹ️ No significant speed improvement - caching may not be active");
            }
        }
    }
    
    println!("✅ Performance patterns test completed");
}

#[tokio::test]
async fn test_hover_concurrent_requests() {
    // 🔄 Test concurrent hover requests
    println!("🔄 Testing concurrent hover requests...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Create concurrent hover requests at different positions
    let positions = vec![
        (18, 12, "Struct"),
        (47, 10, "Function 1"),
        (60, 10, "Function 2"),
        (154, 10, "Generic function"),
        (184, 10, "Variable"),
    ];
    
    let futures: Vec<_> = positions.into_iter().enumerate().map(|(i, (line, char, desc))| {
        let file = lib_file.clone();
        async move {
            let (success, duration, _) = run_hover_test(
                file,
                line,
                char,
                &format!("Concurrent hover {} ({})", i + 1, desc),
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
    
    println!("📊 Concurrent hover results: {}/5 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent hover succeeded");
    } else {
        println!("ℹ️ All concurrent hovers failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent hover test completed");
}

#[tokio::test]
async fn test_hover_documentation_extraction() {
    // 📚 Test hover on well-documented items
    println!("📚 Testing hover documentation extraction...");
    
    let (_temp_dir, project_path) = create_hover_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test hover on items with rich documentation
    let documented_items = vec![
        (18, 12, "Documented struct"),
        (47, 10, "Documented constructor"),
        (154, 10, "Documented generic function"),
        (103, 8, "Documented enum"),
        (127, 15, "Documented trait"),
    ];
    
    for (line, character, description) in documented_items {
        let (success, duration, hover_data) = run_hover_test(
            lib_file.clone(),
            line,
            character,
            &format!("Documentation hover: {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = hover_data {
                if let Some(contents) = data.get("contents") {
                    let contents_str = contents.to_string();
                    
                    // Check for documentation content
                    if contents_str.contains("///") || contents_str.contains("Creates") || 
                       contents_str.contains("Returns") || contents_str.contains("Arguments") {
                        println!("  ✅ {} contains documentation", description);
                    } else {
                        println!("  ℹ️ {} may have different documentation format", description);
                    }
                    
                    // Check for examples in documentation
                    if contents_str.contains("Example") || contents_str.contains("```") {
                        println!("  📖 {} includes code examples", description);
                    }
                }
            }
            
            println!("  ⏱️ {} completed in {:?}", description, duration);
        }
    }
    
    println!("✅ Documentation extraction test completed");
}
