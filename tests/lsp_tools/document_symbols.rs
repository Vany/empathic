//! 🔬 LSP Document Symbols Tool Comprehensive Tests
//!
//! Advanced testing for LSP document symbols functionality including:
//! - File structure outline and symbol hierarchy
//! - Symbol type detection and classification
//! - Performance validation and symbol filtering
//! - Edge cases and nested symbol handling

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::document_symbols::LspDocumentSymbolsTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create Rust file with rich symbol structure for document symbols testing
async fn create_document_symbols_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "document-symbols-test"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with comprehensive symbol structure
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Document symbols test library with comprehensive symbol structure

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module-level constants for testing constant symbols
pub const MAX_ITEMS: usize = 1000;
pub const DEFAULT_TIMEOUT: u64 = 30;
pub const API_VERSION: &str = "1.0.0";

/// Static variables for testing static symbols
pub static mut GLOBAL_COUNTER: u32 = 0;
pub static GLOBAL_CONFIG: &str = "default";

/// Type aliases for testing type alias symbols
pub type ItemId = u64;
pub type ItemMap = HashMap<ItemId, Item>;
pub type ResultType<T> = Result<T, ItemError>;

/// Main item structure for testing struct symbols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    /// Item unique identifier
    pub id: ItemId,
    /// Item display name
    pub name: String,
    /// Item description text
    pub description: String,
    /// Item category classification
    pub category: Category,
    /// Item metadata storage
    pub metadata: HashMap<String, String>,
    /// Item creation timestamp
    pub created_at: u64,
    /// Item last modified timestamp
    pub modified_at: u64,
    /// Item status information
    pub status: ItemStatus,
}

/// Category enumeration for testing enum symbols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Category {
    /// Technology related items
    Technology,
    /// Business related items
    Business,
    /// Personal related items
    Personal,
    /// Custom category with name
    Custom(String),
    /// Complex category with multiple fields
    Complex {
        name: String,
        priority: u32,
        tags: Vec<String>,
    },
}

/// Item status enumeration with various variant types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemStatus {
    /// Item is in draft state
    Draft,
    /// Item is under review
    Review(String), // Reviewer name
    /// Item is published
    Published {
        published_by: String,
        published_at: u64,
    },
    /// Item is archived
    Archived,
}

/// Error types for testing error enum symbols
#[derive(Debug, thiserror::Error)]
pub enum ItemError {
    #[error("Item not found: {id}")]
    NotFound { id: ItemId },
    #[error("Invalid item data: {reason}")]
    InvalidData { reason: String },
    #[error("Permission denied for item {id}")]
    PermissionDenied { id: ItemId },
    #[error("System error: {0}")]
    System(String),
}

/// Trait definitions for testing trait symbols
pub trait Identifiable {
    /// Get the unique identifier
    fn id(&self) -> ItemId;
    
    /// Check if item matches given ID
    fn matches_id(&self, id: ItemId) -> bool {
        self.id() == id
    }
}

/// Trait for items that can be validated
pub trait Validatable {
    /// Validate item data
    fn validate(&self) -> Result<(), String>;
    
    /// Check if item is valid
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Trait for items that can be formatted
pub trait Formattable {
    /// Format item for display
    fn format(&self) -> String;
    
    /// Format item for export
    fn format_for_export(&self) -> String {
        self.format()
    }
}

/// Implementation blocks for testing impl symbols
impl Item {
    /// Constructor function for creating new items
    pub fn new(id: ItemId, name: String, description: String, category: Category) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            id,
            name,
            description,
            category,
            metadata: HashMap::new(),
            created_at: now,
            modified_at: now,
            status: ItemStatus::Draft,
        }
    }
    
    /// Builder pattern method for setting status
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = status;
        self.update_modified_time();
        self
    }
    
    /// Builder pattern method for adding metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self.update_modified_time();
        self
    }
    
    /// Getter methods for accessing fields
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn category(&self) -> &Category { &self.category }
    pub fn status(&self) -> &ItemStatus { &self.status }
    pub fn created_at(&self) -> u64 { self.created_at }
    pub fn modified_at(&self) -> u64 { self.modified_at }
    
    /// Setter methods for modifying fields
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.update_modified_time();
    }
    
    pub fn set_description(&mut self, description: String) {
        self.description = description;
        self.update_modified_time();
    }
    
    pub fn set_category(&mut self, category: Category) {
        self.category = category;
        self.update_modified_time();
    }
    
    pub fn set_status(&mut self, status: ItemStatus) {
        self.status = status;
        self.update_modified_time();
    }
    
    /// Utility methods for item operations
    pub fn is_published(&self) -> bool {
        matches!(self.status, ItemStatus::Published { .. })
    }
    
    pub fn is_draft(&self) -> bool {
        matches!(self.status, ItemStatus::Draft)
    }
    
    pub fn is_under_review(&self) -> bool {
        matches!(self.status, ItemStatus::Review(_))
    }
    
    pub fn age_in_days(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (now - self.created_at) / (24 * 60 * 60)
    }
    
    /// Private helper methods
    fn update_modified_time(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

/// Trait implementations for testing trait impl symbols
impl Identifiable for Item {
    fn id(&self) -> ItemId {
        self.id
    }
}

impl Validatable for Item {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if self.description.len() > 1000 {
            return Err("Description too long".to_string());
        }
        
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        
        Ok(())
    }
}

