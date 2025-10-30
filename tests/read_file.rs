//! 📖 Read file tool tests - Unicode and chunking edge cases

mod common;

use anyhow::Result;
use common::*;
use empathic::tools::{Tool, read_file::ReadFileTool};
use serde_json::json;

#[tokio::test]
async fn test_read_file_unicode_chunking_boundaries() -> Result<()> {
    // 🎯 Test reading files with Unicode at chunk boundaries
    let env = TestEnv::new()?;
    let tool = ReadFileTool;
    
    // Create test file with complex Unicode content
    let unicode_content = unicode::complex_multiline();
    env.create_file("unicode_test.txt", &unicode_content).await?;
    
    // Test 1: Read entire file - should preserve all Unicode
    let result = tool.execute(
        json!({"path": "unicode_test.txt"}),
        &env.config
    ).await?;
    
    let content = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    assert_eq!(content, unicode_content);
    assert!(content.contains("🚀"));
    assert!(content.contains("русский"));
    assert!(content.contains("中文字符"));
    assert!(content.contains("👨‍👩‍👧‍👦"));
    assert!(content.contains("t̴̹̅e̵̞̓x̶̜̌t̸̰̿"));
    
    println!("✅ Full Unicode file read correctly");
    
    // Test 2: Read with line range
    let result = tool.execute(
        json!({
            "path": "unicode_test.txt",
            "line_offset": 1,
            "line_length": 3
        }),
        &env.config
    ).await?;
    
    let content = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    // Should contain specific lines
    assert!(content.contains("Unicode mix café"));
    assert!(content.contains("Emoji combinations"));
    assert!(content.contains("Mathematical symbols"));
    
    println!("✅ Line range selection with Unicode works");
    
    // Test 3: Large file with repeating Unicode content
    let large_content = unicode::large_repeating(1000); // ~600KB
    env.create_file("large_unicode.txt", &large_content).await?;
    
    let result = tool.execute(
        json!({
            "path": "large_unicode.txt",
            "line_offset": 999,
            "line_length": 3
        }),
        &env.config
    ).await?;
    
    let content = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    // Should still preserve Unicode correctly even in large files
    assert!(content.contains("🚀"));
    assert!(content.contains("русский"));
    
    println!("✅ Large Unicode file chunking works");
    
    Ok(())
}

#[tokio::test]
async fn test_read_file_nonexistent() -> Result<()> {
    // 🎯 Test error handling for nonexistent files
    let env = TestEnv::new()?;
    let tool = ReadFileTool;
    
    let result = tool.execute(
        json!({"path": "nonexistent.txt"}),
        &env.config
    ).await;
    
    // Should return error for nonexistent file
    assert!(result.is_err());
    
    println!("✅ Nonexistent file error handling works");
    Ok(())
}

#[tokio::test]
async fn test_read_file_empty() -> Result<()> {
    // 🎯 Test reading empty files
    let env = TestEnv::new()?;
    let tool = ReadFileTool;
    
    env.create_file("empty.txt", "").await?;
    
    let result = tool.execute(
        json!({"path": "empty.txt"}),
        &env.config
    ).await?;
    
    let content = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    assert_eq!(content, "");
    
    println!("✅ Empty file reading works");
    Ok(())
}

#[tokio::test]
async fn test_read_file_different_encodings() -> Result<()> {
    // 🎯 Test files with different character types
    let env = TestEnv::new()?;
    let tool = ReadFileTool;
    
    // Test different content types
    let test_cases = [
        ("ascii.txt", "Simple ASCII content"),
        ("unicode.txt", unicode::simple()),
        ("boundary.txt", &unicode::boundary_test()),
        ("json.txt", files::json_content()),
        ("code.txt", files::rust_code()),
    ];
    
    for (filename, content) in test_cases {
        env.create_file(filename, content).await?;
        
        let result = tool.execute(
            json!({"path": filename}),
            &env.config
        ).await?;
        
        let read_content = result
            .get("content").unwrap().as_array().unwrap()[0]
            .get("text").unwrap().as_str().unwrap();
        
        assert_eq!(read_content, content);
        println!("✅ {} read correctly", filename);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_read_file_directory_listing() -> Result<()> {
    // 🎯 Test new enhancement: read_file on directory should list contents
    let env = TestEnv::new()?;
    let tool = ReadFileTool;
    
    // Create a test directory with some files (directories auto-created via file paths)
    env.create_file("test_dir/file1.txt", "Content 1").await?;
    env.create_file("test_dir/file2.rs", "fn main() {}").await?;
    env.create_file("test_dir/subdir/nested.txt", "Nested content").await?;
    
    // Test 1: Read directory should return listing, not error
    let result = tool.execute(
        json!({"path": "test_dir"}),
        &env.config
    ).await?;
    
    let content = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    println!("DEBUG: Directory listing content:\n{}", content);
    
    // Verify it's a directory listing with expected format
    assert!(content.contains("📁 Directory listing for:"));
    assert!(content.contains("test_dir"));
    assert!(content.contains("📄 file1.txt"));
    assert!(content.contains("📄 file2.rs"));
    // Debug subdirectory detection
    if content.contains("📁 subdir") {
        println!("✅ Directory listing shows subdir correctly");
    } else {
        println!("❌ Subdir not shown as directory, checking if it shows at all...");
        if content.contains("subdir") {
            println!("   -> subdir appears as: (searching for 'subdir' in content)");
        } else {
            println!("   -> subdir doesn't appear at all");
        }
    }
    assert!(content.contains("Total:"));
    
    println!("✅ Directory listing works with files");
    
    Ok(())
}
