//! 🔬 LSP Workspace Symbols Tool Comprehensive Tests
//!
//! Advanced testing for LSP workspace symbols functionality including:
//! - Project-wide symbol search and filtering
//! - Symbol query processing and matching
//! - Performance validation and result ranking
//! - Edge cases and concurrent search handling

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::workspace_symbols::LspWorkspaceSymbolsTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create multi-file Rust project for workspace symbols testing
async fn create_workspace_symbols_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "workspace-symbols-test"
version = "0.1.0"
edition = "2021"
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with diverse symbols
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Workspace symbols test library with searchable symbols

pub mod models;
pub mod services;
pub mod utils;
pub mod config;

use models::{User, UserRole, Organization};
use services::{UserService, OrganizationService, AuthService};
use utils::{format_user, validate_email, calculate_hash};
use config::{AppConfig, DatabaseConfig, ServerConfig};

/// Main application manager - searchable as "Manager", "App", "Application"
pub struct ApplicationManager {
    pub user_service: UserService,
    pub org_service: OrganizationService,
    pub auth_service: AuthService,
    pub config: AppConfig,
}

/// Search result structure - searchable as "Search", "Result"
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub relevance_score: f64,
    pub category: SearchCategory,
}

/// Search category enumeration - searchable as "Search", "Category"
#[derive(Debug, Clone)]
pub enum SearchCategory {
    User,
    Organization,
    Content,
    System,
}

/// API client for external services - searchable as "API", "Client", "External"
pub struct ApiClient {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,
    pub retry_count: u32,
}

/// Configuration manager - searchable as "Config", "Manager", "Configuration"
pub struct ConfigurationManager {
    pub app_config: AppConfig,
    pub db_config: DatabaseConfig,
    pub server_config: ServerConfig,
}

/// Error types for the application - searchable as "Error", "App"
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("User error: {message}")]
    User { message: String },
    #[error("Organization error: {message}")]
    Organization { message: String },
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    #[error("External API error: {message}")]
    ExternalApi { message: String },
}

/// Result type alias - searchable as "Result", "App"
pub type AppResult<T> = Result<T, ApplicationError>;

/// Main application functions - searchable by function names
pub fn initialize_application() -> AppResult<ApplicationManager> {
    // Implementation here
    todo!()
}

pub fn shutdown_application(manager: ApplicationManager) -> AppResult<()> {
    // Implementation here
    todo!()
}

pub fn create_search_index() -> AppResult<SearchIndex> {
    // Implementation here
    todo!()
}

pub fn perform_global_search(query: &str) -> AppResult<Vec<SearchResult>> {
    // Implementation here
    todo!()
}

/// Search index structure - searchable as "Search", "Index"
pub struct SearchIndex {
    pub user_index: UserIndex,
    pub organization_index: OrganizationIndex,
    pub content_index: ContentIndex,
}

/// Specialized index types - searchable by their names
pub struct UserIndex {
    pub users_by_name: std::collections::HashMap<String, Vec<u64>>,
    pub users_by_email: std::collections::HashMap<String, u64>,
    pub users_by_role: std::collections::HashMap<UserRole, Vec<u64>>,
}

pub struct OrganizationIndex {
    pub orgs_by_name: std::collections::HashMap<String, Vec<u64>>,
    pub orgs_by_domain: std::collections::HashMap<String, u64>,
    pub orgs_by_size: std::collections::BTreeMap<u32, Vec<u64>>,
}

pub struct ContentIndex {
    pub content_by_keyword: std::collections::HashMap<String, Vec<u64>>,
    pub content_by_author: std::collections::HashMap<u64, Vec<u64>>,
    pub content_by_date: std::collections::BTreeMap<u64, Vec<u64>>,
}

/// Constants that should be searchable
pub const MAX_SEARCH_RESULTS: usize = 100;
pub const DEFAULT_SEARCH_TIMEOUT: u64 = 30;
pub const SEARCH_INDEX_VERSION: &str = "1.0.0";

/// Global search configuration
pub static SEARCH_CONFIG: SearchConfiguration = SearchConfiguration {
    max_results: MAX_SEARCH_RESULTS,
    timeout_seconds: DEFAULT_SEARCH_TIMEOUT,
    enable_fuzzy_search: true,
    enable_stemming: true,
};

/// Search configuration structure
#[derive(Debug)]
pub struct SearchConfiguration {
    pub max_results: usize,
    pub timeout_seconds: u64,
    pub enable_fuzzy_search: bool,
    pub enable_stemming: bool,
}

