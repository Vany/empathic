use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;
use std::env;

use crate::config::Config;
use crate::tools::Tool;

pub struct EnvTool;

#[async_trait]
impl Tool for EnvTool {
    fn name(&self) -> &'static str {
        "env"
    }
    
    fn description(&self) -> &'static str {
        "🌍 Get full environment variables"
    }
    
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }
    
    async fn execute(&self, _args: Value, config: &Config) -> Result<Value> {
        // Get all environment variables
        let mut env_vars: std::collections::HashMap<String, String> = env::vars().collect();
        
        // Add configured paths to PATH
        if !config.add_path.is_empty() {
            let current_path = env::var("PATH").unwrap_or_default();
            let additional_paths: Vec<String> = config.add_path
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            
            let new_path = format!("{}:{}", additional_paths.join(":"), current_path);
            env_vars.insert("PATH".to_string(), new_path);
        }

        // Add ROOT_DIR to the environment variables for clarity
        env_vars.insert("ROOT_DIR".to_string(), config.root_dir.to_string_lossy().to_string());

        // Format as properly structured JSON with correct MCP content type
        crate::tools::format_json_response(&env_vars)
    }
}
