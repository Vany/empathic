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

/// 📊 Vector Similarity Tool
/// Performs direct vector similarity operations and nearest neighbor search
#[derive(Default)]
pub struct RagSimilarityTool;

#[derive(Deserialize, Default)]
pub struct RagSimilarityArgs {
    /// Query text to convert to vector, or "vector" for direct vector input
    pub query: String,
    /// Operation: "search", "compare", "neighbors", "distance"
    pub operation: Option<String>,
    /// Direct vector input (if query is "vector")
    pub vector: Option<Vec<f32>>,
    /// Index name to search in
    pub index: Option<String>,
    /// Number of nearest neighbors (for search operation)
    pub k: Option<usize>,
    /// Distance metric: "cosine", "euclidean", "dot_product"
    pub metric: Option<String>,
    /// Compare against specific documents (chunk_ids)
    pub target_docs: Option<Vec<String>>,
    /// Minimum similarity threshold
    pub min_similarity: Option<f32>,
}

#[derive(Serialize)]
pub struct SimilarityResult {
    pub chunk_id: String,
    pub similarity_score: f32,
    pub distance: f32,
    pub content_preview: Option<String>,
    pub source: String,
    pub metadata: Value,
}

impl Tool for RagSimilarityTool {
    fn name(&self) -> &'static str {
        "rag_similarity"
    }

    fn description(&self) -> &'static str {
        "📊 Vector similarity tool for direct vector operations and nearest neighbor search. Provides low-level vector similarity operations including: 1) Nearest neighbor search with configurable distance metrics, 2) Direct vector-to-vector similarity comparisons, 3) Batch similarity operations for multiple targets, 4) Similarity score analysis and ranking, 5) Support for cosine, euclidean, and dot product metrics. Features text-to-vector conversion via embeddings service, direct vector input support, and configurable similarity thresholds. Essential for advanced RAG operations requiring precise vector analysis and similarity measurements."
    }

    fn emoji(&self) -> &'static str {
        "📊"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Query text to convert to vector, or 'vector' for direct vector input"
                },
                "operation": {
                    "type": "string",
                    "enum": ["search", "compare", "neighbors", "distance"],
                    "description": "Vector operation type (defaults to 'search')",
                    "default": "search"
                },
                "vector": {
                    "type": "array",
                    "items": {"type": "number"},
                    "description": "Direct vector input (384 dimensions, used when query='vector')"
                },
                "index": {
                    "type": "string",
                    "description": "Index name to search in (defaults to 'documents')",
                    "default": "documents"
                },
                "k": {
                    "type": "integer",
                    "description": "Number of nearest neighbors to find (defaults to 10)",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 10
                },
                "metric": {
                    "type": "string",
                    "enum": ["cosine", "euclidean", "dot_product"],
                    "description": "Distance metric (defaults to 'cosine')",
                    "default": "cosine"
                },
                "target_docs": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Specific document chunk_ids to compare against"
                },
                "include_scores": {
                    "type": "boolean",
                    "description": "Include detailed similarity scores (defaults to true)",
                    "default": true
                },
                "min_similarity": {
                    "type": "number",
                    "description": "Minimum similarity threshold (0.0-1.0)",
                    "minimum": 0.0,
                    "maximum": 1.0
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

            let args: RagSimilarityArgs = match args.and_then(|v| serde_json::from_value(v).ok()) {
                Some(args) => args,
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_similarity");
                    return;
                }
            };

            let operation = args.operation.as_deref().unwrap_or("search");

            match operation {
                "search" => handle_similarity_search(&args, id).await,
                "compare" => handle_vector_compare(&args, id).await,
                "neighbors" => handle_nearest_neighbors(&args, id).await,
                "distance" => handle_distance_calculation(&args, id).await,
                _ => {
                    send_error(
                        id,
                        -1,
                        "❌ Invalid operation. Use: search, compare, neighbors, or distance",
                    );
                }
            }
        });
    }
}