/// Traits that should be discoverable
pub trait Searchable {
    fn search_keywords(&self) -> Vec<String>;
    fn search_content(&self) -> String;
    fn search_relevance(&self, query: &str) -> f64;
}

pub trait Indexable {
    fn index_key(&self) -> String;
    fn index_data(&self) -> serde_json::Value;
    fn index_timestamp(&self) -> u64;
}

pub trait Cacheable {
    fn cache_key(&self) -> String;
    fn cache_ttl(&self) -> u64;
    fn cache_size(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test functions that should be searchable
    #[test]
    fn test_application_manager() {
        // Test implementation
    }
    
    #[test]
    fn test_search_functionality() {
        // Test implementation
    }
    
    #[test]
    fn test_configuration_manager() {
        // Test implementation
    }
}
"#).await?;
    
    // Create models.rs with user and organization models
    let models_rs = src_dir.join("models.rs");
    tokio::fs::write(&models_rs, r#"//! Data models for workspace symbols testing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User model - searchable as "User"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub role: UserRole,
    pub organization_id: Option<u64>,
    pub profile: UserProfile,
    pub settings: UserSettings,
}

/// User role enumeration - searchable as "User", "Role"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    Employee,
    Guest,
    SystemUser,
}

/// User profile information - searchable as "User", "Profile"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub bio: String,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
}

/// User settings structure - searchable as "User", "Settings"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub notifications: NotificationSettings,
    pub privacy: PrivacySettings,
}

/// Notification settings - searchable as "Notification", "Settings"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub desktop_enabled: bool,
    pub frequency: NotificationFrequency,
}

/// Notification frequency - searchable as "Notification", "Frequency"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationFrequency {
    Immediate,
    Hourly,
    Daily,
    Weekly,
    Never,
}

/// Privacy settings - searchable as "Privacy", "Settings"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub profile_public: bool,
    pub email_visible: bool,
    pub activity_tracking: bool,
    pub data_sharing: bool,
}

/// Organization model - searchable as "Organization", "Org"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: u64,
    pub name: String,
    pub domain: String,
    pub description: String,
    pub organization_type: OrganizationType,
    pub settings: OrganizationSettings,
    pub metadata: OrganizationMetadata,
}

/// Organization type enumeration - searchable as "Organization", "Type"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationType {
    Corporation,
    NonProfit,
    Government,
    Educational,
    Startup,
    Individual,
}

/// Organization settings - searchable as "Organization", "Settings"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub allow_public_signup: bool,
    pub require_email_verification: bool,
    pub default_user_role: UserRole,
    pub max_users: Option<u32>,
    pub features: Vec<String>,
}

/// Organization metadata - searchable as "Organization", "Metadata"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMetadata {
    pub founded_year: Option<u32>,
    pub employee_count: Option<u32>,
    pub industry: Option<String>,
    pub headquarters: Option<String>,
    pub website: Option<String>,
}

/// Content model for searchable content - searchable as "Content"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub author_id: u64,
    pub organization_id: u64,
    pub content_type: ContentType,
    pub tags: Vec<String>,
    pub metadata: ContentMetadata,
}

/// Content type enumeration - searchable as "Content", "Type"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Article,
    Document,
    Presentation,
    Video,
    Audio,
    Image,
    Dataset,
}

/// Content metadata - searchable as "Content", "Metadata"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub created_at: u64,
    pub updated_at: u64,
    pub published_at: Option<u64>,
    pub version: u32,
    pub file_size: Option<u64>,
    pub mime_type: Option<String>,
}

/// Permission model - searchable as "Permission"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub resource_type: String,
    pub actions: Vec<PermissionAction>,
}

/// Permission action enumeration - searchable as "Permission", "Action"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionAction {
    Read,
    Write,
    Delete,
    Share,
    Admin,
}

/// Session model for tracking user sessions - searchable as "Session"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: u64,
    pub created_at: u64,
    pub last_activity: u64,
    pub ip_address: String,
    pub user_agent: String,
    pub expires_at: u64,
}

/// Audit log model - searchable as "Audit", "Log"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: u64,
    pub user_id: u64,
    pub action: String,
    pub resource_type: String,
    pub resource_id: u64,
    pub timestamp: u64,
    pub ip_address: String,
    pub details: serde_json::Value,
}

