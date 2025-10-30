//! ✍️ Write file tool tests - Line replacement and Unicode handling

mod common;

use anyhow::Result;
use common::*;
use empathic::tools::{Tool, write_file::WriteFileTool};
use serde_json::json;

#[tokio::test]
async fn test_write_file_basic_operations() -> Result<()> {
    // 🎯 Basic write operations
    let env = TestEnv::new()?;
    let tool = WriteFileTool;
    
    // Test 1: Create new file
    let result = tool.execute(
        json!({
            "path": "new_file.txt",
            "content": "Hello, World! 🌍"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Verify file was created
    let file_path = env.root_path.join("new_file.txt");
    assert!(verify_file_content(&file_path, "Hello, World! 🌍").await?);
    
    println!("✅ Basic file creation works");
    
    // Test 2: Overwrite existing file
    let result = tool.execute(
        json!({
            "path": "new_file.txt",
            "content": unicode::simple()
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    assert!(verify_file_content(&file_path, unicode::simple()).await?);
    
    println!("✅ File overwriting works");
    
    Ok(())
}

#[tokio::test]
async fn test_write_file_line_replacement() -> Result<()> {
    // 🎯 Test complex line replacement scenarios
    let env = TestEnv::new()?;
    let tool = WriteFileTool;
    
    // Create file with multiple lines
    let original = scenarios::write_file_original();
    env.create_file("replace_test.txt", &original).await?;
    
    // Test 1: Replace middle lines
    let result = tool.execute(
        json!({
            "path": "replace_test.txt",
            "content": scenarios::write_file_replacement(),
            "start": 1,
            "end": 2
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Verify replacement worked
    let file_path = env.root_path.join("replace_test.txt");
    let content = tokio::fs::read_to_string(&file_path).await?;
    assert!(content.contains("NEW: Replacement content"));
    assert!(content.contains("🚀 unicode"));
    
    println!("✅ Line replacement works");
    
    // Test 2: Single line replacement
    let result = tool.execute(
        json!({
            "path": "replace_test.txt",
            "content": "SINGLE_LINE_REPLACEMENT",
            "start": 0,
            "end": 0
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    let content = tokio::fs::read_to_string(&file_path).await?;
    assert!(content.contains("SINGLE_LINE_REPLACEMENT"));
    
    println!("✅ Single line replacement works");
    
    Ok(())
}

#[tokio::test]
async fn test_write_file_unicode_content() -> Result<()> {
    // 🎯 Test Unicode content in various scenarios
    let env = TestEnv::new()?;
    let tool = WriteFileTool;
    
    // Test 1: Write complex Unicode content
    let unicode_content = unicode::complex_multiline();
    let result = tool.execute(
        json!({
            "path": unicode::unicode_filename(),
            "content": unicode_content
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Verify Unicode is preserved
    let file_path = env.root_path.join(unicode::unicode_filename());
    assert!(verify_file_content(&file_path, &unicode_content).await?);
    
    println!("✅ Unicode content writing works");
    
    // Test 2: Multi-line Unicode replacement
    env.create_file("multi_unicode.txt", &files::large_text(10)).await?;
    
    let multiline_unicode = format!("{}\n{}\n{}", 
        unicode::simple(),
        unicode::boundary_test(),
        "Final line with émojis 🎉"
    );
    
    let result = tool.execute(
        json!({
            "path": "multi_unicode.txt",
            "content": multiline_unicode,
            "start": 3,
            "end": 6
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    let file_path = env.root_path.join("multi_unicode.txt");
    let content = tokio::fs::read_to_string(&file_path).await?;
    assert!(content.contains("Hello 🌍"));
    assert!(content.contains("🎉"));
    
    println!("✅ Multi-line Unicode replacement works");
    
    Ok(())
}

#[tokio::test]
async fn test_write_file_edge_cases() -> Result<()> {
    // 🎯 Test edge cases and error conditions
    let env = TestEnv::new()?;
    let tool = WriteFileTool;
    
    // Test 1: Empty content (deletion)
    env.create_file("delete_test.txt", &files::large_text(5)).await?;
    
    let result = tool.execute(
        json!({
            "path": "delete_test.txt",
            "content": "",
            "start": 1,
            "end": 3
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    println!("✅ Empty content replacement (deletion) works");
    
    // Test 2: Write to nested directory (should create dirs)
    let result = tool.execute(
        json!({
            "path": "nested/deep/file.txt",
            "content": "Nested file content"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    let nested_file = env.root_path.join("nested/deep/file.txt");
    assert!(verify_file_content(&nested_file, "Nested file content").await?);
    
    println!("✅ Nested directory creation works");
    
    Ok(())
}

#[tokio::test]
async fn test_write_file_large_content() -> Result<()> {
    // 🎯 Test writing large files
    let env = TestEnv::new()?;
    let tool = WriteFileTool;
    
    // Create large Unicode content
    let large_content = unicode::large_repeating(150); // ~75KB to ensure > 50KB
    
    let result = tool.execute(
        json!({
            "path": "large_file.txt",
            "content": large_content
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Verify large file was written correctly
    let file_path = env.root_path.join("large_file.txt");
    let written_content = tokio::fs::read_to_string(&file_path).await?;
    assert_eq!(written_content, large_content);
    
    // Check file size
    let size = file_size(&file_path).await?;
    assert!(size > 50_000); // Should be around 60KB
    
    println!("✅ Large file writing works ({}KB)", size / 1024);
    
    Ok(())
}