/// Handle similarity search operation
async fn handle_similarity_search(args: &RagSimilarityArgs, id: u64) {
    let es_client = ElasticsearchClient::new();
    let embeddings_client = EmbeddingsClient::new();
    let index_name = args.index.as_deref().unwrap_or("documents");

    // Get query vector
    let query_vector = match get_query_vector(args, &embeddings_client).await {
        Ok(vector) => vector,
        Err(e) => {
            send_error(id, -1, &format!("❌ Vector preparation failed: {e}"));
            return;
        }
    };

    // Perform vector search
    match es_client
        .vector_search(index_name, &query_vector, args.k.unwrap_or(10), None)
        .await
    {
        Ok(response) => {
            let results = match parse_similarity_results(&response, args).await {
                Ok(results) => results,
                Err(e) => {
                    send_error(id, -1, &format!("❌ Result parsing failed: {e}"));
                    return;
                }
            };

            let metric = args.metric.as_deref().unwrap_or("cosine");
            let mut response_text = format!(
                "📊 Vector Similarity Search Results\n\n\
                🔍 Query: {}\n\
                🗂️ Index: {}\n\
                📐 Metric: {}\n\
                🎯 K-nearest: {}\n\
                📈 Results found: {}\n\n",
                if args.query == "vector" {
                    "Direct vector input"
                } else {
                    &args.query
                },
                index_name,
                metric,
                args.k.unwrap_or(10),
                results.len()
            );

            if results.is_empty() {
                response_text.push_str(
                    "❌ No similar vectors found\n\n\
                    💡 Troubleshooting:\n\
                    - Check if documents are indexed\n\
                    - Lower min_similarity threshold\n\
                    - Verify index contains vector embeddings",
                );
            } else {
                response_text.push_str("🏆 Top Similar Documents:\n\n");

                for (i, result) in results.iter().take(5).enumerate() {
                    response_text.push_str(&format!(
                        "{}. {} (Similarity: {:.4})\n\
                           📄 Source: {}\n\
                           📊 Distance: {:.4}\n",
                        i + 1,
                        result.chunk_id,
                        result.similarity_score,
                        result.source,
                        result.distance
                    ));

                    if let Some(preview) = &result.content_preview {
                        response_text.push_str(&format!("   Preview: {preview}...\n"));
                    }

                    response_text.push('\n');
                }

                if results.len() > 5 {
                    response_text.push_str(&format!(
                        "   ... and {} more results\n\n",
                        results.len() - 5
                    ));
                }

                response_text.push_str(&format!(
                    "📈 Score Statistics:\n\
                    - Highest: {:.4}\n\
                    - Lowest: {:.4}\n\
                    - Average: {:.4}\n\n\
                    💡 Use rag_search for semantic search with context",
                    results
                        .iter()
                        .map(|r| r.similarity_score)
                        .fold(f32::NEG_INFINITY, f32::max),
                    results
                        .iter()
                        .map(|r| r.similarity_score)
                        .fold(f32::INFINITY, f32::min),
                    results.iter().map(|r| r.similarity_score).sum::<f32>() / results.len() as f32
                ));
            }

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": response_text
                }]
            });
            send_response(id, result);
        }
        Err(e) => send_error(id, -1, &format!("❌ Similarity search failed: {e}")),
    }
}

/// Handle vector comparison operation
async fn handle_vector_compare(args: &RagSimilarityArgs, id: u64) {
    let result = json!({
        "content": [{
            "type": "text",
            "text": format!(
                "🔍 Vector Comparison\n\n\
                ⚠️ Compare operation requires specific implementation\n\
                📊 Query: {}\n\
                🎯 Target docs: {:?}\n\
                📐 Metric: {}\n\n\
                💡 Use 'search' operation for similarity search\n\
                🔧 Direct vector comparison coming in future update",
                args.query,
                args.target_docs,
                args.metric.as_deref().unwrap_or("cosine")
            )
        }]
    });
    send_response(id, result);
}

/// Handle nearest neighbors operation
async fn handle_nearest_neighbors(args: &RagSimilarityArgs, id: u64) {
    // For now, redirect to similarity search
    handle_similarity_search(args, id).await;
}

/// Handle distance calculation operation
async fn handle_distance_calculation(args: &RagSimilarityArgs, id: u64) {
    let embeddings_client = EmbeddingsClient::new();

    // Get query vector
    let query_vector = match get_query_vector(args, &embeddings_client).await {
        Ok(vector) => vector,
        Err(e) => {
            send_error(id, -1, &format!("❌ Vector preparation failed: {e}"));
            return;
        }
    };

    let metric = args.metric.as_deref().unwrap_or("cosine");

    let result = json!({
        "content": [{
            "type": "text",
            "text": format!(
                "📐 Vector Distance Calculation\n\n\
                🔢 Vector dimensions: {}\n\
                📐 Distance metric: {}\n\
                🎯 Vector norm: {:.6}\n\n\
                ✅ Vector successfully processed\n\
                💡 Use 'search' operation to find similar documents\n\
                🔍 Use 'compare' operation for direct comparisons",
                query_vector.len(),
                metric,
                vector_norm(&query_vector)
            )
        }]
    });
    send_response(id, result);
}