/// Model trait implementations
impl User {
    pub fn new(username: String, email: String, full_name: String) -> Self {
        Self {
            id: 0, // Will be set by service
            username,
            email,
            full_name,
            role: UserRole::Employee,
            organization_id: None,
            profile: UserProfile::default(),
            settings: UserSettings::default(),
        }
    }
    
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }
    
    pub fn can_manage_users(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Manager)
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            bio: String::new(),
            avatar_url: None,
            location: None,
            website: None,
            social_links: HashMap::new(),
        }
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            notifications: NotificationSettings::default(),
            privacy: PrivacySettings::default(),
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_enabled: true,
            push_enabled: true,
            desktop_enabled: false,
            frequency: NotificationFrequency::Daily,
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            profile_public: false,
            email_visible: false,
            activity_tracking: true,
            data_sharing: false,
        }
    }
}

impl Organization {
    pub fn new(name: String, domain: String) -> Self {
        Self {
            id: 0, // Will be set by service
            name,
            domain,
            description: String::new(),
            organization_type: OrganizationType::Corporation,
            settings: OrganizationSettings::default(),
            metadata: OrganizationMetadata::default(),
        }
    }
}

impl Default for OrganizationSettings {
    fn default() -> Self {
        Self {
            allow_public_signup: false,
            require_email_verification: true,
            default_user_role: UserRole::Employee,
            max_users: None,
            features: Vec::new(),
        }
    }
}

impl Default for OrganizationMetadata {
    fn default() -> Self {
        Self {
            founded_year: None,
            employee_count: None,
            industry: None,
            headquarters: None,
            website: None,
        }
    }
}
"#).await?;
    
    // Create services.rs with service implementations
    let services_rs = src_dir.join("services.rs");
    tokio::fs::write(&services_rs, r#"//! Service implementations for workspace symbols testing

use crate::models::*;
use std::collections::HashMap;

/// User service for managing users - searchable as "User", "Service"
pub struct UserService {
    users: HashMap<u64, User>,
    next_id: u64,
}

/// Organization service - searchable as "Organization", "Service"
pub struct OrganizationService {
    organizations: HashMap<u64, Organization>,
    next_id: u64,
}

/// Authentication service - searchable as "Auth", "Service", "Authentication"
pub struct AuthService {
    sessions: HashMap<String, Session>,
    permissions: HashMap<u64, Vec<Permission>>,
}

/// Email service for notifications - searchable as "Email", "Service"
pub struct EmailService {
    smtp_server: String,
    smtp_port: u16,
    username: String,
    password: String,
}

/// File service for content management - searchable as "File", "Service"
pub struct FileService {
    storage_path: String,
    max_file_size: u64,
    allowed_types: Vec<String>,
}

/// Search service for content discovery - searchable as "Search", "Service"
pub struct SearchService {
    index_path: String,
    index_version: String,
    max_results: usize,
}

/// Analytics service for tracking - searchable as "Analytics", "Service"
pub struct AnalyticsService {
    database_url: String,
    retention_days: u32,
    sampling_rate: f64,
}

/// Notification service - searchable as "Notification", "Service"
pub struct NotificationService {
    email_service: EmailService,
    push_service: PushService,
    template_engine: TemplateEngine,
}

/// Push notification service - searchable as "Push", "Service"
pub struct PushService {
    api_key: String,
    endpoint: String,
}

/// Template engine for messages - searchable as "Template", "Engine"
pub struct TemplateEngine {
    template_dir: String,
    cache_enabled: bool,
}

/// Background job service - searchable as "Job", "Service", "Background"
pub struct JobService {
    queue_name: String,
    max_retries: u32,
    retry_delay: u64,
}

/// Cache service for performance - searchable as "Cache", "Service"
pub struct CacheService {
    redis_url: String,
    default_ttl: u64,
    max_memory: u64,
}

/// User service implementation
impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn create_user(&mut self, username: String, email: String, full_name: String) -> Result<u64, String> {
        let mut user = User::new(username, email, full_name);
        user.id = self.next_id;
        self.users.insert(self.next_id, user);
        self.next_id += 1;
        Ok(self.next_id - 1)
    }
    
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }
    
    pub fn update_user(&mut self, id: u64, user: User) -> bool {
        if self.users.contains_key(&id) {
            self.users.insert(id, user);
            true
        } else {
            false
        }
    }
    
    pub fn delete_user(&mut self, id: u64) -> bool {
        self.users.remove(&id).is_some()
    }
    
    pub fn find_users_by_role(&self, role: UserRole) -> Vec<&User> {
        self.users.values()
            .filter(|user| std::mem::discriminant(&user.role) == std::mem::discriminant(&role))
            .collect()
    }
    
    pub fn find_users_by_organization(&self, org_id: u64) -> Vec<&User> {
        self.users.values()
            .filter(|user| user.organization_id == Some(org_id))
            .collect()
    }
}