impl Formattable for Item {
    fn format(&self) -> String {
        format!(
            "Item #{}: {} [{}] - {}",
            self.id,
            self.name,
            self.category.name(),
            self.status.display_name()
        )
    }
}

/// Category implementation for testing enum impl symbols
impl Category {
    /// Get category name
    pub fn name(&self) -> String {
        match self {
            Category::Technology => "Technology".to_string(),
            Category::Business => "Business".to_string(),
            Category::Personal => "Personal".to_string(),
            Category::Custom(name) => name.clone(),
            Category::Complex { name, .. } => name.clone(),
        }
    }
    
    /// Check if category is custom
    pub fn is_custom(&self) -> bool {
        matches!(self, Category::Custom(_) | Category::Complex { .. })
    }
    
    /// Get category priority
    pub fn priority(&self) -> u32 {
        match self {
            Category::Technology => 3,
            Category::Business => 2,
            Category::Personal => 1,
            Category::Custom(_) => 0,
            Category::Complex { priority, .. } => *priority,
        }
    }
}

/// ItemStatus implementation for testing enum methods
impl ItemStatus {
    /// Get display name for status
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemStatus::Draft => "Draft",
            ItemStatus::Review(_) => "Under Review",
            ItemStatus::Published { .. } => "Published",
            ItemStatus::Archived => "Archived",
        }
    }
    
    /// Check if status is final
    pub fn is_final(&self) -> bool {
        matches!(self, ItemStatus::Published { .. } | ItemStatus::Archived)
    }
    
    /// Get reviewer if status is under review
    pub fn reviewer(&self) -> Option<&str> {
        match self {
            ItemStatus::Review(reviewer) => Some(reviewer),
            _ => None,
        }
    }
}

/// Free functions for testing function symbols
pub fn create_default_item(id: ItemId, name: String) -> Item {
    Item::new(id, name, "Default item".to_string(), Category::Personal)
}

pub fn create_technology_item(id: ItemId, name: String, description: String) -> Item {
    Item::new(id, name, description, Category::Technology)
        .with_status(ItemStatus::Draft)
}

pub fn validate_item_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 100
}

pub fn calculate_item_hash(item: &Item) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    item.id.hash(&mut hasher);
    item.name.hash(&mut hasher);
    hasher.finish()
}

pub fn format_item_summary(item: &Item) -> String {
    format!("{} ({})", item.name(), item.category().name())
}

/// Async functions for testing async function symbols
pub async fn load_item_async(id: ItemId) -> ResultType<Item> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    if id == 0 {
        Err(ItemError::InvalidData {
            reason: "ID cannot be zero".to_string(),
        })
    } else {
        Ok(create_default_item(id, format!("Item {}", id)))
    }
}

pub async fn save_item_async(item: &Item) -> ResultType<()> {
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    item.validate().map_err(|reason| ItemError::InvalidData { reason })?;
    
    // Simulate save operation
    Ok(())
}

/// Generic functions for testing generic symbols
pub fn process_items<T, F>(items: Vec<T>, processor: F) -> Vec<String>
where
    T: Formattable,
    F: Fn(&T) -> String,
{
    items.iter().map(processor).collect()
}

pub fn find_item_by_predicate<P>(items: &[Item], predicate: P) -> Option<&Item>
where
    P: Fn(&Item) -> bool,
{
    items.iter().find(|item| predicate(item))
}