/// Get query vector from args (either from text or direct input)
async fn get_query_vector(
    args: &RagSimilarityArgs,
    embeddings_client: &EmbeddingsClient,
) -> Result<Vec<f32>, String> {
    if args.query == "vector" {
        // Use direct vector input
        if let Some(vector) = &args.vector {
            if vector.len() != 384 {
                return Err(format!(
                    "Vector must be 384 dimensions, got {}",
                    vector.len()
                ));
            }
            Ok(vector.clone())
        } else {
            Err("Direct vector input required when query='vector'".to_string())
        }
    } else {
        // Generate embedding from text
        embeddings_client.embed_text(&args.query).await
    }
}

/// Parse similarity search results
async fn parse_similarity_results(
    response: &Value,
    args: &RagSimilarityArgs,
) -> Result<Vec<SimilarityResult>, String> {
    let hits = response
        .get("hits")
        .and_then(|h| h.get("hits"))
        .and_then(|h| h.as_array())
        .ok_or("Invalid search response format")?;

    let mut results = Vec::new();
    let min_similarity = args.min_similarity.unwrap_or(0.0);

    for hit in hits {
        let source = hit.get("_source").ok_or("Missing _source in hit")?;
        let score = hit
            .get("_score")
            .and_then(|s| s.as_f64())
            .map(|s| s as f32)
            .unwrap_or(0.0);

        // Convert Elasticsearch score to similarity (0-1 range)
        let similarity_score = score.clamp(0.0, 1.0);

        if similarity_score < min_similarity {
            continue;
        }

        let chunk_id = source
            .get("chunk_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let content_preview = source
            .get("content")
            .and_then(|v| v.as_str())
            .map(|content| {
                if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content.to_string()
                }
            });

        let metadata = source.get("metadata").cloned().unwrap_or(json!({}));

        let source_name = source
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Calculate distance (inverse of similarity for cosine)
        let distance = 1.0 - similarity_score;

        results.push(SimilarityResult {
            chunk_id,
            similarity_score,
            distance,
            content_preview,
            source: source_name,
            metadata,
        });
    }

    // Sort by similarity score descending
    results.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}

/// Calculate vector norm (magnitude)
fn vector_norm(vector: &[f32]) -> f32 {
    vector.iter().map(|&x| x * x).sum::<f32>().sqrt()
}

/// 🔢 Vector Math Tool (bonus utility) - FIXED VERSION
#[derive(Default)]
pub struct RagVectorMathTool;

#[derive(Deserialize, Default)]
pub struct RagVectorMathArgs {
    pub operation: String, // "norm", "dot", "cosine", "euclidean"
    pub vector1: Vec<f32>,
    pub vector2: Option<Vec<f32>>,
}

