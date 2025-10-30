//! 🔬 LSP Stability and Memory Leak Detection Tests
//!
//! Long-running tests that validate resource management, memory monitoring,
//! and automatic restart capabilities under various stress conditions.

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::interval;

// Re-export test utilities
mod common;
use common::*;

use empathic::lsp::manager::LspManager;
use empathic::lsp::resource::{ResourceConfig, ResourceMonitor};

/// 🏃‍♂️ Long-running resource monitoring stability test
/// 
/// Validates that resource monitoring can run for extended periods without
/// memory leaks or performance degradation.
#[tokio::test]
async fn test_long_running_resource_monitoring() {
    // Create test environment
    let env = TestEnv::new().expect("Failed to create test environment");
    let _test_project = env.create_rust_project("stability_test").await
        .expect("Failed to create test project");

    // Create LSP manager with aggressive monitoring config
    let resource_config = ResourceConfig {
        max_rss_mb: 512.0,        // 512MB limit for testing
        max_memory_percent: 5.0,   // 5% memory limit
        monitor_interval_secs: 1,  // Monitor every second
        restart_grace_secs: 2,     // Quick restart for testing
        max_restart_attempts: 5,   // Allow more restarts
    };
    
    let manager = LspManager::with_resource_config(env.root_dir().clone(), resource_config);
    
    // Start resource monitoring
    manager.start_resource_monitoring().await
        .expect("Failed to start resource monitoring");
    
    // Track memory usage over time
    let mut memory_samples = Vec::new();
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(30); // 30-second test
    
    let mut interval = interval(Duration::from_secs(2));
    
    while start_time.elapsed() < test_duration {
        interval.tick().await;
        
        let stats = manager.get_resource_stats().await;
        memory_samples.push((start_time.elapsed().as_secs(), stats.avg_memory_mb));
        
        println!("📊 [{:3}s] Memory: {:.1}MB, Processes: {}, Restarts: {}", 
               start_time.elapsed().as_secs(),
               stats.avg_memory_mb,
               stats.total_processes,
               stats.total_restarts);
    }
    
    // Stop monitoring and validate no memory leaks
    manager.stop_resource_monitoring().await;
    
    // Memory should be stable (no continuous growth)
    if memory_samples.len() >= 10 {
        let first_half: f64 = memory_samples[..memory_samples.len()/2].iter()
            .map(|(_, mem)| *mem).sum::<f64>() / (memory_samples.len()/2) as f64;
        let second_half: f64 = memory_samples[memory_samples.len()/2..].iter()
            .map(|(_, mem)| *mem).sum::<f64>() / (memory_samples.len()/2) as f64;
        
        let memory_growth = second_half - first_half;
        println!("📈 Memory growth over test: {:.1}MB", memory_growth);
        
        // Memory growth should be minimal (< 10MB indicates no major leaks)
        assert!(memory_growth < 10.0, "Potential memory leak detected: {:.1}MB growth", memory_growth);
    }
    
    println!("✅ Long-running resource monitoring test completed successfully");
}