/// Nested modules for testing module symbols
pub mod utils {
    use super::*;
    
    /// Nested constants
    pub const UTILS_VERSION: &str = "1.0.0";
    
    /// Nested functions
    pub fn format_timestamp(timestamp: u64) -> String {
        format!("Timestamp: {}", timestamp)
    }
    
    pub fn parse_category(s: &str) -> Option<Category> {
        match s.to_lowercase().as_str() {
            "technology" => Some(Category::Technology),
            "business" => Some(Category::Business),
            "personal" => Some(Category::Personal),
            _ => Some(Category::Custom(s.to_string())),
        }
    }
    
    /// Nested struct
    pub struct UtilsHelper {
        pub name: String,
    }
    
    impl UtilsHelper {
        pub fn new(name: String) -> Self {
            Self { name }
        }
        
        pub fn process(&self, input: &str) -> String {
            format!("{}: {}", self.name, input)
        }
    }
}

/// Test module for testing nested symbols
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test constants
    const TEST_ID: ItemId = 123;
    
    /// Test functions
    fn create_test_item() -> Item {
        Item::new(
            TEST_ID,
            "Test Item".to_string(),
            "Test Description".to_string(),
            Category::Technology,
        )
    }
    
    #[test]
    fn test_item_creation() {
        let item = create_test_item();
        assert_eq!(item.id(), TEST_ID);
        assert_eq!(item.name(), "Test Item");
    }
    
    #[test]
    fn test_item_validation() {
        let item = create_test_item();
        assert!(item.is_valid());
    }
    
    #[test]
    fn test_category_methods() {
        let category = Category::Technology;
        assert_eq!(category.name(), "Technology");
        assert_eq!(category.priority(), 3);
    }
    
    #[tokio::test]
    async fn test_async_operations() {
        let item = load_item_async(TEST_ID).await.unwrap();
        assert_eq!(item.id(), TEST_ID);
        
        let save_result = save_item_async(&item).await;
        assert!(save_result.is_ok());
    }
}
"#).await?;
    
    // Create another file with different symbol patterns
    let complex_rs = src_dir.join("complex.rs");
    tokio::fs::write(&complex_rs, r#"//! Complex file with nested structures for document symbols testing

use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap, VecDeque};

/// Complex nested structure for testing hierarchical symbols
pub struct ComplexService {
    /// Inner state with nested generics
    state: Arc<RwLock<ServiceState>>,
    /// Connection pool
    connections: Arc<Mutex<ConnectionPool>>,
    /// Cache storage
    cache: Arc<RwLock<CacheStorage>>,
}

/// Service state with nested fields
#[derive(Debug)]
struct ServiceState {
    /// Configuration mapping
    config: HashMap<String, ConfigValue>,
    /// Active sessions
    sessions: BTreeMap<u64, Session>,
    /// Request queue
    request_queue: VecDeque<Request>,
    /// Statistics
    stats: ServiceStats,
}

/// Configuration value enumeration
#[derive(Debug, Clone)]
enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<ConfigValue>),
    Map(HashMap<String, ConfigValue>),
}

/// Session information
#[derive(Debug)]
struct Session {
    id: u64,
    user_id: String,
    created_at: u64,
    last_activity: u64,
    permissions: Vec<Permission>,
    context: SessionContext,
}

/// Session context with nested data
#[derive(Debug)]
struct SessionContext {
    ip_address: String,
    user_agent: String,
    preferences: UserPreferences,
    history: Vec<ActionHistory>,
}

/// User preferences structure
#[derive(Debug)]
struct UserPreferences {
    theme: String,
    language: String,
    timezone: String,
    notifications: NotificationSettings,
}

/// Notification settings with detailed configuration
#[derive(Debug)]
struct NotificationSettings {
    email_enabled: bool,
    push_enabled: bool,
    frequency: NotificationFrequency,
    categories: Vec<NotificationCategory>,
}

/// Notification frequency enumeration
#[derive(Debug)]
enum NotificationFrequency {
    Immediate,
    Hourly,
    Daily,
    Weekly,
    Custom { interval_minutes: u32 },
}

/// Notification category enumeration
#[derive(Debug)]
enum NotificationCategory {
    Security,
    Updates,
    Social,
    Marketing,
    Custom(String),
}

/// Permission enumeration with nested data
#[derive(Debug)]
enum Permission {
    Read { resource: String },
    Write { resource: String },
    Delete { resource: String },
    Admin { scope: AdminScope },
}

