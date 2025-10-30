//! 🌍 Environment Variables Tool - Clean ToolBuilder implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

use crate::tools::{ToolBuilder, SchemaBuilder};
use crate::config::Config;
use crate::error::EmpathicResult;

/// 🌍 Environment Variables Tool using modern ToolBuilder pattern
pub struct EnvTool;

#[derive(Deserialize)]
pub struct EnvArgs {
    // No arguments needed for env tool
}

#[derive(Serialize)]
pub struct EnvOutput {
    /// All environment variables including PATH enhancements and ROOT_DIR
    env_vars: HashMap<String, String>,
    /// Number of environment variables returned
    count: usize,
    /// Whether PATH was enhanced with additional paths
    path_enhanced: bool,
    /// Whether ROOT_DIR was injected
    root_dir_injected: bool,
}

#[async_trait]
impl ToolBuilder for EnvTool {
    type Args = EnvArgs;
    type Output = EnvOutput;

    fn name() -> &'static str {
        "env"
    }
    
    fn description() -> &'static str {
        "🌍 Get full environment variables"
    }
    
    fn schema() -> serde_json::Value {
        SchemaBuilder::new()
            .build()
    }
    
    async fn run(_args: Self::Args, config: &Config) -> EmpathicResult<Self::Output> {
        // Get all environment variables
        let mut env_vars: HashMap<String, String> = env::vars().collect();
        let _original_count = env_vars.len();
        
        // Add configured paths to PATH
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

        // Add ROOT_DIR to the environment variables for clarity
        let root_dir_injected = !env_vars.contains_key("ROOT_DIR");
        env_vars.insert("ROOT_DIR".to_string(), config.root_dir.to_string_lossy().to_string());
        
        Ok(EnvOutput {
            count: env_vars.len(),
            path_enhanced,
            root_dir_injected,
            env_vars,
        })
    }
}

// 🔧 Implement Tool trait using the builder pattern
crate::impl_tool_for_builder!(EnvTool);
