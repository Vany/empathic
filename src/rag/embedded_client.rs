//! 🚀 Drop-in replacement for ElasticsearchClient using embedded Tantivy
//! Same API surface but uses pure Rust implementation

pub use embedded_client_impl::*;
mod embedded_client_impl {
    use crate::rag::embedded_rag::get_embedded_rag;
    use serde_json::{Value, json};

    /// 🚀 Drop-in replacement for ElasticsearchClient using embedded Tantivy
    /// Same API surface but uses pure Rust implementation
    pub struct ElasticsearchClient {
        _base_url: String, // Kept for API compatibility
    }

    impl ElasticsearchClient {
        pub fn new() -> Self {
            Self {
                _base_url: "embedded://localhost".to_string(),
            }
        }

        /// 📥 Index a document with vector embedding
        pub async fn index_document(
            &self,
            index: &str,
            doc_id: Option<&str>,
            document: &Value,
        ) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let mut engine = rag.lock().await;

            // Extract fields from document
            let content = document
                .get("content")
                .and_then(|c| c.as_str())
                .ok_or("Missing content field")?;

            let embedding = document
                .get("embedding")
                .and_then(|e| e.as_array())
                .ok_or("Missing embedding field")?
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect::<Vec<f32>>();

            if embedding.len() != 384 {
                return Err(format!(
                    "Invalid embedding dimensions: expected 384, got {}",
                    embedding.len()
                ));
            }

            let metadata = document.get("metadata").cloned().unwrap_or(json!({}));
            let source = document
                .get("source")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
            let chunk_index = document
                .get("chunk_index")
                .and_then(|i| i.as_u64())
                .unwrap_or(0) as usize;

            let chunk_id = engine
                .index_document(
                    index,
                    crate::rag::embedded_rag::embedded::IndexDocumentParams {
                        doc_id,
                        content,
                        embedding,
                        metadata,
                        source,
                        chunk_index,
                    },
                )
                .await?;

            Ok(json!({
                "_id": chunk_id,
                "_index": index,
                "result": "created",
                "_version": 1
            }))
        }

        /// 🔍 Search documents using vector similarity
        pub async fn vector_search(
            &self,
            index: &str,
            query_vector: &[f32],
            size: usize,
            filters: Option<&Value>,
        ) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let mut engine = rag.lock().await;

