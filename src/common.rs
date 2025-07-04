use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

#[derive(Serialize)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: String,
}

pub fn send_response(id: u64, result: Value) {
    let response = Response {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    };
    // 🎯 KEEP THIS println! - This is MCP protocol JSON that MUST go to stdout
    // Claude.ai desktop reads these JSON responses from stdout
    println!("{}", serde_json::to_string(&response).unwrap());
}

pub fn send_error(id: u64, code: i32, message: &str) {
    let response = Response {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(ErrorInfo {
            code,
            message: message.to_string(),
        }),
    };
    // 🎯 KEEP THIS println! - This is MCP protocol JSON that MUST go to stdout
    // Claude.ai desktop reads these JSON error responses from stdout
    println!("{}", serde_json::to_string(&response).unwrap());
}
