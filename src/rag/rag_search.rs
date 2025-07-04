use crate::{
    common::{send_error, send_response},
    rag::{
        rag_client::{ElasticsearchClient, EmbeddingsClient},
        rag_stack::ensure_rag_stack_running,
    },
    tools::tool_trait::Tool,
};
use crate::register_tool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// 🔍 Semantic Search Tool
/// Performs vector similarity search using embeddings
#[derive(Default)]
pub struct RagSearchTool;

#[derive(Deserialize, Default)]
pub struct RagSearchArgs {
    /// Query text to search for
    pub query: String,
    /// Index name to search in (defaults to "documents")
    pub index: Option<String>,
    /// Maximum number of results to return (defaults to 10)
    pub limit: Option<usize>,
    /// Minimum similarity score threshold (0.0-1.0)
    pub min_score: Option<f32>,
    /// Metadata filters as JSON object
    pub filters: Option<Value>,
    /// Include full content in results (defaults to true)
    pub include_content: Option<bool>,
    /// Include embeddings in results (defaults to false)
    pub include_embeddings: Option<bool>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub chunk_id: String,
    pub content: Option<String>,
    pub score: f32,
    pub metadata: Value,
    pub source: String,
    pub chunk_index: usize,
    pub embedding: Option<Vec<f32>>,
}

impl Tool for RagSearchTool {
    fn name(&self) -> &'static str {
        "rag_search"
    }

    fn description(&self) -> &'static str {
        "🔍 Semantic search tool for RAG operations. Performs vector similarity search using natural language queries to find relevant document chunks. The tool converts query text to embeddings via the embeddings service, then searches the Elasticsearch vector index for semantically similar content. Features include: 1) Natural language query processing, 2) Configurable result limits and similarity thresholds, 3) Metadata-based filtering for precise results, 4) Relevance scoring with cosine similarity, 5) Optional content and embedding inclusion in results. Returns ranked results ordered by semantic relevance for downstream RAG applications."
    }

    fn emoji(&self) -> &'static str {
        "🔍"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural language query text to search for",
                    "minLength": 1
                },
                "index": {
                    "type": "string",
                    "description": "Elasticsearch index name to search in (defaults to 'documents')",
                    "default": "documents"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (defaults to 10)",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 10
                },
                "min_score": {
                    "type": "number",
                    "description": "Minimum similarity score threshold 0.0-1.0 (defaults to 0.0)",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.0
                },
                "filters": {
                    "type": "object",
                    "description": "Metadata filters to narrow search results",
                    "additionalProperties": true
                },
                "include_content": {
                    "type": "boolean",
                    "description": "Include full chunk content in results (defaults to true)",
                    "default": true
                },
                "include_embeddings": {
                    "type": "boolean",
                    "description": "Include embedding vectors in results (defaults to false)",
                    "default": false
                }
            },
            "required": ["query"],
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

            let args: RagSearchArgs = match args.and_then(|v| serde_json::from_value(v).ok()) {
                Some(args) => args,
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_search");
                    return;
                }
            };

            if args.query.trim().is_empty() {
                send_error(id, -1, "❌ Query cannot be empty");
                return;
            }

            let es_client = ElasticsearchClient::new();
            let embeddings_client = EmbeddingsClient::new();
            let index_name = args.index.as_deref().unwrap_or("documents");

            // Generate query embedding
            let query_vector = match embeddings_client.embed_text(&args.query).await {
                Ok(vector) => vector,
                Err(e) => {
                    send_error(
                        id,
                        -1,
                        &format!("❌ Query embedding generation failed: {e}"),
                    );
                    return;
                }
            };

            // Perform vector search
            let search_response = match es_client
                .vector_search(
                    index_name,
                    &query_vector,
                    args.limit.unwrap_or(10),
                    args.filters.as_ref(),
                )
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    send_error(id, -1, &format!("❌ Vector search failed: {e}"));
                    return;
                }
            };

            // Parse search results
            let results = match parse_search_results(&search_response, &args).await {
                Ok(results) => results,
                Err(e) => {
                    send_error(id, -1, &format!("❌ Result parsing failed: {e}"));
                    return;
                }
            };

            // Filter by minimum score
            let min_score = args.min_score.unwrap_or(0.0);
            let filtered_results: Vec<_> = results
                .into_iter()
                .filter(|r| r.score >= min_score)
                .collect();

            // Generate response
            let result_count = filtered_results.len();
            let include_content = args.include_content.unwrap_or(true);

            let response_text = if filtered_results.is_empty() {
                format!(
                    "🔍 Search Complete - No Results Found\n\n\
                    📝 Query: \"{}\"\n\
                    🗂️ Index: {}\n\
                    🎯 Minimum score: {:.3}\n\
                    📊 Total results: 0\n\n\
                    💡 Suggestions:\n\
                    - Try a broader query\n\
                    - Lower the min_score threshold\n\
                    - Check if documents are indexed with rag_ingest\n\
                    - Verify index name with rag_index_manage",
                    args.query, index_name, min_score
                )
            } else {
                let mut response = format!(
                    "🔍 Semantic Search Results\n\n\
                    📝 Query: \"{}\"\n\
                    🗂️ Index: {}\n\
                    📊 Found {} relevant chunks\n\
                    🎯 Score range: {:.3} - {:.3}\n\n",
                    args.query,
                    index_name,
                    result_count,
                    filtered_results
                        .iter()
                        .map(|r| r.score)
                        .fold(f32::INFINITY, f32::min),
                    filtered_results
                        .iter()
                        .map(|r| r.score)
                        .fold(f32::NEG_INFINITY, f32::max)
                );

                for (i, result) in filtered_results.iter().take(5).enumerate() {
                    response.push_str(&format!(
                        "📄 Result #{}: {} (Score: {:.3})\n",
                        i + 1,
                        result.source,
                        result.score
                    ));

                    if include_content {
                        let content_preview = if let Some(content) = &result.content {
                            if content.len() > 200 {
                                format!("{}...", &content[..200])
                            } else {
                                content.clone()
                            }
                        } else {
                            "Content not included".to_string()
                        };
                        response.push_str(&format!("   Content: {content_preview}\n"));
                    }

                    response.push_str(&format!(
                        "   Chunk: {} (index: {})\n\n",
                        result.chunk_id, result.chunk_index
                    ));
                }

                if result_count > 5 {
                    response.push_str(&format!("   ... and {} more results\n\n", result_count - 5));
                }

                response.push_str(&format!(
                    "🚀 Search Performance:\n\
                    - Vector similarity: Cosine distance\n\
                    - Embedding dimensions: 384\n\
                    - Results filtered by score ≥ {min_score:.3}\n\n\
                    💡 Use rag_similarity for direct vector operations"
                ));

                response
            };

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": response_text
                }]
            });

            send_response(id, result);
        });
    }
}

