use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::{Result, Context};
use std::process::Stdio;
use tokio::process::Command;
use std::env;

use crate::config::Config;
use crate::tools::Tool;

/// 🏃‍♂️ Git tool for executing git commands in project directory
pub struct GitTool;

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &'static str {
        "git"
    }
    
    fn description(&self) -> &'static str {
        "🐙 Execute git commands in project directory"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Git command arguments (e.g., ['status'], ['commit', '-m', 'message'])"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for execution directory"
                }
            },
            "required": ["args"]
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        execute_command("git", args, config).await
    }
}

/// 📦 Cargo tool for executing cargo commands in project directory
pub struct CargoTool;

#[async_trait]
impl Tool for CargoTool {
    fn name(&self) -> &'static str {
        "cargo"
    }
    
    fn description(&self) -> &'static str {
        "🦀 Execute cargo commands in project directory"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Cargo command arguments (e.g., ['build'], ['test', '--release'])"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for execution directory"
                }
            },
            "required": ["args"]
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        execute_command("cargo", args, config).await
    }
}

/// 🔨 Make tool for executing make commands in project directory
pub struct MakeTool;

#[async_trait]
impl Tool for MakeTool {
    fn name(&self) -> &'static str {
        "make"
    }
    
    fn description(&self) -> &'static str {
        "🔨 Execute make commands in project directory"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Make targets and arguments (e.g., ['build'], ['clean', 'install'])"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for execution directory"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        execute_command("make", args, config).await
    }
}

/// 🐚 Shell tool for executing shell commands in project directory
pub struct ShellTool;

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &'static str {
        "shell"
    }
    
    fn description(&self) -> &'static str {
        "🐚 Execute shell commands in project directory"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for execution directory"
                }
            },
            "required": ["command"]
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        let command_str = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("command is required"))?;
        
        let project = args.get("project").and_then(|v| v.as_str());
        let working_dir = config.project_path(project);
        
        // Prepare environment with additional paths
        let mut env_vars = std::collections::HashMap::new();
        if !config.add_path.is_empty() {
            let current_path = env::var("PATH").unwrap_or_default();
            let additional_paths: Vec<String> = config.add_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            let new_path = format!("{}:{}", additional_paths.join(":"), current_path);
            env_vars.insert("PATH".to_string(), new_path);
        }
        
        // Use bash for shell command execution
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
           .arg(command_str)
           .current_dir(&working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        
        let output = cmd.output().await
            .with_context(|| format!("Failed to execute shell command: {command_str}"))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let response = json!({
            "command": command_str,
            "working_dir": working_dir.to_string_lossy(),
            "exit_code": output.status.code().unwrap_or(-1),
            "stdout": stdout.trim_end(),
            "stderr": stderr.trim_end(),
            "success": output.status.success()
        });
        
        Ok(crate::tools::format_json_response(&response)?)
    }
}

/// 🐘 Gradle tool for executing gradle commands in project directory
pub struct GradleTool;

#[async_trait]
impl Tool for GradleTool {
    fn name(&self) -> &'static str {
        "gradle"
    }
    
    fn description(&self) -> &'static str {
        "🐘 Execute gradle commands in project directory"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Gradle task arguments (e.g., ['build'], ['clean', 'test'])"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for execution directory"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        execute_command("gradle", args, config).await
    }
}

/// 📦 NPM tool for executing npm commands in project directory
pub struct NpmTool;

#[async_trait]
impl Tool for NpmTool {
    fn name(&self) -> &'static str {
        "npm"
    }
    
    fn description(&self) -> &'static str {
        "📦 Execute npm commands in project directory"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "NPM command arguments (e.g., ['install'], ['run', 'build'])"
                },
                "project": {
                    "type": "string",
                    "description": "Project name for execution directory"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, args: Value, config: &Config) -> Result<Value> {
        execute_command("npm", args, config).await
    }
}

/// Generic command execution helper 🔧
async fn execute_command(command: &str, args: Value, config: &Config) -> Result<Value> {
    let command_args = args.get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    
    let project = args.get("project").and_then(|v| v.as_str());
    let working_dir = config.project_path(project);
    
    // Prepare environment with additional paths
    let mut env_vars = std::collections::HashMap::new();
    if !config.add_path.is_empty() {
        let current_path = env::var("PATH").unwrap_or_default();
        let additional_paths: Vec<String> = config.add_path
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        let new_path = format!("{}:{}", additional_paths.join(":"), current_path);
        env_vars.insert("PATH".to_string(), new_path);
    }
    
    let mut cmd = Command::new(command);
    cmd.args(&command_args)
       .current_dir(&working_dir)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    
    let output = cmd.output().await
        .with_context(|| format!("Failed to execute {command} command"))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let response = json!({
        "command": command,
        "args": command_args,
        "working_dir": working_dir.to_string_lossy(),
        "exit_code": output.status.code().unwrap_or(-1),
        "stdout": stdout.trim_end(),
        "stderr": stderr.trim_end(),
        "success": output.status.success()
    });
    
    crate::tools::format_json_response(&response)
}