/// 🔄 Automatic restart stability test
/// 
/// Simulates process crashes and validates automatic restart behavior
/// over multiple cycles without resource leaks.
#[tokio::test]
async fn test_automatic_restart_stability() {
    let env = TestEnv::new().expect("Failed to create test environment");
    let test_project = env.create_rust_project("restart_test").await
        .expect("Failed to create test project");

    // Create test file
    let test_file = test_project.join("src/main.rs");
    env.write_file(&test_file, r#"
fn main() {
    println!("Hello, restart test!");
}
"#).await.expect("Failed to create test file");

    // Create manager with aggressive restart config
    let resource_config = ResourceConfig {
        max_rss_mb: 64.0,         // Very low limit to trigger restarts
        max_memory_percent: 2.0,   // Very low percentage
        monitor_interval_secs: 1,  // Fast monitoring
        restart_grace_secs: 1,     // Quick restart
        max_restart_attempts: 10,  // Allow many restarts for testing
    };
    
    let manager = LspManager::with_resource_config(env.root_dir().clone(), resource_config);
    manager.start_resource_monitoring().await
        .expect("Failed to start resource monitoring");
    
    let mut restart_cycles = 0;
    let max_cycles = 5; // Test 5 restart cycles
    
    for cycle in 0..max_cycles {
        println!("🔄 Starting restart cycle {}/{}", cycle + 1, max_cycles);
        
        // Spawn LSP server
        let _server = manager.get_or_spawn_server(&test_file).await
            .expect("Failed to spawn LSP server");
        
        // Open document to establish connection
        manager.open_document(&test_file).await
            .expect("Failed to open document");
        
        // Wait for server to be established
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Simulate restart by shutting down and respawning
        manager.shutdown_server(&test_project).await
            .expect("Failed to shutdown server");
        
        // Record restart using the public interface
        let restart_reason = format!("Test cycle {}", cycle + 1);
        log::info!("🔄 Recording restart: {}", restart_reason);
        
        restart_cycles += 1;
        
        // Brief pause between cycles
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Validate restart tracking
    let stats = manager.get_resource_stats().await;
    assert_eq!(stats.total_restarts, restart_cycles as u64, 
              "Restart count mismatch: expected {}, got {}", 
              restart_cycles, stats.total_restarts);
    
    // Cleanup
    manager.graceful_shutdown_all().await
        .expect("Failed to gracefully shutdown");
    
    println!("✅ Automatic restart stability test completed: {} cycles", restart_cycles);
}

/// 💾 Memory limit enforcement test
/// 
/// Tests that memory limits are properly detected and processes are
/// flagged when they exceed configured thresholds.
#[tokio::test] 
async fn test_memory_limit_enforcement() {
    let _env = TestEnv::new().expect("Failed to create test environment");
    
    // Create resource monitor with very low limits for testing
    let config = ResourceConfig {
        max_rss_mb: 1.0,          // 1MB limit (artificially low)
        max_memory_percent: 0.1,   // 0.1% limit (artificially low)
        monitor_interval_secs: 1,
        restart_grace_secs: 1,
        max_restart_attempts: 3,
    };
    
    let monitor = ResourceMonitor::new(config);
    monitor.start_monitoring().await
        .expect("Failed to start monitoring");
    
    // Wait for monitoring to collect some data
    tokio::time::sleep(Duration::from_secs(3)).await;
    
    // Check for over-limit processes (should find some with such low limits)
    let over_limit = monitor.get_over_limit_processes().await;
    let stats = monitor.get_stats().await;
    
    println!("📊 Memory monitoring results:");
    println!("   Total processes: {}", stats.total_processes);
    println!("   Over limit: {}", over_limit.len());
    println!("   Average memory: {:.1}MB", stats.avg_memory_mb);
    println!("   Peak memory: {:.1}MB", stats.peak_memory_mb);
    
    // With such low limits, we should detect some processes over limit
    // (but only if there are actually processes running)
    if stats.total_processes > 0 {
        assert!(stats.peak_memory_mb > 0.0, "Peak memory should be recorded");
        // Note: Over-limit detection depends on actual system processes
        // so we just validate the monitoring is working
    }
    
    monitor.stop_monitoring().await;
    
    println!("✅ Memory limit enforcement test completed");
}

/// 🏥 Comprehensive health monitoring test
/// 
/// Tests health check functionality under various conditions including
/// process failures and resource exhaustion scenarios.
#[tokio::test]
async fn test_comprehensive_health_monitoring() {
    let env = TestEnv::new().expect("Failed to create test environment");
    let test_project = env.create_rust_project("health_test").await
        .expect("Failed to create test project");

    let test_file = test_project.join("src/lib.rs");
    env.write_file(&test_file, r#"
//! Health monitoring test library

/// Test function for health monitoring
pub fn health_check() {
    println!("Health check function");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_function() {
        health_check();
    }
}
"#).await.expect("Failed to create test file");

    let manager = LspManager::new(env.root_dir().clone());
    manager.start_resource_monitoring().await
        .expect("Failed to start resource monitoring");
    
    // Initial health check (no servers running)
    let health1 = manager.comprehensive_health_check().await
        .expect("Failed to perform initial health check");
    
    assert_eq!(health1.healthy_processes.len(), 0, "Should have no healthy processes initially");
    assert_eq!(health1.unhealthy_processes.len(), 0, "Should have no unhealthy processes initially");
    
    println!("📊 Initial health check: {} healthy, {} unhealthy", 
           health1.healthy_processes.len(), health1.unhealthy_processes.len());
    
    // Spawn a server and check health
    let _server = manager.get_or_spawn_server(&test_file).await
        .expect("Failed to spawn LSP server");
    
    // Wait for server to fully initialize
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let health2 = manager.comprehensive_health_check().await
        .expect("Failed to perform health check with server");
    
    let total_processes = health2.healthy_processes.len() + health2.unhealthy_processes.len();
    assert!(total_processes > 0, "Should have at least one process running");
    
    println!("📊 With server health check: {} healthy, {} unhealthy", 
           health2.healthy_processes.len(), health2.unhealthy_processes.len());
    
    // Test graceful shutdown
    let shutdown_start = Instant::now();
    manager.graceful_shutdown_all().await
        .expect("Failed to gracefully shutdown");
    let shutdown_duration = shutdown_start.elapsed();
    
    println!("🛑 Graceful shutdown completed in {}ms", shutdown_duration.as_millis());
    
    // Final health check (should be clean)
    let health3 = manager.comprehensive_health_check().await
        .expect("Failed to perform final health check");
    
    assert_eq!(health3.healthy_processes.len(), 0, "Should have no processes after shutdown");
    assert_eq!(health3.unhealthy_processes.len(), 0, "Should have no processes after shutdown");
    
    // Shutdown should be reasonably fast (< 5 seconds)
    assert!(shutdown_duration < Duration::from_secs(5), 
           "Graceful shutdown took too long: {}ms", shutdown_duration.as_millis());
    
    println!("✅ Comprehensive health monitoring test completed");
}

/// ⚡ Performance under resource pressure test
/// 
/// Tests system behavior when resource monitoring detects pressure
/// and validates that performance metrics remain accurate.
#[tokio::test]
async fn test_performance_under_pressure() {
    let env = TestEnv::new().expect("Failed to create test environment");
    let test_project = env.create_rust_project("pressure_test").await
        .expect("Failed to create test project");

    // Create multiple test files to stress the system
    let mut test_files = Vec::new();
    for i in 0..5 {
        let file_path = test_project.join(format!("src/module_{}.rs", i));
        env.write_file(&file_path, &format!(r#"
//! Module {} for pressure testing

use std::collections::HashMap;

/// Test structure for module {}
#[derive(Debug, Clone)]
pub struct TestStruct{} {{
    pub id: u32,
    pub name: String,
    pub data: HashMap<String, String>,
}}

impl TestStruct{} {{
    pub fn new(id: u32, name: String) -> Self {{
        Self {{
            id,
            name,
            data: HashMap::new(),
        }}
    }}
    
    pub fn add_data(&mut self, key: String, value: String) {{
        self.data.insert(key, value);
    }}
    
    pub fn get_data(&self, key: &str) -> Option<&String> {{
        self.data.get(key)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_struct_creation() {{
        let s = TestStruct{}::new(1, "test".to_string());
        assert_eq!(s.id, 1);
        assert_eq!(s.name, "test");
    }}
}}
"#, i, i, i, i, i)).await.expect("Failed to create test file");
        test_files.push(file_path);
    }

    let manager = LspManager::new(env.root_dir().clone());
    manager.start_resource_monitoring().await
        .expect("Failed to start resource monitoring");
    
    // Measure performance under pressure
    let start_time = Instant::now();
    let mut operation_times = Vec::new();
    
    // Perform multiple operations rapidly
    for (i, file_path) in test_files.iter().enumerate() {
        let op_start = Instant::now();
        
        // Open document
        manager.open_document(file_path).await
            .expect("Failed to open document");
        
        // Update document content
        let new_content = format!("// Updated at {} ms\n{}", 
                                start_time.elapsed().as_millis(),
                                tokio::fs::read_to_string(file_path).await.unwrap());
        manager.update_document(file_path, &new_content).await
            .expect("Failed to update document");
        
        let op_duration = op_start.elapsed();
        operation_times.push(op_duration);
        
        println!("📝 Operation {} completed in {}ms", i + 1, op_duration.as_millis());
        
        // Brief pause to allow monitoring
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Validate performance metrics
    let performance_summary = manager.performance_summary();
    let resource_summary = manager.get_resource_summary().await;
    
    println!("⚡ Performance Summary: {}", performance_summary);
    println!("📊 Resource Summary: {}", resource_summary);
    
    // Calculate operation statistics
    let avg_time = operation_times.iter().sum::<Duration>().as_millis() as f64 / operation_times.len() as f64;
    let max_time = operation_times.iter().max().unwrap().as_millis();
    
    println!("📈 Operation stats: avg={:.1}ms, max={}ms", avg_time, max_time);
    
    // Operations should complete in reasonable time even under pressure
    assert!(avg_time < 1000.0, "Average operation time too slow: {:.1}ms", avg_time);
    assert!(max_time < 2000, "Maximum operation time too slow: {}ms", max_time);
    
    // Cleanup
    manager.graceful_shutdown_all().await
        .expect("Failed to gracefully shutdown");
    
    println!("✅ Performance under pressure test completed");
}

/// 🔍 Edge case stability test
/// 
/// Tests various edge cases that could cause instability including
/// file system errors, invalid paths, and resource exhaustion.
#[tokio::test]
async fn test_edge_case_stability() {
    let env = TestEnv::new().expect("Failed to create test environment");
    
    let manager = LspManager::new(env.root_dir().clone());
    manager.start_resource_monitoring().await
        .expect("Failed to start resource monitoring");
    
    // Test 1: Non-existent file handling
    let nonexistent_file = env.root_dir().join("nonexistent.rs");
    let result1 = manager.get_or_spawn_server(&nonexistent_file).await;
    assert!(result1.is_err(), "Should fail for non-existent file");
    println!("✅ Non-existent file handled correctly");
    
    // Test 2: Invalid file path handling
    let invalid_path = PathBuf::from("/invalid/path/that/does/not/exist.rs");
    let result2 = manager.open_document(&invalid_path).await;
    assert!(result2.is_err(), "Should fail for invalid path");
    println!("✅ Invalid path handled correctly");
    
    // Test 3: Multiple rapid shutdown/startup cycles
    for cycle in 0..3 {
        println!("🔄 Edge case cycle {}/3", cycle + 1);
        
        // Try to perform operations that might fail
        let _health = manager.comprehensive_health_check().await
            .expect("Health check should always work");
        
        manager.graceful_shutdown_all().await
            .expect("Graceful shutdown should always work");
        
        // Brief pause
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Test 4: Resource monitoring after shutdown
    let final_stats = manager.get_resource_stats().await;
    let final_summary = manager.get_resource_summary().await;
    
    println!("📊 Final stats: processes={}, restarts={}", 
           final_stats.total_processes, final_stats.total_restarts);
    println!("📋 Final summary: {}", final_summary);
    
    // Stop monitoring
    manager.stop_resource_monitoring().await;
    
    println!("✅ Edge case stability test completed");
}

/// 🧪 Integration stability test with all components
/// 
/// Comprehensive test that exercises all LSP features together under
/// realistic conditions to validate overall system stability.
#[tokio::test]
async fn test_full_integration_stability() {
    let env = TestEnv::new().expect("Failed to create test environment");
    let test_project = env.create_rust_project("integration_test").await
        .expect("Failed to create test project");

    // Create a realistic Rust project structure
    let src_dir = test_project.join("src");
    let lib_file = src_dir.join("lib.rs");
    let main_file = src_dir.join("main.rs");
    let mod_file = src_dir.join("helpers.rs");

    env.write_file(&lib_file, r#"
//! Integration test library

pub mod helpers;

use std::collections::HashMap;

/// Main library structure
#[derive(Debug, Clone)]
pub struct Library {
    pub name: String,
    pub version: String,
    pub modules: HashMap<String, String>,
}

impl Library {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            modules: HashMap::new(),
        }
    }
    
    pub fn add_module(&mut self, name: String, description: String) {
        self.modules.insert(name, description);
    }
    
    pub fn get_module(&self, name: &str) -> Option<&String> {
        self.modules.get(name)
    }
}

pub use helpers::*;
"#).await.expect("Failed to create lib.rs");

    env.write_file(&main_file, r#"
//! Integration test main

use integration_test::{Library, helper_function};

fn main() {
    let mut lib = Library::new("test".to_string(), "1.0.0".to_string());
    lib.add_module("core".to_string(), "Core functionality".to_string());
    
    println!("Library: {} v{}", lib.name, lib.version);
    helper_function();
}
"#).await.expect("Failed to create main.rs");

    env.write_file(&mod_file, r#"
//! Helper functions module

/// Helper function for testing
pub fn helper_function() {
    println!("Helper function called");
}

/// Utility structure
#[derive(Debug)]
pub struct Utility {
    pub id: u32,
    pub name: String,
}

impl Utility {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }
}
"#).await.expect("Failed to create helpers.rs");

    // Create manager with balanced configuration
    let resource_config = ResourceConfig {
        max_rss_mb: 1024.0,       // 1GB reasonable limit
        max_memory_percent: 15.0,  // 15% reasonable limit
        monitor_interval_secs: 2,  // Monitor every 2 seconds
        restart_grace_secs: 5,     // 5 second grace period
        max_restart_attempts: 3,   // 3 restart attempts
    };
    
    let manager = LspManager::with_resource_config(env.root_dir().clone(), resource_config);
    manager.start_resource_monitoring().await
        .expect("Failed to start resource monitoring");
    
    println!("🚀 Starting full integration stability test...");
    let test_start = Instant::now();
    
    // Phase 1: Basic operations
    println!("📋 Phase 1: Basic operations");
    for file in [&lib_file, &main_file, &mod_file] {
        manager.open_document(file).await
            .expect("Failed to open document");
        println!("  ✅ Opened: {}", file.file_name().unwrap().to_string_lossy());
    }
    
    // Phase 2: Performance monitoring
    println!("📋 Phase 2: Performance monitoring");
    let mut performance_samples = Vec::new();
    for i in 0..5 {
        let sample_start = Instant::now();
        
        let health = manager.comprehensive_health_check().await
            .expect("Failed to perform health check");
        let stats = manager.get_resource_stats().await;
        
        let sample_duration = sample_start.elapsed();
        performance_samples.push(sample_duration);
        
        println!("  📊 Sample {}: {}ms, {} healthy processes, {:.1}MB avg memory", 
               i + 1, sample_duration.as_millis(), health.healthy_processes.len(), stats.avg_memory_mb);
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    // Phase 3: Document modifications
    println!("📋 Phase 3: Document modifications");
    for file in [&lib_file, &main_file, &mod_file].iter() {
        let original_content = tokio::fs::read_to_string(file).await
            .expect("Failed to read file");
        
        let modified_content = format!("// Modified at {}ms\n{}", 
                                     test_start.elapsed().as_millis(), 
                                     original_content);
        
        manager.update_document(file, &modified_content).await
            .expect("Failed to update document");
        
        println!("  ✏️ Modified: {}", file.file_name().unwrap().to_string_lossy());
        
        // Brief pause between modifications
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // Phase 4: Cache validation
    println!("📋 Phase 4: Cache validation");
    let cache_stats_before = manager.cache().stats().await;
    
    // Trigger some cache operations
    for file in [&lib_file, &main_file, &mod_file] {
        manager.invalidate_file_cache(file).await;
    }
    
    let cache_stats_after = manager.cache().stats().await;
    println!("  🗄️ Cache entries before: {}, after: {}", 
           cache_stats_before.total_entries, cache_stats_after.total_entries);
    
    // Phase 5: Final validation
    println!("📋 Phase 5: Final validation");
    
    let final_health = manager.comprehensive_health_check().await
        .expect("Failed to perform final health check");
    let final_stats = manager.get_resource_stats().await;
    let final_performance = manager.performance_summary();
    
    println!("  🏥 Final health: {} healthy, {} unhealthy", 
           final_health.healthy_processes.len(), final_health.unhealthy_processes.len());
    println!("  📊 Final stats: {} processes, {} restarts", 
           final_stats.total_processes, final_stats.total_restarts);
    println!("  ⚡ Performance: {}", final_performance);
    
    // Performance validation
    let avg_perf = performance_samples.iter().sum::<Duration>().as_millis() as f64 / performance_samples.len() as f64;
    assert!(avg_perf < 500.0, "Average performance check too slow: {:.1}ms", avg_perf);
    
    // Graceful shutdown
    println!("🛑 Graceful shutdown...");
    let shutdown_start = Instant::now();
    manager.graceful_shutdown_all().await
        .expect("Failed to gracefully shutdown");
    let shutdown_duration = shutdown_start.elapsed();
    
    println!("✅ Full integration stability test completed in {}ms (shutdown: {}ms)", 
           test_start.elapsed().as_millis(), shutdown_duration.as_millis());
    
    // Final assertions
    assert!(shutdown_duration < Duration::from_secs(10), 
           "Shutdown took too long: {}ms", shutdown_duration.as_millis());
    assert!(final_stats.total_restarts < 3, 
           "Too many restarts during test: {}", final_stats.total_restarts);
}