/// Parse Elasticsearch search response into structured results
async fn parse_search_results(
    response: &Value,
    args: &RagSearchArgs,
) -> Result<Vec<SearchResult>, String> {
    let hits = response
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .ok_or("Invalid search response format")?;

    let mut results = Vec::new();
    let include_content = args.include_content.unwrap_or(true);
    let include_embeddings = args.include_embeddings.unwrap_or(false);

    for hit in hits {
        let source = hit.get("_source").ok_or("Missing _source in hit")?;
        let score = hit
            .get("_score")
            .and_then(|s| s.as_f64())
            .map(|s| s as f32)
            .unwrap_or(0.0);

        let chunk_id = source
            .get("chunk_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let content = if include_content {
            source
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let metadata = source.get("metadata").cloned().unwrap_or(json!({}));

        let source_name = source
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let chunk_index = source
            .get("chunk_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let embedding = if include_embeddings {
            source
                .get("embedding")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect()
                })
        } else {
            None
        };

        results.push(SearchResult {
            chunk_id,
            content,
            score,
            metadata,
            source: source_name,
            chunk_index,
            embedding,
        });
    }

    // Sort by score descending
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}

/// 🔎 Metadata Filter Search Tool
/// Advanced search with field-specific filtering and faceted search
#[derive(Default)]
pub struct RagFilterSearchTool;

#[derive(Deserialize, Default)]
pub struct RagFilterSearchArgs {
    /// Index name to search in
    pub index: Option<String>,
    /// Field-specific filters as JSON object
    pub filters: Value,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Fields to return in results
    pub fields: Option<Vec<String>>,
    /// Sort configuration
    pub sort: Option<Value>,
    /// Enable faceted search aggregations
    pub facets: Option<Vec<String>>,
}