            engine
                .vector_search(index, query_vector, size, filters)
                .await
        }

        /// 🔀 Hybrid search (vector + keyword)
        pub async fn hybrid_search(
            &self,
            index: &str,
            query_text: &str,
            query_vector: &[f32],
            size: usize,
        ) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let mut engine = rag.lock().await;

            engine
                .hybrid_search(index, query_text, query_vector, size)
                .await
        }

        /// 🗂️ Create index with vector mapping
        pub async fn create_index(
            &self,
            index: &str,
            _settings: Option<&Value>,
        ) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let mut engine = rag.lock().await;

            engine.ensure_index(index).await?;

            Ok(json!({
                "acknowledged": true,
                "shards_acknowledged": true,
                "index": index
            }))
        }

        /// 🗑️ Delete index
        pub async fn delete_index(&self, index: &str) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let mut engine = rag.lock().await;

            engine.delete_index(index).await?;

            Ok(json!({
                "acknowledged": true
            }))
        }

        /// 📋 List all indices
        pub async fn list_indices(&self) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let engine = rag.lock().await;

            let indices = engine.list_indices().await?;

            // Format as Elasticsearch cat indices response
            let formatted_indices: Vec<Value> = indices
                .into_iter()
                .map(|index_name| {
                    json!({
                        "health": "green",
                        "status": "open",
                        "index": index_name,
                        "pri": "1",
                        "rep": "0",
                        "docs.count": "unknown",
                        "docs.deleted": "0",
                        "store.size": "~1mb"
                    })
                })
                .collect();

            Ok(json!(formatted_indices))
        }

        /// 📊 Get index statistics
        pub async fn index_stats(&self, index: &str) -> Result<Value, String> {
            // Simplified stats for embedded engine
            Ok(json!({
                "_all": {
                    "primaries": {
                        "docs": {
                            "count": 0,
                            "deleted": 0
                        },
                        "store": {
                            "size_in_bytes": 1024
                        }
                    },
                    "total": {
                        "docs": {
                            "count": 0,
                            "deleted": 0
                        },
                        "store": {
                            "size_in_bytes": 1024
                        }
                    }
                },
                "indices": {
                    index: {
                        "primaries": {
                            "docs": {
                                "count": 0,
                                "deleted": 0
                            }
                        }
                    }
                }
            }))
        }

        /// 🏥 Check cluster health
        pub async fn cluster_health(&self) -> Result<Value, String> {
            let rag = get_embedded_rag();
            let engine = rag.lock().await;

            Ok(engine.health().await)
        }

        /// 🔍 Execute raw Elasticsearch query with enhanced support
        pub async fn execute_raw_query(&self, index: &str, query: &Value) -> Result<Value, String> {
            eprintln!("🔍 [8] execute_raw_query called for index: {index}");
            let rag = get_embedded_rag();
            let mut engine = rag.lock().await;
            eprintln!("🔍 [9] Got RAG engine lock");

            // Ensure index exists
            engine.ensure_index(index).await?;
            eprintln!("🔍 [10] Index ensured");

            // Extract query parameters
            let size = query.get("size").and_then(|s| s.as_u64()).unwrap_or(10) as usize;
            eprintln!("🔍 [11] Query size: {size}");

            // Handle different query types
            if let Some(query_obj) = query.get("query") {
                eprintln!("🔍 [12] Processing query object...");
                match query_obj {
                    // Match all query
                    Value::Object(map) if map.contains_key("match_all") => {
                        // Return all documents (vector search with dummy vector)
                        let dummy_vector = vec![0.0f32; 384];
                        engine.vector_search(index, &dummy_vector, size, None).await
                    }

                    // Boolean query support
                    Value::Object(map) if map.contains_key("bool") => {
                        eprintln!("🔍 [13] Detected bool query, calling handle_bool_query...");
                        self.handle_bool_query(&mut engine, index, &map["bool"], size).await
                    }

                    // Match query (text search)
                    Value::Object(map) if map.contains_key("match") => {
                        if let Some(match_obj) = map["match"].as_object() {
                            // Extract field and query text
                            for (field, value) in match_obj {
                                if field == "content" {
                                    if let Some(query_text) = value.as_str() {
                                        // Use hybrid search with minimal vector component
                                        let dummy_vector = vec![0.0f32; 384];
                                        return engine
                                            .hybrid_search(index, query_text, &dummy_vector, size)
                                            .await;
                                    }
                                }
                            }
                        }
                        Ok(json!({
                            "hits": {
                                "total": { "value": 0 },
                                "hits": []
                            }
                        }))
                    }

                    // KNN vector search
                    Value::Object(map) if map.contains_key("knn") => {
                        if let Some(knn) = map["knn"].as_object() {
                            if let Some(vector) = knn.get("vector").and_then(|v| v.as_array()) {
                                let query_vector: Vec<f32> = vector
                                    .iter()
                                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                                    .collect();

                                if query_vector.len() == 384 {
                                    return engine
                                        .vector_search(index, &query_vector, size, None)
                                        .await;
                                }
                            }
                        }
                        Err("Invalid KNN query format".to_string())
                    }

                    // Default case
                    _ => Ok(json!({
                        "hits": {
                            "total": { "value": 0 },
                            "hits": []
                        },
                        "aggregations": {}
                    })),
                }
            } else {
                // No query specified, return empty results
                Ok(json!({
                    "hits": {
                        "total": { "value": 0 },
                        "hits": []
                    }
                }))
            }
        }

        /// Handle boolean queries
        async fn handle_bool_query(
            &self,
            engine: &mut crate::rag::embedded_rag::embedded::EmbeddedRagEngine,
            index: &str,
            bool_query: &Value,
            size: usize,
        ) -> Result<Value, String> {
            eprintln!("🔍 [14] handle_bool_query started");
            eprintln!("🔍 [15] Using already-locked engine instance");
            eprintln!("🔍 [16] Got RAG engine lock in handle_bool_query");

            // Check for filter/term clauses (metadata filtering)
            if let Some(must_clauses) = bool_query.get("must").and_then(|m| m.as_array()) {
                eprintln!("🔍 [17] Processing must clauses: {}", must_clauses.len());
                let mut filters = json!({});
                let mut has_filters = false;

                for clause in must_clauses {
                    if let Some(term_obj) = clause.get("term").and_then(|t| t.as_object()) {
                        eprintln!("🔍 [18] Found term clause");
                        // Handle term queries as metadata filters
                        for (field, value) in term_obj {
                            filters[field] = value.clone();
                            has_filters = true;
                        }
                    }
                }

                // Use filter search if we have metadata filters
                if has_filters {
                    eprintln!("🔍 [19] About to call engine.filter_search...");
                    return engine.filter_search(index, &filters, size).await;
                }

                // Handle text search in must clauses
                for clause in must_clauses {
                    if let Some(match_obj) = clause.get("match").and_then(|m| m.as_object()) {
                        for (field, value) in match_obj {
                            if field == "content" && value.is_string() {
                                let query_text = value.as_str().unwrap();
                                let dummy_vector = vec![0.0f32; 384];
                                return engine
                                    .hybrid_search(index, query_text, &dummy_vector, size)
                                    .await;
                            }
                        }
                    }
                }
            }

            // Default empty result
            Ok(json!({
                "hits": {
                    "total": { "value": 0 },
                    "hits": []
                }
            }))
        }
    }

    /// 🧠 Embedded Embeddings Client using local Candle models
    /// Replaces the HTTP-based embeddings service
    pub struct EmbeddingsClient {
        _base_url: String, // Kept for API compatibility
    }

    impl EmbeddingsClient {
        pub fn new() -> Self {
            Self {
                _base_url: "embedded://localhost".to_string(),
            }
        }

        /// 🔢 Generate embedding vector for text using local Candle model
        pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String> {
            // Use the existing candle embeddings implementation
            use crate::rag::embeddings_native::EmbeddingsBackend;
            use std::sync::OnceLock;
            use tokio::sync::Mutex as AsyncMutex;

            static EMBEDDINGS: OnceLock<AsyncMutex<Option<EmbeddingsBackend>>> = OnceLock::new();

            let embeddings_mutex = EMBEDDINGS.get_or_init(|| AsyncMutex::new(None));
            let mut embeddings_guard = embeddings_mutex.lock().await;

            // Initialize embeddings backend if not already done
            if embeddings_guard.is_none() {
                eprintln!("🧠 Initializing native embeddings backend...");
                match EmbeddingsBackend::new().await {
                    Ok(backend) => {
                        *embeddings_guard = Some(backend);
                        eprintln!("✅ Native embeddings ready!");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to initialize embeddings: {e}");
                        eprintln!("🔄 Falling back to deterministic hash-based embeddings");
                        return Ok(self.hash_based_embedding(text));
                    }
                }
            }

            // Use real embeddings if available
            if let Some(ref backend) = *embeddings_guard {
                match backend.embed_text(text).await {
                    Ok(embedding) => Ok(embedding),
                    Err(e) => {
                        eprintln!("⚠️ Embeddings error ({e}), using fallback");
                        Ok(self.hash_based_embedding(text))
                    }
                }
            } else {
                Ok(self.hash_based_embedding(text))
            }
        }

        /// 🔄 Fallback deterministic embedding generation
        fn hash_based_embedding(&self, text: &str) -> Vec<f32> {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let hash = hasher.finish();

            // Generate deterministic 384-dim vector from hash
            let mut embedding = Vec::with_capacity(384);
            let mut seed = hash;
            for _ in 0..384 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let val = (seed as f32) / (u64::MAX as f32) * 2.0 - 1.0;
                embedding.push(val * 0.1); // Small values for reasonable similarity
            }

            embedding
        }
    }
}
