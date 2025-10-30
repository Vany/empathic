//! 🔧 Tool Base - Common functionality for all MCP tools
//! 
//! Provides base traits and builders to eliminate duplication across tools

use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{EmpathicResult, EmpathicError};

/// 🏗️ Tool Builder trait - simplifies tool creation
#[async_trait]
pub trait ToolBuilder: Send + Sync {
    type Args: for<'de> serde::Deserialize<'de> + Send;
    type Output: serde::Serialize + Send;

    fn name() -> &'static str;
    fn description() -> &'static str;
    fn schema() -> Value;
    
    async fn run(args: Self::Args, config: &Config) -> EmpathicResult<Self::Output>;
}

/// 📋 Auto-implement Tool for any ToolBuilder
/// 
/// Note: This is a helper macro, not a blanket impl to avoid conflicts with LSP tools
#[macro_export]
macro_rules! impl_tool_for_builder {
    ($tool_type:ty) => {
        #[async_trait::async_trait]
        impl $crate::tools::Tool for $tool_type {
            fn name(&self) -> &'static str {
                <$tool_type as $crate::tools::ToolBuilder>::name()
            }
            
            fn description(&self) -> &'static str {
                <$tool_type as $crate::tools::ToolBuilder>::description()
            }
            
            fn schema(&self) -> serde_json::Value {
                <$tool_type as $crate::tools::ToolBuilder>::schema()
            }
            
            async fn execute(&self, args: serde_json::Value, config: &$crate::config::Config) -> $crate::error::EmpathicResult<serde_json::Value> {
                let parsed_args = serde_json::from_value(args)
                    .map_err(|e| $crate::error::EmpathicError::JsonProcessing { source: e })?;
                
                let output = <$tool_type as $crate::tools::ToolBuilder>::run(parsed_args, config).await?;
                $crate::tools::format_json_response(&output)
            }
        }
    };
}

/// 🔧 Helper functions for common tool patterns
/// Extract required string parameter with context
pub fn require_string(args: &Value, param: &str) -> EmpathicResult<String> {
    args.get(param)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| EmpathicError::McpParameterMissing { parameter: param.to_string() })
}

/// Extract optional string parameter
pub fn optional_string(args: &Value, param: &str) -> Option<String> {
    args.get(param).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Extract optional integer parameter
pub fn optional_int(args: &Value, param: &str) -> Option<i64> {
    args.get(param).and_then(|v| v.as_i64())
}

/// Extract optional bool parameter with default
pub fn bool_param_or(args: &Value, param: &str, default: bool) -> bool {
    args.get(param).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// 📝 Standard MCP text response format
pub fn format_text_response(text: &str) -> Value {
    json!({
        "content": [
            {
                "type": "text", 
                "text": text
            }
        ]
    })
}

/// 📊 Standard MCP JSON response format
pub fn format_json_response<T: serde::Serialize>(data: &T) -> EmpathicResult<Value> {
    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(data)?
            }
        ]
    }))
}

/// 🎯 Path validation and resolution helpers
/// Get default path for file operations when project is set but path is not provided
/// Returns "." (project root) if project is set and path is None, otherwise returns the provided path
pub fn default_fs_path(provided_path: Option<String>, project: Option<&str>) -> String {
    match (provided_path, project) {
        (Some(path), _) => path,
        (None, Some(_)) => ".".to_string(), // Default to project root when project is set
        (None, None) => ".".to_string(),     // Default to current directory
    }
}

/// Resolve file path relative to project or root directory
pub fn resolve_file_path(
    file_path: &str, 
    project: Option<&str>, 
    config: &Config
) -> EmpathicResult<PathBuf> {
    let working_dir = match project {
        Some(proj) => config.project_path(Some(proj)),
        None => config.root_dir.clone(),
    };
    
    let resolved_path = working_dir.join(file_path);
    
    // Validate path is within working directory (security check)
    if !resolved_path.starts_with(&working_dir) {
        return Err(EmpathicError::InvalidPath { path: resolved_path });
    }
    
    Ok(resolved_path)
}

/// Validate file exists and return canonical path
pub fn validate_file_exists(path: &Path) -> EmpathicResult<PathBuf> {
    if !path.exists() {
        return Err(EmpathicError::FileNotFound { path: path.to_path_buf() });
    }
    
    if !path.is_file() {
        return Err(EmpathicError::InvalidPath { path: path.to_path_buf() });
    }
    
    Ok(path.to_path_buf())
}

/// Validate directory exists and return canonical path
pub fn validate_dir_exists(path: &Path) -> EmpathicResult<PathBuf> {
    if !path.exists() {
        return Err(EmpathicError::FileNotFound { path: path.to_path_buf() });
    }
    
    if !path.is_dir() {
        return Err(EmpathicError::InvalidPath { path: path.to_path_buf() });
    }
    
    Ok(path.to_path_buf())
}

