use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 🚀 JSON-RPC 2.0 response macros - eliminates boilerplate
#[macro_export]
macro_rules! json_rpc_response {
    ($id:expr, $result:expr) => {
        $crate::mcp::protocol::JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: $id,
            result: Some($result),
            error: None,
        }
    };
}

#[macro_export]
macro_rules! json_rpc_error {
    ($id:expr, $code:expr, $message:expr) => {
        $crate::mcp::protocol::JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: $id,
            result: None,
            error: Some($crate::mcp::protocol::JsonRpcError {
                code: $code,
                message: $message.to_string(),
                data: None,
            }),
        }
    };
}

/// 📨 JSON-RPC 2.0 Request Structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// 📨 JSON-RPC 2.0 Response Structure
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// ❌ JSON-RPC 2.0 Error Structure
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// 🎯 MCP Initialize Response
#[derive(Debug, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: Capabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// 🔧 MCP Server Capabilities
#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub tools: Option<ToolsCapability>,
}

/// 🛠️ Tools Capability Configuration
#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

/// 📋 Server Information
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// 🔧 Tools List Response
#[derive(Debug, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolInfo>,
}

/// 🔧 Individual Tool Information
#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}