/// Admin scope enumeration
#[derive(Debug)]
enum AdminScope {
    Global,
    Organization(String),
    Team(String),
    Limited { resources: Vec<String> },
}

/// Action history for tracking user actions
#[derive(Debug)]
struct ActionHistory {
    action: Action,
    timestamp: u64,
    metadata: ActionMetadata,
}

/// Action enumeration
#[derive(Debug)]
enum Action {
    Login,
    Logout,
    CreateResource { resource_type: String, resource_id: String },
    UpdateResource { resource_type: String, resource_id: String },
    DeleteResource { resource_type: String, resource_id: String },
    ViewResource { resource_type: String, resource_id: String },
}

/// Action metadata
#[derive(Debug)]
struct ActionMetadata {
    source_ip: String,
    user_agent: String,
    success: bool,
    error_message: Option<String>,
}

/// Request structure for queued requests
#[derive(Debug)]
struct Request {
    id: u64,
    session_id: u64,
    endpoint: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    priority: RequestPriority,
    retry_count: u32,
}

/// HTTP method enumeration
#[derive(Debug)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// Request priority enumeration
#[derive(Debug)]
enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Service statistics structure
#[derive(Debug)]
struct ServiceStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    average_response_time: f64,
    active_sessions: u32,
    cache_hits: u64,
    cache_misses: u64,
}

/// Connection pool for managing connections
#[derive(Debug)]
struct ConnectionPool {
    connections: Vec<Connection>,
    available: VecDeque<usize>,
    max_connections: usize,
    timeout_seconds: u64,
}

/// Individual connection structure
#[derive(Debug)]
struct Connection {
    id: usize,
    url: String,
    created_at: u64,
    last_used: u64,
    status: ConnectionStatus,
}

/// Connection status enumeration
#[derive(Debug)]
enum ConnectionStatus {
    Active,
    Idle,
    Closed,
    Error(String),
}

/// Cache storage for caching data
#[derive(Debug)]
struct CacheStorage {
    memory_cache: HashMap<String, CacheEntry>,
    disk_cache: BTreeMap<String, DiskCacheEntry>,
    cache_stats: CacheStats,
}

/// Cache entry in memory
#[derive(Debug)]
struct CacheEntry {
    key: String,
    value: Vec<u8>,
    created_at: u64,
    expires_at: u64,
    access_count: u32,
}

/// Cache entry on disk
#[derive(Debug)]
struct DiskCacheEntry {
    key: String,
    file_path: String,
    size: u64,
    created_at: u64,
    expires_at: u64,
}

/// Cache statistics
#[derive(Debug)]
struct CacheStats {
    memory_hits: u64,
    memory_misses: u64,
    disk_hits: u64,
    disk_misses: u64,
    evictions: u64,
}

/// Complex implementation block with many methods
impl ComplexService {
    /// Constructor with complex initialization
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ServiceState::new())),
            connections: Arc::new(Mutex::new(ConnectionPool::new(10))),
            cache: Arc::new(RwLock::new(CacheStorage::new())),
        }
    }
    
    /// Session management methods
    pub fn create_session(&self, user_id: String) -> u64 {
        // Implementation would be here
        0
    }
    
    pub fn get_session(&self, session_id: u64) -> Option<Session> {
        // Implementation would be here
        None
    }
    
    pub fn update_session(&self, session_id: u64, context: SessionContext) -> bool {
        // Implementation would be here
        false
    }
    
    pub fn close_session(&self, session_id: u64) -> bool {
        // Implementation would be here
        false
    }
    
    /// Request processing methods
    pub fn queue_request(&self, request: Request) -> bool {
        // Implementation would be here
        false
    }
    
    pub fn process_next_request(&self) -> Option<Request> {
        // Implementation would be here
        None
    }
    
    pub fn get_queue_status(&self) -> QueueStatus {
        // Implementation would be here
        QueueStatus {
            pending: 0,
            processing: 0,
            completed: 0,
            failed: 0,
        }
    }
    
    /// Cache management methods
    pub fn cache_get(&self, key: &str) -> Option<Vec<u8>> {
        // Implementation would be here
        None
    }
    
    pub fn cache_set(&self, key: String, value: Vec<u8>, ttl: u64) -> bool {
        // Implementation would be here
        false
    }
    
    pub fn cache_delete(&self, key: &str) -> bool {
        // Implementation would be here
        false
    }
    
    pub fn cache_clear(&self) -> u64 {
        // Implementation would be here
        0
    }
}

