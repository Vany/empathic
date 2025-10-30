//! 📦 NPM Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::EmpathicResult;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use super::executor_utils::{execute_command, CommandOutput};

/// 📦 NPM Tool using modern ToolBuilder pattern
pub struct NpmTool;

#[derive(Deserialize)]
pub struct NpmArgs {
    #[serde(default)]
    args: Vec<String>,
    project: Option<String>,
}

pub type NpmOutput = CommandOutput;

#[async_trait]
impl ToolBuilder for NpmTool {
    type Args = NpmArgs;
    type Output = NpmOutput;

    fn name() -> &'static str {
        "npm"
    }
    
    fn description() -> &'static str {
        "📦 Execute npm commands in project directory"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .optional_array("args", "NPM command arguments (e.g., ['install'], ['run', 'build'])")
            .optional_string("project", "Project name for execution directory")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        execute_command("npm", args.args, args.project.as_deref(), config).await
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(NpmTool);
