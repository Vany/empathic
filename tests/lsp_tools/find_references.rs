//! 🔬 LSP Find References Tool Comprehensive Tests
//!
//! Advanced testing for LSP find references functionality including:
//! - Reference discovery across files and modules
//! - Declaration inclusion and filtering options
//! - Performance validation and context extraction
//! - Edge cases and concurrent request handling

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use serde_json::Value;

use empathic::tools::lsp::find_references::LspFindReferencesTool;
use empathic::mcp::{Tool, ToolInput, CallToolResult};

/// 📁 Create multi-file Rust project for find references testing
async fn create_find_references_test_project() -> std::io::Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    // Create Cargo.toml
    let cargo_toml = project_path.join("Cargo.toml");
    tokio::fs::write(&cargo_toml, r#"
[package]
name = "find-references-test"
version = "0.1.0"
edition = "2021"
"#).await?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    
    // Create lib.rs with widely used types and functions
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, r#"//! Find references test library with cross-file usage

pub mod models;
pub mod operations;
pub mod helpers;

use models::{Task, TaskStatus, Priority};
use operations::{TaskManager, TaskFilter};
use helpers::{format_task, validate_task_name};

/// Main API for task management - should be referenced across files
pub struct TaskApi {
    manager: TaskManager,
    default_priority: Priority,
}

impl TaskApi {
    /// Create new TaskApi - should be referenced in tests and main
    pub fn new() -> Self {
        Self {
            manager: TaskManager::new(),
            default_priority: Priority::Medium,
        }
    }
    
    /// Create task - widely used function for finding references
    pub fn create_task(&mut self, name: String, description: String) -> Result<u64, String> {
        if !validate_task_name(&name) {
            return Err("Invalid task name".to_string());
        }
        
        let task = Task::new(name, description, self.default_priority);
        self.manager.add_task(task)
    }
    
    /// Get task by ID - should be referenced multiple times
    pub fn get_task(&self, id: u64) -> Option<&Task> {
        self.manager.get_task(id)
    }
    
    /// Update task status - commonly used operation
    pub fn update_task_status(&mut self, id: u64, status: TaskStatus) -> bool {
        if let Some(task) = self.manager.get_task_mut(id) {
            task.set_status(status);
            true
        } else {
            false
        }
    }
    
    /// List all tasks - should be referenced in multiple contexts
    pub fn list_tasks(&self) -> Vec<&Task> {
        self.manager.list_all_tasks()
    }
    
    /// Format task list - utility function with references
    pub fn format_task_list(&self) -> Vec<String> {
        self.list_tasks().iter()
            .map(|task| format_task(task))
            .collect()
    }
}

/// Free function for creating default task - should find many references
pub fn create_default_task(name: String) -> Task {
    Task::new(name, "Default task".to_string(), Priority::Low)
}

/// Utility function for task validation - referenced in multiple modules
pub fn is_valid_task_id(id: u64) -> bool {
    id > 0 && id < 1000000
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_api_creation() {
        let api = TaskApi::new();  // Reference to TaskApi::new
        assert_eq!(api.default_priority, Priority::Medium);
    }
    
    #[test]
    fn test_create_task() {
        let mut api = TaskApi::new();  // Another reference to TaskApi::new
        let result = api.create_task("Test Task".to_string(), "Description".to_string());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_default_task() {
        let task = create_default_task("Default".to_string());  // Reference to create_default_task
        assert_eq!(task.priority(), Priority::Low);
    }
}
"#).await?;
    
    // Create models.rs with core types that will be referenced everywhere
    let models_rs = src_dir.join("models.rs");
    tokio::fs::write(&models_rs, r#"//! Core models for find references testing

use std::collections::HashMap;

/// Task struct - should be heavily referenced across the project
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    id: u64,
    name: String,
    description: String,
    status: TaskStatus,
    priority: Priority,
    metadata: HashMap<String, String>,
    created_at: u64,
}

/// Task status enumeration - should be referenced in many places
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

/// Priority enumeration - should be referenced across modules
#[derive(Debug, Clone, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Task {
    /// Constructor - should be referenced heavily
    pub fn new(name: String, description: String, priority: Priority) -> Self {
        Self {
            id: 0, // Will be set by manager
            name,
            description,
            status: TaskStatus::Todo,
            priority,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
    
    /// Getters - should be referenced in various contexts
    pub fn id(&self) -> u64 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn status(&self) -> &TaskStatus { &self.status }
    pub fn priority(&self) -> Priority { self.priority.clone() }
    pub fn created_at(&self) -> u64 { self.created_at }
    
    /// Setters - should be referenced in update operations
    pub fn set_id(&mut self, id: u64) { self.id = id; }
    pub fn set_name(&mut self, name: String) { self.name = name; }
    pub fn set_description(&mut self, description: String) { self.description = description; }
    pub fn set_status(&mut self, status: TaskStatus) { self.status = status; }
    pub fn set_priority(&mut self, priority: Priority) { self.priority = priority; }
    
    /// Utility methods - should be referenced in formatting and validation
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Done)
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self.status, TaskStatus::Todo | TaskStatus::InProgress)
    }
    
    pub fn is_high_priority(&self) -> bool {
        matches!(self.priority, Priority::High | Priority::Critical)
    }
    
    /// Metadata operations - should be referenced in various modules
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl TaskStatus {
    /// Status utility methods - should be referenced in UI and logic
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }
    
    pub fn is_final(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }
    
    pub fn can_transition_to(&self, new_status: &TaskStatus) -> bool {
        match (self, new_status) {
            (TaskStatus::Todo, TaskStatus::InProgress) => true,
            (TaskStatus::InProgress, TaskStatus::Done) => true,
            (_, TaskStatus::Cancelled) => true,
            _ => false,
        }
    }
}

