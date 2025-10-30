//! 🐚 Shell Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use crate::error::EmpathicError;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Stdio;
use tokio::process::Command;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::error::EmpathicResult;

/// 🐚 Shell Tool using modern ToolBuilder pattern
pub struct ShellTool;

#[derive(Deserialize)]
pub struct ShellArgs {
    command: String,
    project: Option<String>,
}

#[derive(Serialize)]
pub struct ShellOutput {
    command: String,
    working_dir: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
    success: bool,
    path_enhanced: bool,
}

#[async_trait]
impl ToolBuilder for ShellTool {
    type Args = ShellArgs;
    type Output = ShellOutput;

    fn name() -> &'static str {
        "shell"
    }
    
    fn description() -> &'static str {
        "🐚 Execute shell commands in project directory"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_string("command", "Shell command to execute")
            .optional_string("project", "Project name for execution directory")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let working_dir = config.project_path(args.project.as_deref());
        
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
        
        // Use bash for shell command execution
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
           .arg(&args.command)
           .current_dir(&working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        
        let output = cmd.output().await
            .map_err(|e| EmpathicError::ToolExecutionFailed {
                tool_name: "shell".to_string(),
                message: format!("Failed to execute command '{}': {}", args.command, e),
            })?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        Ok(ShellOutput {
            command: args.command,
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
crate::impl_tool_for_builder!(ShellTool);
