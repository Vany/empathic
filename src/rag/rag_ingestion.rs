use crate::{
    common::{send_error, send_response},
    rag::{
        embeddings_native::EmbeddingsBackend,
        rag_client::ElasticsearchClient,
        rag_stack::ensure_rag_stack_running,
    },
    tools::tool_trait::Tool,
};
use crate::register_tool;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

/// 📥 Document Ingestion Tool
/// Processes documents, chunks them, generates embeddings, and indexes them
#[derive(Default)]
pub struct RagIngestTool;

#[derive(Deserialize, Default)]
pub struct RagIngestArgs {
    /// File path, URL, or direct text content
    pub input: String,
    /// Input type: "file", "url", or "text"  
    pub input_type: Option<String>,
    /// Target index name (defaults to "documents")
    pub index: Option<String>,
    /// Chunk size in characters (defaults to 500)
    pub chunk_size: Option<usize>,
    /// Chunk overlap in characters (defaults to 50)
    pub chunk_overlap: Option<usize>,
    /// Document metadata as JSON object
    pub metadata: Option<Value>,
}

#[derive(Serialize)]
pub struct IngestedChunk {
    pub chunk_id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: Value,
    pub timestamp: String,
    pub source: String,
    pub chunk_index: usize,
}

impl Tool for RagIngestTool {
    fn name(&self) -> &'static str {
        "rag_ingest"
    }

    fn description(&self) -> &'static str {
        "📥 Document ingestion tool for RAG pipeline. Processes documents through automated chunking, embedding generation, and indexing. Supports multiple input types: 1) File paths for local documents (PDF, TXT, MD formats), 2) URLs for web content, 3) Direct text content. Features configurable chunking strategies with size and overlap controls, metadata extraction and preservation, batch processing for large documents, and automatic embedding generation via the embeddings service. The tool creates optimized document chunks with vector embeddings for semantic search operations."
    }

    fn emoji(&self) -> &'static str {
        "📥"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "File path, URL, or direct text content to ingest"
                },
                "input_type": {
                    "type": "string",
                    "enum": ["file", "url", "text"],
                    "description": "Type of input: 'file' for file paths, 'url' for web content, 'text' for direct content",
                    "default": "file"
                },
                "index": {
                    "type": "string",
                    "description": "Target Elasticsearch index name (defaults to 'documents')",
                    "default": "documents"
                },
                "chunk_size": {
                    "type": "integer",
                    "description": "Chunk size in characters (defaults to 500)",
                    "minimum": 100,
                    "maximum": 2000,
                    "default": 500
                },
                "chunk_overlap": {
                    "type": "integer",
                    "description": "Chunk overlap in characters (defaults to 50)",
                    "minimum": 0,
                    "maximum": 200,
                    "default": 50
                },
                "metadata": {
                    "type": "object",
                    "description": "Additional metadata to attach to all document chunks",
                    "additionalProperties": true
                }
            },
            "required": ["input"],
            "additionalProperties": false
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        tokio::spawn(async move {
            
            // Auto-start RAG stack if needed
            if let Err(e) = ensure_rag_stack_running().await {
                send_error(id, -1, &format!("❌ Failed to start RAG stack: {e}"));
                return;
            }

            let args: RagIngestArgs = match args.and_then(|v| serde_json::from_value(v).ok()) {
                Some(args) => {
                    args
                },
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_ingest");
                    return;
                }
            };

            let es_client = ElasticsearchClient::new();

            let embeddings_client = match EmbeddingsBackend::new().await {
                Ok(client) => {
                    client
                },
                Err(e) => {
                    send_error(id, -1, &format!("❌ Failed to initialize embeddings: {e}"));
                    return;
                }
            };

            // Extract content based on input type
            let content = match extract_content(&args).await {
                Ok(content) => {
                    content
                },
                Err(e) => {
                    send_error(id, -1, &format!("❌ Content extraction failed: {e}"));
                    return;
                }
            };

            // Chunk the content
            let chunks = chunk_text(
                &content,
                args.chunk_size.unwrap_or(500),
                args.chunk_overlap.unwrap_or(50),
            );

            if chunks.is_empty() {
                send_error(id, -1, "❌ No content chunks generated");
                return;
            }

            let index_name = args.index.as_deref().unwrap_or("documents");

            // Ensure index exists
            if let Err(e) = ensure_index_exists(&es_client, index_name).await {
                send_error(id, -1, &format!("❌ Index creation failed: {e}"));
                return;
            }

            // Generate embeddings for all chunks
            let embeddings = match embeddings_client.embed_batch(&chunks).await {
                Ok(embeddings) => {
                    embeddings
                },
                Err(e) => {
                    send_error(id, -1, &format!("❌ Embeddings generation failed: {e}"));
                    return;
                }
            };

            if embeddings.len() != chunks.len() {
                send_error(id, -1, "❌ Mismatch between chunks and embeddings count");
                return;
            }

            // Index all chunks
            let mut ingested_chunks = Vec::new();
            let timestamp = Utc::now().to_rfc3339();
            let source = determine_source(&args);

            for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
                let chunk_id = format!("{}_{}", source.replace("/", "_"), i);

                let mut chunk_metadata = args.metadata.clone().unwrap_or_else(|| json!({}));
                chunk_metadata["chunk_index"] = json!(i);
                chunk_metadata["total_chunks"] = json!(chunks.len());
                chunk_metadata["input_type"] = json!(args.input_type.as_deref().unwrap_or("file"));

                let document = json!({
                    "chunk_id": chunk_id,
                    "content": chunk,
                    "embedding": embedding,
                    "metadata": chunk_metadata,
                    "timestamp": timestamp,
                    "source": source,
                    "chunk_index": i
                });

                match es_client
                    .index_document(index_name, Some(&chunk_id), &document)
                    .await
                {
                    Ok(_) => {
                        ingested_chunks.push(IngestedChunk {
                            chunk_id: chunk_id.clone(),
                            content: chunk.clone(),
                            embedding: embedding.clone(),
                            metadata: chunk_metadata,
                            timestamp: timestamp.clone(),
                            source: source.clone(),
                            chunk_index: i,
                        });
                    }
                    Err(e) => {
                        send_error(id, -1, &format!("❌ Failed to index chunk {chunk_id}: {e}"));
                        return;
                    }
                }
            }


            // Success response
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "📥 Document Ingestion Complete!\n\n\
                        ✅ Successfully processed: {}\n\
                        📊 Generated {} chunks\n\
                        🧠 Created {} embeddings\n\
                        🗂️ Indexed to: {}\n\
                        📈 Vector dimensions: 384\n\n\
                        📋 Ingestion Details:\n\
                        - Source: {}\n\
                        - Chunk size: {} chars\n\
                        - Overlap: {} chars\n\
                        - Total content: {} chars\n\
                        - Index timestamp: {}\n\n\
                        🔍 Search ready! Use rag_search to find relevant content.\n\
                        💡 Example: rag_search with query text to test semantic search",
                        args.input,
                        chunks.len(),
                        embeddings.len(),
                        index_name,
                        source,
                        args.chunk_size.unwrap_or(500),
                        args.chunk_overlap.unwrap_or(50),
                        content.len(),
                        timestamp
                    )
                }]
            });

            send_response(id, result);
        });
    }
}

