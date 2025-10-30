//! 🔌 LSP Client - JSON-RPC communication layer
//!
//! Handles JSON-RPC 2.0 communication with LSP servers over stdin/stdout.
//! Manages request correlation, async responses, and LSP initialization.

use crate::lsp::types::{LspError, LspResult};
use lsp_types::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tokio::time::timeout;
use url::Url;

/// 📡 JSON-RPC message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// 🔧 LSP client for JSON-RPC communication
pub struct LspClient {
    /// Project path this client serves
    project_path: std::path::PathBuf,
    /// Request ID counter
    next_id: AtomicU64,
    /// Pending requests waiting for responses
    pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
    /// Message sender to LSP server
    message_sender: mpsc::UnboundedSender<String>,
    /// Server capabilities after initialization
    capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Default request timeout
    timeout_duration: Duration,
    /// Notification broadcaster for LSP notifications
    notification_tx: broadcast::Sender<JsonRpcNotification>,
}

impl std::fmt::Debug for LspClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LspClient")
            .field("project_path", &self.project_path)
            .field("next_id", &self.next_id.load(Ordering::SeqCst))
            .field("timeout_duration", &self.timeout_duration)
            .field("notification_subscribers", &self.notification_tx.receiver_count())
            .finish_non_exhaustive()
    }
}

impl Clone for LspClient {
    fn clone(&self) -> Self {
        Self {
            project_path: self.project_path.clone(),
            next_id: AtomicU64::new(self.next_id.load(Ordering::SeqCst)),
            pending_requests: self.pending_requests.clone(),
            message_sender: self.message_sender.clone(),
            capabilities: self.capabilities.clone(),
            timeout_duration: self.timeout_duration,
            notification_tx: self.notification_tx.clone(),
        }
    }
}