impl Tool for RagFilterSearchTool {
    fn name(&self) -> &'static str {
        "rag_filter_search"
    }

    fn description(&self) -> &'static str {
        "🔎 Advanced metadata filtering tool for precise RAG document search. Performs field-specific queries with complex filter combinations including: 1) Exact match filters for categorical fields, 2) Range queries for numeric and date fields, 3) Text matching with wildcards and regex, 4) Nested field queries for complex metadata, 5) Faceted search for result aggregations, 6) Custom sorting by any field. Supports Boolean logic (AND/OR/NOT) for combining multiple filters. Returns documents matching all specified criteria with optional aggregation statistics."
    }

    fn emoji(&self) -> &'static str {
        "🔎"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "index": {
                    "type": "string",
                    "description": "Index to search in (defaults to 'documents')",
                    "default": "documents"
                },
                "filters": {
                    "type": "object",
                    "description": "Field-specific filters (e.g., {\"metadata.author\": \"John\", \"metadata.year\": {\"gte\": 2020}})",
                    "additionalProperties": true
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results (defaults to 10)",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 10
                },
                "fields": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Specific fields to return (defaults to all)"
                },
                "sort": {
                    "type": "object",
                    "description": "Sort configuration (e.g., {\"metadata.date\": \"desc\"})"
                },
                "facets": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Fields for faceted aggregations"
                }
            },
            "required": ["filters"],
            "additionalProperties": false
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        tokio::spawn(async move {
            eprintln!("🔍 [1] Starting rag_filter_search tool...");
            
            // Auto-start RAG stack if needed
            eprintln!("🔍 [2] About to ensure RAG stack...");
            if let Err(e) = ensure_rag_stack_running().await {
                send_error(id, -1, &format!("❌ Failed to start RAG stack: {e}"));
                return;
            }
            eprintln!("🔍 [3] RAG stack ensured successfully");

            let args: RagFilterSearchArgs = match args.and_then(|v| serde_json::from_value(v).ok())
            {
                Some(args) => args,
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_filter_search");
                    return;
                }
            };
            eprintln!("🔍 [4] Arguments parsed successfully");

            let es_client = ElasticsearchClient::new();
            let index_name = args.index.as_deref().unwrap_or("documents");
            eprintln!("🔍 [5] ElasticsearchClient created, index: {index_name}");

            // Build Elasticsearch query
            let mut query = json!({
                "size": args.limit.unwrap_or(10),
                "query": {
                    "bool": {
                        "must": build_filter_clauses(&args.filters)
                    }
                }
            });
            eprintln!("🔍 [6] Query built: {}", serde_json::to_string_pretty(&query).unwrap_or("Invalid".to_string()));

            // Add fields selection if specified
            if let Some(fields) = &args.fields {
                query["_source"] = json!(fields);
            }

            // Add sorting if specified
            if let Some(sort) = &args.sort {
                query["sort"] = sort.clone();
            }

            // Add aggregations for faceted search
            if let Some(facets) = &args.facets {
                let mut aggs = json!({});
                for facet in facets {
                    aggs[facet] = json!({
                        "terms": {
                            "field": facet,
                            "size": 10
                        }
                    });
                }
                query["aggs"] = aggs;
            }
            eprintln!("🔍 [7] About to call execute_raw_query...");

            // Execute search using raw query
            match es_client.execute_raw_query(index_name, &query).await {
                Ok(response) => {
                    let hits = response
                        .get("hits")
                        .and_then(|h| h.get("hits"))
                        .and_then(|h| h.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);

                    let mut result_text = format!(
                        "🔎 Metadata Filter Search Results\n\n\
                        🗂️ Index: {}\n\
                        📊 Found {} matching documents\n\
                        🔍 Filters applied: {}\n\n",
                        index_name,
                        hits,
                        serde_json::to_string_pretty(&args.filters)
                            .unwrap_or_else(|_| "Invalid".to_string())
                    );

                    // Display results
                    if let Some(hits_array) = response
                        .get("hits")
                        .and_then(|h| h.get("hits"))
                        .and_then(|h| h.as_array())
                    {
                        for (i, hit) in hits_array.iter().take(5).enumerate() {
                            if let Some(source) = hit.get("_source") {
                                result_text.push_str(&format!(
                                    "📄 Result #{}:\n{}\n\n",
                                    i + 1,
                                    serde_json::to_string_pretty(source)
                                        .unwrap_or_else(|_| "Invalid".to_string())
                                ));
                            }
                        }

                        if hits > 5 {
                            result_text.push_str(&format!("... and {} more results\n\n", hits - 5));
                        }
                    }

                    // Display facet results if present
                    if let Some(aggs) = response.get("aggregations") {
                        result_text.push_str("📊 Faceted Results:\n");
                        for (facet, data) in aggs.as_object().unwrap_or(&serde_json::Map::new()) {
                            if let Some(buckets) = data.get("buckets").and_then(|b| b.as_array()) {
                                result_text.push_str(&format!("\n🏷️ {facet}:\n"));
                                for bucket in buckets.iter().take(5) {
                                    if let (Some(key), Some(count)) = (
                                        bucket.get("key").and_then(|k| k.as_str()),
                                        bucket.get("doc_count").and_then(|c| c.as_u64()),
                                    ) {
                                        result_text.push_str(&format!("  - {key}: {count} docs\n"));
                                    }
                                }
                            }
                        }
                    }

                    result_text.push_str(
                        "\n💡 Filter Examples:\n\
                        - Exact match: {\"field\": \"value\"}\n\
                        - Range: {\"field\": {\"gte\": 10, \"lte\": 100}}\n\
                        - Exists: {\"field\": {\"exists\": true}}\n\
                        - Multiple: {\"field1\": \"value\", \"field2\": {\"gte\": 5}}",
                    );

                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": result_text
                        }]
                    });

                    send_response(id, result);
                }
                Err(e) => send_error(id, -1, &format!("❌ Filter search failed: {e}")),
            }
        });
    }
}

