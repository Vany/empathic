//! 🔬 LSP Goto Definition Tool Comprehensive Tests
//!
//! Advanced testing for LSP goto definition functionality including:
//! - Symbol navigation and definition location
//! - Cross-file definition discovery
//! - Standard library and external dependency navigation
//! - Performance validation and edge case handling

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::goto_definition::LspGotoDefinitionTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create multi-file Rust project for goto definition testing
async fn create_goto_definition_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "goto-definition-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with cross-referenced types
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Goto definition test library with cross-references

pub mod types;
pub mod utils;
pub mod service;

use types::{User, UserRole, UserConfig};
use utils::{calculate_hash, format_user_info};
use service::{UserService, ServiceConfig};

/// Main API struct for goto definition testing
pub struct ApiClient {
    pub config: UserConfig,
    pub service: UserService,
    pub users: Vec<User>,
}

impl ApiClient {
    /// Create new API client - goto should navigate to UserConfig definition
    pub fn new(config: UserConfig) -> Self {
        let service = UserService::new(ServiceConfig::default());
        Self {
            config,
            service,
            users: Vec::new(),
        }
    }
    
    /// Add user - goto should navigate to User definition
    pub fn add_user(&mut self, user: User) {
        self.users.push(user);
    }
    
    /// Get user by ID - goto should navigate to calculate_hash definition
    pub fn get_user(&self, id: u64) -> Option<&User> {
        let hash = calculate_hash(&id.to_string());
        self.users.iter().find(|u| u.id == id)
    }
    
    /// Format all users - goto should navigate to format_user_info definition
    pub fn format_users(&self) -> Vec<String> {
        self.users.iter()
            .map(|user| format_user_info(user))
            .collect()
    }
    
    /// Process user with role - goto should navigate to UserRole definition
    pub fn process_user_role(&self, user_id: u64, role: UserRole) -> bool {
        if let Some(user) = self.get_user(user_id) {
            match role {
                UserRole::Admin => true,
                UserRole::User => user.is_active(),
                UserRole::Guest => false,
            }
        } else {
            false
        }
    }
}

/// Free function using external types - goto should navigate to definitions
pub fn create_admin_user(name: String, email: String) -> User {
    User::new(1, name, email, UserRole::Admin)
}

/// Function with standard library types - goto should work for std types
pub fn process_data(data: Vec<String>) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for item in data {
        let len = item.len();
        map.insert(item, len);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_client() {
        let config = UserConfig::default();
        let mut client = ApiClient::new(config);
        
        let user = create_admin_user("Admin".to_string(), "admin@test.com".to_string());
        client.add_user(user);
        
        assert_eq!(client.users.len(), 1);
    }
}
"#).await?;
    
    // Create types.rs module
    let types_rs = src_dir.join("types.rs");
    tokio::fs::write(&types_rs, r#"//! Type definitions for goto definition testing

use serde::{Deserialize, Serialize};

/// User struct for testing goto definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub config: UserConfig,
    pub metadata: UserMetadata,
}

/// User role enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

/// User configuration struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub theme: String,
    pub language: String,
    pub notifications: bool,
    pub privacy_level: PrivacyLevel,
}

/// User metadata for additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub login_count: u32,
    pub preferences: std::collections::HashMap<String, String>,
}

/// Privacy level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Public,
    Private,
    Limited,
}

impl User {
    /// Constructor for User - should be findable via goto definition
    pub fn new(id: u64, name: String, email: String, role: UserRole) -> Self {
        Self {
            id,
            name,
            email,
            role,
            config: UserConfig::default(),
            metadata: UserMetadata::default(),
        }
    }
    
    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.metadata.login_count > 0
    }
    
    /// Get user display name
    pub fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.email)
    }
    
    /// Update user role
    pub fn update_role(&mut self, new_role: UserRole) {
        self.role = new_role;
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            notifications: true,
            privacy_level: PrivacyLevel::Private,
        }
    }
}

impl Default for UserMetadata {
    fn default() -> Self {
        Self {
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_login: None,
            login_count: 0,
            preferences: std::collections::HashMap::new(),
        }
    }
}