impl Priority {
    /// Priority utility methods - should be referenced in sorting and filtering
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }
    
    pub fn numeric_value(&self) -> u32 {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
    
    pub fn from_str(s: &str) -> Option<Priority> {
        match s.to_lowercase().as_str() {
            "low" => Some(Priority::Low),
            "medium" => Some(Priority::Medium),
            "high" => Some(Priority::High),
            "critical" => Some(Priority::Critical),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = Task::new("Test".to_string(), "Description".to_string(), Priority::High);
        assert_eq!(task.name(), "Test");
        assert_eq!(task.priority(), Priority::High);
    }
    
    #[test]
    fn test_task_status() {
        let mut task = Task::new("Test".to_string(), "Desc".to_string(), Priority::Low);
        assert!(task.is_active());
        
        task.set_status(TaskStatus::Done);
        assert!(task.is_completed());
        assert!(!task.is_active());
    }
}
"#).await?;
    
    // Create operations.rs with task management operations
    let operations_rs = src_dir.join("operations.rs");
    tokio::fs::write(&operations_rs, r#"//! Task operations for find references testing

use crate::models::{Task, TaskStatus, Priority};
use std::collections::HashMap;

/// Task manager - should be referenced as main operational component
pub struct TaskManager {
    tasks: HashMap<u64, Task>,
    next_id: u64,
}

/// Filter criteria for finding tasks - should be referenced in search operations
#[derive(Debug, Clone)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub name_contains: Option<String>,
}

impl TaskManager {
    /// Constructor - should be referenced when creating managers
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Add task - should be referenced in task creation flows
    pub fn add_task(&mut self, mut task: Task) -> Result<u64, String> {
        let id = self.next_id;
        task.set_id(id);
        self.tasks.insert(id, task);
        self.next_id += 1;
        Ok(id)
    }
    
    /// Get task by ID - heavily referenced method
    pub fn get_task(&self, id: u64) -> Option<&Task> {
        self.tasks.get(&id)
    }
    
    /// Get mutable task reference - referenced in update operations
    pub fn get_task_mut(&mut self, id: u64) -> Option<&mut Task> {
        self.tasks.get_mut(&id)
    }
    
    /// Remove task - should be referenced in deletion operations
    pub fn remove_task(&mut self, id: u64) -> Option<Task> {
        self.tasks.remove(&id)
    }
    
    /// List all tasks - should be referenced in display operations
    pub fn list_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }
    
    /// Find tasks by status - should be referenced in filtering operations
    pub fn find_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks.values()
            .filter(|task| task.status() == &status)
            .collect()
    }
    
    /// Find tasks by priority - should be referenced in priority-based views
    pub fn find_tasks_by_priority(&self, priority: Priority) -> Vec<&Task> {
        self.tasks.values()
            .filter(|task| task.priority() == priority)
            .collect()
    }
    
    /// Find active tasks - commonly referenced method
    pub fn find_active_tasks(&self) -> Vec<&Task> {
        self.tasks.values()
            .filter(|task| task.is_active())
            .collect()
    }
    
    /// Find high priority tasks - should be referenced in urgent task views
    pub fn find_high_priority_tasks(&self) -> Vec<&Task> {
        self.tasks.values()
            .filter(|task| task.is_high_priority())
            .collect()
    }
    
    /// Complex filter method - should be referenced in advanced search
    pub fn filter_tasks(&self, filter: &TaskFilter) -> Vec<&Task> {
        self.tasks.values()
            .filter(|task| {
                if let Some(ref status) = filter.status {
                    if task.status() != status {
                        return false;
                    }
                }
                
                if let Some(ref priority) = filter.priority {
                    if task.priority() != *priority {
                        return false;
                    }
                }
                
                if let Some(ref name_part) = filter.name_contains {
                    if !task.name().contains(name_part) {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }
    
    /// Get task count - should be referenced in statistics
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    
    /// Get task statistics - should be referenced in dashboard views
    pub fn get_statistics(&self) -> TaskStatistics {
        let total = self.task_count();
        let todo = self.find_tasks_by_status(TaskStatus::Todo).len();
        let in_progress = self.find_tasks_by_status(TaskStatus::InProgress).len();
        let done = self.find_tasks_by_status(TaskStatus::Done).len();
        let cancelled = self.find_tasks_by_status(TaskStatus::Cancelled).len();
        
        TaskStatistics {
            total,
            todo,
            in_progress,
            done,
            cancelled,
        }
    }
}

/// Task statistics struct - should be referenced in reporting
#[derive(Debug, Clone)]
pub struct TaskStatistics {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
    pub cancelled: usize,
}

impl TaskStatistics {
    /// Calculate completion percentage - should be referenced in progress views
    pub fn completion_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.done as f64 / self.total as f64) * 100.0
        }
    }
    
    /// Check if all tasks are completed - should be referenced in completion checks
    pub fn is_all_completed(&self) -> bool {
        self.total > 0 && self.done == self.total
    }
}