/// Check if file has expected extension
pub fn validate_file_extension(path: &Path, expected: &str) -> EmpathicResult<()> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext == expected => Ok(()),
        Some(ext) => Err(EmpathicError::FileOperationFailed {
            operation: "validate extension".to_string(),
            path: path.to_path_buf(),
            reason: format!("Expected .{} file, got .{} file", expected, ext),
        }),
        None => Err(EmpathicError::FileOperationFailed {
            operation: "validate extension".to_string(),
            path: path.to_path_buf(),
            reason: format!("File has no extension, expecting .{} file", expected),
        }),
    }
}

/// 🔧 Schema builder for common parameter patterns
pub struct SchemaBuilder {
    required: Vec<&'static str>,
    properties: serde_json::Map<String, Value>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            properties: serde_json::Map::new(),
        }
    }
    
    pub fn required_string(mut self, name: &'static str, desc: &str) -> Self {
        self.required.push(name);
        self.properties.insert(name.to_string(), json!({
            "type": "string",
            "description": desc
        }));
        self
    }
    
    pub fn optional_string(mut self, name: &'static str, desc: &str) -> Self {
        self.properties.insert(name.to_string(), json!({
            "type": "string", 
            "description": desc
        }));
        self
    }
    
    pub fn required_array(mut self, name: &'static str, desc: &str) -> Self {
        self.required.push(name);
        self.properties.insert(name.to_string(), json!({
            "type": "array",
            "items": {"type": "string"},
            "description": desc
        }));
        self
    }
    
    pub fn optional_array(mut self, name: &'static str, desc: &str) -> Self {
        self.properties.insert(name.to_string(), json!({
            "type": "array",
            "items": {"type": "string"},
            "description": desc
        }));
        self
    }
    
    pub fn optional_integer(mut self, name: &'static str, desc: &str, minimum: Option<i64>) -> Self {
        let mut prop = json!({
            "type": "integer",
            "description": desc
        });
        
        if let Some(min) = minimum {
            prop["minimum"] = json!(min);
        }
        
        self.properties.insert(name.to_string(), prop);
        self
    }
    
    pub fn optional_bool(mut self, name: &'static str, desc: &str, default: Option<bool>) -> Self {
        let mut prop = json!({
            "type": "boolean",
            "description": desc
        });
        
        if let Some(def) = default {
            prop["default"] = json!(def);
        }
        
        self.properties.insert(name.to_string(), prop);
        self
    }
    
    pub fn build(self) -> Value {
        json!({
            "type": "object",
            "required": self.required,
            "properties": self.properties,
            "additionalProperties": false
        })
    }
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 🧪 Tests
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Deserialize)]
    struct TestArgs {
        name: String,
        count: Option<i32>,
    }
    
    #[derive(Serialize)]
    struct TestOutput {
        message: String,
        processed: i32,
    }
    
    struct TestTool;
    
    #[async_trait]
    impl ToolBuilder for TestTool {
        type Args = TestArgs;
        type Output = TestOutput;
        
        fn name() -> &'static str {
            "test_tool"
        }
        
        fn description() -> &'static str {
            "🧪 Test tool for builder pattern"
        }
        
        fn schema() -> Value {
            SchemaBuilder::new()
                .required_string("name", "Name parameter")
                .optional_integer("count", "Optional count", Some(0))
                .build()
        }
        
        async fn run(args: Self::Args, _config: &Config) -> EmpathicResult<Self::Output> {
            Ok(TestOutput {
                message: format!("Hello, {}!", args.name),
                processed: args.count.unwrap_or(1),
            })
        }
    }
    
    #[tokio::test]
    async fn test_tool_builder_pattern() {
        // Test the ToolBuilder trait methods directly
        assert_eq!(TestTool::name(), "test_tool");
        assert!(TestTool::description().contains("Test tool"));
        
        let schema = TestTool::schema();
        assert_eq!(schema["required"], json!(["name"]));
        assert!(schema["properties"]["name"]["type"] == "string");
    }
    
    #[test]
    fn test_schema_builder() {
        let schema = SchemaBuilder::new()
            .required_string("file_path", "Path to file")
            .optional_bool("recursive", "Recursive flag", Some(false))
            .build();
            
        assert_eq!(schema["required"], json!(["file_path"]));
        assert_eq!(schema["properties"]["recursive"]["default"], json!(false));
    }
    
    #[test]
    fn test_parameter_extraction() {
        let args = json!({
            "name": "test",
            "count": 42,
            "flag": true
        });
        
        assert_eq!(require_string(&args, "name").unwrap(), "test");
        assert_eq!(optional_int(&args, "count"), Some(42));
        assert!(bool_param_or(&args, "flag", false));
        assert!(bool_param_or(&args, "missing", true));
    }
}
