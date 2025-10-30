//! 🌍 Environment tool tests - Variable access and filtering

mod common;

use anyhow::Result;
use common::*;
use empathic::tools::{Tool, env::EnvTool};
use serde_json::json;

#[tokio::test]
async fn test_env_basic_functionality() -> Result<()> {
    // 🎯 Test basic environment variable access
    let env = TestEnv::new()?;
    let tool = EnvTool;
    
    // Test getting all environment variables
    let result = tool.execute(json!({}), &env.config).await?;
    
    // For env tool, the response is JSON as a string in MCP format
    let content_text = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    let env_vars: serde_json::Value = serde_json::from_str(content_text)?;
    let response = env_vars.as_object().unwrap();
    let variables = response.get("env_vars").unwrap().as_object().unwrap();
    
    // Should contain common environment variables
    assert!(variables.contains_key("PATH") || variables.contains_key("Path"));
    
    // Should have multiple variables
    assert!(variables.len() > 5);
    
    // Should include our ROOT_DIR
    assert!(variables.contains_key("ROOT_DIR"));
    
    // Check response structure
    assert!(response.contains_key("count"));
    assert!(response.contains_key("path_enhanced"));
    assert!(response.contains_key("root_dir_injected"));
    
    println!("✅ Environment variable access works ({} vars)", variables.len());
    
    Ok(())
}

#[tokio::test]
async fn test_env_no_project_context() -> Result<()> {
    // 🎯 Test env tool works without project context
    let env = TestEnv::new()?;
    let tool = EnvTool;
    
    // Test with empty args
    let result = tool.execute(json!({}), &env.config).await?;
    
    let content_text = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    let env_vars: serde_json::Value = serde_json::from_str(content_text)?;
    let response = env_vars.as_object().unwrap();
    let variables = response.get("env_vars").unwrap().as_object().unwrap();
    
    // Should still work even without project-specific context
    assert!(!variables.is_empty());
    
    println!("✅ Environment tool works without project context");
    
    Ok(())
}

#[tokio::test] 
async fn test_env_common_variables() -> Result<()> {
    // 🎯 Test that common environment variables are accessible
    let env = TestEnv::new()?;
    let tool = EnvTool;
    
    let result = tool.execute(json!({}), &env.config).await?;
    
    let content_text = result
        .get("content").unwrap().as_array().unwrap()[0]
        .get("text").unwrap().as_str().unwrap();
    
    let env_vars: serde_json::Value = serde_json::from_str(content_text)?;
    let response = env_vars.as_object().unwrap();
    let variables = response.get("env_vars").unwrap().as_object().unwrap();
    
    // Common variables that should exist on most systems
    let common_vars = [
        "PATH", "Path",  // PATH (case varies by OS)
        "HOME", "USERPROFILE",  // Home directory (varies by OS)
        "USER", "USERNAME",  // Username (varies by OS)
    ];
    
    let mut found_vars = Vec::new();
    for var in &common_vars {
        if variables.contains_key(*var) {
            found_vars.push(*var);
            let value = variables[*var].as_str().unwrap_or("");
            assert!(!value.is_empty(), "Variable {} should not be empty", var);
        }
    }
    
    // Should find at least some common variables
    assert!(!found_vars.is_empty(), "Should find at least one common environment variable");
    
    println!("✅ Found common environment variables: {:?}", found_vars);
    
    Ok(())
}

#[tokio::test]
async fn test_env_response_structure() -> Result<()> {
    // 🎯 Test the structure of environment response
    let env = TestEnv::new()?;
    let tool = EnvTool;
    
    let result = tool.execute(json!({}), &env.config).await?;
    
    // Check MCP response structure
    assert!(result.get("content").is_some());
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);
    assert_eq!(content[0]["type"].as_str().unwrap(), "text");
    
    let content_text = content[0]["text"].as_str().unwrap();
    let env_vars: serde_json::Value = serde_json::from_str(content_text)?;
    let response = env_vars.as_object().unwrap();
    let variables = response.get("env_vars").unwrap().as_object().unwrap();
    
    // All values should be strings
    for (key, value) in variables {
        assert!(value.is_string(), "Variable {} should be a string", key);
        assert!(!key.is_empty(), "Variable name should not be empty");
    }
    
    println!("✅ Environment response structure is correct");
    
    Ok(())
}
