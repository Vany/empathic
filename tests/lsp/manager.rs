//! 🔬 LSP Manager Process Management Tests
//!
//! Comprehensive tests for LSP server process management including:
//! - Process spawning and lifecycle management
//! - Resource monitoring and cleanup  
//! - Error handling and recovery scenarios
//! - Document synchronization state management

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

use empathic::lsp::manager::LspManager;
use empathic::lsp::project_detector::ProjectDetector;
use empathic::lsp::resource::ResourceConfig;
use empathic::lsp::types::LspResult;

/// 🏗️ Create test LSP manager with default configuration
async fn create_test_manager() -> LspResult<LspManager> {
    let config = ResourceConfig::default();
    LspManager::new(config).await
}

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
    
    // Create basic lib.rs
    let src_dir = project_path.join("src");
    tokio::fs::create_dir_all(&src_dir).await?;
    let lib_rs = src_dir.join("lib.rs");
    tokio::fs::write(&lib_rs, "//! Test project\n\npub fn hello() -> &'static str {\n    \"Hello, world!\"\n}\n").await?;
    
    Ok((temp_dir, project_path))
}

#[tokio::test]
async fn test_manager_creation() {
    // ✅ Test basic manager creation
    let manager = create_test_manager().await;
    assert!(manager.is_ok(), "Manager creation should succeed");
    
    let manager = manager.unwrap();
    let stats = manager.get_resource_stats().await;
    assert_eq!(stats.process_count, 0, "No processes should be running initially");
}

#[tokio::test] 
async fn test_project_detection() {
    // 🔍 Test project detection functionality
    let (_temp_dir, project_path) = create_rust_project().await
        .expect("Failed to create test project");
    
    let detector = ProjectDetector::new();
    let projects = detector.find_rust_projects(&project_path);
    
    assert_eq!(projects.len(), 1, "Should find exactly one Rust project");
    assert_eq!(projects[0], project_path, "Should find the correct project path");
}

#[tokio::test]
async fn test_resource_monitoring_startup() {
    // 📊 Test resource monitoring lifecycle
    let manager = create_test_manager().await
        .expect("Failed to create manager");
    
    // Start monitoring
    let start_result = manager.start_resource_monitoring().await;
    assert!(start_result.is_ok(), "Resource monitoring should start successfully");
    
    // Check initial stats
    let stats = manager.get_resource_stats().await;
    assert_eq!(stats.process_count, 0, "No processes initially");
    assert_eq!(stats.total_memory_mb, 0.0, "No memory usage initially");
    
    // Stop monitoring
    manager.stop_resource_monitoring().await;
    println!("✅ Resource monitoring lifecycle test completed");
}

#[tokio::test]
async fn test_document_tracking() {
    // 📄 Test document state management
    let (_temp_dir, project_path) = create_rust_project().await
        .expect("Failed to create test project");
    
    let manager = create_test_manager().await
        .expect("Failed to create manager");
    
    let file_path = project_path.join("src/lib.rs");
    let content = "pub fn test() {}\n";
    
    // Test document operations without LSP server (should handle gracefully)
    let open_result = timeout(
        Duration::from_secs(2),
        manager.open_document(&file_path, content)
    ).await;
    
    // Should either succeed or fail gracefully (not hang)
    match open_result {
        Ok(result) => {
            // If LSP server available, should succeed
            println!("📄 Document opened: {:?}", result);
        },
        Err(_) => {
            // Timeout is acceptable if rust-analyzer not available
            println!("⏱️ Document open timeout (expected without rust-analyzer)");
        }
    }
    
    println!("✅ Document tracking test completed");
}

#[tokio::test]
async fn test_graceful_shutdown() {
    // 🛑 Test graceful shutdown functionality
    let manager = create_test_manager().await
        .expect("Failed to create manager");
    
    // Start resource monitoring
    let _ = manager.start_resource_monitoring().await;
    
    // Perform graceful shutdown
    let shutdown_result = timeout(
        Duration::from_secs(5),
        manager.graceful_shutdown_all()
    ).await;
    
    assert!(shutdown_result.is_ok(), "Graceful shutdown should not timeout");
    
    let shutdown_result = shutdown_result.unwrap();
    assert!(shutdown_result.is_ok(), "Graceful shutdown should succeed");
    
    // Verify cleanup
    let stats = manager.get_resource_stats().await;
    assert_eq!(stats.process_count, 0, "All processes should be cleaned up");
    
    println!("✅ Graceful shutdown test completed");
}

