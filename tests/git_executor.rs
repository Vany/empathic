//! 🐙 Git executor tool tests - Commands, workflows, and Unicode handling

mod common;

use anyhow::Result;
use common::*;
use empathic::tools::{Tool, git::GitTool};
use serde_json::json;

async fn init_git_repo(env: &TestEnv, project_name: &str) -> Result<()> {
    let tool = GitTool;
    
    // Initialize repo
    let result = tool.execute(
        json!({
            "args": ["init"],
            "project": project_name
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Configure git user (required for commits)
    for (key, value) in [("user.name", "Test User"), ("user.email", "test@example.com")] {
        let result = tool.execute(
            json!({
                "args": ["config", key, value],
                "project": project_name
            }),
            &env.config
        ).await?;
        
        let parsed = McpResult::parse(result)?;
        assert_mcp_success(&parsed);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_git_basic_commands() -> Result<()> {
    // 🎯 Test basic git command execution
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    let _project_dir = env.create_project("test_repo").await?;
    init_git_repo(&env, "test_repo").await?;
    
    // Test git status
    let result = tool.execute(
        json!({
            "args": ["status", "--porcelain"],
            "project": "test_repo"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    assert_eq!(parsed.exit_code(), Some(0));
    assert!(parsed.working_dir().unwrap().contains("test_repo"));
    
    println!("✅ Basic git commands work");
    
    Ok(())
}

#[tokio::test]
async fn test_git_invalid_commands() -> Result<()> {
    // 🎯 Test error handling for invalid git commands
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    // Test invalid command
    let result = tool.execute(
        json!({
            "args": ["nonexistent-command", "--invalid-flag"]
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_failure(&parsed);
    assert_ne!(parsed.exit_code().unwrap_or(0), 0);
    assert!(!parsed.stderr().unwrap_or("").is_empty());
    
    println!("✅ Invalid command error handling works");
    
    // Test git command in non-git directory
    let result = tool.execute(
        json!({"args": ["status"]}),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_failure(&parsed);
    let stderr = parsed.stderr().unwrap_or("");
    assert!(stderr.contains("not a git repository") || stderr.contains("fatal"));
    
    println!("✅ Non-git directory error handling works");
    
    Ok(())
}

#[tokio::test]
async fn test_git_workflow() -> Result<()> {
    // 🎯 Test complete git workflow with files and commits
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    let project_dir = env.create_project("workflow_repo").await?;
    init_git_repo(&env, "workflow_repo").await?;
    
    // Create test files
    let test_files = scenarios::git_test_files();
    create_test_files(&project_dir, &test_files).await?;
    
    // Add files to git
    for (filename, _) in &test_files {
        let result = tool.execute(
            json!({
                "args": ["add", filename],
                "project": "workflow_repo"
            }),
            &env.config
        ).await?;
        
        let parsed = McpResult::parse(result)?;
        assert_mcp_success(&parsed);
    }
    
    // Commit files
    let result = tool.execute(
        json!({
            "args": ["commit", "-m", "Add test files with emoji 🚀"],
            "project": "workflow_repo"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Check git log
    let result = tool.execute(
        json!({
            "args": ["log", "--oneline", "-1"],
            "project": "workflow_repo"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    assert!(parsed.stdout().unwrap().contains("Add test files with emoji 🚀"));
    
    println!("✅ Complete git workflow works");
    
    Ok(())
}

#[tokio::test]
async fn test_git_unicode_handling() -> Result<()> {
    // 🎯 Test git with Unicode filenames, content, and commit messages
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    // Create project with Unicode name
    let unicode_project = "тест_репо";
    let project_dir = env.create_project(unicode_project).await?;
    init_git_repo(&env, unicode_project).await?;
    
    // Create Unicode file
    let unicode_filename = unicode::unicode_filename();
    let unicode_content = unicode::complex_multiline();
    create_test_files(&project_dir, &[(unicode_filename, &unicode_content)]).await?;
    
    // Add and commit with Unicode commit message
    let result = tool.execute(
        json!({
            "args": ["add", unicode_filename],
            "project": unicode_project
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    let commit_msg = unicode::commit_message();
    let result = tool.execute(
        json!({
            "args": ["commit", "-m", commit_msg],
            "project": unicode_project
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    // Verify Unicode is preserved in log
    let result = tool.execute(
        json!({
            "args": ["log", "--oneline", "-1"],
            "project": unicode_project
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    let stdout = parsed.stdout().unwrap();
    assert!(stdout.contains("Добавить файл 📁"));
    assert!(stdout.contains("मिश्रित भाषा"));
    
    println!("✅ Unicode handling works");
    
    Ok(())
}

#[tokio::test]
async fn test_git_branch_operations() -> Result<()> {
    // 🎯 Test git branch creation and switching
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    let project_dir = env.create_project("branch_repo").await?;
    init_git_repo(&env, "branch_repo").await?;
    
    // Create initial commit
    create_test_files(&project_dir, &[("initial.txt", "initial content")]).await?;
    
    let result = tool.execute(
        json!({
            "args": ["add", "initial.txt"],
            "project": "branch_repo"
        }),
        &env.config
    ).await?;
    assert_mcp_success(&McpResult::parse(result)?);
    
    let result = tool.execute(
        json!({
            "args": ["commit", "-m", "Initial commit"],
            "project": "branch_repo"
        }),
        &env.config
    ).await?;
    assert_mcp_success(&McpResult::parse(result)?);
    
    // Create new branch
    let result = tool.execute(
        json!({
            "args": ["checkout", "-b", "feature-branch"],
            "project": "branch_repo"
        }),
        &env.config
    ).await?;
    assert_mcp_success(&McpResult::parse(result)?);
    
    // Check current branch
    let result = tool.execute(
        json!({
            "args": ["branch", "--show-current"],
            "project": "branch_repo"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    assert_eq!(parsed.stdout().unwrap().trim(), "feature-branch");
    
    // List all branches
    let result = tool.execute(
        json!({
            "args": ["branch", "-a"],
            "project": "branch_repo"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    let stdout = parsed.stdout().unwrap();
    assert!(stdout.contains("feature-branch"));
    assert!(stdout.contains("master") || stdout.contains("main"));
    
    println!("✅ Branch operations work");
    
    Ok(())
}

#[tokio::test]
async fn test_git_complex_arguments() -> Result<()> {
    // 🎯 Test complex git commands with multiple flags
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    let _project_dir = env.create_project("complex_repo").await?;
    init_git_repo(&env, "complex_repo").await?;
    
    // Test complex git log command
    let result = tool.execute(
        json!({
            "args": [
                "log",
                "--oneline",
                "--graph", 
                "--decorate",
                "--all",
                "--max-count=10",
                "--pretty=format:%h %s %an %ad",
                "--date=short"
            ],
            "project": "complex_repo"
        }),
        &env.config
    ).await?;
    
    // Should succeed even with no commits (empty output)
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    assert_eq!(parsed.exit_code(), Some(0));
    
    println!("✅ Complex argument parsing works");
    
    Ok(())
}

#[tokio::test]
async fn test_git_large_output() -> Result<()> {
    // 🎯 Test git commands that produce large output
    let env = TestEnv::new()?;
    let tool = GitTool;
    
    let project_dir = env.create_project("large_output_repo").await?;
    init_git_repo(&env, "large_output_repo").await?;
    
    // Create many files
    let many_files: Vec<_> = (0..50)
        .map(|i| (format!("file_{:02}.txt", i), format!("Content of file {}", i)))
        .collect();
    
    for (filename, content) in &many_files {
        create_test_files(&project_dir, &[(filename, content)]).await?;
    }
    
    // Test git status with many files
    let result = tool.execute(
        json!({
            "args": ["status", "--porcelain"],
            "project": "large_output_repo"
        }),
        &env.config
    ).await?;
    
    let parsed = McpResult::parse(result)?;
    assert_mcp_success(&parsed);
    
    let stdout = parsed.stdout().unwrap();
    let file_count = stdout.lines().count();
    assert_eq!(file_count, 50);
    assert!(stdout.len() > 500); // Should be substantial output
    
    println!("✅ Large output handling works ({}KB)", stdout.len() / 1024);
    
    Ok(())
}
