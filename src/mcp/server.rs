use std::sync::Arc;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};

use crate::error::EmpathicResult;

use crate::config::Config;
use crate::tools::{Tool, get_all_tools};
use crate::lsp::LspManager;
use crate::mcp::protocol::JsonRpcRequest;
use crate::mcp::handlers::RequestHandler;

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
#[inline(always)]
fn log(config: &Config, level: &str, msg: &str) {
    if should_log(&config.log_level, level) {
        eprintln!("{level}: {msg}");
    }
}

/// 🚀 MCP JSON-RPC 2.0 Server 
pub struct McpServer {
    config: Config,
    tools: HashMap<String, Box<dyn Tool>>,
    /// 🧠 LSP manager for semantic code analysis
    lsp_manager: Arc<LspManager>,
}

impl McpServer {
    pub fn new(mut config: Config) -> Self {
        log(&config, "INFO", "🚀 Creating new MCP server instance");
        
        // Create LSP manager for semantic analysis and file synchronization
        let lsp_manager = Arc::new(LspManager::new(config.root_dir.clone()));
        
        // Set LSP manager in config so tools can access it
        config.set_lsp_manager(lsp_manager.clone());
        
        let tools = get_all_tools().into_iter()
            .map(|tool| (tool.name().to_string(), tool))
            .collect::<HashMap<_, _>>();
        
        log(&config, "INFO", &format!("🔧 Registered {} tools", tools.len()));
        log(&config, "INFO", "🧠 LSP manager initialized for file synchronization");
        
        Self {
            config,
            tools,
            lsp_manager,
        }
    }
    
    pub async fn run(&mut self) -> EmpathicResult<()> {
        log(&self.config, "INFO", "🚀 MCP server initialized");
        
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = TokioBufReader::new(stdin);
        let mut line = String::new();
        let mut request_count = 0;
        
        let handler = RequestHandler::new(&self.config, &self.tools);
        
        loop {
            log(&self.config, "DEBUG", &format!("📋 Loop iteration {request_count}, clearing line buffer"));
            line.clear();
            
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    log(&self.config, "INFO", "🚀 MCP server initialized");
                    break;
                },
                Ok(bytes_read) => {
                    log(&self.config, "DEBUG", &format!("📨 Read {bytes_read} bytes from stdin"));
                    
                    request_count += 1;
                    
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            log(&self.config, "INFO", &format!("✅ Parsed JSON-RPC request: {}", request.method));
                            
                            if let Some(response) = handler.handle_request(request).await {
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
                                    },
                                    Err(e) => {
                                        log(&self.config, "ERROR", &format!("❌ Failed to serialize response: {e}"));
                                        return Err(e.into());
                                    }
                                }
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
        }
        
        log(&self.config, "INFO", "🧠 Shutting down LSP servers before exit");
        if let Err(e) = self.lsp_manager.shutdown_all().await {
            log(&self.config, "ERROR", &format!("❌ Error shutting down LSP servers: {}", e));
        }
        
        log(&self.config, "INFO", "✅ MCP server shutdown complete");
        Ok(())
    }
}
