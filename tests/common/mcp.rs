//! 🔧 MCP response parsing utilities

use anyhow::{anyhow, Result};
use serde_json::Value;

/// 📦 Parsed MCP tool execution result
#[derive(Debug, Clone)]
pub struct McpResult {
    pub raw: Value,
    pub content: Value,
}

impl McpResult {
    /// Parse MCP response format: {"content": [{"text": "...", "type": "text"}]}
    pub fn parse(mcp_response: Value) -> Result<Self> {
        let content_text = mcp_response
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
            .ok_or_else(|| anyhow!("❌ Invalid MCP response format: missing content[0].text"))?;
        
        let content: Value = serde_json::from_str(content_text)
            .map_err(|e| anyhow!("❌ Failed to parse JSON content: {}", e))?;

        Ok(Self {
            raw: mcp_response,
            content,
        })
    }

    /// Get success status from parsed content
    pub fn success(&self) -> bool {
        self.content.get("success")
            .and_then(|s| s.as_bool())
            .unwrap_or(false)
    }

    /// Get exit code (for executor tools)
    pub fn exit_code(&self) -> Option<i64> {
        self.content.get("exit_code")
            .and_then(|c| c.as_i64())
    }

    /// Get stdout output
    pub fn stdout(&self) -> Option<&str> {
        self.content.get("stdout")
            .and_then(|s| s.as_str())
    }

    /// Get stderr output
    pub fn stderr(&self) -> Option<&str> {
        self.content.get("stderr")
            .and_then(|s| s.as_str())
    }

    /// Get working directory (for executor tools)
    pub fn working_dir(&self) -> Option<&str> {
        self.content.get("working_dir")
            .and_then(|d| d.as_str())
    }

    /// Get bytes written (for write operations)
    pub fn bytes_written(&self) -> Option<u64> {
        self.content.get("bytes_written")
            .and_then(|b| b.as_u64())
    }

    /// Get file content (for read operations)  
    pub fn file_content(&self) -> Option<&str> {
        // For read_file tool, content is directly in the MCP text field
        self.raw.get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
    }

    /// Get any string field by name
    pub fn get_str(&self, field: &str) -> Option<&str> {
        self.content.get(field)
            .and_then(|v| v.as_str())
    }

    /// Get any bool field by name
    pub fn get_bool(&self, field: &str) -> Option<bool> {
        self.content.get(field)
            .and_then(|v| v.as_bool())
    }

    /// Get any number field by name
    pub fn get_i64(&self, field: &str) -> Option<i64> {
        self.content.get(field)
            .and_then(|v| v.as_i64())
    }
}

/// 🎯 Quick success assertion with detailed error info
pub fn assert_mcp_success(result: &McpResult) {
    if !result.success() {
        panic!("❌ MCP operation failed:\n  Exit code: {:?}\n  Stderr: {:?}\n  Raw: {}", 
               result.exit_code(), 
               result.stderr(),
               serde_json::to_string_pretty(&result.content).unwrap_or_default());
    }
}

/// 🎯 Quick failure assertion with detailed error info
pub fn assert_mcp_failure(result: &McpResult) {
    if result.success() {
        panic!("❌ Expected MCP operation to fail but it succeeded:\n  Raw: {}", 
               serde_json::to_string_pretty(&result.content).unwrap_or_default());
    }
}

/// 🔍 Helper to debug MCP responses during test development
pub fn debug_mcp_response(result: &McpResult, label: &str) {
    println!("🔍 {} MCP Response:", label);
    println!("  Success: {}", result.success());
    if let Some(code) = result.exit_code() {
        println!("  Exit code: {}", code);
    }
    if let Some(stdout) = result.stdout() {
        println!("  Stdout: {:?}", stdout);
    }
    if let Some(stderr) = result.stderr() {
        println!("  Stderr: {:?}", stderr);
    }
    println!("  Content: {}", serde_json::to_string_pretty(&result.content).unwrap_or_default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_result_parsing() {
        let mcp_response = json!({
            "content": [{
                "text": "{\"success\": true, \"exit_code\": 0, \"stdout\": \"hello\"}",
                "type": "text"
            }]
        });

        let result = McpResult::parse(mcp_response).unwrap();
        assert!(result.success());
        assert_eq!(result.exit_code(), Some(0));
        assert_eq!(result.stdout(), Some("hello"));
    }

    #[test]
    fn test_mcp_result_file_content() {
        let mcp_response = json!({
            "content": [{
                "text": "file content here",
                "type": "text"
            }]
        });

        let result = McpResult::parse(json!({})).unwrap_or_else(|_| {
            McpResult {
                raw: mcp_response.clone(),
                content: json!({}),
            }
        });
        
        // For read operations, content is in the text field
        assert_eq!(result.file_content(), Some("file content here"));
    }
}