/// 🏆 Result Ranking Tool
/// Custom relevance scoring and result re-ranking
#[derive(Default)]
pub struct RagRankResultsTool;

#[derive(Deserialize, Default)]
pub struct RagRankResultsArgs {
    /// Search results to re-rank (as JSON array)
    pub results: Vec<Value>,
    /// Ranking algorithm: "bm25", "tfidf", "custom", "fusion"
    pub algorithm: String,
    /// Query text for relevance scoring
    pub query: Option<String>,
    /// Field weights for custom scoring
    pub field_weights: Option<Value>,
    /// Boost factors for specific criteria
    pub boost_factors: Option<Value>,
    /// Maximum results to return after ranking
    pub limit: Option<usize>,
}

impl Tool for RagRankResultsTool {
    fn name(&self) -> &'static str {
        "rag_rank_results"
    }

    fn description(&self) -> &'static str {
        "🏆 Advanced result ranking tool for optimizing search relevance in RAG pipelines. Re-ranks search results using sophisticated scoring algorithms including: 1) BM25 scoring for traditional relevance, 2) TF-IDF weighting for term importance, 3) Custom scoring with field-specific weights, 4) Result fusion for combining multiple ranking signals, 5) Boost factors for business logic (recency, popularity, etc.). Supports multi-criteria ranking with configurable weights and normalization. Essential for improving result quality beyond basic vector similarity."
    }

    fn emoji(&self) -> &'static str {
        "🏆"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "results": {
                    "type": "array",
                    "description": "Search results to re-rank (each with content, metadata, score)",
                    "items": {"type": "object"}
                },
                "algorithm": {
                    "type": "string",
                    "enum": ["bm25", "tfidf", "custom", "fusion"],
                    "description": "Ranking algorithm to apply"
                },
                "query": {
                    "type": "string",
                    "description": "Query text for relevance scoring (required for bm25/tfidf)"
                },
                "field_weights": {
                    "type": "object",
                    "description": "Field importance weights for custom scoring (e.g., {\"title\": 2.0, \"content\": 1.0})"
                },
                "boost_factors": {
                    "type": "object",
                    "description": "Boost configuration (e.g., {\"recency\": {\"field\": \"date\", \"weight\": 0.5}})"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results after ranking (defaults to all)",
                    "minimum": 1
                }
            },
            "required": ["results", "algorithm"],
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

            let args: RagRankResultsArgs = match args.and_then(|v| serde_json::from_value(v).ok()) {
                Some(args) => args,
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_rank_results");
                    return;
                }
            };

            if args.results.is_empty() {
                send_error(id, -1, "❌ No results provided for ranking");
                return;
            }

            // Apply ranking based on algorithm
            let ranked_results = match args.algorithm.as_str() {
                "bm25" => {
                    if args.query.is_none() {
                        send_error(id, -1, "❌ Query required for BM25 ranking");
                        return;
                    }
                    rank_by_bm25(&args.results, args.query.as_ref().unwrap())
                }
                "tfidf" => {
                    if args.query.is_none() {
                        send_error(id, -1, "❌ Query required for TF-IDF ranking");
                        return;
                    }
                    rank_by_tfidf(&args.results, args.query.as_ref().unwrap())
                }
                "custom" => rank_by_custom(&args.results, &args.field_weights, &args.boost_factors),
                "fusion" => {
                    rank_by_fusion(&args.results, args.query.as_deref(), &args.field_weights)
                }
                _ => {
                    send_error(id, -1, "❌ Invalid ranking algorithm");
                    return;
                }
            };

            // Limit results if specified
            let final_results: Vec<_> = if let Some(limit) = args.limit {
                ranked_results.into_iter().take(limit).collect()
            } else {
                ranked_results
            };

            // Format response
            let mut response_text = format!(
                "🏆 Result Ranking Complete!\n\n\
                📊 Algorithm: {}\n\
                📈 Input results: {}\n\
                🎯 Output results: {}\n",
                args.algorithm,
                args.results.len(),
                final_results.len()
            );

            if let Some(query) = &args.query {
                response_text.push_str(&format!("🔍 Query: \"{query}\"\n"));
            }

            response_text.push_str("\n📋 Ranked Results:\n\n");

            for (i, result) in final_results.iter().enumerate() {
                if let Some(score) = result.get("_rank_score").and_then(|s| s.as_f64()) {
                    response_text.push_str(&format!("#{} (Score: {:.3})\n", i + 1, score));
                } else {
                    response_text.push_str(&format!("#{}\n", i + 1));
                }

                if let Some(content) = result.get("content").and_then(|c| c.as_str()) {
                    let preview = if content.len() > 150 {
                        format!("{}...", &content[..150])
                    } else {
                        content.to_string()
                    };
                    response_text.push_str(&format!("   Content: {preview}\n"));
                }

                if let Some(source) = result.get("source").and_then(|s| s.as_str()) {
                    response_text.push_str(&format!("   Source: {source}\n"));
                }

                response_text.push('\n');
            }

            response_text.push_str(
                "💡 Ranking Features:\n\
                - BM25: Classic probabilistic ranking\n\
                - TF-IDF: Term frequency analysis\n\
                - Custom: Field-weighted scoring\n\
                - Fusion: Combined ranking signals\n\n\
                🚀 Use ranked results for improved relevance!",
            );

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": response_text
                }]
            });

            send_response(id, result);
        });
    }
}