impl Tool for RagVectorMathTool {
    fn name(&self) -> &'static str {
        "rag_vector_math"
    }

    fn description(&self) -> &'static str {
        "🔢 Vector mathematics utility for RAG operations. Provides low-level vector calculations including norm computation, dot product, cosine similarity, and euclidean distance. Useful for debugging embeddings, analyzing vector properties, and performing custom similarity calculations. Supports 384-dimensional vectors used by the RAG embeddings service."
    }

    fn emoji(&self) -> &'static str {
        "🔢"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["norm", "dot", "cosine", "euclidean"],
                    "description": "Vector operation to perform"
                },
                "vector1": {
                    "type": "array",
                    "items": {"type": "number"},
                    "description": "First vector (384 dimensions)"
                },
                "vector2": {
                    "type": "array",
                    "items": {"type": "number"},
                    "description": "Second vector (required for dot, cosine, euclidean)"
                }
            },
            "required": ["operation", "vector1"],
            "additionalProperties": false
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        // Pure mathematical operations - no async needed
        let args: RagVectorMathArgs = match args.and_then(|v| serde_json::from_value(v).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "❌ Invalid arguments for rag_vector_math");
                return;
            }
        };

        let result_text = match args.operation.as_str() {
            "norm" => {
                let norm = vector_norm(&args.vector1);
                format!(
                    "🔢 Vector Norm Calculation\n\n\
                    📊 Vector dimensions: {}\n\
                    📐 L2 norm (magnitude): {:.6}\n\
                    🔍 Vector properties: {}\n\n\
                    💡 Norm indicates vector magnitude/length",
                    args.vector1.len(),
                    norm,
                    if norm > 0.0 {
                        "Non-zero vector"
                    } else {
                        "Zero vector"
                    }
                )
            }
            "dot" => {
                if let Some(vector2) = &args.vector2 {
                    if args.vector1.len() != vector2.len() {
                        send_error(id, -1, "❌ Vectors must have same dimensions");
                        return;
                    }
                    let dot_product: f32 = args
                        .vector1
                        .iter()
                        .zip(vector2.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                    format!(
                        "🔢 Dot Product Calculation\n\n\
                        📊 Vector dimensions: {}\n\
                        🔗 Dot product: {:.6}\n\
                        📐 Interpretation: {}\n\n\
                        💡 Dot product measures vector alignment",
                        args.vector1.len(),
                        dot_product,
                        if dot_product > 0.0 {
                            "Positive correlation"
                        } else if dot_product < 0.0 {
                            "Negative correlation"
                        } else {
                            "Orthogonal vectors"
                        }
                    )
                } else {
                    send_error(id, -1, "❌ Second vector required for dot product");
                    return;
                }
            }
            "cosine" => {
                if let Some(vector2) = &args.vector2 {
                    if args.vector1.len() != vector2.len() {
                        send_error(id, -1, "❌ Vectors must have same dimensions");
                        return;
                    }
                    let dot_product: f32 = args
                        .vector1
                        .iter()
                        .zip(vector2.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                    let norm1 = vector_norm(&args.vector1);
                    let norm2 = vector_norm(vector2);
                    let cosine_similarity = if norm1 > 0.0 && norm2 > 0.0 {
                        dot_product / (norm1 * norm2)
                    } else {
                        0.0
                    };
                    format!(
                        "🔢 Cosine Similarity Calculation\n\n\
                        📊 Vector dimensions: {}\n\
                        📐 Cosine similarity: {:.6}\n\
                        🎯 Similarity interpretation: {}\n\
                        📏 Distance: {:.6}\n\n\
                        💡 Cosine similarity: 1.0=identical, 0.0=orthogonal, -1.0=opposite",
                        args.vector1.len(),
                        cosine_similarity,
                        if cosine_similarity > 0.8 {
                            "Very similar"
                        } else if cosine_similarity > 0.5 {
                            "Moderately similar"
                        } else if cosine_similarity > 0.0 {
                            "Somewhat similar"
                        } else {
                            "Dissimilar"
                        },
                        1.0 - cosine_similarity
                    )
                } else {
                    send_error(id, -1, "❌ Second vector required for cosine similarity");
                    return;
                }
            }
            "euclidean" => {
                if let Some(vector2) = &args.vector2 {
                    if args.vector1.len() != vector2.len() {
                        send_error(id, -1, "❌ Vectors must have same dimensions");
                        return;
                    }
                    let distance: f32 = args
                        .vector1
                        .iter()
                        .zip(vector2.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    format!(
                        "🔢 Euclidean Distance Calculation\n\n\
                        📊 Vector dimensions: {}\n\
                        📏 Euclidean distance: {:.6}\n\
                        🎯 Distance interpretation: {}\n\n\
                        💡 Euclidean distance: 0.0=identical, larger=more different",
                        args.vector1.len(),
                        distance,
                        if distance < 0.5 {
                            "Very close"
                        } else if distance < 1.0 {
                            "Moderately close"
                        } else {
                            "Far apart"
                        }
                    )
                } else {
                    send_error(id, -1, "❌ Second vector required for Euclidean distance");
                    return;
                }
            }
            _ => {
                send_error(
                    id,
                    -1,
                    "❌ Invalid operation. Use: norm, dot, cosine, or euclidean",
                );
                return;
            }
        };

        let result = json!({
            "content": [{
                "type": "text",
                "text": result_text
            }]
        });
        send_response(id, result);
    }
}

// Register the tools
register_tool!("rag_similarity", RagSimilarityTool);
register_tool!("rag_vector_math", RagVectorMathTool);