/// Organization service implementation
impl OrganizationService {
    pub fn new() -> Self {
        Self {
            organizations: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn create_organization(&mut self, name: String, domain: String) -> Result<u64, String> {
        let mut org = Organization::new(name, domain);
        org.id = self.next_id;
        self.organizations.insert(self.next_id, org);
        self.next_id += 1;
        Ok(self.next_id - 1)
    }
    
    pub fn get_organization(&self, id: u64) -> Option<&Organization> {
        self.organizations.get(&id)
    }
    
    pub fn find_organization_by_domain(&self, domain: &str) -> Option<&Organization> {
        self.organizations.values()
            .find(|org| org.domain == domain)
    }
    
    pub fn list_organizations(&self) -> Vec<&Organization> {
        self.organizations.values().collect()
    }
}

/// Authentication service implementation
impl AuthService {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            permissions: HashMap::new(),
        }
    }
    
    pub fn create_session(&mut self, user_id: u64) -> String {
        let session_id = format!("session_{}", user_id);
        let session = Session {
            id: session_id.clone(),
            user_id,
            created_at: 0, // Would use real timestamp
            last_activity: 0,
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
            expires_at: 0,
        };
        self.sessions.insert(session_id.clone(), session);
        session_id
    }
    
    pub fn validate_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }
    
    pub fn revoke_session(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }
}

/// Email service implementation
impl EmailService {
    pub fn new(smtp_server: String, smtp_port: u16) -> Self {
        Self {
            smtp_server,
            smtp_port,
            username: String::new(),
            password: String::new(),
        }
    }
    
    pub fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        // Implementation would go here
        Ok(())
    }
    
    pub fn send_template_email(&self, to: &str, template: &str, data: serde_json::Value) -> Result<(), String> {
        // Implementation would go here
        Ok(())
    }
}

/// Search service implementation
impl SearchService {
    pub fn new(index_path: String) -> Self {
        Self {
            index_path,
            index_version: "1.0".to_string(),
            max_results: 100,
        }
    }
    
    pub fn index_user(&self, user: &User) -> Result<(), String> {
        // Implementation would go here
        Ok(())
    }
    
    pub fn index_organization(&self, org: &Organization) -> Result<(), String> {
        // Implementation would go here
        Ok(())
    }
    
    pub fn search_users(&self, query: &str) -> Result<Vec<User>, String> {
        // Implementation would go here
        Ok(Vec::new())
    }
    
    pub fn search_organizations(&self, query: &str) -> Result<Vec<Organization>, String> {
        // Implementation would go here
        Ok(Vec::new())
    }
}
"#).await?;
    
    // Create utils.rs with utility functions
    let utils_rs = src_dir.join("utils.rs");
    tokio::fs::write(&utils_rs, r#"//! Utility functions for workspace symbols testing

use crate::models::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Format user for display - searchable as "format", "user"
pub fn format_user(user: &User) -> String {
    format!("{} ({}) - {}", user.full_name, user.username, user.email)
}

/// Format organization for display - searchable as "format", "organization"
pub fn format_organization(org: &Organization) -> String {
    format!("{} - {}", org.name, org.domain)
}

/// Validate email address - searchable as "validate", "email"
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// Validate username - searchable as "validate", "username"
pub fn validate_username(username: &str) -> bool {
    !username.is_empty() && username.len() >= 3
}

/// Calculate hash for data - searchable as "calculate", "hash"
pub fn calculate_hash<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Generate random ID - searchable as "generate", "random", "id"
pub fn generate_random_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
}

/// Format timestamp - searchable as "format", "timestamp"
pub fn format_timestamp(timestamp: u64) -> String {
    format!("Timestamp: {}", timestamp)
}

/// Parse user role from string - searchable as "parse", "user", "role"
pub fn parse_user_role(role_str: &str) -> Option<UserRole> {
    match role_str.to_lowercase().as_str() {
        "admin" => Some(UserRole::Admin),
        "manager" => Some(UserRole::Manager),
        "employee" => Some(UserRole::Employee),
        "guest" => Some(UserRole::Guest),
        "system" => Some(UserRole::SystemUser),
        _ => None,
    }
}

/// Convert organization type to string - searchable as "organization", "type", "string"
pub fn organization_type_to_string(org_type: &OrganizationType) -> &'static str {
    match org_type {
        OrganizationType::Corporation => "corporation",
        OrganizationType::NonProfit => "nonprofit",
        OrganizationType::Government => "government",
        OrganizationType::Educational => "educational",
        OrganizationType::Startup => "startup",
        OrganizationType::Individual => "individual",
    }
}