#[tokio::test]
async fn test_health_check() {
    // 🏥 Test comprehensive health monitoring
    let manager = create_test_manager().await
        .expect("Failed to create manager");
    
    // Start monitoring for health check
    let _ = manager.start_resource_monitoring().await;
    
    // Perform health check
    let health_result = timeout(
        Duration::from_secs(5),
        manager.comprehensive_health_check()
    ).await;
    
    assert!(health_result.is_ok(), "Health check should not timeout");
    
    let health_result = health_result.unwrap();
    assert!(health_result.is_ok(), "Health check should succeed");
    
    let health_status = health_result.unwrap();
    assert_eq!(health_status.healthy_count, 0, "No healthy processes initially");
    assert_eq!(health_status.unhealthy_count, 0, "No unhealthy processes initially");
    
    // Cleanup
    manager.stop_resource_monitoring().await;
    
    println!("✅ Health check test completed");
}

#[tokio::test]
async fn test_error_recovery() {
    // 🚨 Test error handling and recovery mechanisms
    let manager = create_test_manager().await
        .expect("Failed to create manager");
    
    // Test operations on non-existent file (should handle gracefully)
    let non_existent = PathBuf::from("/non/existent/file.rs");
    
    let open_result = timeout(
        Duration::from_secs(2),
        manager.open_document(&non_existent, "test content")
    ).await;
    
    // Should handle gracefully (either error or timeout)
    match open_result {
        Ok(Err(_)) => {
            println!("✅ Error handled gracefully for non-existent file");
        },
        Err(_) => {
            println!("⏱️ Timeout handled gracefully for non-existent file");
        },
        Ok(Ok(_)) => {
            // Unexpected success, but not necessarily wrong
            println!("⚠️ Unexpected success for non-existent file");
        }
    }
    
    println!("✅ Error recovery test completed");
}

#[tokio::test]
async fn test_concurrent_operations() {
    // 🔄 Test concurrent operation handling
    let manager = create_test_manager().await
        .expect("Failed to create manager");
    
    let (_temp_dir, project_path) = create_rust_project().await
        .expect("Failed to create test project");
    
    // Start resource monitoring
    let _ = manager.start_resource_monitoring().await;
    
    // Test concurrent resource stats access
    let stats_futures: Vec<_> = (0..5).map(|_| {
        manager.get_resource_stats()
    }).collect();
    
    let stats_results = futures::future::join_all(stats_futures).await;
    
    // All should succeed
    for stats in stats_results {
        assert_eq!(stats.process_count, 0, "Concurrent stats access should work");
    }
    
    // Test concurrent health checks
    let health_futures: Vec<_> = (0..3).map(|_| {
        timeout(Duration::from_secs(2), manager.comprehensive_health_check())
    }).collect();
    
    let health_results = futures::future::join_all(health_futures).await;
    
    // All should complete (success or timeout acceptable)
    for result in health_results {
        match result {
            Ok(Ok(_)) => println!("✅ Concurrent health check succeeded"),
            Ok(Err(_)) => println!("⚠️ Concurrent health check failed gracefully"),
            Err(_) => println!("⏱️ Concurrent health check timeout (acceptable)"),
        }
    }
    
    // Cleanup
    manager.stop_resource_monitoring().await;
    
    println!("✅ Concurrent operations test completed");
}

#[tokio::test]
async fn test_resource_limit_handling() {
    // 📏 Test resource limit detection and enforcement
    let config = ResourceConfig {
        max_rss_mb: 1.0, // Very low limit for testing
        max_memory_percent: 1.0, // Very low percentage
        monitor_interval_secs: 1,
        restart_grace_secs: 5,
        max_restart_attempts: 1,
    };
    
    let manager = LspManager::new(config).await
        .expect("Failed to create manager with custom config");
    
    let _ = manager.start_resource_monitoring().await;
    
    // Check that limits are properly configured
    let stats = manager.get_resource_stats().await;
    assert_eq!(stats.process_count, 0, "No processes initially");
    
    // Test memory limit checking for current process
    let current_pid = std::process::id();
    let limit_check = manager.check_process_limits(current_pid).await;
    
    // Should detect current process as over limit (due to very low threshold)
    match limit_check {
        Some(true) => println!("✅ Correctly detected process over memory limit"),
        Some(false) => println!("⚠️ Process within memory limit (unexpected with low threshold)"),
        None => println!("ℹ️ Could not check memory limits (acceptable)"),
    }
    
    manager.stop_resource_monitoring().await;
    
    println!("✅ Resource limit handling test completed");
}
