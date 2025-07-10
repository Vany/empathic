use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

use crate::config::Config;

pub mod env;
pub mod read_file;
pub mod write_file;
pub mod list_files;
pub mod delete_file;
pub mod replace;
pub mod mkdir;
pub mod symlink;
pub mod executor;

/// Tool trait for MCP tools 🔧
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    async fn execute(&self, args: Value, config: &Config) -> Result<Value>;
}



/// Helper function for text-based tool responses
pub fn format_text_response(text: &str) -> Value {
    serde_json::json!({
        "content": [
            {
                "type": "text", 
                "text": text
            }
        ]
    })
}

/// Helper function for structured data responses (when client expects JSON)
pub fn format_json_response<T: serde::Serialize>(data: &T) -> Result<Value> {
    Ok(serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(data)?
            }
        ]
    }))
}

/// Get all registered tools
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(env::EnvTool),
        Box::new(read_file::ReadFileTool),
        Box::new(write_file::WriteFileTool),
        Box::new(list_files::ListFilesTool),
        Box::new(delete_file::DeleteFileTool),
        Box::new(replace::ReplaceTool),
        Box::new(mkdir::MkdirTool),
        Box::new(symlink::SymlinkTool),
        Box::new(executor::GitTool),
        Box::new(executor::CargoTool),
        Box::new(executor::MakeTool),
        Box::new(executor::ShellTool),
        Box::new(executor::GradleTool),
        Box::new(executor::NpmTool),
    ]
}