//! 🐚 Bash Tool - Claude Desktop's expected bash_tool interface
//!
//! This tool provides the `bash_tool` interface that Claude Desktop naturally expects,
//! with `command` and `description` parameters for better context and logging.

use async_trait::async_trait;
use crate::error::EmpathicError;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Stdio;
use tokio::process::Command;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::error::EmpathicResult;

/// 🐚 Bash Tool - Expected interface for Claude Desktop
pub struct BashTool;

#[derive(Deserialize)]
pub struct BashArgs {
    /// The bash command to execute
    command: String,
    /// Description/reasoning for why this command is being run (for context and logging)
    description: String,
}

#[derive(Serialize)]
pub struct BashOutput {
    command: String,
    description: String,
    working_dir: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
    success: bool,
    path_enhanced: bool,
}

#[async_trait]
impl ToolBuilder for BashTool {
    type Args = BashArgs;
    type Output = BashOutput;

    fn name() -> &'static str {
        "bash_tool"
    }
    
    fn description() -> &'static str {
        "🐚 Execute bash commands with context description for logging and debugging"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_string("command", "Bash command to execute")
            .required_string("description", "Why I'm running this command (for context and debugging)")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        // Always use ROOT_DIR as working directory (simpler interface, no project parameter)
        let working_dir = &config.root_dir;
        
        // Log the description for better debugging context
        log::info!("🐚 bash_tool: {} - Running: {}", args.description, args.command);
        
        // Prepare environment with additional paths
        let mut env_vars = std::collections::HashMap::new();
        let path_enhanced = if !config.add_path.is_empty() {
            let current_path = env::var("PATH").unwrap_or_default();
            let additional_paths: Vec<String> = config.add_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let new_path = format!("{}:{}", additional_paths.join(":"), current_path);
            env_vars.insert("PATH".to_string(), new_path);
            true
        } else {
            false
        };
        
        // Use bash for command execution
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
           .arg(&args.command)
           .current_dir(working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        
        let output = cmd.output().await
            .map_err(|e| EmpathicError::ToolExecutionFailed {
                tool_name: "bash_tool".to_string(),
                message: format!("Failed to execute command '{}' ({}): {}", args.command, args.description, e),
            })?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        Ok(BashOutput {
            command: args.command,
            description: args.description,
            working_dir: working_dir.to_string_lossy().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: stdout.trim_end().to_string(),
            stderr: stderr.trim_end().to_string(),
            success: output.status.success(),
            path_enhanced,
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(BashTool);
