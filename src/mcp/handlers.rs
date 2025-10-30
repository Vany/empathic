use std::collections::HashMap;

use crate::config::Config;
use crate::tools::Tool;
use crate::mcp::protocol::*;
use crate::{json_rpc_response, json_rpc_error};
use crate::error::EmpathicError;

/// 🔍 Format comprehensive error message with full context
fn format_detailed_error(error: &EmpathicError, tool_name: &str) -> String {
    let category = error.category();
    let is_recoverable = if error.is_recoverable() { "RECOVERABLE" } else { "FATAL" };
    
    // Base error message
    let mut message = format!(
        "Tool '{}' failed [category: {}, status: {}]\n\nError: {}\n\n",
        tool_name, category, is_recoverable, error
    );
    
    // Add category-specific guidance
    match category {
        "filesystem" => {
            message.push_str("Troubleshooting:\n");
            message.push_str("• Check if the file/directory exists\n");
            message.push_str("• Verify file permissions\n");
            message.push_str("• Ensure the path is correct and accessible\n");
        },
        "execution" => {
            message.push_str("Troubleshooting:\n");
            message.push_str("• Verify the command exists in PATH\n");
            message.push_str("• Check command syntax and arguments\n");
            message.push_str("• Ensure required dependencies are installed\n");
        },
        "lsp" => {
            message.push_str("Troubleshooting:\n");
            message.push_str("• Ensure LSP server (rust-analyzer) is installed\n");
            message.push_str("• Check if the project is a valid Rust project (Cargo.toml exists)\n");
            message.push_str("• Verify LSP server is not crashed (check logs)\n");
        },
        "configuration" => {
            message.push_str("Troubleshooting:\n");
            message.push_str("• Check environment variables (ROOT_DIR, ADD_PATH, LOGLEVEL)\n");
            message.push_str("• Verify configuration file syntax if using one\n");
            message.push_str("• Ensure all required settings are provided\n");
        },
        "protocol" => {
            message.push_str("Troubleshooting:\n");
            message.push_str("• Verify the tool name is correct\n");
            message.push_str("• Check that all required parameters are provided\n");
            message.push_str("• Ensure parameter types match the tool schema\n");
        },
        _ => {
            message.push_str("Contact support if the issue persists.\n");
        }
    }
    
    message
}

/// 🎯 Request Handler - Routes MCP requests to appropriate handlers
pub struct RequestHandler<'a> {
    config: &'a Config,
    tools: &'a HashMap<String, Box<dyn Tool>>,
}

impl<'a> RequestHandler<'a> {
    pub fn new(config: &'a Config, tools: &'a HashMap<String, Box<dyn Tool>>) -> Self {
        Self { config, tools }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        log::debug!("🎯 Handling request: {}", request.method);
        
        // Handle notifications - methods starting with "notifications/" should not receive responses
        if request.method.starts_with("notifications/") {
            log::info!("📨 Ignoring notification: {}", request.method);
            return None;
        }
        
        let response = match request.method.as_str() {
            "initialize" => {
                log::info!("🎯 Handling initialize request");
                self.handle_initialize(request).await
            },
            "tools/list" => {
                log::info!("🔧 Handling tools/list request");
                self.handle_tools_list(request).await
            },
            "tools/call" => {
                log::info!("⚙️  Handling tools/call request");
                self.handle_tools_call(request).await
            },
            "prompts/list" => {
                log::info!("📝 Handling prompts/list request");
                self.handle_prompts_list(request).await
            },
            "resources/list" => {
                log::info!("📂 Handling resources/list request");
                self.handle_resources_list(request).await
            },
            _ => {
                log::error!("❌ Unknown method: {}", request.method);
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
        
        Some(response)
    }
    
    async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        log::info!("🚀 MCP server initialized");
        
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: Capabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            server_info: ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        
        log::info!("✅ Initialize handshake complete");
        
        json_rpc_response!(request.id, serde_json::to_value(result).unwrap())
    }
    
    async fn handle_tools_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tools: Vec<ToolInfo> = self.tools.values()
            .map(|tool| ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.schema(),
            })
            .collect();
        
        log::info!("📦 Tools list prepared with {} tools", tools.len());
        
        let result = ToolsListResult { tools };
        
        json_rpc_response!(request.id, serde_json::to_value(result).unwrap())
    }
    
    async fn handle_tools_call(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params {
            Some(params) => params,
            None => {
                log::error!("❌ Tool call missing params");
                return json_rpc_error!(request.id, -32602, "Invalid params");
            }
        };
        
        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => {
                log::error!("❌ Tool call missing name");
                return json_rpc_error!(request.id, -32602, "Tool name is required");
            }
        };
        
        let tool = match self.tools.get(tool_name) {
            Some(tool) => tool,
            None => {
                return json_rpc_error!(request.id, -32601, &format!("Tool '{tool_name}' not found"));
            }
        };
        
        let arguments = params.get("arguments").cloned().unwrap_or_default();
        
        // 🚀 PROACTIVE LSP SPAWNING (v2.2.5)
        // When ANY tool is called with a `project` parameter, spawn LSP server
        // in background. This allows rust-analyzer to index while user works
        // with regular tools (read_file, write_file, git, etc.)
        // with regular tools (read_file, write_file, git, etc.)
        if let Some(project) = arguments.get("project").and_then(|v| v.as_str())
            && let Some(lsp_manager) = self.config.lsp_manager() {
            let project_path = self.config.project_path(Some(project));
            let lsp_manager = lsp_manager.clone();
            let project = project.to_string(); // Clone the string for the spawned task
            
            // Spawn LSP warmup in background task - don't wait for it
            tokio::spawn(async move {
                if let Err(e) = lsp_manager.get_client(&project_path).await {
                    log::debug!("⚠️ LSP warmup for project '{}' failed: {}", project, e);
                } else {
                    log::info!("🚀 LSP server warmed up for project: {}", project);
                }
            });
        }
        
        // ⏱️ Execute tool with hard timeout protection
        // 🌟 IMPORTANT: All 23 tools (16 MCP + 7 LSP) go through this single execution path
        // Any error from any tool gets enhanced error reporting via format_detailed_error()
        let timeout_duration = self.config.request_timeout;
        log::debug!("⏱️ Executing {} with {}s timeout", tool_name, timeout_duration.as_secs());
        
        match tokio::time::timeout(timeout_duration, tool.execute(arguments, self.config)).await {
            Ok(Ok(result)) => {
                log::debug!("✅ Tool {} completed successfully", tool_name);
                json_rpc_response!(request.id, result)
            },
            Ok(Err(e)) => {
                // 🔍 Generate comprehensive error message with context
                let detailed_error = format_detailed_error(&e, tool_name);
                log::error!("❌ Tool {} failed: {}", tool_name, detailed_error);
                json_rpc_error!(request.id, -32000, &detailed_error)
            },
            Err(_) => {
                let timeout_msg = format!(
                    "⏱️ Tool '{}' exceeded timeout of {}s. Claude Desktop has a 60s hard limit. Consider breaking operation into smaller chunks or optimizing the implementation.",
                    tool_name,
                    timeout_duration.as_secs()
                );
                log::error!("{}", timeout_msg);
                json_rpc_error!(request.id, -32001, &timeout_msg)
            }
        }
    }
    
    async fn handle_prompts_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        json_rpc_response!(request.id, serde_json::json!({ "prompts": [] }))
    }
    
    async fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        json_rpc_response!(request.id, serde_json::json!({ "resources": [] }))
    }
}