use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};

use crate::config::Config;
use crate::tools::{Tool, get_all_tools};

/// 🚀 JSON-RPC 2.0 response macros - eliminates boilerplate
macro_rules! json_rpc_response {
    ($id:expr, $result:expr) => {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: $id,
            result: Some($result),
            error: None,
        }
    };
}

macro_rules! json_rpc_error {
    ($id:expr, $code:expr, $message:expr) => {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: $id,
            result: None,
            error: Some(JsonRpcError {
                code: $code,
                message: $message.to_string(),
                data: None,
            }),
        }
    };
}


/// 🔥 Log level hierarchy: ERROR > WARN > INFO > DEBUG
fn should_log(config_level: &str, message_level: &str) -> bool {
    let level_priority = |level: &str| match level {
        "error" => 3,
        "warn" => 2, 
        "info" => 1,
        "debug" => 0,
        _ => 0, // Unknown levels default to debug
    };
    
    level_priority(message_level.to_lowercase().as_str()) >= level_priority(config_level)
}

/// 🔥 Simplified logging with level filtering
/// 🔥 Simplified logging with level filtering
#[inline(always)]
fn log(config: &Config, level: &str, msg: &str) {
    if should_log(&config.log_level, level) {
        eprintln!("{level}: {msg}");
    }
}

/// MCP JSON-RPC 2.0 Server 🚀
pub struct McpServer {
    config: Config,
    tools: HashMap<String, Box<dyn Tool>>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: Capabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct Capabilities {
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Serialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

impl McpServer {
    pub fn new(config: Config) -> Self {
                log(&config, "INFO", "🚀 Creating new MCP server instance");
        
        let tools = get_all_tools().into_iter()
            .map(|tool| (tool.name().to_string(), tool))
            .collect::<HashMap<_, _>>();
        
                log(&config, "INFO", &format!("🔧 Registered {} tools", tools.len()));
        
        Self {
            config,
            tools,
        }
    }
    
    pub async fn run(&mut self) -> Result<()> {
        log(&self.config, "INFO", "$1");
        
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = TokioBufReader::new(stdin);
        let mut line = String::new();
        let mut request_count = 0;
        
        log(&self.config, "DEBUG", "$1");
        
        loop {
            log(&self.config, "DEBUG", &format!("📋 Loop iteration {request_count}, clearing line buffer"));
            line.clear();
            
            log(&self.config, "DEBUG", "$1");
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    log(&self.config, "INFO", "$1");
                    break;
                },
                Ok(bytes_read) => {
                    log(&self.config, "DEBUG", &format!("📨 Read {bytes_read} bytes from stdin"));
                    
                    request_count += 1;
                    
                    if line.trim().is_empty() {
                        log(&self.config, "DEBUG", "$1");
                        continue;
                    }
                    
                    match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            log(&self.config, "INFO", &format!("✅ Parsed JSON-RPC request: {}", request.method));
                            
                            if let Some(response) = self.handle_request(request).await {
                                match serde_json::to_string(&response) {
                                    Ok(response_json) => {
                                        log(&self.config, "DEBUG", "📤 Sending JSON-RPC response");
                                        
                                        if let Err(e) = stdout.write_all(response_json.as_bytes()).await {
                                            log(&self.config, "ERROR", &format!("❌ Failed to write response: {e}"));
                                            return Err(e.into());
                                        }
                                        
                                        if let Err(e) = stdout.write_all(b"\n").await {
                                            log(&self.config, "ERROR", &format!("❌ Failed to write newline: {e}"));
                                            return Err(e.into());
                                        }
                                        
                                        if let Err(e) = stdout.flush().await {
                                            log(&self.config, "ERROR", &format!("❌ Failed to flush stdout: {e}"));
                                            return Err(e.into());
                                        }
                                        
                                        log(&self.config, "DEBUG", "$1");
                                    },
                                    Err(e) => {
                                        log(&self.config, "ERROR", &format!("❌ Failed to serialize response: {e}"));
                                        return Err(e.into());
                                    }
                                }
                            } else {
                                log(&self.config, "DEBUG", "$1");
                            }
                        },
                        Err(e) => {
                            log(&self.config, "ERROR", &format!("❌ Failed to parse JSON-RPC request: {e}"));
                        }
                    }
                },
                Err(e) => {
                    log(&self.config, "ERROR", &format!("❌ Failed to read from stdin: {e}"));
                    return Err(e.into());
                }
            }
            
