//! 🔨 Make Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::error::EmpathicResult;
use super::executor_utils::{execute_command, CommandOutput};

/// 🔨 Make Tool using modern ToolBuilder pattern
pub struct MakeTool;

#[derive(Deserialize)]
pub struct MakeArgs {
    #[serde(default)]
    args: Vec<String>,
    project: Option<String>,
}

pub type MakeOutput = CommandOutput;

#[async_trait]
impl ToolBuilder for MakeTool {
    type Args = MakeArgs;
    type Output = MakeOutput;

    fn name() -> &'static str {
        "make"
    }
    
    fn description() -> &'static str {
        "🔨 Execute make commands in project directory"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .optional_array("args", "Make targets and arguments (e.g., ['build'], ['clean', 'install'])")
            .optional_string("project", "Project name for execution directory")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        execute_command("make", args.args, args.project.as_deref(), config).await
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(MakeTool);