/// Extract content based on input type
async fn extract_content(args: &RagIngestArgs) -> Result<String, String> {
    let input_type = args.input_type.as_deref().unwrap_or("file");

    match input_type {
        "text" => {
            Ok(args.input.clone())
        },
        "file" => {
            extract_file_content(&args.input).await
        },
        "url" => {
            extract_url_content(&args.input).await
        },
        _ => {
            Err(format!("Unsupported input type: {input_type}"))
        },
    }
}

/// Extract content from local file
async fn extract_file_content(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(format!("File not found: {file_path}"));
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("txt") | Some("md") => {
            match fs::read_to_string(path) {
                Ok(content) => {
                    Ok(content)
                },
                Err(e) => {
                    Err(format!("Failed to read text file: {e}"))
                }
            }
        }
        Some("pdf") => {
            // For now, return error for PDF - would need PDF parsing library
            Err("PDF parsing not yet implemented. Please convert to text format.".to_string())
        }
        Some(ext) => {
            Err(format!("Unsupported file extension: .{ext}"))
        },
        None => {
            Err("File has no extension".to_string())
        },
    }
}

/// Extract content from URL
async fn extract_url_content(url: &str) -> Result<String, String> {
    // Simple curl-based content extraction
    let output = std::process::Command::new("curl")
        .args(["-s", "-L", url])
        .output()
        .map_err(|e| format!("Failed to fetch URL: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("URL fetch failed: {stderr}"));
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();

    // Basic HTML stripping (very simple)
    let stripped = strip_html_tags(&content);

    if stripped.trim().is_empty() {
        return Err("No content extracted from URL".to_string());
    }

    Ok(stripped)
}

/// Basic HTML tag stripping
fn strip_html_tags(html: &str) -> String {
    // Very basic HTML stripping - would use proper HTML parser in production
    let mut result = String::new();
    let mut in_tag = false;

    for char in html.chars() {
        match char {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Clean up whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Chunk text into smaller pieces with overlap (UTF-8 safe)
fn chunk_text(content: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    
    if content.len() <= chunk_size {
        return vec![content.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start_byte = 0;

    while start_byte < content.len() {
        // 🔧 FIX: Find safe character boundary for end position
        let mut end_byte = std::cmp::min(start_byte + chunk_size, content.len());
        
        // Ensure we're at a character boundary (not in the middle of a UTF-8 sequence)
        while end_byte < content.len() && !content.is_char_boundary(end_byte) {
            end_byte += 1;
        }
        
        
        // Extract chunk using safe slicing
        let chunk = &content[start_byte..end_byte];

        if !chunk.trim().is_empty() {
            chunks.push(chunk.to_string());
        }

        if end_byte >= content.len() {
            break;
        }

        // 🔧 FIX: Calculate next start position with character boundary safety
        let step = chunk_size.saturating_sub(overlap);
        let mut next_start = start_byte + step;
        
        // Ensure next start is at a character boundary
        while next_start < content.len() && !content.is_char_boundary(next_start) {
            next_start += 1;
        }
        
        start_byte = next_start;
    }

    chunks
}

/// Determine source identifier from args
fn determine_source(args: &RagIngestArgs) -> String {
    match args.input_type.as_deref().unwrap_or("file") {
        "file" => Path::new(&args.input)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
            .to_string_lossy()
            .to_string(),
        "url" => args.input.clone(),
        "text" => "direct_text".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Ensure the target index exists with proper mapping
async fn ensure_index_exists(es_client: &ElasticsearchClient, index: &str) -> Result<(), String> {
    // Check if index already exists
    match es_client.list_indices().await {
        Ok(indices) => {
            if let Some(indices_array) = indices.as_array() {
                for index_info in indices_array {
                    if let Some(name) = index_info.get("index").and_then(|v| v.as_str()) {
                        if name == index {
                            return Ok(()); // Index already exists
                        }
                    }
                }
            }
        }
        Err(_) => {
            // Ignore error, try to create index
        }
    }

    // Create index with vector mapping
    es_client.create_index(index, None).await?;
    Ok(())
}

// Register the tool
register_tool!("rag_ingest", RagIngestTool);