impl LspClient {
    /// Create a new LSP client with stdin/stdout handles
    pub async fn new(
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        project_path: std::path::PathBuf,
    ) -> LspResult<Self> {

        let (message_tx, message_rx) = mpsc::unbounded_channel::<String>();
        let pending_requests = Arc::new(RwLock::new(HashMap::new()));
        
        // Create notification broadcast channel with capacity for 100 notifications
        let (notification_tx, _) = broadcast::channel(100);

        // 📊 Read LSP_TIMEOUT from environment (default: 60s)
        let timeout_duration = std::env::var("LSP_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(60));

        log::debug!("⏱️ LSP client timeout set to {}s", timeout_duration.as_secs());

        let client = Self {
            project_path,
            next_id: AtomicU64::new(1),
            pending_requests: pending_requests.clone(),
            message_sender: message_tx,
            capabilities: Arc::new(RwLock::new(None)),
            timeout_duration,
            notification_tx: notification_tx.clone(),
        };

        // Spawn communication tasks
        tokio::spawn({
            let pending_requests = pending_requests.clone();
            async move {
                Self::run_communication(stdin, stdout, message_rx, pending_requests, notification_tx).await
            }
        });

        Ok(client)
    }

    /// 🚀 Main communication loop with proper LSP Content-Length protocol
    async fn run_communication(
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        mut message_rx: mpsc::UnboundedReceiver<String>,
        pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
        notification_tx: broadcast::Sender<JsonRpcNotification>,
    ) {
        let mut reader = BufReader::new(stdout);

        loop {
            tokio::select! {
                // Handle outgoing messages with Content-Length headers
                msg = message_rx.recv() => {
                    match msg {
                        Some(message) => {
                            // LSP requires Content-Length header
                            let content = message.as_bytes();
                            let header = format!("Content-Length: {}\r\n\r\n", content.len());
                            
                            if let Err(e) = stdin.write_all(header.as_bytes()).await {
                                log::error!("Failed to write LSP header: {e}");
                                break;
                            }
                            if let Err(e) = stdin.write_all(content).await {
                                log::error!("Failed to write LSP content: {e}");
                                break;
                            }
                            if let Err(e) = stdin.flush().await {
                                log::error!("Failed to flush LSP stdin: {e}");
                                break;
                            }
                        }
                        None => break,
                    }
                }

                // Handle incoming messages with Content-Length protocol
                read_result = Self::read_lsp_message(&mut reader) => {
                    match read_result {
                        Ok(Some(content)) => {
                            if let Err(e) = Self::handle_incoming_message(&content, &pending_requests, &notification_tx).await {
                                log::error!("Failed to handle incoming LSP message: {e}");
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            log::error!("Failed to read LSP message: {e}");
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Read a single LSP message with Content-Length protocol
    async fn read_lsp_message(reader: &mut BufReader<tokio::process::ChildStdout>) -> LspResult<Option<String>> {
        let mut header_line = String::new();
        
        // Read headers until we find Content-Length
        let mut content_length: Option<usize> = None;
        
        loop {
            header_line.clear();
            let bytes_read = reader.read_line(&mut header_line).await
                .map_err(|e| LspError::InvalidResponse { 
                    message: format!("Failed to read header: {e}") 
                })?;
            
            if bytes_read == 0 {
                return Ok(None); // EOF
            }
            
            let header = header_line.trim();
            
            // Empty line signals end of headers
            if header.is_empty() {
                break;
            }
            
            // Parse Content-Length header
            if let Some(len_str) = header.strip_prefix("Content-Length:") {
                content_length = len_str.trim().parse().ok();
            }
            // Ignore other headers (Content-Type, etc.)
        }
        
        // Read the message content
        let content_length = content_length.ok_or_else(|| LspError::InvalidResponse {
            message: "Missing Content-Length header".to_string(),
        })?;
        
        let mut content = vec![0u8; content_length];
        reader.read_exact(&mut content).await
            .map_err(|e| LspError::InvalidResponse {
                message: format!("Failed to read message content: {e}"),
            })?;
        
        String::from_utf8(content).map(Some).map_err(|e| LspError::InvalidResponse {
            message: format!("Invalid UTF-8 in message: {e}"),
        })
    }

    /// Handle an incoming message from LSP server
    async fn handle_incoming_message(
        content: &str,
        pending_requests: &Arc<RwLock<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
        notification_tx: &broadcast::Sender<JsonRpcNotification>,
    ) -> LspResult<()> {
        let content = content.trim();
        if content.is_empty() {
            return Ok(());
        }

        // Parse JSON-RPC message
        let message: JsonRpcMessage = serde_json::from_str(content).map_err(|e| {
            LspError::InvalidResponse {
                message: format!("Failed to parse JSON-RPC: {e}"),
            }
        })?;

        match message {
            JsonRpcMessage::Response(response) => {
                // Find and notify the pending request
                let mut pending = pending_requests.write().await;
                if let Some(sender) = pending.remove(&response.id) {
                    let _ = sender.send(response); // Ignore if receiver is dropped
                }
            }
            JsonRpcMessage::Notification(notification) => {
                // Broadcast notification to all subscribers
                log::debug!("📨 LSP notification: {}", notification.method);
                let _ = notification_tx.send(notification); // Ignore if no subscribers
            }
            JsonRpcMessage::Request(_request) => {
                // LSP servers shouldn't send requests to clients in our use case
                log::warn!("Unexpected request from LSP server");
            }
        }

        Ok(())
    }

    /// 📤 Send a JSON-RPC request and wait for response
    pub async fn send_request<T>(&self, method: &str, params: Option<Value>) -> LspResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (response_tx, response_rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(id, response_tx);
        }

        // Create and send request (message will be framed with Content-Length by run_communication)
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let message = serde_json::to_string(&request)?;
        self.message_sender.send(message).map_err(|_| {
            LspError::JsonRpcError {
                message: "Failed to send message to LSP server".to_string(),
            }
        })?;

        // Wait for response with timeout
        let response = timeout(self.timeout_duration, response_rx)
            .await
            .map_err(|_| LspError::Timeout {
                timeout_secs: self.timeout_duration.as_secs(),
            })?
            .map_err(|_| LspError::JsonRpcError {
                message: "Response channel closed".to_string(),
            })?;

        // Handle response or error
        if let Some(error) = response.error {
            return Err(LspError::JsonRpcError {
                message: format!("LSP error {}: {}", error.code, error.message),
            });
        }

        // result field is REQUIRED in successful responses, but can be null
        // response.result is Option<Value>, where:
        // - None means field is missing (protocol violation)
        // - Some(Value::Null) means result is null (valid, means "no info")
        // - Some(Value::...) means result has data
        let result = response.result.unwrap_or(Value::Null);

        serde_json::from_value(result).map_err(|e| LspError::InvalidResponse {
            message: format!("Failed to deserialize response: {e}"),
        })
    }

    /// 📢 Send a JSON-RPC notification (no response expected)
    pub async fn send_notification(&self, method: &str, params: Option<Value>) -> LspResult<()> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };

        let message = serde_json::to_string(&notification)?;
        self.message_sender.send(message).map_err(|_| {
            LspError::JsonRpcError {
                message: "Failed to send notification to LSP server".to_string(),
            }
        })?;

        Ok(())
    }

    /// 🚀 Initialize the LSP server
    pub async fn initialize(&self) -> LspResult<InitializeResult> {
        let client_capabilities = ClientCapabilities {
            workspace: Some(WorkspaceClientCapabilities {
                configuration: Some(true),
                did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                    dynamic_registration: Some(false),
                    relative_pattern_support: Some(true),
                }),
                ..Default::default()
            }),
            text_document: Some(TextDocumentClientCapabilities {
                hover: Some(HoverClientCapabilities {
                    dynamic_registration: Some(false),
                    content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                }),
                completion: Some(CompletionClientCapabilities {
                    dynamic_registration: Some(false),
                    completion_item: Some(CompletionItemCapability {
                        snippet_support: Some(true),
                        resolve_support: Some(CompletionItemCapabilityResolveSupport {
                            properties: vec!["documentation".to_string(), "detail".to_string()],
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                definition: Some(GotoCapability {
                    dynamic_registration: Some(false),
                    link_support: Some(true),
                }),
                references: Some(ReferenceClientCapabilities {
                    dynamic_registration: Some(false),
                }),
                document_symbol: Some(DocumentSymbolClientCapabilities {
                    dynamic_registration: Some(false),
                    hierarchical_document_symbol_support: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let root_url = Url::from_file_path(&self.project_path).unwrap();
        let root_uri = Uri::from_str(root_url.as_str()).unwrap();
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            initialization_options: None,
            capabilities: client_capabilities,
            trace: Some(TraceValue::Off),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri.clone(),
                name: self.project_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unnamed")
                    .to_string(),
            }]),
            client_info: Some(ClientInfo {
                name: "empathic".to_string(),
                version: Some("2.0.0".to_string()),
            }),
            locale: None,
            work_done_progress_params: lsp_types::WorkDoneProgressParams::default(),
            ..Default::default()
        };

        let result: InitializeResult = self
            .send_request("initialize", Some(serde_json::to_value(params)?))
            .await?;

        // Store server capabilities
        {
            let mut capabilities = self.capabilities.write().await;
            *capabilities = Some(result.capabilities.clone());
        }

        // Send initialized notification
        self.send_notification("initialized", Some(json!({})))
            .await?;

        Ok(result)
    }

    /// Get server capabilities
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.read().await.clone()
    }

    /// Get project path
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    // 🧠 LSP-specific request methods

    /// 🎯 Send hover request
    pub async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        self.send_request("textDocument/hover", Some(serde_json::to_value(params)?)).await
    }

    /// 🎯 Send completion request
    pub async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        self.send_request("textDocument/completion", Some(serde_json::to_value(params)?)).await
    }

    /// 🎯 Send goto definition request
    pub async fn goto_definition(&self, params: GotoDefinitionParams) -> LspResult<Option<GotoDefinitionResponse>> {
        self.send_request("textDocument/definition", Some(serde_json::to_value(params)?)).await
    }

    /// 🎯 Send find references request
    pub async fn find_references(&self, params: ReferenceParams) -> LspResult<Option<Vec<Location>>> {
        self.send_request("textDocument/references", Some(serde_json::to_value(params)?)).await
    }

    /// 🎯 Send document symbols request
    pub async fn document_symbols(&self, params: DocumentSymbolParams) -> LspResult<Option<DocumentSymbolResponse>> {
        self.send_request("textDocument/documentSymbol", Some(serde_json::to_value(params)?)).await
    }

    /// 🎯 Send workspace symbols request
    pub async fn workspace_symbols(&self, params: WorkspaceSymbolParams) -> LspResult<Option<Vec<SymbolInformation>>> {
        self.send_request("workspace/symbol", Some(serde_json::to_value(params)?)).await
    }

    /// 🔍 Get server capabilities after initialization
    pub async fn get_capabilities(&self) -> Option<ServerCapabilities> {
        let caps = self.capabilities.read().await;
        caps.clone()
    }
    /// 🛑 Shutdown the LSP server gracefully
    pub async fn shutdown(&self) -> LspResult<()> {
        log::info!("🛑 Shutting down LSP server for project: {}", self.project_path().display());
        
        // Step 1: Send shutdown request
        match self.send_request::<Value>("shutdown", None).await {
            Ok(_) => log::debug!("✅ Shutdown request successful"),
            Err(e) => log::warn!("⚠️  Shutdown request failed: {e}"),
        }
        
        // Step 2: Send exit notification (terminates the server)
        match self.send_notification("exit", None).await {
            Ok(_) => log::debug!("✅ Exit notification sent"),
            Err(e) => log::warn!("⚠️  Exit notification failed: {e}"),
        }
        
        Ok(())
    }

    /// 📡 Subscribe to LSP notifications
    /// Returns a receiver that will get all notifications from the LSP server
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<JsonRpcNotification> {
        self.notification_tx.subscribe()
    }

    /// 🎯 Wait for a specific notification type with timeout
    /// Useful for waiting for diagnostics after opening a document
    pub async fn wait_for_notification(
        &self,
        method: &str,
        timeout_duration: Duration,
    ) -> LspResult<JsonRpcNotification> {
        let mut rx = self.subscribe_notifications();
        
        let result = timeout(timeout_duration, async {
            loop {
                match rx.recv().await {
                    Ok(notification) if notification.method == method => {
                        return Ok(notification);
                    }
                    Ok(_) => continue, // Not the notification we want
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        log::warn!("Notification listener lagged, skipped {} notifications", skipped);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(LspError::JsonRpcError {
                            message: "Notification channel closed".to_string(),
                        });
                    }
                }
            }
        })
        .await;

        match result {
            Ok(Ok(notification)) => Ok(notification),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(LspError::Timeout {
                timeout_secs: timeout_duration.as_secs(),
            }),
        }
    }
}