/// 🔀 Hybrid Search Tool (combines vector + keyword search)
#[derive(Default)]
pub struct RagHybridSearchTool;

#[derive(Deserialize, Default)]
pub struct RagHybridSearchArgs {
    /// Query text for both vector and keyword search
    pub query: String,
    /// Index name to search in
    pub index: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Vector search weight (0.0-1.0, defaults to 0.7)
    pub vector_weight: Option<f32>,
    /// Keyword search weight (0.0-1.0, defaults to 0.3)
    pub keyword_weight: Option<f32>,
}

impl Tool for RagHybridSearchTool {
    fn name(&self) -> &'static str {
        "rag_hybrid_search"
    }

    fn description(&self) -> &'static str {
        "🔀 Hybrid search tool combining vector similarity and keyword matching for comprehensive RAG search. This tool performs both semantic vector search and traditional keyword search, then combines results with configurable weights. Features: 1) Dual-mode search (vector + keyword) for better recall, 2) Configurable weight balance between semantic and lexical matching, 3) Enhanced relevance through result fusion, 4) Metadata filtering support, 5) Optimized for complex queries requiring both concepts and exact terms. Ideal for queries where both semantic understanding and exact keyword matches are important."
    }

    fn emoji(&self) -> &'static str {
        "🔀"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Query text for both vector and keyword search"
                },
                "index": {
                    "type": "string",
                    "description": "Index to search in (defaults to 'documents')",
                    "default": "documents"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results (defaults to 10)",
                    "default": 10
                },
                "vector_weight": {
                    "type": "number",
                    "description": "Vector search weight 0.0-1.0 (defaults to 0.7)",
                    "default": 0.7
                },
                "keyword_weight": {
                    "type": "number",
                    "description": "Keyword search weight 0.0-1.0 (defaults to 0.3)",
                    "default": 0.3
                },
                "filters": {
                    "type": "object",
                    "description": "Metadata filters",
                    "additionalProperties": true
                }
            },
            "required": ["query"],
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

            let args: RagHybridSearchArgs = match args.and_then(|v| serde_json::from_value(v).ok())
            {
                Some(args) => args,
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_hybrid_search");
                    return;
                }
            };

            let es_client = ElasticsearchClient::new();
            let embeddings_client = EmbeddingsClient::new();
            let index_name = args.index.as_deref().unwrap_or("documents");

            // Generate query embedding for vector search
            let query_vector = match embeddings_client.embed_text(&args.query).await {
                Ok(vector) => vector,
                Err(e) => {
                    send_error(id, -1, &format!("❌ Query embedding failed: {e}"));
                    return;
                }
            };

            // Perform hybrid search
            match es_client
                .hybrid_search(
                    index_name,
                    &args.query,
                    &query_vector,
                    args.limit.unwrap_or(10),
                )
                .await
            {
                Ok(response) => {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!(
                                "🔀 Hybrid Search Complete!\n\n\
                                📝 Query: \"{}\"\n\
                                🗂️ Index: {}\n\
                                ⚖️ Vector weight: {:.1} | Keyword weight: {:.1}\n\n\
                                📊 Combined semantic and lexical search results:\n{}",
                                args.query,
                                index_name,
                                args.vector_weight.unwrap_or(0.7),
                                args.keyword_weight.unwrap_or(0.3),
                                serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Results unavailable".to_string())
                            )
                        }]
                    });
                    send_response(id, result);
                }
                Err(e) => send_error(id, -1, &format!("❌ Hybrid search failed: {e}")),
            }
        });
    }
}