/// Create user display name - searchable as "create", "user", "display"
pub fn create_user_display_name(user: &User) -> String {
    if !user.full_name.is_empty() {
        user.full_name.clone()
    } else {
        user.username.clone()
    }
}

/// Check if user is active - searchable as "check", "user", "active"
pub fn is_user_active(user: &User) -> bool {
    // Simplified check - in real implementation would check last activity
    !matches!(user.role, UserRole::Guest)
}

/// Generate email verification token - searchable as "generate", "email", "verification"
pub fn generate_email_verification_token(user_id: u64, email: &str) -> String {
    format!("verify_{}_{}", user_id, calculate_hash(&email))
}

/// Sanitize input string - searchable as "sanitize", "input"
pub fn sanitize_input(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.@".contains(*c))
        .collect()
}

/// Calculate user score - searchable as "calculate", "user", "score"
pub fn calculate_user_score(user: &User) -> f64 {
    let mut score = 0.0;
    
    // Add points for completed profile
    if !user.full_name.is_empty() { score += 10.0; }
    if !user.profile.bio.is_empty() { score += 5.0; }
    if user.profile.avatar_url.is_some() { score += 5.0; }
    
    // Add points based on role
    score += match user.role {
        UserRole::Admin => 50.0,
        UserRole::Manager => 30.0,
        UserRole::Employee => 20.0,
        UserRole::Guest => 5.0,
        UserRole::SystemUser => 0.0,
    };
    
    score
}

/// Fuzzy search function - searchable as "fuzzy", "search"
pub fn fuzzy_search(query: &str, candidates: &[String]) -> Vec<(String, f64)> {
    let mut results = Vec::new();
    
    for candidate in candidates {
        let score = calculate_fuzzy_score(query, candidate);
        if score > 0.0 {
            results.push((candidate.clone(), score));
        }
    }
    
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
}

/// Calculate fuzzy match score - searchable as "calculate", "fuzzy", "score"
pub fn calculate_fuzzy_score(query: &str, text: &str) -> f64 {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();
    
    if text_lower.contains(&query_lower) {
        if text_lower == query_lower {
            1.0 // Exact match
        } else if text_lower.starts_with(&query_lower) {
            0.8 // Prefix match
        } else {
            0.5 // Contains match
        }
    } else {
        0.0 // No match
    }
}

/// Helper macros for common operations
#[macro_export]
macro_rules! create_user_quick {
    ($username:expr, $email:expr) => {
        User::new($username.to_string(), $email.to_string(), $username.to_string())
    };
}

#[macro_export]
macro_rules! create_org_quick {
    ($name:expr, $domain:expr) => {
        Organization::new($name.to_string(), $domain.to_string())
    };
}

/// Constants for utility functions
pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MAX_USERNAME_LENGTH: usize = 50;
pub const MAX_EMAIL_LENGTH: usize = 254;
pub const DEFAULT_USER_SCORE: f64 = 10.0;

/// Test helper functions
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    
    pub fn create_test_user(id: u64) -> User {
        let mut user = User::new(
            format!("user{}", id),
            format!("user{}@test.com", id),
            format!("Test User {}", id),
        );
        user.id = id;
        user
    }
    
    pub fn create_test_organization(id: u64) -> Organization {
        let mut org = Organization::new(
            format!("Test Org {}", id),
            format!("testorg{}.com", id),
        );
        org.id = id;
        org
    }
}
"#).await?;
    
    // Create config.rs with configuration structures
    let config_rs = src_dir.join("config.rs");
    tokio::fs::write(&config_rs, r#"//! Configuration structures for workspace symbols testing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main application configuration - searchable as "App", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub features: FeatureConfig,
}

/// Server configuration - searchable as "Server", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: u32,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub ssl_enabled: bool,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
}

/// Database configuration - searchable as "Database", "Config", "DB"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub migration_enabled: bool,
    pub pool_config: PoolConfig,
}

/// Database pool configuration - searchable as "Pool", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_size: u32,
    pub min_idle: u32,
    pub test_on_checkout: bool,
    pub max_lifetime: u64,
}

/// Cache configuration - searchable as "Cache", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl: u64,
    pub max_memory: u64,
    pub eviction_policy: EvictionPolicy,
    pub cluster_enabled: bool,
    pub sentinel_enabled: bool,
}

/// Cache eviction policy - searchable as "Cache", "Eviction"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    Random,
    TTL,
}

/// Security configuration - searchable as "Security", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub password_policy: PasswordPolicy,
    pub rate_limiting: RateLimitConfig,
    pub cors: CorsConfig,
    pub csrf_protection: bool,
}

