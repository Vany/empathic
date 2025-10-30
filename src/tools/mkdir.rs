//! 📁 Mkdir Tool - Modern ToolBuilder implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::error::{EmpathicResult, EmpathicError};

/// 📁 Create Directory Tool using modern ToolBuilder pattern
pub struct MkdirTool;

#[derive(Deserialize)]
pub struct MkdirArgs {
    path: String,
    project: Option<String>,
}

#[derive(Serialize)]
pub struct MkdirOutput {
    success: bool,
    path: String,
    working_dir: String,
}

#[async_trait]
impl ToolBuilder for MkdirTool {
    type Args = MkdirArgs;
    type Output = MkdirOutput;

    fn name() -> &'static str {
        "mkdir"
    }
    
    fn description() -> &'static str {
        "📁 Create directories with parent creation"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .required_string("path", "Directory path to create")
            .optional_string("project", "Project name for path resolution")
            .build()
    }
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        let working_dir = config.project_path(args.project.as_deref());
        let create_path = working_dir.join(&args.path);
        
        tokio::fs::create_dir_all(&create_path).await
            .map_err(|e| EmpathicError::DirectoryCreationFailed {
                path: create_path.clone(),
                reason: e.to_string(),
            })?;
        
        Ok(MkdirOutput {
            success: true,
            path: create_path.to_string_lossy().to_string(),
            working_dir: working_dir.to_string_lossy().to_string(),
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(MkdirTool);
