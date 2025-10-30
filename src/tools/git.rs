//! 🐙 Git Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use crate::error::EmpathicResult;
use serde::Deserialize;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use super::executor_utils::{execute_command, CommandOutput};

/// 🐙 Git Tool using modern ToolBuilder pattern
pub struct GitTool;

#[derive(Deserialize)]
pub struct GitArgs {
    args: Vec<String>,
    project: Option<String>,
}

pub type GitOutput = CommandOutput;

#[async_trait]
impl ToolBuilder for GitTool {
    type Args = GitArgs;
    type Output = GitOutput;

    fn name() -> &'static str {
        "git"
    }
    
    fn description() -> &'static str {
        "🐙 Execute git commands in project directory"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_array("args", "Git command arguments (e.g., ['status'], ['commit', '-m', 'message'])")
            .optional_string("project", "Project name for execution directory")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        execute_command("git", args.args, args.project.as_deref(), config).await
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(GitTool);
