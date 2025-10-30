//! 📂 Document Tracking and Synchronization
//!
//! Manages document state synchronization between empathic and LSP servers.
//! Tracks open documents, content changes, and handles didOpen/didClose/didChange events.

use crate::lsp::performance::LspMetrics;
use crate::lsp::types::{LspError, LspResult};
use crate::lsp::client::LspClient;
use crate::lsp::ProjectDetector;
use lsp_types::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

/// Type alias for async function returning LSP client
type GetClientFn<'a> = &'a dyn Fn(&Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = LspResult<LspClient>> + Send + '_>>;

/// Type alias for async function returning document tracker
type GetTrackerFn<'a> = &'a dyn Fn(&Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = LspResult<&'a mut DocumentTracker>> + Send + '_>>;

/// 📂 Tracks synchronized documents for an LSP server with performance monitoring
#[derive(Debug, Clone)]
pub struct DocumentTracker {
    /// Set of file URIs that are currently open in the LSP server
    open_documents: HashSet<Uri>,
    /// Last known content of each document (for change detection)
    document_content: HashMap<Uri, String>,
    /// Performance metrics for this tracker
    #[allow(dead_code)]
    metrics: Arc<LspMetrics>,
}

impl DocumentTracker {
    pub fn new(metrics: Arc<LspMetrics>) -> Self {
        Self {
            open_documents: HashSet::new(),
            document_content: HashMap::new(),
            metrics,
        }
    }

    pub fn is_open(&self, uri: &Uri) -> bool {
        self.open_documents.contains(uri)
    }

    pub fn add_document(&mut self, uri: Uri, content: String) {
        self.open_documents.insert(uri.clone());
        self.document_content.insert(uri, content);
        
        log::debug!("📂 Opened document, total open: {}", self.open_documents.len());
    }

    pub fn remove_document(&mut self, uri: &Uri) {
        self.open_documents.remove(uri);
        self.document_content.remove(uri);
        
        log::debug!("📂 Closed document, total open: {}", self.open_documents.len());
    }

    pub fn update_content(&mut self, uri: &Uri, content: String) {
        if self.open_documents.contains(uri) {
            self.document_content.insert(uri.clone(), content);
            log::debug!("📝 Updated document content: {}", uri.as_str());
        }
    }

    #[allow(dead_code)]
    pub fn get_content(&self, uri: &Uri) -> Option<&String> {
        self.document_content.get(uri)
    }

    pub fn open_document_count(&self) -> usize {
        self.open_documents.len()
    }
}

/// 📄 Document Operations Handler
pub struct DocumentOperations<'a> {
    detector: &'a ProjectDetector,
    get_client_fn: GetClientFn<'a>,
    get_tracker_fn: GetTrackerFn<'a>,
}

impl<'a> DocumentOperations<'a> {
    pub fn new(
        detector: &'a ProjectDetector,
        get_client_fn: GetClientFn<'a>,
        get_tracker_fn: GetTrackerFn<'a>,
    ) -> Self {
        Self {
            detector,
            get_client_fn,
            get_tracker_fn,
        }
    }

    /// 📂 Open a document in the LSP server (textDocument/didOpen)
    pub async fn open_document(&self, file_path: &Path) -> LspResult<()> {
        let _project = self.detector.find_project_for_file(file_path)?.ok_or_else(|| LspError::NoServerAvailable {
            file_path: file_path.to_path_buf(),
        })?;

        let file_url = Url::from_file_path(file_path).map_err(|_| LspError::InvalidRequest {
            message: format!("Invalid file path: {}", file_path.display()),
        })?;
        let file_uri = Uri::from_str(file_url.as_str()).unwrap();

        // Check if already open using the provided tracker function
        {
            let tracker = (self.get_tracker_fn)(file_path).await?;
            if tracker.is_open(&file_uri) {
                return Ok(()); // Already open
            }
        }

        // Read file content
        let content = tokio::fs::read_to_string(file_path).await.map_err(|e| {
            LspError::InvalidRequest {
                message: format!("Failed to read file {}: {}", file_path.display(), e),
            }
        })?;

        // Get client using the provided function
        let client = (self.get_client_fn)(file_path).await?;
        
        let language_id = if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
            "rust"
        } else {
            "text"
        };

        // Send didOpen notification
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: file_uri.clone(),
                language_id: language_id.to_string(),
                version: 1,
                text: content.clone(),
            },
        };

        client.send_notification("textDocument/didOpen", Some(serde_json::to_value(params)?)).await?;

        // Track the document using the provided tracker function
        {
            let tracker = (self.get_tracker_fn)(file_path).await?;
            tracker.add_document(file_uri, content);
        }

        log::debug!("📄 Opened document: {}", file_path.display());
        Ok(())
    }

    /// 📄 Close a document in the LSP server (textDocument/didClose)
    pub async fn close_document(&self, file_path: &Path) -> LspResult<()> {
        let _project = self.detector.find_project_for_file(file_path)?.ok_or_else(|| LspError::NoServerAvailable {
            file_path: file_path.to_path_buf(),
        })?;

        let file_url = Url::from_file_path(file_path).map_err(|_| LspError::InvalidRequest {
            message: format!("Invalid file path: {}", file_path.display()),
        })?;
        let file_uri = Uri::from_str(file_url.as_str()).unwrap();

        // Check if the document is open
        {
            let tracker = (self.get_tracker_fn)(file_path).await?;
            if !tracker.is_open(&file_uri) {
                return Ok(()); // Not open, nothing to do
            }
        }

        // Get client and send didClose
        let client = (self.get_client_fn)(file_path).await?;

        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: file_uri.clone() },
        };

        client.send_notification("textDocument/didClose", Some(serde_json::to_value(params)?)).await?;

        // Remove from tracking
        {
            let tracker = (self.get_tracker_fn)(file_path).await?;
            tracker.remove_document(&file_uri);
        }

        log::debug!("📄 Closed document: {}", file_path.display());
        Ok(())
    }

    /// 📝 Update document content in the LSP server (textDocument/didChange)
    pub async fn update_document(&self, file_path: &Path, new_content: String) -> LspResult<()> {
        let _project = self.detector.find_project_for_file(file_path)?.ok_or_else(|| LspError::NoServerAvailable {
            file_path: file_path.to_path_buf(),
        })?;

        let file_url = Url::from_file_path(file_path).map_err(|_| LspError::InvalidRequest {
            message: format!("Invalid file path: {}", file_path.display()),
        })?;
        let file_uri = Uri::from_str(file_url.as_str()).unwrap();

        // Check if the document is open
        {
            let tracker = (self.get_tracker_fn)(file_path).await?;
            if !tracker.is_open(&file_uri) {
                // Document not open, open it first
                self.open_document(file_path).await?;
                return Ok(());
            }
        }

        // Get client and send didChange
        let client = (self.get_client_fn)(file_path).await?;

        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: file_uri.clone(),
                version: 1, // Simple version number, LSP servers typically ignore this
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None, // Full document change
                range_length: None,
                text: new_content.clone(),
            }],
        };

        client.send_notification("textDocument/didChange", Some(serde_json::to_value(params)?)).await?;

        // Update content in tracker
        {
            let tracker = (self.get_tracker_fn)(file_path).await?;
            tracker.update_content(&file_uri, new_content);
        }

        log::debug!("📝 Updated document: {}", file_path.display());
        Ok(())
    }
}