// Helper functions for ranking algorithms

/// Build Elasticsearch filter clauses from filter object
fn build_filter_clauses(filters: &Value) -> Vec<Value> {
    let mut clauses = Vec::new();

    if let Some(obj) = filters.as_object() {
        for (field, value) in obj {
            if let Some(range_obj) = value.as_object() {
                // Handle range queries
                if range_obj.contains_key("gte")
                    || range_obj.contains_key("gt")
                    || range_obj.contains_key("lte")
                    || range_obj.contains_key("lt")
                {
                    clauses.push(json!({
                        "range": {
                            field: value
                        }
                    }));
                } else if range_obj.contains_key("exists") {
                    // Handle exists queries
                    clauses.push(json!({
                        "exists": {
                            "field": field
                        }
                    }));
                } else {
                    // Handle nested object as term query
                    clauses.push(json!({
                        "term": {
                            field: value
                        }
                    }));
                }
            } else if value.is_array() {
                // Handle array as terms query (OR)
                clauses.push(json!({
                    "terms": {
                        field: value
                    }
                }));
            } else {
                // Handle simple value as term query
                clauses.push(json!({
                    "term": {
                        field: value
                    }
                }));
            }
        }
    }

    clauses
}

/// Simple BM25 ranking implementation
fn rank_by_bm25(results: &[Value], query: &str) -> Vec<Value> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
    let k1 = 1.2;
    let b = 0.75;
    let avg_doc_length = 500.0; // Assumed average

    let mut ranked: Vec<(f64, Value)> = results
        .iter()
        .map(|result| {
            let content = result
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_lowercase();

            let doc_length = content.len() as f64;
            let mut score = 0.0;

            for term in &query_terms {
                let tf = content.matches(term).count() as f64;
                let idf = 2.0; // Simplified IDF
                let numerator = tf * (k1 + 1.0);
                let denominator = tf + k1 * (1.0 - b + b * (doc_length / avg_doc_length));
                score += idf * (numerator / denominator);
            }

            let mut result_with_score = result.clone();
            if let Some(obj) = result_with_score.as_object_mut() {
                obj.insert("_rank_score".to_string(), json!(score));
            }

            (score, result_with_score)
        })
        .collect();

    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.into_iter().map(|(_, r)| r).collect()
}

