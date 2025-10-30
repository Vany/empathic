//! 🦀 Cargo Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use crate::error::EmpathicResult;
use serde::Deserialize;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use super::executor_utils::{execute_command, CommandOutput};

/// 🦀 Cargo Tool using modern ToolBuilder pattern
pub struct CargoTool;

#[derive(Deserialize)]
pub struct CargoArgs {
    args: Vec<String>,
    project: Option<String>,
}

pub type CargoOutput = CommandOutput;

#[async_trait]
impl ToolBuilder for CargoTool {
    type Args = CargoArgs;
    type Output = CargoOutput;

    fn name() -> &'static str {
        "cargo"
    }
    
    fn description() -> &'static str {
        "🦀 Execute cargo commands in project directory"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_array("args", "Cargo command arguments (e.g., ['build'], ['test', '--release'])")
            .optional_string("project", "Project name for execution directory")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        execute_command("cargo", args.args, args.project.as_deref(), config).await
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(CargoTool);
