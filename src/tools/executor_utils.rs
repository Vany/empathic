//! 🔧 Executor Utilities - Shared command execution logic

use serde::Serialize;
use std::env;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::Config;
use crate::error::{EmpathicResult, EmpathicError};

#[derive(Serialize)]
pub struct CommandOutput {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub path_enhanced: bool,
}

/// Generic command execution helper 🔧
/// 
/// ✅ FIXED: Always returns CommandOutput, never errors on non-zero exit codes
/// Non-zero exit codes are common and legitimate (git status, failed tests, etc.)
pub async fn execute_command(
    command: &str, 
    args: Vec<String>, 
    project: Option<&str>, 
    config: &Config
) -> EmpathicResult<CommandOutput> {
    let working_dir = config.project_path(project);
    
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
    
    let mut cmd = Command::new(command);
    cmd.args(&args)
       .current_dir(&working_dir)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    
    let output = cmd.output().await
        .map_err(|_e| EmpathicError::CommandNotFound { command: command.to_string() })?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();
    
    // ✅ ALWAYS return the output - don't error on non-zero exit codes!
    // Commands like `git status`, `cargo test`, `make` often return non-zero legitimately
    Ok(CommandOutput {
        command: command.to_string(),
        args,
        working_dir: working_dir.to_string_lossy().to_string(),
        exit_code,
        stdout: stdout.trim_end().to_string(),
        stderr: stderr.trim_end().to_string(),
        success,
        path_enhanced,
    })
}