            log(&self.config, "ERROR", "Fixed");
        }
        
        log(&self.config, "INFO", "$1");
        Ok(())
    }
    
    async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        log(&self.config, "DEBUG", &format!("🎯 Handling request: {}", request.method));
        
        // Handle notifications - methods starting with "notifications/" should not receive responses
        if request.method.starts_with("notifications/") {
            log(&self.config, "INFO", &format!("📨 Ignoring notification: {}", request.method));
            return None;
        }
        
        let response = match request.method.as_str() {
            "initialize" => {
                log(&self.config, "INFO", "$1");
                self.handle_initialize(request).await
            },
            "tools/list" => {
                log(&self.config, "INFO", "$1");
                self.handle_tools_list(request).await
            },
            "tools/call" => {
                log(&self.config, "INFO", "$1");
                self.handle_tools_call(request).await
            },
            "prompts/list" => {
                log(&self.config, "INFO", "$1");
                self.handle_prompts_list(request).await
            },
            "resources/list" => {
                log(&self.config, "INFO", "$1");
                self.handle_resources_list(request).await
            },
            _ => {
                log(&self.config, "ERROR", &format!("❌ Unknown method: {}", request.method));
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                }
            },
        };
        
        log(&self.config, "DEBUG", "$1");
        Some(response)
    }
    
    async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        log(&self.config, "INFO", "$1");
        
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: Capabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            server_info: ServerInfo {
                name: "empathic".to_string(), // todo get mane from cargo
                version: "1.0.0".to_string(), // todo get version from cargo
            },
        };
        
        log(&self.config, "INFO", "✅ Initialize handshake complete");
        log(&self.config, "INFO", "✅ Initialize handshake complete");
        
        json_rpc_response!(request.id, serde_json::to_value(result).unwrap())
    }
    
    async fn handle_tools_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        log(&self.config, "DEBUG", "$1");
        
        let tools: Vec<ToolInfo> = self.tools.iter()
            .map(|(name, tool)| ToolInfo {
                name: name.clone(),
                description: tool.description().to_string(),
                input_schema: tool.schema(),
            })
            .collect();
        
        log(&self.config, "INFO", &format!("📦 Tools list prepared with {} tools", tools.len()));
        
        let result = ToolsListResult { tools };
        
        json_rpc_response!(request.id, serde_json::to_value(result).unwrap())
    }
    
    async fn handle_tools_call(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        log(&self.config, "DEBUG", "$1");
        
        let params = match request.params {
            Some(params) => params,
            None => {
                log(&self.config, "Uerror", "❌ Tool call missing params");
                return json_rpc_error!(request.id, -32602, "Invalid params");
            }
        };
        
        
        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                log(&self.config, "Uerror", "❌ Tool call missing name");
                return json_rpc_error!(request.id, -32602, "Tool name is required");
            }
        };
        
        log(&self.config, "ERROR", "Fixed");
        
        let tool = match self.tools.get(tool_name) {
            Some(tool) => tool,
            None => {
                log(&self.config, "ERROR", "Fixed");
                return json_rpc_error!(request.id, -32601, &format!("Tool '{tool_name}' not found"));
            }
        };
        
        let arguments = params.get("arguments").cloned().unwrap_or_default();
        
                match tool.execute(arguments, &self.config).await {
            Ok(result) => {
                log(&self.config, "ERROR", "Fixed");
                json_rpc_response!(request.id, result)
            },
            Err(e) => {
                log(&self.config, "ERROR", "Fixed");
                json_rpc_error!(request.id, -32000, &e.to_string())
            }
        }
    }
    
    async fn handle_prompts_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        log(&self.config, "DEBUG", "$1");
        json_rpc_response!(request.id, serde_json::json!({ "prompts": [] }))
    }
    
    async fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        log(&self.config, "DEBUG", "$1");
        json_rpc_response!(request.id, serde_json::json!({ "resources": [] }))
    }
}