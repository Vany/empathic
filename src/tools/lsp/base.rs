//! 🧠 Base LSP Tool - Common functionality for all LSP tools
//!
//! Eliminates 60%+ code duplication across LSP tools by providing shared logic

use crate::config::Config;
use crate::error::{EmpathicResult, EmpathicError};
use crate::lsp::manager::LspManager;
use crate::tools::{Tool, format_json_response};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;

/// 🎯 Common input for all LSP tools
pub trait LspInput: DeserializeOwned + Send {
    fn file_path(&self) -> &str;
    fn project(&self) -> &str;
}

/// 🎯 Common output for all LSP tools  
pub trait LspOutput: Serialize {
    fn set_file_path(&mut self, path: String);
    fn set_project(&mut self, project: String);
}

/// 🧠 Base trait for LSP tools - handles all common logic
#[async_trait]
pub trait BaseLspTool: Send + Sync {
    type Input: LspInput;
    type Output: LspOutput;

    /// Tool name (e.g., "lsp_hover")
    fn name() -> &'static str where Self: Sized;
    
    /// Tool description with emoji
    fn description() -> &'static str where Self: Sized;
    
    /// Additional schema properties beyond file_path and project
    fn additional_schema() -> Value where Self: Sized {
        json!({})
    }
    
    /// Required parameters beyond file_path and project
    fn additional_required() -> Vec<&'static str> where Self: Sized {
        vec![]
    }
    
    /// Core LSP operation - only this needs to be implemented per tool
    async fn execute_lsp(
        &self,
        input: Self::Input,
        file_path: PathBuf,
        config: &Config,
    ) -> EmpathicResult<Self::Output>;
}

/// 🛠️ Auto-implement Tool trait for any BaseLspTool
#[async_trait]
impl<T: BaseLspTool + 'static> Tool for T {
    fn name(&self) -> &'static str {
        T::name()
    }

    fn description(&self) -> &'static str {
        T::description()
    }

    fn schema(&self) -> Value {
        let mut properties = json!({
            "file_path": {
                "type": "string",
                "description": "Path to the Rust file to analyze"
            },
            "project": {
                "type": "string", 
                "description": "Project name for path resolution"
            }
        });

        // Merge additional properties
        if let Some(additional_props) = T::additional_schema().as_object()
            && let Some(props) = properties.as_object_mut() {
            for (key, value) in additional_props {
                props.insert(key.clone(), value.clone());
            }
        }

        let mut required = vec!["file_path", "project"];
        required.extend(T::additional_required());

        json!({
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: Value, config: &Config) -> EmpathicResult<Value> {
        // 📥 Parse input
        let input: T::Input = serde_json::from_value(args)?;

        // 📋 Store needed values before moving input
        let file_path_str = input.file_path().to_string();
        let project_str = input.project().to_string();

        // 🎯 Resolve and validate file path
        let file_path = validate_lsp_file_path(
            &file_path_str,
            &project_str,
            config,
        )?;

        // 🚀 Execute LSP operation
        // Note: LSP server is spawned proactively in mcp/handlers.rs for ALL tools
        let mut output = self.execute_lsp(input, file_path.clone(), config).await?;

        // 📤 Set common output fields
        output.set_file_path(file_path.to_string_lossy().to_string());
        output.set_project(project_str);

        format_json_response(&output)
    }
}

/// 🔍 Validate LSP file path with all common checks
pub fn validate_lsp_file_path(
    file_path: &str,
    project: &str,
    config: &Config,
) -> EmpathicResult<PathBuf> {
    // 🎯 Resolve file path relative to project directory
    let working_dir = config.project_path(Some(project));
    let file_path = working_dir.join(file_path);

    // 🛡️ Security: Validate file is within project directory
    if !file_path.starts_with(&working_dir) {
        return Err(EmpathicError::FileAccessDenied { path: file_path.clone() });
    }

    // 📁 Validate file exists
    if !file_path.exists() {
        return Err(EmpathicError::FileNotFound { path: file_path.clone() });
    }

    // 🦀 Check if this is a Rust file
    if let Some(extension) = file_path.extension() {
        if extension != "rs" {
            return Err(EmpathicError::tool_failed("lsp_tools", "Only supports Rust files (.rs)"));
        }
    } else {
        return Err(EmpathicError::tool_failed("lsp_tools", "File has no extension, expecting .rs file"));
    }

    Ok(file_path)
}

/// 🔧 Get LSP manager from config with helpful error message
pub fn get_lsp_manager(config: &Config) -> EmpathicResult<&std::sync::Arc<LspManager>> {
    config.lsp_manager()
        .ok_or_else(|| EmpathicError::tool_failed("lsp_manager", "LSP manager not available"))
}

/// 🎯 Position helper for tools that need line/character
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// 🎯 Range helper for LSP responses
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RangeInfo {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

impl RangeInfo {
    pub fn from_lsp_range(range: &lsp_types::Range) -> Self {
        Self {
            start_line: range.start.line,
            start_character: range.start.character,
            end_line: range.end.line,
            end_character: range.end.character,
        }
    }
}

/// 🧪 Tests
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    struct TestInput {
        file_path: String,
        project: String,
        test_param: Option<String>,
    }

    impl LspInput for TestInput {
        fn file_path(&self) -> &str {
            &self.file_path
        }

        fn project(&self) -> &str {
            &self.project
        }
    }

    #[derive(Serialize)]
    struct TestOutput {
        file_path: String,
        project: String,
        result: String,
    }

    impl LspOutput for TestOutput {
        fn set_file_path(&mut self, path: String) {
            self.file_path = path;
        }

        fn set_project(&mut self, project: String) {
            self.project = project;
        }
    }

    struct TestLspTool;

    #[async_trait]
    impl BaseLspTool for TestLspTool {
        type Input = TestInput;
        type Output = TestOutput;

        fn name() -> &'static str {
            "test_lsp"
        }

        fn description() -> &'static str {
            "🧪 Test LSP tool"
        }

        fn additional_schema() -> Value {
            json!({
                "test_param": {
                    "type": "string",
                    "description": "Test parameter"
                }
            })
        }

        async fn execute_lsp(
            &self,
            input: Self::Input,
            _file_path: PathBuf,
            _config: &Config,
        ) -> EmpathicResult<Self::Output> {
            Ok(TestOutput {
                file_path: String::new(), // Will be set by base
                project: String::new(),   // Will be set by base
                result: input.test_param.unwrap_or_else(|| "default".to_string()),
            })
        }
    }

    #[test]
    fn test_schema_generation() {
        let tool = TestLspTool;
        let schema = tool.schema();

        assert_eq!(tool.name(), "test_lsp");
        assert!(tool.description().contains("Test LSP"));

        // Should have base properties
        assert!(schema["properties"]["file_path"].is_object());
        assert!(schema["properties"]["project"].is_object());
        
        // Should have additional properties
        assert!(schema["properties"]["test_param"].is_object());
        
        // Required should include base fields
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("file_path")));
        assert!(required.contains(&json!("project")));
    }
}