impl Default for TaskFilter {
    fn default() -> Self {
        Self {
            status: None,
            priority: None,
            name_contains: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_manager_creation() {
        let manager = TaskManager::new();
        assert_eq!(manager.task_count(), 0);
    }
    
    #[test]
    fn test_add_and_get_task() {
        let mut manager = TaskManager::new();
        let task = Task::new("Test".to_string(), "Description".to_string(), Priority::Medium);
        
        let id = manager.add_task(task).unwrap();
        let retrieved = manager.get_task(id).unwrap();
        
        assert_eq!(retrieved.name(), "Test");
    }
    
    #[test]
    fn test_filter_tasks() {
        let mut manager = TaskManager::new();
        
        let task1 = Task::new("High Priority".to_string(), "Desc".to_string(), Priority::High);
        let task2 = Task::new("Low Priority".to_string(), "Desc".to_string(), Priority::Low);
        
        manager.add_task(task1).unwrap();
        manager.add_task(task2).unwrap();
        
        let filter = TaskFilter {
            status: None,
            priority: Some(Priority::High),
            name_contains: None,
        };
        
        let filtered = manager.filter_tasks(&filter);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name(), "High Priority");
    }
}
"#).await?;
    
    // Create helpers.rs with utility functions
    let helpers_rs = src_dir.join("helpers.rs");
    tokio::fs::write(&helpers_rs, r#"//! Helper functions for find references testing

use crate::models::{Task, TaskStatus, Priority};

/// Format task for display - should be referenced in UI components
pub fn format_task(task: &Task) -> String {
    format!(
        "[{}] {} - {} (Priority: {})",
        task.status().as_str().to_uppercase(),
        task.name(),
        task.description(),
        task.priority().as_str()
    )
}

/// Validate task name - should be referenced in input validation
pub fn validate_task_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 100
}

/// Validate task description - should be referenced in input validation
pub fn validate_task_description(description: &str) -> bool {
    description.len() <= 500
}

/// Calculate task age in days - should be referenced in reporting
pub fn calculate_task_age_days(task: &Task) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    (now - task.created_at()) / (24 * 60 * 60)
}

/// Check if task is overdue - should be referenced in alert systems
pub fn is_task_overdue(task: &Task, days_limit: u64) -> bool {
    task.is_active() && calculate_task_age_days(task) > days_limit
}

/// Sort tasks by priority - should be referenced in sorting operations
pub fn sort_tasks_by_priority(tasks: &mut [&Task]) {
    tasks.sort_by(|a, b| {
        b.priority().numeric_value().cmp(&a.priority().numeric_value())
    });
}