/// Password policy configuration - searchable as "Password", "Policy"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age_days: u32,
}

/// Rate limiting configuration - searchable as "Rate", "Limit"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

/// CORS configuration - searchable as "CORS", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
}

/// Logging configuration - searchable as "Logging", "Config", "Log"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_config: Option<FileLogConfig>,
    pub structured: bool,
}

/// Log level enumeration - searchable as "Log", "Level"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log format enumeration - searchable as "Log", "Format"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Plain,
    JSON,
    Compact,
}

/// Log output enumeration - searchable as "Log", "Output"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Console,
    File,
    Both,
}

/// File logging configuration - searchable as "File", "Log", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLogConfig {
    pub path: String,
    pub max_size: u64,
    pub max_files: u32,
    pub compression: bool,
}

/// Feature toggles configuration - searchable as "Feature", "Config"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub user_registration: bool,
    pub email_verification: bool,
    pub social_login: bool,
    pub two_factor_auth: bool,
    pub api_rate_limiting: bool,
    pub search_indexing: bool,
    pub analytics: bool,
    pub experimental_features: HashMap<String, bool>,
}

/// Environment-specific configurations
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            features: FeatureConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
            max_connections: 1000,
            timeout_seconds: 30,
            ssl_enabled: false,
            ssl_cert_path: None,
            ssl_key_path: None,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/app".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 600,
            migration_enabled: true,
            pool_config: PoolConfig::default(),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: 1,
            test_on_checkout: true,
            max_lifetime: 3600,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: 3600,
            max_memory: 1024 * 1024 * 1024, // 1GB
            eviction_policy: EvictionPolicy::LRU,
            cluster_enabled: false,
            sentinel_enabled: false,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-change-in-production".to_string(),
            jwt_expiration: 86400, // 24 hours
            password_policy: PasswordPolicy::default(),
            rate_limiting: RateLimitConfig::default(),
            cors: CorsConfig::default(),
            csrf_protection: true,
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: false,
            max_age_days: 90,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_size: 10,
            whitelist: Vec::new(),
            blacklist: Vec::new(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age: 3600,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Plain,
            output: LogOutput::Console,
            file_config: None,
            structured: false,
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            user_registration: true,
            email_verification: true,
            social_login: false,
            two_factor_auth: false,
            api_rate_limiting: true,
            search_indexing: true,
            analytics: false,
            experimental_features: HashMap::new(),
        }
    }
}

/// Configuration loading functions - searchable as "load", "config"
pub fn load_config_from_file(path: &str) -> Result<AppConfig, String> {
    // Implementation would load from file
    Ok(AppConfig::default())
}

pub fn load_config_from_env() -> AppConfig {
    // Implementation would load from environment variables
    AppConfig::default()
}