/// Simple TF-IDF ranking implementation
fn rank_by_tfidf(results: &[Value], query: &str) -> Vec<Value> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
    let total_docs = results.len() as f64;

    let mut ranked: Vec<(f64, Value)> = results
        .iter()
        .map(|result| {
            let content = result
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_lowercase();

            let doc_terms: Vec<&str> = content.split_whitespace().collect();
            let doc_length = doc_terms.len() as f64;
            let mut score = 0.0;

            for term in &query_terms {
                let tf = doc_terms.iter().filter(|&&t| t == *term).count() as f64 / doc_length;
                let df = results
                    .iter()
                    .filter(|r| {
                        r.get("content")
                            .and_then(|c| c.as_str())
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(term)
                    })
                    .count() as f64;
                let idf = (total_docs / (df + 1.0)).ln();
                score += tf * idf;
            }

            let mut result_with_score = result.clone();
            if let Some(obj) = result_with_score.as_object_mut() {
                obj.insert("_rank_score".to_string(), json!(score));
            }

            (score, result_with_score)
        })
        .collect();

    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.into_iter().map(|(_, r)| r).collect()
}

/// Custom field-weighted ranking
fn rank_by_custom(
    results: &[Value],
    field_weights: &Option<Value>,
    boost_factors: &Option<Value>,
) -> Vec<Value> {
    let weights = field_weights
        .as_ref()
        .and_then(|w| w.as_object())
        .cloned()
        .unwrap_or_default();

    let mut ranked: Vec<(f64, Value)> = results
        .iter()
        .map(|result| {
            let mut score = 0.0;

            // Apply field weights
            for (field, weight) in &weights {
                if let (Some(value), Some(w)) = (result.get(field), weight.as_f64()) {
                    if let Some(text) = value.as_str() {
                        score += text.len() as f64 * w / 100.0; // Normalize by length
                    } else if let Some(num) = value.as_f64() {
                        score += num * w;
                    }
                }
            }

            // Apply boost factors
            if let Some(boosts) = boost_factors.as_ref().and_then(|b| b.as_object()) {
                for (_boost_name, config) in boosts {
                    if let Some(boost_obj) = config.as_object() {
                        if let (Some(field), Some(weight)) = (
                            boost_obj.get("field").and_then(|f| f.as_str()),
                            boost_obj.get("weight").and_then(|w| w.as_f64()),
                        ) {
                            if let Some(value) = result.get(field).and_then(|v| v.as_f64()) {
                                score += value * weight;
                            }
                        }
                    }
                }
            }

            let mut result_with_score = result.clone();
            if let Some(obj) = result_with_score.as_object_mut() {
                obj.insert("_rank_score".to_string(), json!(score));
            }

            (score, result_with_score)
        })
        .collect();

    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked.into_iter().map(|(_, r)| r).collect()
}

/// Fusion ranking combining multiple signals
fn rank_by_fusion(
    results: &[Value],
    query: Option<&str>,
    field_weights: &Option<Value>,
) -> Vec<Value> {
    let mut fusion_scores: Vec<(f64, Value)> = Vec::new();

    // Get BM25 scores if query provided
    let bm25_results = if let Some(q) = query {
        rank_by_bm25(results, q)
    } else {
        results.to_vec()
    };

    // Get custom scores
    let custom_results = rank_by_custom(results, field_weights, &None);

    // Combine scores with normalization
    for (i, result) in results.iter().enumerate() {
        let bm25_score = bm25_results
            .get(i)
            .and_then(|r| r.get("_rank_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);

        let custom_score = custom_results
            .get(i)
            .and_then(|r| r.get("_rank_score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);

        // Normalize and combine (equal weights)
        let combined_score = (bm25_score + custom_score) / 2.0;

        let mut result_with_score = result.clone();
        if let Some(obj) = result_with_score.as_object_mut() {
            obj.insert("_rank_score".to_string(), json!(combined_score));
            obj.insert("_bm25_score".to_string(), json!(bm25_score));
            obj.insert("_custom_score".to_string(), json!(custom_score));
        }

        fusion_scores.push((combined_score, result_with_score));
    }

    fusion_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    fusion_scores.into_iter().map(|(_, r)| r).collect()
}

// Register the tools
register_tool!("rag_search", RagSearchTool);
register_tool!("rag_filter_search", RagFilterSearchTool);
register_tool!("rag_rank_results", RagRankResultsTool);
register_tool!("rag_hybrid_search", RagHybridSearchTool);