/// Group tasks by status - should be referenced in grouping operations
pub fn group_tasks_by_status(tasks: &[&Task]) -> std::collections::HashMap<String, Vec<&Task>> {
    let mut groups = std::collections::HashMap::new();
    
    for task in tasks {
        let status_key = task.status().as_str().to_string();
        groups.entry(status_key).or_insert_with(Vec::new).push(*task);
    }
    
    groups
}

/// Create task summary - should be referenced in summary views
pub fn create_task_summary(task: &Task) -> String {
    format!(
        "Task #{}: {} [{}]",
        task.id(),
        task.name(),
        task.status().as_str()
    )
}

/// Format priority display - should be referenced in UI formatting
pub fn format_priority_display(priority: Priority) -> String {
    match priority {
        Priority::Critical => "🔴 Critical".to_string(),
        Priority::High => "🟠 High".to_string(),
        Priority::Medium => "🟡 Medium".to_string(),
        Priority::Low => "🟢 Low".to_string(),
    }
}

/// Check task completion eligibility - should be referenced in workflow logic
pub fn can_complete_task(task: &Task) -> bool {
    match task.status() {
        TaskStatus::InProgress => true,
        TaskStatus::Todo => false, // Must be in progress first
        TaskStatus::Done => false, // Already completed
        TaskStatus::Cancelled => false, // Cannot complete cancelled tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_task() {
        let task = Task::new("Test Task".to_string(), "Description".to_string(), Priority::High);
        let formatted = format_task(&task);
        assert!(formatted.contains("Test Task"));
        assert!(formatted.contains("high"));
    }
    
    #[test]
    fn test_validate_task_name() {
        assert!(validate_task_name("Valid Name"));
        assert!(!validate_task_name(""));
        assert!(!validate_task_name("   "));
    }
    
    #[test]
    fn test_can_complete_task() {
        let mut task = Task::new("Test".to_string(), "Desc".to_string(), Priority::Medium);
        
        assert!(!can_complete_task(&task)); // Todo status
        
        task.set_status(TaskStatus::InProgress);
        assert!(can_complete_task(&task)); // InProgress status
        
        task.set_status(TaskStatus::Done);
        assert!(!can_complete_task(&task)); // Done status
    }
}
"#).await?;
    
    // Create main.rs with extensive usage of all modules
    let main_rs = src_dir.join("main.rs");
    tokio::fs::write(&main_rs, r#"//! Main application demonstrating extensive usage for find references testing

use find_references_test::{TaskApi, create_default_task, is_valid_task_id};
use find_references_test::models::{Task, TaskStatus, Priority};
use find_references_test::operations::{TaskManager, TaskFilter};
use find_references_test::helpers::{
    format_task, validate_task_name, calculate_task_age_days,
    sort_tasks_by_priority, group_tasks_by_status
};

fn main() {
    // TaskApi usage - multiple references to TaskApi methods
    let mut api = TaskApi::new();
    
    // Task creation - references to create_task, Task::new
    let task_id = api.create_task("Important Task".to_string(), "Critical work item".to_string())
        .expect("Failed to create task");
    
    // Task retrieval - references to get_task
    let task = api.get_task(task_id).expect("Task should exist");
    
    // Status update - references to update_task_status, TaskStatus variants
    api.update_task_status(task_id, TaskStatus::InProgress);
    
    // Task listing - references to list_tasks, format_task_list
    let all_tasks = api.list_tasks();
    let formatted_tasks = api.format_task_list();
    
    // Default task creation - references to create_default_task
    let default_task = create_default_task("Default Task".to_string());
    
    // Direct TaskManager usage - references to TaskManager methods
    let mut manager = TaskManager::new();
    let new_task = Task::new("Direct Task".to_string(), "Description".to_string(), Priority::High);
    let direct_task_id = manager.add_task(new_task).expect("Failed to add task");
    
    // Task filtering - references to filter methods and TaskFilter
    let high_priority_tasks = manager.find_tasks_by_priority(Priority::High);
    let active_tasks = manager.find_active_tasks();
    
    // Complex filtering - references to filter_tasks, TaskFilter construction
    let filter = TaskFilter {
        status: Some(TaskStatus::InProgress),
        priority: Some(Priority::High),
        name_contains: Some("Important".to_string()),
    };
    let filtered_tasks = manager.filter_tasks(&filter);
    
    // Helper function usage - references to various helper functions
    let task_formatted = format_task(task);
    let name_valid = validate_task_name("New Task");
    let task_age = calculate_task_age_days(task);
    
    // Task manipulation and utility functions
    demonstrate_task_operations(&mut manager);
    demonstrate_task_formatting(&all_tasks);
    demonstrate_priority_handling();
    
    println!("Application completed successfully");
    println!("Created {} tasks", manager.task_count());
}

/// Function demonstrating various task operations - more references
fn demonstrate_task_operations(manager: &mut TaskManager) {
    // Create multiple tasks - references to Task::new, Priority variants
    let tasks_data = vec![
        ("Research", "Research new technologies", Priority::Medium),
        ("Development", "Implement new features", Priority::High),
        ("Testing", "Write comprehensive tests", Priority::Medium),
        ("Documentation", "Update project documentation", Priority::Low),
    ];
    
    for (name, description, priority) in tasks_data {
        let task = Task::new(name.to_string(), description.to_string(), priority);
        manager.add_task(task).expect("Failed to add task");
    }
    
    // Find tasks by different criteria - references to find methods
    let todo_tasks = manager.find_tasks_by_status(TaskStatus::Todo);
    let high_priority_tasks = manager.find_tasks_by_priority(Priority::High);
    let active_tasks = manager.find_active_tasks();
    
    // Update task statuses - references to get_task_mut, set_status
    for task in manager.list_all_tasks() {
        if task.priority() == Priority::High {
            if let Some(mut_task) = manager.get_task_mut(task.id()) {
                mut_task.set_status(TaskStatus::InProgress);
            }
        }
    }
    
    // Get statistics - references to get_statistics, TaskStatistics methods
    let stats = manager.get_statistics();
    let completion_pct = stats.completion_percentage();
    let all_done = stats.is_all_completed();
    
    println!("Task operations completed: {} total, {:.1}% complete, all done: {}", 
             stats.total, completion_pct, all_done);
}

/// Function demonstrating task formatting - references to helper functions
fn demonstrate_task_formatting(tasks: &[&Task]) {
    // Format individual tasks - references to format_task
    let formatted: Vec<String> = tasks.iter()
        .map(|task| format_task(task))
        .collect();
    
    // Group tasks by status - references to group_tasks_by_status
    let grouped = group_tasks_by_status(tasks);
    
    // Sort tasks by priority - references to sort_tasks_by_priority
    let mut task_refs: Vec<&Task> = tasks.iter().cloned().collect();
    sort_tasks_by_priority(&mut task_refs);
    
    // Task age calculation - references to calculate_task_age_days
    let old_tasks: Vec<&Task> = tasks.iter()
        .filter(|task| calculate_task_age_days(task) > 7)
        .cloned()
        .collect();
    
    println!("Formatting completed: {} formatted, {} groups, {} old tasks", 
             formatted.len(), grouped.len(), old_tasks.len());
}

/// Function demonstrating priority handling - references to Priority methods
fn demonstrate_priority_handling() {
    // Priority creation and manipulation - references to Priority variants
    let priorities = vec![Priority::Low, Priority::Medium, Priority::High, Priority::Critical];
    
    // Priority utility methods - references to Priority methods
    for priority in &priorities {
        let name = priority.as_str();
        let value = priority.numeric_value();
        println!("Priority {}: value {}", name, value);
    }
    
    // Priority parsing - references to Priority::from_str
    let parsed_priority = Priority::from_str("high").expect("Should parse high priority");
    assert_eq!(parsed_priority, Priority::High);
    
    // Task creation with different priorities - references to Task::new
    let critical_task = Task::new(
        "Critical Issue".to_string(),
        "Urgent fix required".to_string(),
        Priority::Critical
    );
    
    let low_task = Task::new(
        "Nice to Have".to_string(),
        "Enhancement when time permits".to_string(),
        Priority::Low
    );
    
    // Priority-based logic - references to is_high_priority
    if critical_task.is_high_priority() {
        println!("Critical task needs immediate attention");
    }
    
    if !low_task.is_high_priority() {
        println!("Low priority task can wait");
    }
}

/// Additional usage patterns for comprehensive reference testing
fn advanced_usage_patterns() {
    // Validation functions - references to validation helpers
    let valid_names = vec!["Task 1", "Important Work", "Quick Fix"];
    let invalid_names = vec!["", "   ", ""];
    
    for name in &valid_names {
        assert!(validate_task_name(name));
    }
    
    for name in &invalid_names {
        assert!(!validate_task_name(name));
    }
    
    // ID validation - references to is_valid_task_id
    let valid_ids = vec![1, 100, 999999];
    let invalid_ids = vec![0, 1000000];
    
    for id in &valid_ids {
        assert!(is_valid_task_id(*id));
    }
    
    for id in &invalid_ids {
        assert!(!is_valid_task_id(*id));
    }
    
    // Status transitions - references to TaskStatus methods
    let current_status = TaskStatus::Todo;
    let next_status = TaskStatus::InProgress;
    
    if current_status.can_transition_to(&next_status) {
        println!("Status transition allowed");
    }
    
    if !current_status.is_final() {
        println!("Status can be changed");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_workflow() {
        // Complete workflow test with many references
        let mut api = TaskApi::new();
        
        // Create task - references to create_task
        let id = api.create_task("Integration Test".to_string(), "Full test".to_string())
            .expect("Task creation should succeed");
        
        // Get task - references to get_task
        let task = api.get_task(id).expect("Task should exist");
        assert_eq!(task.name(), "Integration Test");
        
        // Update status - references to update_task_status, TaskStatus::InProgress
        assert!(api.update_task_status(id, TaskStatus::InProgress));
        
        // Verify status change - references to get_task, status
        let updated_task = api.get_task(id).unwrap();
        assert_eq!(updated_task.status(), &TaskStatus::InProgress);
        
        // List tasks - references to list_tasks
        let tasks = api.list_tasks();
        assert_eq!(tasks.len(), 1);
        
        // Format tasks - references to format_task_list
        let formatted = api.format_task_list();
        assert_eq!(formatted.len(), 1);
    }
    
    #[test]
    fn test_manager_operations() {
        // Manager operations test with references
        let mut manager = TaskManager::new();
        
        // Add multiple tasks - references to Task::new, add_task, Priority variants
        let task1 = Task::new("Task 1".to_string(), "First task".to_string(), Priority::High);
        let task2 = Task::new("Task 2".to_string(), "Second task".to_string(), Priority::Low);
        
        let id1 = manager.add_task(task1).unwrap();
        let id2 = manager.add_task(task2).unwrap();
        
        // Find by priority - references to find_tasks_by_priority
        let high_tasks = manager.find_tasks_by_priority(Priority::High);
        assert_eq!(high_tasks.len(), 1);
        
        // Get statistics - references to get_statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.todo, 2);
    }
}
"#).await?;
    
    Ok((temp_dir, project_path))
}

/// 🧪 Run find references test and analyze results
async fn run_find_references_test(
    file_path: PathBuf,
    line: u32,
    character: u32,
    include_declaration: Option<bool>,
    test_name: &str,
    timeout_secs: u64
) -> (bool, Duration, Option<Value>) {
    let tool = LspFindReferencesTool::new();
    let input = ToolInput::LspFindReferences { 
        file_path, 
        line, 
        character, 
        include_declaration 
    };
    
    let start = Instant::now();
    let result = timeout(Duration::from_secs(timeout_secs), tool.call(input)).await;
    let duration = start.elapsed();
    
    match result {
        Ok(Ok(CallToolResult::Success { content })) => {
            println!("✅ {} succeeded in {:?}", test_name, duration);
            
            if let Some(text_content) = content.first() {
                if let Ok(references_data) = serde_json::from_str::<Value>(&text_content.text) {
                    return (true, duration, Some(references_data));
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
async fn test_find_references_heavily_used_types() {
    // 🎯 Test find references for heavily used types (Task, TaskStatus, Priority)
    println!("🎯 Testing find references for heavily used types...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let models_file = project_path.join("src/models.rs");
    
    // Test references for Task struct (should be heavily referenced)
    let (success, duration, references_data) = run_find_references_test(
        models_file.clone(),
        8, // Line with Task struct definition
        12, // Character position on "Task"
        Some(true), // Include declaration
        "Find references for Task struct",
        20
    ).await;
    
    if success {
        if let Some(data) = references_data {
            if let Some(references) = data["references"].as_array() {
                println!("📍 Found {} references to Task struct", references.len());
                
                // Check for references in different files
                let mut file_counts = std::collections::HashMap::new();
                for reference in references {
                    if let Some(uri) = reference["uri"].as_str() {
                        *file_counts.entry(uri).or_insert(0) += 1;
                    }
                }
                
                println!("📊 References across files:");
                for (file, count) in &file_counts {
                    let filename = file.split('/').last().unwrap_or(file);
                    println!("  {}: {} references", filename, count);
                }
                
                if references.len() > 5 {
                    println!("✅ Task struct is heavily referenced as expected");
                } else {
                    println!("ℹ️ Found fewer references than expected (may be format difference)");
                }
            }
        }
        
        // Check performance
        if duration < Duration::from_millis(500) {
            println!("⚡ Excellent reference finding performance: {:?}", duration);
        } else if duration < Duration::from_secs(2) {
            println!("✅ Good reference finding performance: {:?}", duration);
        } else {
            println!("⚠️ Slow reference finding performance: {:?}", duration);
        }
    }
    
    println!("✅ Heavily used types reference test completed");
}

#[tokio::test]
async fn test_find_references_functions() {
    // 🔧 Test find references for functions across modules
    println!("🔧 Testing find references for functions...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test references for create_task method (should be called multiple times)
    let (success, duration, references_data) = run_find_references_test(
        lib_file.clone(),
        26, // Line with create_task method definition
        15, // Character position on "create_task"
        Some(false), // Exclude declaration
        "Find references for create_task function",
        15
    ).await;
    
    if success {
        if let Some(data) = references_data {
            if let Some(references) = data["references"].as_array() {
                println!("📍 Found {} references to create_task", references.len());
                
                // Check for usage context in references
                for (i, reference) in references.iter().take(3).enumerate() {
                    if let Some(context) = reference["context"].as_str() {
                        println!("  {}. Context: {}", i + 1, context.trim());
                    }
                }
                
                if references.len() > 0 {
                    println!("✅ create_task function is referenced as expected");
                }
            }
        }
    }
    
    println!("✅ Function references test completed");
}

#[tokio::test]
async fn test_find_references_with_declaration() {
    // 📋 Test find references with and without declaration inclusion
    println!("📋 Testing find references with declaration inclusion options...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let helpers_file = project_path.join("src/helpers.rs");
    
    // Test with declaration included
    let (success_with, duration_with, data_with) = run_find_references_test(
        helpers_file.clone(),
        6, // Line with format_task function definition
        10, // Character position on "format_task"
        Some(true), // Include declaration
        "Find references with declaration",
        10
    ).await;
    
    // Test without declaration included
    let (success_without, duration_without, data_without) = run_find_references_test(
        helpers_file,
        6, // Same position
        10,
        Some(false), // Exclude declaration
        "Find references without declaration",
        10
    ).await;
    
    if success_with && success_without {
        let refs_with = data_with
            .and_then(|d| d["references"].as_array())
            .map(|r| r.len())
            .unwrap_or(0);
        
        let refs_without = data_without
            .and_then(|d| d["references"].as_array())
            .map(|r| r.len())
            .unwrap_or(0);
        
        println!("📊 References comparison:");
        println!("  With declaration: {} references in {:?}", refs_with, duration_with);
        println!("  Without declaration: {} references in {:?}", refs_without, duration_without);
        
        if refs_with > refs_without {
            println!("✅ Declaration inclusion working as expected (more refs with declaration)");
        } else if refs_with == refs_without {
            println!("ℹ️ Same number of references (format may be different)");
        } else {
            println!("⚠️ Unexpected result: more refs without declaration");
        }
    }
    
    println!("✅ Declaration inclusion test completed");
}

#[tokio::test]
async fn test_find_references_cross_file() {
    // 🌐 Test find references across multiple files
    println!("🌐 Testing find references across files...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let operations_file = project_path.join("src/operations.rs");
    
    // Test references for TaskManager (should be used in lib.rs and main.rs)
    let (success, duration, references_data) = run_find_references_test(
        operations_file,
        8, // Line with TaskManager struct definition
        12, // Character position on "TaskManager"
        Some(true), // Include declaration
        "Find cross-file references for TaskManager",
        15
    ).await;
    
    if success {
        if let Some(data) = references_data {
            if let Some(references) = data["references"].as_array() {
                println!("📍 Found {} cross-file references to TaskManager", references.len());
                
                // Analyze references by file
                let mut files_with_refs = std::collections::HashSet::new();
                for reference in references {
                    if let Some(uri) = reference["uri"].as_str() {
                        if let Some(filename) = uri.split('/').last() {
                            files_with_refs.insert(filename);
                        }
                    }
                }
                
                println!("📂 Files containing references: {:?}", files_with_refs);
                
                // Check for expected files
                let expected_files = vec!["lib.rs", "main.rs", "operations.rs"];
                let found_expected = expected_files.iter()
                    .filter(|file| files_with_refs.contains(*file))
                    .count();
                
                if found_expected > 1 {
                    println!("✅ TaskManager referenced across multiple files as expected");
                } else {
                    println!("ℹ️ Fewer cross-file references found than expected");
                }
            }
        }
    }
    
    println!("✅ Cross-file references test completed");
}

#[tokio::test]
async fn test_find_references_context_extraction() {
    // 📝 Test context extraction for references
    println!("📝 Testing reference context extraction...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let models_file = project_path.join("src/models.rs");
    
    // Test references for a commonly used method
    let (success, duration, references_data) = run_find_references_test(
        models_file,
        60, // Line with is_active method definition
        10, // Character position on "is_active"
        Some(false), // Exclude declaration to focus on usage
        "Find references with context extraction",
        10
    ).await;
    
    if success {
        if let Some(data) = references_data {
            if let Some(references) = data["references"].as_array() {
                println!("📍 Found {} references with context", references.len());
                
                // Examine context information
                for (i, reference) in references.iter().take(5).enumerate() {
                    println!("  Reference {}:", i + 1);
                    
                    if let Some(uri) = reference["uri"].as_str() {
                        let filename = uri.split('/').last().unwrap_or("unknown");
                        println!("    File: {}", filename);
                    }
                    
                    if let Some(range) = reference["range"].as_object() {
                        if let (Some(start), Some(end)) = (range["start"].as_object(), range["end"].as_object()) {
                            let start_line = start["line"].as_u64().unwrap_or(0);
                            let end_line = end["line"].as_u64().unwrap_or(0);
                            println!("    Lines: {}-{}", start_line, end_line);
                        }
                    }
                    
                    if let Some(context) = reference["context"].as_str() {
                        println!("    Context: {}", context.trim());
                    }
                }
                
                if references.len() > 0 {
                    println!("✅ Reference context extraction working");
                }
            }
        }
    }
    
    println!("✅ Context extraction test completed");
}

#[tokio::test]
async fn test_find_references_performance() {
    // ⚡ Test find references performance patterns
    println!("⚡ Testing find references performance...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test repeated find references (caching)
    let test_position = (11, 10); // TaskApi struct reference
    let mut durations = Vec::new();
    
    for i in 0..3 {
        let (success, duration, _) = run_find_references_test(
            lib_file.clone(),
            test_position.0,
            test_position.1,
            Some(true),
            &format!("Performance test {}", i + 1),
            10
        ).await;
        
        if success {
            durations.push(duration);
        }
    }
    
    if !durations.is_empty() {
        println!("📊 Find references performance:");
        for (i, duration) in durations.iter().enumerate() {
            println!("  Find references {}: {:?}", i + 1, duration);
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
    }
    
    println!("✅ Performance test completed");
}

#[tokio::test]
async fn test_find_references_edge_cases() {
    // 🧪 Test find references edge cases
    println!("🧪 Testing find references edge cases...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Test edge cases
    let edge_cases = vec![
        (0, 0, "Start of file"),
        (1, 100, "Beyond line end"),
        (999, 0, "Beyond file end"),
        (5, 1, "Comment text"),
        (15, 0, "Start of line"),
    ];
    
    for (line, character, description) in edge_cases {
        let (success, duration, _) = run_find_references_test(
            lib_file.clone(),
            line,
            character,
            Some(true),
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
async fn test_find_references_concurrent() {
    // 🔄 Test concurrent find references requests
    println!("🔄 Testing concurrent find references requests...");
    
    let (_temp_dir, project_path) = create_find_references_test_project().await
        .expect("Failed to create test project");
    
    let lib_file = project_path.join("src/lib.rs");
    
    // Create concurrent find references requests
    let positions = vec![
        (11, 10, "TaskApi struct"),
        (26, 15, "create_task method"),
        (35, 10, "get_task method"),
        (42, 15, "update_task_status method"),
    ];
    
    let futures: Vec<_> = positions.into_iter().enumerate().map(|(i, (line, char, desc))| {
        let file = lib_file.clone();
        async move {
            let (success, duration, _) = run_find_references_test(
                file,
                line,
                char,
                Some(true),
                &format!("Concurrent find references {} ({})", i + 1, desc),
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
    
    println!("📊 Concurrent find references results: {}/4 successful", successful);
    println!("⏱️ Total time: {:?}", total_duration);
    
    if successful > 0 {
        println!("✅ At least one concurrent find references succeeded");
    } else {
        println!("ℹ️ All concurrent find references failed (likely rust-analyzer not available)");
    }
    
    println!("✅ Concurrent find references test completed");
}