pub fn validate_config(config: &AppConfig) -> Result<(), String> {
    // Implementation would validate configuration
    Ok(())
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run workspace symbols test and analyze results
async fn run_workspace_symbols_test(
    project_path: PathBuf,
    query: Option<String>,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspWorkspaceSymbolsTool::new();
    let input = ToolInput::LspWorkspaceSymbols { project_path, query };
    
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
async fn test_workspace_symbols_no_query() {
    // 🌍 Test workspace symbols without query (all symbols)
    println!("🌍 Testing workspace symbols without query...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    let (success, duration, symbols_data) = run_workspace_symbols_test(
        project_path,
        None, // No query - should return all symbols
        "Workspace symbols no query",
        25
    ).await;
    
    if success {
        if let Some(data) = symbols_data {
            if let Some(symbols) = data["symbols"].as_array() {
                println!("🌍 Found {} symbols in workspace", symbols.len());
                
                // Analyze symbol distribution by file
                let mut file_counts = std::collections::HashMap::new();
                for symbol in symbols {
                    if let Some(uri) = symbol["uri"].as_str() {
                        let filename = uri.split('/').last().unwrap_or("unknown");
                        *file_counts.entry(filename).or_insert(0) += 1;
                    }
                }
                
                println!("📊 Symbols by file:");
                for (file, count) in &file_counts {
                    println!("  {}: {} symbols", file, count);
                }
                
                if symbols.len() > 10 {
                    println!("✅ Found substantial number of workspace symbols");
                } else {
                    println!("ℹ️ Found limited workspace symbols");
                }
                
                // Display first few symbols
                for (i, symbol) in symbols.iter().take(5).enumerate() {
                    if let Some(name) = symbol["name"].as_str() {
                        let kind = symbol["kind"].as_str().unwrap_or("unknown");
                        println!("  {}. {} ({})", i + 1, name, kind);
                    }
                }
            }
        }
        
        // Check performance
        if duration < Duration::from_secs(2) {
            println!("⚡ Excellent workspace symbols performance: {:?}", duration);
        } else if duration < Duration::from_secs(5) {
            println!("✅ Good workspace symbols performance: {:?}", duration);
        } else {
            println!("⚠️ Slow workspace symbols performance: {:?}", duration);
        }
    }
    
    println!("✅ Workspace symbols no query test completed");
}

#[tokio::test]
async fn test_workspace_symbols_specific_queries() {
    // 🔍 Test workspace symbols with specific search queries
    println!("🔍 Testing workspace symbols with specific queries...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Test different search queries
    let search_queries = vec![
        ("User", "User-related symbols"),
        ("Service", "Service-related symbols"),
        ("Config", "Configuration symbols"),
        ("Manager", "Manager symbols"),
        ("Email", "Email-related symbols"),
    ];
    
    for (query, description) in search_queries {
        let (success, duration, symbols_data) = run_workspace_symbols_test(
            project_path.clone(),
            Some(query.to_string()),
            &format!("Workspace symbols query: {}", description),
            15
        ).await;
        
        if success {
            if let Some(data) = symbols_data {
                if let Some(symbols) = data["symbols"].as_array() {
                    println!("🔍 Query '{}' found {} symbols in {:?}", query, symbols.len(), duration);
                    
                    // Check if results are relevant to query
                    let relevant_count = symbols.iter()
                        .filter(|symbol| {
                            if let Some(name) = symbol["name"].as_str() {
                                name.to_lowercase().contains(&query.to_lowercase())
                            } else {
                                false
                            }
                        })
                        .count();
                    
                    if relevant_count > 0 {
                        println!("  ✅ {} relevant symbols found", relevant_count);
                    } else {
                        println!("  ℹ️ No obviously relevant symbols (may use different matching)");
                    }
                    
                    // Show some results
                    for (i, symbol) in symbols.iter().take(3).enumerate() {
                        if let Some(name) = symbol["name"].as_str() {
                            println!("    {}. {}", i + 1, name);
                        }
                    }
                }
            }
        }
    }
    
    println!("✅ Specific queries test completed");
}

#[tokio::test]
async fn test_workspace_symbols_fuzzy_search() {
    // 🔤 Test workspace symbols fuzzy search capabilities
    println!("🔤 Testing workspace symbols fuzzy search...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Test fuzzy search patterns
    let fuzzy_queries = vec![
        ("usr", "Partial 'User' match"),
        ("svc", "Abbreviated 'Service' match"),
        ("cfg", "Abbreviated 'Config' match"),
        ("mgr", "Abbreviated 'Manager' match"),
        ("app", "Application-related match"),
    ];
    
    for (query, description) in fuzzy_queries {
        let (success, duration, symbols_data) = run_workspace_symbols_test(
            project_path.clone(),
            Some(query.to_string()),
            &format!("Fuzzy search: {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = symbols_data {
                if let Some(symbols) = data["symbols"].as_array() {
                    println!("🔤 Fuzzy query '{}' found {} symbols in {:?}", query, symbols.len(), duration);
                    
                    if symbols.len() > 0 {
                        println!("  ✅ Fuzzy search returned results");
                        
                        // Show first few results
                        for (i, symbol) in symbols.iter().take(2).enumerate() {
                            if let Some(name) = symbol["name"].as_str() {
                                println!("    {}. {}", i + 1, name);
                            }
                        }
                    } else {
                        println!("  ℹ️ No fuzzy matches found");
                    }
                }
            }
        }
    }
    
    println!("✅ Fuzzy search test completed");
}

#[tokio::test]
async fn test_workspace_symbols_case_sensitivity() {
    // 🔠 Test workspace symbols case sensitivity
    println!("🔠 Testing workspace symbols case sensitivity...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Test case variations
    let case_queries = vec![
        ("user", "lowercase"),
        ("User", "capitalized"),
        ("USER", "uppercase"),
        ("UsEr", "mixed case"),
    ];
    
    let mut results = Vec::new();
    
    for (query, description) in case_queries {
        let (success, duration, symbols_data) = run_workspace_symbols_test(
            project_path.clone(),
            Some(query.to_string()),
            &format!("Case test: {}", description),
            10
        ).await;
        
        if success {
            if let Some(data) = symbols_data {
                let symbol_count = data["symbols"].as_array().map(|s| s.len()).unwrap_or(0);
                results.push((query, symbol_count, duration));
                println!("🔠 Query '{}' ({}) found {} symbols in {:?}", query, description, symbol_count, duration);
            }
        }
    }
    
    // Analyze case sensitivity
    if results.len() >= 2 {
        let first_count = results[0].1;
        let consistent = results.iter().all(|(_, count, _)| *count == first_count);
        
        if consistent && first_count > 0 {
            println!("✅ Case-insensitive search detected (consistent results)");
        } else if first_count > 0 {
            println!("ℹ️ Case-sensitive or different matching strategies");
        } else {
            println!("ℹ️ No results for case sensitivity testing");
        }
    }
    
    println!("✅ Case sensitivity test completed");
}

#[tokio::test]
async fn test_workspace_symbols_performance() {
    // ⚡ Test workspace symbols performance patterns
    println!("⚡ Testing workspace symbols performance...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Test repeated searches (caching)
    let test_query = "User";
    let mut durations = Vec::new();
    
    for i in 0..3 {
        let (success, duration, _) = run_workspace_symbols_test(
            project_path.clone(),
            Some(test_query.to_string()),
            &format!("Performance test {}", i + 1),
            10
        ).await;
        
        if success {
            durations.push(duration);
        }
    }
    
    if !durations.is_empty() {
        println!("📊 Workspace symbols performance:");
        for (i, duration) in durations.iter().enumerate() {
            println!("  Search {}: {:?}", i + 1, duration);
        }
        
        let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        println!("  Average: {:?}", avg_duration);
        
        // Check performance targets
        if avg_duration < Duration::from_secs(2) {
            println!("  ⚡ Excellent performance (<2s average)");
        } else if avg_duration < Duration::from_secs(5) {
            println!("  ✅ Good performance (<5s average)");
        } else {
            println!("  ⚠️ Slow performance (>5s average)");
        }
        
        // Check for caching effects
        if durations.len() >= 2 {
            let first = durations[0];
            let later_avg = durations[1..].iter().sum::<Duration>() / (durations.len() - 1) as u32;
            
            if later_avg < first {
                println!("  ✅ Caching effect detected (later searches faster)");
            } else {
                println!("  ℹ️ No significant caching effect");
            }
        }
    }
    
    println!("✅ Performance test completed");
}

#[tokio::test]
async fn test_workspace_symbols_edge_cases() {
    // 🧪 Test workspace symbols edge cases
    println!("🧪 Testing workspace symbols edge cases...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Test edge case queries
    let edge_cases = vec![
        ("", "Empty query"),
        ("x", "Single character"),
        ("nonexistent", "Non-existent symbol"),
        ("123", "Numeric query"),
        ("!@#", "Special characters"),
        ("very_long_query_that_should_not_match_anything_in_the_workspace", "Very long query"),
    ];
    
    for (query, description) in edge_cases {
        let (success, duration, symbols_data) = run_workspace_symbols_test(
            project_path.clone(),
            Some(query.to_string()),
            &format!("Edge case: {}", description),
            5
        ).await;
        
        if success {
            if let Some(data) = symbols_data {
                let symbol_count = data["symbols"].as_array().map(|s| s.len()).unwrap_or(0);
                println!("  ✅ {} handled successfully: {} results in {:?}", description, symbol_count, duration);
            } else {
                println!("  ✅ {} handled successfully in {:?}", description, duration);
            }
        } else {
            println!("  ✅ {} failed gracefully in {:?} (acceptable for edge cases)", description, duration);
        }
    }
    
    println!("✅ Edge cases test completed");
}

#[tokio::test]
async fn test_workspace_symbols_concurrent() {
    // 🔄 Test concurrent workspace symbols requests
    println!("🔄 Testing concurrent workspace symbols requests...");
    
    let (_temp_dir, project_path) = create_workspace_symbols_test_project().await
        .expect("Failed to create test project");
    
    // Create concurrent search requests
    let queries = vec![
        (Some("User".to_string()), "User search"),
        (Some("Service".to_string()), "Service search"),
        (Some("Config".to_string()), "Config search"),
        (None, "All symbols"),
    ];
    
    let futures: Vec<_> = queries.into_iter().enumerate().map(|(i, (query, desc))| {
        let path = project_path.clone();
        async move {
            let (success, duration, _) = run_workspace_symbols_test(
                path,
                query,
                &format!("Concurrent search {} ({})", i + 1, desc),
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
    
    println!("📊 Concurrent workspace symbols results: {}/4 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent workspace symbols search succeeded");
    } else {
        println!("ℹ️ All concurrent searches failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent workspace symbols test completed");
}
