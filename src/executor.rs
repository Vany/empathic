use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::resources::{get_allowed_dir, get_base_dir, validate_path};

/// 🎯 Unified execution arguments for all tools
#[derive(Deserialize, Default, Clone)]
pub struct ExecutorArgs {
    // 📍 Common path/directory arguments
    pub path: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub project_dir: Option<String>,
    
    // 🔊 Say-specific arguments
    pub text: Option<String>,
    pub voice: Option<String>,
    
    // 🐚 Shell-specific arguments  
    pub command: Option<String>,
}

/// 🚀 Command execution result
#[derive(Debug, Clone)]
pub struct ExecutorResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
}

/// 🔧 **CORE SYSTEM EXECUTOR** - Execute any command with system process management
pub fn execute_system_command(
    command_name: &str,
    command_args: &[String],
    working_dir: &str,
    env_vars: Option<&HashMap<String, String>>,
) -> Result<ExecutorResult, String> {
    
    if !Path::new(working_dir).exists() {
        return Err(format!("🚫 Working directory does not exist: {working_dir}"));
    }
    
    let mut cmd = Command::new(command_name);
    cmd.current_dir(working_dir);
    
    // 📝 Add command arguments
    if !command_args.is_empty() {
        cmd.args(command_args);
    }
    
    // 🌍 Set environment variables if provided
    if let Some(env) = env_vars {
        for (key, value) in env {
            cmd.env(key, value);
        }
    }
    
    let output = cmd.output()
        .map_err(|e| format!("🚫 Failed to execute '{command_name}': {e}"))?;
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.is_empty() { 
        stdout.to_string() 
    } else { 
        format!("{stdout}\n{stderr}") 
    };
    
    Ok(ExecutorResult {
        success: output.status.success(),
        output: combined,
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// 📂 **PATH VALIDATION** - Resolve and validate project paths safely
pub fn resolve_project_path(base_dir: &str, project_dir: Option<&str>) -> String {
    match project_dir {
        Some(dir) if dir.starts_with('/') => dir.to_string(),
        Some(dir) => format!("{base_dir}/{dir}"),
        None => base_dir.to_string(),
    }
}

/// 🛡️ **GIT PATH VALIDATOR** - Validate git repository paths
pub fn validate_git_path(path: Option<&str>) -> Result<String, String> {
    let repo_path = path.map(|p| p.to_string()).unwrap_or_else(get_allowed_dir);
    validate_path(&repo_path)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("🚫 Invalid git repository path: {e}"))
}

/// 📦 **CARGO PROJECT VALIDATOR** - Validate cargo project directories
pub fn validate_cargo_project(project_dir: Option<&str>) -> Result<String, String> {
    let base_dir = get_base_dir();
    let project_path = resolve_project_path(&base_dir, project_dir);
    
    if !Path::new(&project_path).exists() {
        return Err(format!("🚫 Project directory does not exist: {project_path}"));
    }
    
    Ok(project_path)
}

/// 🔍 **CARGO.TOML VALIDATOR** - Ensure Cargo.toml exists for operations that need it
pub fn validate_cargo_toml(project_path: &str, operation: &str) -> Result<(), String> {
    // Skip validation for 'clean' operation
    if operation == "clean" {
        return Ok(());
    }
    
    if !Path::new(&format!("{project_path}/Cargo.toml")).exists() {
        return Err(format!("🚫 No Cargo.toml found in: {project_path}"));
    }
    
    Ok(())
}

/// 🔊 **TTS COMMAND RESOLVER** - Get platform-specific TTS command
pub fn get_tts_command() -> &'static str {
    if cfg!(target_os = "macos") { "say" } else { "espeak" }
}

/// 🐚 **SHELL COMMAND RESOLVER** - Get platform-specific shell command  
pub fn get_shell_command() -> &'static str {
    if cfg!(windows) { "cmd" } else { "bash" }
}

/// 🐚 **SHELL ARGS BUILDER** - Build platform-specific shell arguments
pub fn build_shell_args(command: &str) -> Vec<String> {
    if cfg!(windows) {
        vec!["/C".to_string(), command.to_string()]
    } else {
        vec!["-c".to_string(), command.to_string()]
    }
}

// 🎯 =============================================================================
// 🎯 SPECIFIC EXECUTOR FUNCTIONS - Used by macro invocations
// 🎯 =============================================================================

/// 🗂️ **GIT EXECUTOR** - Handle git command execution
pub fn execute_git_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    let repo_path = validate_git_path(args.path.as_deref())?;
    
    let git_args = args.args.unwrap_or_default();
    
    execute_system_command("git", &git_args, &repo_path, args.env.as_ref())
}

/// 📦 **CARGO EXECUTOR** - Handle cargo command execution with subcommand
pub fn execute_cargo_tool(subcmd: &str, args: ExecutorArgs) -> Result<ExecutorResult, String> {
    let project_path = validate_cargo_project(args.project_dir.as_deref())?;
    validate_cargo_toml(&project_path, subcmd)?;
    
    let mut cargo_args = vec![subcmd.to_string()];
    if let Some(ref extra_args) = args.args {
        cargo_args.extend_from_slice(extra_args);
    }
    
    execute_system_command("cargo", &cargo_args, &project_path, args.env.as_ref())
}

/// 📦 **CARGO SPECIFIC TOOLS** - Individual wrapper functions for each subcommand
pub fn execute_cargo_check_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("check", args)
}

pub fn execute_cargo_test_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("test", args)
}

pub fn execute_cargo_build_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("build", args)
}

pub fn execute_cargo_run_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("run", args)
}

pub fn execute_cargo_clean_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("clean", args)
}

pub fn execute_cargo_clippy_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("clippy", args)
}

pub fn execute_cargo_fmt_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    execute_cargo_tool("fmt", args)
}

/// 🔨 **MAKE EXECUTOR** - Handle make command execution
pub fn execute_make_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    use crate::resources::get_base_dir;
    
    let base_dir = get_base_dir();
    let target_dir = resolve_project_path(&base_dir, args.path.as_deref());
    
    let make_args = args.args.unwrap_or_default();
    
    execute_system_command("make", &make_args, &target_dir, args.env.as_ref())
}

/// 🔊 **SAY EXECUTOR** - Handle TTS command execution
pub fn execute_say_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    use crate::resources::get_base_dir;
    
    let text = args.text.ok_or("🚫 Text to speak is required")?;
    
    let tts_cmd = get_tts_command();
    let base_dir = get_base_dir();
    
    let mut tts_args = Vec::new();
    
    if cfg!(target_os = "macos") {
        if let Some(voice) = args.voice {
            tts_args.push("-v".to_string());
            tts_args.push(voice);
        }
        tts_args.push(text.clone());
    } else {
        tts_args.push(text.clone());
    }
    
    let mut result = execute_system_command(tts_cmd, &tts_args, &base_dir, args.env.as_ref())?;
    // Override output for TTS
    result.output = format!("🔊 Spoke: \"{text}\"");
    Ok(result)
}

/// 🐚 **SHELL EXECUTOR** - Handle shell command execution
pub fn execute_shell_tool(args: ExecutorArgs) -> Result<ExecutorResult, String> {
    use crate::resources::get_base_dir;
    
    let command = args.command.ok_or("🚫 Shell command is required")?;
    
    let base_dir = get_base_dir();
    let work_dir = resolve_project_path(&base_dir, args.path.as_deref());
    
    let shell_cmd = get_shell_command();
    let shell_args = build_shell_args(&command);
    
    execute_system_command(shell_cmd, &shell_args, &work_dir, args.env.as_ref())
}