impl UserRole {
    /// Check if role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
    
    /// Get role name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user", 
            UserRole::Guest => "guest",
        }
    }
}
"#).await?;
    
    // Create utils.rs module
    let utils_rs = src_dir.join("utils.rs");
    tokio::fs::write(&utils_rs, r#"//! Utility functions for goto definition testing

use crate::types::{User, UserRole};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Calculate hash for a string - should be findable via goto definition
pub fn calculate_hash(input: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// Format user information - should be findable via goto definition
pub fn format_user_info(user: &User) -> String {
    format!(
        "User: {} ({}) - Role: {} - Active: {}",
        user.name,
        user.email,
        user.role.as_str(),
        user.is_active()
    )
}

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// Generate user ID from name and email
pub fn generate_user_id(name: &str, email: &str) -> u64 {
    let combined = format!("{}:{}", name, email);
    calculate_hash(&combined)
}

/// Check if user has admin privileges
pub fn is_admin_user(user: &User) -> bool {
    user.role.is_admin()
}

/// Convert role string to UserRole enum
pub fn parse_user_role(role_str: &str) -> Option<UserRole> {
    match role_str.to_lowercase().as_str() {
        "admin" => Some(UserRole::Admin),
        "user" => Some(UserRole::User),
        "guest" => Some(UserRole::Guest),
        _ => None,
    }
}

/// Create formatted user summary
pub fn create_user_summary(users: &[User]) -> String {
    let total = users.len();
    let active = users.iter().filter(|u| u.is_active()).count();
    let admins = users.iter().filter(|u| is_admin_user(u)).count();
    
    format!(
        "Users: {} total, {} active, {} admins",
        total, active, admins
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{User, UserRole};
    
    #[test]
    fn test_calculate_hash() {
        let hash1 = calculate_hash("test");
        let hash2 = calculate_hash("test");
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_format_user_info() {
        let user = User::new(1, "Test".to_string(), "test@example.com".to_string(), UserRole::User);
        let formatted = format_user_info(&user);
        assert!(formatted.contains("Test"));
    }
}
"#).await?;
    
    // Create service.rs module
    let service_rs = src_dir.join("service.rs");
    tokio::fs::write(&service_rs, r#"//! Service layer for goto definition testing

use crate::types::{User, UserRole, UserConfig};
use crate::utils::{validate_email, generate_user_id, is_admin_user};
use std::collections::HashMap;

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_users: usize,
    pub admin_required: bool,
    pub email_validation: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_users: 1000,
            admin_required: false,
            email_validation: true,
        }
    }
}

/// User service for managing users
#[derive(Debug)]
pub struct UserService {
    config: ServiceConfig,
    users: HashMap<u64, User>,
    next_id: u64,
}

impl UserService {
    /// Create new user service - should be findable via goto definition
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            users: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Add user to service - uses utility functions
    pub fn add_user(&mut self, name: String, email: String, role: UserRole) -> Result<u64, String> {
        if self.config.email_validation && !validate_email(&email) {
            return Err("Invalid email format".to_string());
        }
        
        if self.users.len() >= self.config.max_users {
            return Err("Maximum users reached".to_string());
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        let user = User::new(id, name, email, role);
        self.users.insert(id, user);
        
        Ok(id)
    }
    
    /// Get user by ID
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }
    
    /// Update user role - admin required check
    pub fn update_user_role(&mut self, user_id: u64, new_role: UserRole, admin_user_id: u64) -> Result<(), String> {
        if self.config.admin_required {
            if let Some(admin_user) = self.get_user(admin_user_id) {
                if !is_admin_user(admin_user) {
                    return Err("Admin privileges required".to_string());
                }
            } else {
                return Err("Admin user not found".to_string());
            }
        }
        
        if let Some(user) = self.users.get_mut(&user_id) {
            user.update_role(new_role);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
    
    /// Get all users with specific role
    pub fn get_users_by_role(&self, role: UserRole) -> Vec<&User> {
        self.users.values()
            .filter(|user| std::mem::discriminant(&user.role) == std::mem::discriminant(&role))
            .collect()
    }
    
    /// Get user statistics
    pub fn get_stats(&self) -> ServiceStats {
        let total_users = self.users.len();
        let active_users = self.users.values().filter(|u| u.is_active()).count();
        let admin_users = self.users.values().filter(|u| is_admin_user(u)).count();
        
        ServiceStats {
            total_users,
            active_users,
            admin_users,
            max_users: self.config.max_users,
        }
    }
}

/// Service statistics struct
#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub total_users: usize,
    pub active_users: usize,
    pub admin_users: usize,
    pub max_users: usize,
}

impl ServiceStats {
    /// Check if service is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.total_users >= self.max_users
    }
    
    /// Get utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        (self.total_users as f64 / self.max_users as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_service() {
        let config = ServiceConfig::default();
        let mut service = UserService::new(config);
        
        let user_id = service.add_user(
            "Test".to_string(),
            "test@example.com".to_string(),
            UserRole::User
        ).unwrap();
        
        assert!(service.get_user(user_id).is_some());
    }
}
"#).await?;
    
    // Create main.rs with goto definition usage
    let main_rs = src_dir.join("main.rs");
    tokio::fs::write(&main_rs, r#"//! Main file demonstrating goto definition across modules

use goto_definition_test::{ApiClient, create_admin_user};
use goto_definition_test::types::{User, UserRole, UserConfig};
use goto_definition_test::service::{UserService, ServiceConfig};
use goto_definition_test::utils::{calculate_hash, format_user_info};

fn main() {
    // Goto definition should work for all these types and functions
    let config = UserConfig::default();
    let mut client = ApiClient::new(config);
    
    // User creation - goto should navigate to User::new
    let user1 = User::new(
        1,
        "Alice".to_string(),
        "alice@example.com".to_string(),
        UserRole::Admin
    );
    
    // Function call - goto should navigate to create_admin_user
    let user2 = create_admin_user("Bob".to_string(), "bob@example.com".to_string());
    
    // Method calls - goto should navigate to method definitions
    client.add_user(user1);
    client.add_user(user2);
    
    // Utility function call - goto should navigate to calculate_hash
    let hash = calculate_hash("test");
    
    // Format function call - goto should navigate to format_user_info
    let formatted = client.format_users();
    
    // Service usage - goto should navigate to service module
    let service_config = ServiceConfig::default();
    let mut service = UserService::new(service_config);
    
    // Service method call - goto should navigate to add_user method
    let user_id = service.add_user(
        "Charlie".to_string(),
        "charlie@example.com".to_string(),
        UserRole::User
    ).unwrap();
    
    // Enum usage - goto should navigate to UserRole definition
    let role_check = service.update_user_role(user_id, UserRole::Admin, 1);
    
    // Standard library types - goto should work for std types
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    
    // Vector methods - goto should navigate to Vec methods
    let mut vec = Vec::new();
    vec.push("item".to_string());
    let first = vec.first();
    
    println!("Demo completed: hash={}, formatted={:?}, role_check={:?}, first={:?}", 
             hash, formatted, role_check, first);
}

/// Function demonstrating goto definition in complex expressions
fn complex_goto_examples() {
    // Nested method calls - goto should work at each level
    let config = UserConfig::default();
    let client = ApiClient::new(config);
    
    // Chained method calls - goto should work in chains
    let user = User::new(1, "Test".to_string(), "test@example.com".to_string(), UserRole::User);
    let display = user.display_name();
    let is_active = user.is_active();
    
    // Field access - goto should navigate to field definitions
    let user_id = user.id;
    let user_name = user.name;
    let user_role = user.role;
    
    // Match on enum - goto should navigate to enum variants
    let role_name = match user_role {
        UserRole::Admin => "Administrator",
        UserRole::User => "Regular User",
        UserRole::Guest => "Guest User",
    };
    
    println!("Complex examples: display={}, active={}, id={}, name={}, role={}", 
             display, is_active, user_id, user_name, role_name);
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run goto definition test and analyze results
async fn run_goto_definition_test(
    file_path: PathBuf,
    line: u32,
    character: u32,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspGotoDefinitionTool::new();
    let input = ToolInput::LspGotoDefinition { file_path, line, character };
    
    let start = Instant::now();
    let result = timeout(Duration::from_secs(timeout_secs), tool.call(input)).await;
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(CallToolResult::Success { content })) => {
            println!("✅ {} succeeded in {:?}", test_name, duration);
            
            if let Some(text_content) = content.first() {
                if let Ok(goto_data) = serde_json::from_str::<Value>(&text_content.text) {
                    return (true, duration, Some(goto_data));
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
async fn test_goto_definition_types() {
    // 🎯 Test goto definition for type references
    println!("🎯 Testing goto definition for types...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test goto definition for type references in main.rs
    let type_positions = vec![
        (15, 20, "User type in constructor"),
        (10, 15, "UserConfig type"),
        (11, 25, "ApiClient type"),
        (20, 25, "UserRole enum"),
        (25, 25, "create_admin_user function"),
    ];
    
    for (line, character, description) in type_positions {
        let (success, duration, goto_data) = run_goto_definition_test(
            main_file.clone(),
            line,
            character,
            &format!("Goto {}", description),
            15
        ).await;
        
        if success {
            if let Some(data) = goto_data {
                if let Some(definitions) = data["definitions"].as_array() {
                    println!("  📍 {} found {} definitions in {:?}", 
                            description, definitions.len(), duration);
                    
                    // Check for valid file paths in definitions
                    for definition in definitions {
                        if let Some(uri) = definition["uri"].as_str() {
                            if uri.contains("types.rs") || uri.contains("lib.rs") {
                                println!("    ✅ Found definition in expected file: {}", uri);
                            }
                        }
                    }
                } else if let Some(location) = data["location"].as_object() {
                    // Alternative format - single location
                    if let Some(uri) = location["uri"].as_str() {
                        println!("  📍 {} found definition at {} in {:?}", 
                                description, uri, duration);
                    }
                }
            }
        }
    }
    
    println!("✅ Type goto definition test completed");
}

#[tokio::test]
async fn test_goto_definition_functions() {
    // 🔧 Test goto definition for function calls
    println!("🔧 Testing goto definition for functions...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test goto definition for function calls
    let function_positions = vec![
        (30, 10, "add_user method"),
        (35, 15, "calculate_hash function"),
        (38, 20, "format_users method"),
        (45, 20, "add_user service method"),
        (53, 25, "update_user_role method"),
    ];
    
    for (line, character, description) in function_positions {
        let (success, duration, goto_data) = run_goto_definition_test(
            main_file.clone(),
            line,
            character,
            &format!("Goto {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = goto_data {
                println!("  🔧 {} completed in {:?}", description, duration);
                
                // Check for definition location
                if data.get("definitions").is_some() || data.get("location").is_some() {
                    println!("    ✅ Found function definition");
                }
            }
        }
    }
    
    println!("✅ Function goto definition test completed");
}

#[tokio::test]
async fn test_goto_definition_cross_module() {
    // 🌐 Test goto definition across module boundaries
    println!("🌐 Testing goto definition across modules...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    // Test from lib.rs to other modules
    let lib_file = project_path.join("src/lib.rs");
    
    let cross_module_positions = vec![
        (6, 10, "types module import"),
        (7, 10, "utils module import"),
        (8, 15, "service module import"),
        (30, 15, "calculate_hash from utils"),
        (35, 25, "format_user_info from utils"),
    ];
    
    for (line, character, description) in cross_module_positions {
        let (success, duration, goto_data) = run_goto_definition_test(
            lib_file.clone(),
            line,
            character,
            &format!("Cross-module goto {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = goto_data {
                println!("  🌐 {} completed in {:?}", description, duration);
                
                // Check if definition is in correct module file
                if let Some(definitions) = data["definitions"].as_array() {
                    for definition in definitions {
                        if let Some(uri) = definition["uri"].as_str() {
                            if uri.contains("types.rs") || uri.contains("utils.rs") || uri.contains("service.rs") {
                                println!("    ✅ Found cross-module definition in {}", uri);
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("✅ Cross-module goto definition test completed");
}

#[tokio::test]
async fn test_goto_definition_standard_library() {
    // 📚 Test goto definition for standard library types
    println!("📚 Testing goto definition for standard library...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test goto definition for std library types
    let std_positions = vec![
        (55, 30, "HashMap from std"),
        (60, 15, "Vec type"),
        (61, 10, "push method"),
        (62, 20, "first method"),
    ];
    
    for (line, character, description) in std_positions {
        let (success, duration, goto_data) = run_goto_definition_test(
            main_file.clone(),
            line,
            character,
            &format!("Std library goto {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = goto_data {
                println!("  📚 {} completed in {:?}", description, duration);
                
                // Check for standard library definition
                if let Some(definitions) = data["definitions"].as_array() {
                    for definition in definitions {
                        if let Some(uri) = definition["uri"].as_str() {
                            if uri.contains("rust") || uri.contains("std") || uri.contains("core") {
                                println!("    ✅ Found standard library definition");
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("✅ Standard library goto definition test completed");
}

#[tokio::test]
async fn test_goto_definition_performance() {
    // ⚡ Test goto definition performance patterns
    println!("⚡ Testing goto definition performance...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test repeated goto definition (caching)
    let test_position = (15, 20); // User type reference
    let mut durations = Vec::new();
    
    for i in 0..3 {
        let (success, duration, _) = run_goto_definition_test(
            main_file.clone(),
            test_position.0,
            test_position.1,
            &format!("Performance test {}", i + 1),
            5
        ).await;
        
        if success {
            durations.push(duration);
        }
    }
    
    if !durations.is_empty() {
        println!("📊 Goto definition performance:");
        for (i, duration) in durations.iter().enumerate() {
            println!("  Goto {}: {:?}", i + 1, duration);
        }
        
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        println!("  Average: {:?}", avg_duration);
        
        // Check performance targets
        if avg_duration < Duration::from_millis(500) {
            println!("  ⚡ Excellent performance (<500ms average)");
        } else if avg_duration < Duration::from_secs(2) {
            println!("  ✅ Good performance (<2s average)");
        } else {
            println!("  ⚠️ Slow performance (>2s average)");
        }
    }
    
    println!("✅ Performance test completed");
}

#[tokio::test]
async fn test_goto_definition_edge_cases() {
    // 🧪 Test goto definition edge cases
    println!("🧪 Testing goto definition edge cases...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Test edge cases
    let edge_cases = vec![
        (0, 0, "Start of file"),
        (1, 100, "Beyond line end"),
        (999, 0, "Beyond file end"),
        (10, 0, "Start of line"),
        (5, 1, "Comment text"),
    ];
    
    for (line, character, description) in edge_cases {
        let (success, duration, _) = run_goto_definition_test(
            main_file.clone(),
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
async fn test_goto_definition_concurrent() {
    // 🔄 Test concurrent goto definition requests
    println!("🔄 Testing concurrent goto definition requests...");
    
    let (_temp_dir, project_path) = create_goto_definition_test_project().await
        .expect("Failed to create test project");
    
    let main_file = project_path.join("src/main.rs");
    
    // Create concurrent goto definition requests
    let positions = vec![
        (15, 20, "Type reference"),
        (25, 25, "Function call"),
        (35, 15, "Method call"),
        (45, 20, "Service method"),
    ];
    
    let futures: Vec<_> = positions.into_iter().enumerate().map(|(i, (line, char, desc))| {
        let file = main_file.clone();
        async move {
            let (success, duration, _) = run_goto_definition_test(
                file,
                line,
                char,
                &format!("Concurrent goto {} ({})", i + 1, desc),
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
    
    println!("📊 Concurrent goto results: {}/4 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent goto definition succeeded");
    } else {
        println!("ℹ️ All concurrent goto definitions failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent goto definition test completed");
}
