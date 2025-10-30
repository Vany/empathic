use async_trait::async_trait;
use serde_json::Value;

use crate::config::Config;
use crate::error::EmpathicResult;

pub mod tool_base;
pub mod env;
pub mod read_file;
pub mod write_file;
pub mod list_files;
pub mod delete_file;
pub mod replace;
pub mod str_replace;
pub mod mkdir;
pub mod symlink;
pub mod executor_utils;
pub mod shell;
pub mod bash_tool;
pub mod git;
pub mod cargo;
pub mod make;
pub mod gradle;
pub mod npm;
pub mod lsp;

/// Tool trait for MCP tools 🔧
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    async fn execute(&self, args: Value, config: &Config) -> EmpathicResult<Value>;
}

// Re-export tool base functionality
pub use tool_base::{
    ToolBuilder, SchemaBuilder,
    require_string, optional_string, optional_int, bool_param_or,
    default_fs_path, resolve_file_path, validate_file_exists, validate_dir_exists, validate_file_extension,
    format_text_response, format_json_response
};

/// Get all registered tools
pub fn get_all_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(env::EnvTool),
        Box::new(read_file::ReadFileTool),
        Box::new(write_file::WriteFileTool),
        Box::new(list_files::ListFilesTool),
        Box::new(delete_file::DeleteFileTool),
        Box::new(replace::ReplaceTool),
        Box::new(str_replace::StrReplaceTool),
        Box::new(mkdir::MkdirTool),
        Box::new(symlink::SymlinkTool),
        Box::new(shell::ShellTool),
        Box::new(bash_tool::BashTool),
        Box::new(git::GitTool),
        Box::new(cargo::CargoTool),
        Box::new(make::MakeTool),
        Box::new(gradle::GradleTool),
        Box::new(npm::NpmTool),
        // 🧠 LSP Tools
        Box::new(lsp::LspDiagnosticsTool),
        Box::new(lsp::LspHoverTool),
        Box::new(lsp::LspCompletionTool),
        Box::new(lsp::LspGotoDefinitionTool),
        Box::new(lsp::LspFindReferencesTool),
        Box::new(lsp::LspDocumentSymbolsTool),
        Box::new(lsp::LspWorkspaceSymbolsTool),
    ]
}