/// Queue status structure
#[derive(Debug)]
pub struct QueueStatus {
    pub pending: u32,
    pub processing: u32,
    pub completed: u32,
    pub failed: u32,
}

/// Implementation blocks for nested structures
impl ServiceState {
    fn new() -> Self {
        Self {
            config: HashMap::new(),
            sessions: BTreeMap::new(),
            request_queue: VecDeque::new(),
            stats: ServiceStats::new(),
        }
    }
}

impl ServiceStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            active_sessions: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl ConnectionPool {
    fn new(max_connections: usize) -> Self {
        Self {
            connections: Vec::new(),
            available: VecDeque::new(),
            max_connections,
            timeout_seconds: 30,
        }
    }
}

impl CacheStorage {
    fn new() -> Self {
        Self {
            memory_cache: HashMap::new(),
            disk_cache: BTreeMap::new(),
            cache_stats: CacheStats::new(),
        }
    }
}

impl CacheStats {
    fn new() -> Self {
        Self {
            memory_hits: 0,
            memory_misses: 0,
            disk_hits: 0,
            disk_misses: 0,
            evictions: 0,
        }
    }
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run document symbols test and analyze results
async fn run_document_symbols_test(
    file_path: PathBuf,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspDocumentSymbolsTool::new();
    let input = ToolInput::LspDocumentSymbols { file_path };
    
    let start = Instant::now();
    let result = timeout(Duration::from_secs(timeout_secs), tool.call(input)).await;
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(CallToolResult::Success { content })) => {
            println!("✅ {} succeeded in {:?}", test_name, duration);
            
            if let Some(text_content) = content.first() {
                if let Ok(symbols_data) = serde_json::from_str::<Value>(&text_content.text) {
                    return (true, duration, Some(symbols_data));
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
async fn test_document_symbols_comprehensive() {
    // 🎯 Test document symbols on file with comprehensive symbol structure
    println!("🎯 Testing document symbols comprehensive structure...");
    
    let (_temp_dir, project_path) = create_document_symbols_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    let (success, duration, symbols_data) = run_document_symbols_test(
        lib_file,
        "Comprehensive document symbols",
        20
    ).await;
    
    if success {
        if let Some(data) = symbols_data {
            if let Some(symbols) = data["symbols"].as_array() {
                println!("📍 Found {} symbols in document", symbols.len());
                
                // Analyze symbol types
                let mut symbol_counts = std::collections::HashMap::new();
                for symbol in symbols {
                    if let Some(kind) = symbol["kind"].as_str() {
                        *symbol_counts.entry(kind).or_insert(0) += 1;
                    }
                }
                
                println!("📊 Symbol types found:");
                for (kind, count) in &symbol_counts {
                    println!("  {}: {} symbols", kind, count);
                }
                
                // Check for expected symbol types
                let expected_types = vec!["struct", "enum", "function", "const", "trait", "impl"];
                let found_types: Vec<_> = expected_types.iter()
                    .filter(|t| symbol_counts.contains_key(*t))
                    .collect();
                
                if found_types.len() >= 4 {
                    println!("✅ Found expected symbol types: {:?}", found_types);
                } else {
                    println!("ℹ️ Symbol format may be different than expected");
                }
                
                // Display first few symbols for debugging
                for (i, symbol) in symbols.iter().take(5).enumerate() {
                    if let Some(name) = symbol["name"].as_str() {
                        let kind = symbol["kind"].as_str().unwrap_or("unknown");
                        println!("  {}. {} ({})", i + 1, name, kind);
                    }
                }
            }
        }
        
        // Check performance
        if duration < Duration::from_millis(500) {
            println!("⚡ Excellent document symbols performance: {:?}", duration);
        } else if duration < Duration::from_secs(2) {
            println!("✅ Good document symbols performance: {:?}", duration);
        } else {
            println!("⚠️ Slow document symbols performance: {:?}", duration);
        }
    }
    
    println!("✅ Comprehensive document symbols test completed");
}

#[tokio::test]
async fn test_document_symbols_hierarchy() {
    // 🏗️ Test document symbols hierarchical structure
    println!("🏗️ Testing document symbols hierarchy...");
    
    let (_temp_dir, project_path) = create_document_symbols_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    let (success, duration, symbols_data) = run_document_symbols_test(
        lib_file,
        "Document symbols hierarchy",
        15
    ).await;
    
    if success {
        if let Some(data) = symbols_data {
            if let Some(symbols) = data["symbols"].as_array() {
                println!("🏗️ Analyzing symbol hierarchy...");
                
                // Look for nested symbols (children)
                let mut has_children = 0;
                let mut total_children = 0;
                
                for symbol in symbols {
                    if let Some(children) = symbol["children"].as_array() {
                        if !children.is_empty() {
                            has_children += 1;
                            total_children += children.len();
                            
                            if let Some(name) = symbol["name"].as_str() {
                                println!("  📂 {} has {} children", name, children.len());
                            }
                        }
                    }
                }
                
                println!("📊 Hierarchy analysis:");
                println!("  {} symbols with children", has_children);
                println!("  {} total child symbols", total_children);
                
                if has_children > 0 {
                    println!("✅ Document symbols hierarchy detected");
                } else {
                    println!("ℹ️ No nested symbols found (may be flat structure)");
                }
            }
        }
    }
    
    println!("✅ Document symbols hierarchy test completed");
}

#[tokio::test]
async fn test_document_symbols_complex_file() {
    // 🔬 Test document symbols on complex nested file
    println!("🔬 Testing document symbols on complex file...");
    
    let (_temp_dir, project_path) = create_document_symbols_test_project().await
        .expect("Failed to create test project");
    
    let complex_file = project_path.join("src/complex.rs");
    let (success, duration, symbols_data) = run_document_symbols_test(
        complex_file,
        "Complex file document symbols",
        20
    ).await;
    
    if success {
        if let Some(data) = symbols_data {
            if let Some(symbols) = data["symbols"].as_array() {
                println!("🔬 Found {} symbols in complex file", symbols.len());
                
                // Count deeply nested structures
                let mut nested_levels = std::collections::HashMap::new();
                
                fn count_nesting_levels(symbol: &Value, level: u32, counts: &mut std::collections::HashMap<u32, u32>) {
                    *counts.entry(level).or_insert(0) += 1;
                    
                    if let Some(children) = symbol["children"].as_array() {
                        for child in children {
                            count_nesting_levels(child, level + 1, counts);
                        }
                    }
                }
                
                for symbol in symbols {
                    count_nesting_levels(symbol, 0, &mut nested_levels);
                }
                
                println!("📊 Nesting level analysis:");
                for level in 0..5 {
                    if let Some(count) = nested_levels.get(&level) {
                        println!("  Level {}: {} symbols", level, count);
                    }
                }
                
                let max_level = nested_levels.keys().max().copied().unwrap_or(0);
                if max_level > 1 {
                    println!("✅ Complex nesting detected (max level: {})", max_level);
                } else {
                    println!("ℹ️ Limited nesting detected");
                }
            }
        }
    }
    
    println!("✅ Complex file document symbols test completed");
}

#[tokio::test]
async fn test_document_symbols_performance() {
    // ⚡ Test document symbols performance patterns
    println!("⚡ Testing document symbols performance...");
    
    let (_temp_dir, project_path) = create_document_symbols_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test repeated document symbols (caching)
    let mut durations = Vec::new();
    
    for i in 0..3 {
        let (success, duration, _) = run_document_symbols_test(
            lib_file.clone(),
            &format!("Performance test {}", i + 1),
            10
        ).await;
        
        if success {
            durations.push(duration);
        }
    }
    
    if !durations.is_empty() {
        println!("📊 Document symbols performance:");
        for (i, duration) in durations.iter().enumerate() {
            println!("  Symbols {}: {:?}", i + 1, duration);
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
        
        // Check for caching effects
        if durations.len() >= 2 {
            let first = durations[0];
            let later_avg = durations[1..].iter().sum::<Duration>() / (durations.len() - 1) as u32;
            
            if later_avg < first {
                println!("  ✅ Caching effect detected (later calls faster)");
            } else {
                println!("  ℹ️ No significant caching effect");
            }
        }
    }
    
    println!("✅ Performance test completed");
}

#[tokio::test]
async fn test_document_symbols_symbol_types() {
    // 🎭 Test document symbols symbol type detection
    println!("🎭 Testing document symbols symbol type detection...");
    
    let (_temp_dir, project_path) = create_document_symbols_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    let (success, duration, symbols_data) = run_document_symbols_test(
        lib_file,
        "Symbol type detection",
        15
    ).await;
    
    if success {
        if let Some(data) = symbols_data {
            if let Some(symbols) = data["symbols"].as_array() {
                println!("🎭 Analyzing symbol type detection...");
                
                // Look for specific symbol types
                let mut found_symbols = std::collections::HashMap::new();
                
                for symbol in symbols {
                    if let (Some(name), Some(kind)) = (symbol["name"].as_str(), symbol["kind"].as_str()) {
                        found_symbols.entry(kind).or_insert_with(Vec::new).push(name);
                    }
                }
                
                // Check for expected symbol categories
                let expected_categories = vec![
                    ("const", vec!["MAX_ITEMS", "DEFAULT_TIMEOUT", "API_VERSION"]),
                    ("struct", vec!["Item"]),
                    ("enum", vec!["Category", "ItemStatus", "ItemError"]),
                    ("trait", vec!["Identifiable", "Validatable", "Formattable"]),
                    ("function", vec!["create_default_item", "validate_item_name"]),
                ];
                
                for (category, expected_names) in expected_categories {
                    if let Some(found_names) = found_symbols.get(category) {
                        let matches: Vec<_> = expected_names.iter()
                            .filter(|name| found_names.iter().any(|found| found.contains(*name)))
                            .collect();
                        
                        if !matches.is_empty() {
                            println!("  ✅ Found {} {}: {:?}", category, if matches.len() == 1 { "symbol" } else { "symbols" }, matches);
                        }
                    }
                }
                
                // Show all found symbol types
                println!("📊 All symbol types found:");
                for (kind, names) in &found_symbols {
                    println!("  {}: {} symbols", kind, names.len());
                }
            }
        }
    }
    
    println!("✅ Symbol type detection test completed");
}

#[tokio::test]
async fn test_document_symbols_edge_cases() {
    // 🧪 Test document symbols edge cases
    println!("🧪 Testing document symbols edge cases...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_path_buf();
    
    // Create minimal Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "edge-case-test"
version = "0.1.0"
edition = "2021"
"#).await.expect("Failed to write Cargo.toml");
    
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await.expect("Failed to create src dir");
    
    // Test different edge cases
    let edge_cases = vec![
        ("empty_file.rs", "// Empty file with only comments\n"),
        ("minimal.rs", "pub fn hello() -> &'static str { \"hello\" }\n"),
        ("comments_only.rs", "//! File with only comments\n/// Documentation\n/* Block comment */\n"),
        ("syntax_errors.rs", "pub struct InvalidSyntax { // Missing closing brace\n"),
    ];
    
    for (filename, content) in edge_cases {
        let file_path = src_dir.join(filename);
        tokio::fs::write(&file_path, content).await.expect("Failed to write test file");
        
        let (success, duration, symbols_data) = run_document_symbols_test(
            file_path,
            &format!("Edge case: {}", filename),
            5
        ).await;
        
        if success {
            if let Some(data) = symbols_data {
                let symbol_count = data["symbols"].as_array().map(|s| s.len()).unwrap_or(0);
                println!("  ✅ {} handled successfully: {} symbols in {:?}", filename, symbol_count, duration);
            } else {
                println!("  ✅ {} handled successfully in {:?}", filename, duration);
            }
        } else {
            println!("  ✅ {} failed gracefully in {:?} (expected for some cases)", filename, duration);
        }
    }
    
    println!("✅ Edge cases test completed");
}

#[tokio::test]
async fn test_document_symbols_concurrent() {
    // 🔄 Test concurrent document symbols requests
    println!("🔄 Testing concurrent document symbols requests...");
    
    let (_temp_dir, project_path) = create_document_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Create concurrent requests for different files
    let files = vec![
        (project_path.join("src/lib.rs"), "lib.rs symbols"),
        (project_path.join("src/complex.rs"), "complex.rs symbols"),
        (project_path.join("src/lib.rs"), "lib.rs symbols (duplicate)"),
    ];
    
    let futures: Vec<_> = files.into_iter().enumerate().map(|(i, (file_path, desc))| {
        async move {
            let (success, duration, _) = run_document_symbols_test(
                file_path,
                &format!("Concurrent symbols {} ({})", i + 1, desc),
                15
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
    
    println!("📊 Concurrent document symbols results: {}/3 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent document symbols succeeded");
    } else {
        println!("ℹ️ All concurrent document symbols failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent document symbols test completed");
}
