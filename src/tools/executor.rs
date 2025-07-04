use crate::{create_executor_tool, register_executor_tool};
use serde_json::json;

// 🗂️ =============================================================================
// 🗂️ GIT TOOL - Unified version control operations  
// 🗂️ =============================================================================

create_executor_tool!(
    GitTool,
    crate::executor::execute_git_tool,    // Specific executor function
    "git",                                // MCP tool name
    "🗂️ Execute git commands with full git syntax support",
    "🗂️",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Git command and arguments (e.g. ['status'], ['commit', '-m', 'message'], ['log', '--oneline'])"
            },
            "path": {
                "type": "string",
                "description": "Repository path (optional, defaults to project root)"
            }
        },
        "required": ["args"]
    })
);
register_executor_tool!("git", GitTool);

// 📦 =============================================================================
// 📦 CARGO TOOLS - Rust package management and build operations
// 📦 =============================================================================

create_executor_tool!(
    CargoCheckTool,
    crate::executor::execute_cargo_check_tool,  // Specific executor function
    "cargo_check",                              // MCP tool name
    "⚙️ Run cargo check on the project",
    "⚙️",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo check (optional)"
            },
            "project_dir": {
                "type": "string", 
                "description": "Project subdirectory name or full path. If just a name, resolved relative to ROOT_DIR."
            }
        }
    })
);
register_executor_tool!("cargo_check", CargoCheckTool);

create_executor_tool!(
    CargoTestTool,
    crate::executor::execute_cargo_test_tool,   // Specific executor function
    "cargo_test",
    "🧪 Run cargo test on the project",
    "🧪",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo test (optional)"
            },
            "project_dir": {
                "type": "string",
                "description": "Project subdirectory name or full path"
            }
        }
    })
);
register_executor_tool!("cargo_test", CargoTestTool);

create_executor_tool!(
    CargoBuildTool,
    crate::executor::execute_cargo_build_tool,  // Specific executor function
    "cargo_build",
    "🏗️ Run cargo build on the project",
    "🏗️",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo build (optional)"
            },
            "project_dir": {
                "type": "string",
                "description": "Project subdirectory name or full path"
            }
        }
    })
);
register_executor_tool!("cargo_build", CargoBuildTool);

create_executor_tool!(
    CargoRunTool,
    crate::executor::execute_cargo_run_tool,    // Specific executor function
    "cargo_run",
    "🚀 Run cargo run on the project",
    "🚀",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo run (optional)"
            },
            "project_dir": {
                "type": "string",
                "description": "Project subdirectory name or full path"
            }
        }
    })
);
register_executor_tool!("cargo_run", CargoRunTool);

create_executor_tool!(
    CargoCleanTool,
    crate::executor::execute_cargo_clean_tool,  // Specific executor function
    "cargo_clean",
    "🧹 Run cargo clean on the project",
    "🧹",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo clean (optional)"
            },
            "project_dir": {
                "type": "string",
                "description": "Project subdirectory name or full path"
            }
        }
    })
);
register_executor_tool!("cargo_clean", CargoCleanTool);

create_executor_tool!(
    CargoClippyTool,
    crate::executor::execute_cargo_clippy_tool, // Specific executor function
    "cargo_clippy",
    "📋 Run cargo clippy on the project",
    "📋",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo clippy (optional)"
            },
            "project_dir": {
                "type": "string",
                "description": "Project subdirectory name or full path"
            }
        }
    })
);
register_executor_tool!("cargo_clippy", CargoClippyTool);

create_executor_tool!(
    CargoFmtTool,
    crate::executor::execute_cargo_fmt_tool,    // Specific executor function
    "cargo_fmt",
    "🎨 Run cargo fmt on the project",
    "🎨",
    json!({
        "type": "object",
        "properties": {
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Additional arguments for cargo fmt (optional)"
            },
            "project_dir": {
                "type": "string",
                "description": "Project subdirectory name or full path"
            }
        }
    })
);
register_executor_tool!("cargo_fmt", CargoFmtTool);

// 🔨 =============================================================================
// 🔨 MAKE TOOL - Build automation and task execution
// 🔨 =============================================================================

create_executor_tool!(
    MakeTool,
    crate::executor::execute_make_tool,         // Specific executor function
    "make",                                     // MCP tool name
    "🔨 Execute make in specified folder with specified arguments and environment",
    "🔨",
    json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Target folder path (absolute or relative to ROOT_DIR)"
            },
            "args": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Make arguments/targets (optional)"
            },
            "env": {
                "type": "object",
                "additionalProperties": {"type": "string"},
                "description": "Environment variables to set (optional)"
            }
        }
    })
);
register_executor_tool!("make", MakeTool);

// 🔊 =============================================================================
// 🔊 SAY TOOL - Text-to-speech synthesis
// 🔊 =============================================================================

create_executor_tool!(
    SayTool,
    crate::executor::execute_say_tool,          // Specific executor function
    "say",                                      // MCP tool name
    "🔊 Cross-platform text-to-speech synthesis",
    "🔊",
    json!({
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "Text to speak"
            },
            "voice": {
                "type": "string",
                "description": "Voice to use (optional, e.g., 'Alex', 'Samantha')"
            }
        },
        "required": ["text"]
    })
);
register_executor_tool!("say", SayTool);

// 🐚 =============================================================================
// 🐚 SHELL TOOL - Command-line execution with environment control
// 🐚 =============================================================================

create_executor_tool!(
    ShellTool,
    crate::executor::execute_shell_tool,        // Specific executor function
    "shell",                                    // MCP tool name
    "🐚 Execute bash commands with environment and path control",
    "🐚",
    json!({
        "type": "object",
        "properties": {
            "command": {
                "type": "string",
                "description": "Shell command to execute"
            },
            "path": {
                "type": "string",
                "description": "Working directory (optional, defaults to ROOT_DIR)"
            },
            "env": {
                "type": "object",
                "additionalProperties": {"type": "string"},
                "description": "Environment variables to set (optional)"
            }
        },
        "required": ["command"]
    })
);
register_executor_tool!("shell", ShellTool);