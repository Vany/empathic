use crate::{
    common::{send_error, send_response},
    rag::{
        rag_client::ElasticsearchClient,
        rag_stack::ensure_rag_stack_running,
    },
    tools::tool_trait::Tool,
};
use crate::register_tool;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// 🗂️ Index Management Tool
/// Manages Elasticsearch indices for RAG operations
#[derive(Default)]
pub struct RagIndexManageTool;

#[derive(Deserialize, Default)]
pub struct RagIndexManageArgs {
    /// Operation: "create", "delete", "list", "stats", "health"
    pub operation: String,
    /// Index name (required for create, delete, stats)
    pub index: Option<String>,
    /// Custom index settings for create operation
    pub settings: Option<Value>,
    /// Force deletion (required for delete operation)
    pub force: Option<bool>,
}

#[derive(Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub health: String,
    pub status: String,
    pub docs_count: Option<u64>,
    pub store_size: Option<String>,
    pub creation_date: Option<String>,
}

impl Tool for RagIndexManageTool {
    fn name(&self) -> &'static str {
        "rag_index_manage"
    }

    fn description(&self) -> &'static str {
        "🗂️ Index management tool for RAG operations. Provides comprehensive Elasticsearch index lifecycle management including: 1) Index creation with optimized vector search mappings, 2) Index deletion with safety checks, 3) Index listing with health and statistics, 4) Index health monitoring and diagnostics, 5) Custom mapping and settings configuration. Features automatic vector field configuration for 384-dimensional embeddings, cosine similarity optimization, and production-ready index settings. Essential for maintaining RAG document storage and ensuring optimal search performance."
    }

    fn emoji(&self) -> &'static str {
        "🗂️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["create", "delete", "list", "stats", "health"],
                    "description": "Index operation to perform"
                },
                "index": {
                    "type": "string",
                    "description": "Index name (required for create, delete, stats operations)"
                },
                "settings": {
                    "type": "object",
                    "description": "Custom index settings and mappings for create operation",
                    "additionalProperties": true
                },
                "force": {
                    "type": "boolean",
                    "description": "Force deletion without confirmation (required for delete)",
                    "default": false
                }
            },
            "required": ["operation"],
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

            let args: RagIndexManageArgs = match args.and_then(|v| serde_json::from_value(v).ok()) {
                Some(args) => args,
                None => {
                    send_error(id, -1, "❌ Invalid arguments for rag_index_manage");
                    return;
                }
            };
            let es_client = ElasticsearchClient::new();

            match args.operation.as_str() {
                "create" => handle_create_index(&es_client, &args, id).await,
                "delete" => handle_delete_index(&es_client, &args, id).await,
                "list" => handle_list_indices(&es_client, id).await,
                "stats" => handle_index_stats(&es_client, &args, id).await,
                "health" => handle_cluster_health(&es_client, id).await,
                _ => {
                    send_error(
                        id,
                        -1,
                        "❌ Invalid operation. Use: create, delete, list, stats, or health",
                    );
                }
            }
        });
    }
}

/// Handle index creation
async fn handle_create_index(es_client: &ElasticsearchClient, args: &RagIndexManageArgs, id: u64) {
    let index_name = match &args.index {
        Some(name) if !name.trim().is_empty() => name.trim(),
        _ => {
            send_error(id, -1, "❌ Index name is required for create operation");
            return;
        }
    };

    // Check if index already exists
    match es_client.list_indices().await {
        Ok(indices) => {
            if let Some(indices_array) = indices.as_array() {
                for index_info in indices_array {
                    if let Some(name) = index_info.get("index").and_then(|v| v.as_str()) {
                        if name == index_name {
                            send_error(id, -1, &format!("❌ Index '{index_name}' already exists"));
                            return;
                        }
                    }
                }
            }
        }
        Err(e) => {
            send_error(id, -1, &format!("❌ Failed to check existing indices: {e}"));
            return;
        }
    }

    // Create index
    match es_client
        .create_index(index_name, args.settings.as_ref())
        .await
    {
        Ok(response) => {
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🗂️ Index Created Successfully!\n\n\
                        ✅ Index: {}\n\
                        🔧 Configuration:\n\
                        - Vector field: embedding (384 dimensions)\n\
                        - Similarity: cosine\n\
                        - Content field: text with standard analyzer\n\
                        - Metadata field: object (dynamic)\n\
                        - Shards: 1 (development optimized)\n\
                        - Replicas: 0 (single node)\n\n\
                        📊 Creation Response:\n{}\n\n\
                        🚀 Ready for document ingestion!\n\
                        💡 Use rag_ingest to add documents to this index",
                        index_name,
                        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Response unavailable".to_string())
                    )
                }]
            });
            send_response(id, result);
        }
        Err(e) => send_error(
            id,
            -1,
            &format!("❌ Failed to create index '{index_name}': {e}"),
        ),
    }
}

/// Handle index deletion
async fn handle_delete_index(es_client: &ElasticsearchClient, args: &RagIndexManageArgs, id: u64) {
    let index_name = match &args.index {
        Some(name) if !name.trim().is_empty() => name.trim(),
        _ => {
            send_error(id, -1, "❌ Index name is required for delete operation");
            return;
        }
    };

    if !args.force.unwrap_or(false) {
        send_error(
            id,
            -1,
            "❌ Force confirmation required for deletion. Set 'force': true",
        );
        return;
    }

    // Get index stats before deletion
    let stats_before = es_client
        .index_stats(index_name)
        .await
        .unwrap_or_else(|_| json!({}));
    let doc_count = stats_before
        .get("indices")
        .and_then(|i| i.get(index_name))
        .and_then(|idx| idx.get("total"))
        .and_then(|t| t.get("docs"))
        .and_then(|d| d.get("count"))
        .and_then(|c| c.as_u64())
        .unwrap_or(0);

    match es_client.delete_index(index_name).await {
        Ok(_) => {
            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🗑️ Index Deleted Successfully!\n\n\
                        ✅ Deleted: {}\n\
                        📊 Documents removed: {}\n\
                        💾 Vector embeddings deleted\n\
                        🧹 Storage space freed\n\n\
                        ⚠️  This action is irreversible!\n\
                        💡 Create a new index with: rag_index_manage operation=create",
                        index_name,
                        doc_count
                    )
                }]
            });
            send_response(id, result);
        }
        Err(e) => send_error(
            id,
            -1,
            &format!("❌ Failed to delete index '{index_name}': {e}"),
        ),
    }
}

/// Handle listing all indices
/// Handle listing all indices
async fn handle_list_indices(es_client: &ElasticsearchClient, id: u64) {
    match es_client.list_indices().await {
        Ok(indices) => {
            let empty_vec = vec![];
            let indices_array = indices.as_array().unwrap_or(&empty_vec);

            if indices_array.is_empty() {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": "📋 No Indices Found\n\n\
                                🗂️ No Elasticsearch indices exist yet.\n\n\
                                💡 Create your first index:\n\
                                - Use rag_index_manage with operation=create\n\
                                - Or use rag_ingest (creates index automatically)\n\n\
                                🚀 Example: rag_index_manage operation=create index=documents"
                    }]
                });
                send_response(id, result);
                return;
            }

            let mut response = "📋 Elasticsearch Indices\n\n".to_string();

            for (i, index_info) in indices_array.iter().enumerate() {
                let name = index_info
                    .get("index")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let health = index_info
                    .get("health")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let status = index_info
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let docs_count = index_info
                    .get("docs.count")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                let store_size = index_info
                    .get("store.size")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0b");

                let health_emoji = match health {
                    "green" => "🟢",
                    "yellow" => "🟡",
                    "red" => "🔴",
                    _ => "⚪",
                };

                response.push_str(&format!(
                    "{}. {} {} ({})\n\
                       📊 Documents: {} | Size: {}\n\
                       🔍 Status: {}\n\n",
                    i + 1,
                    health_emoji,
                    name,
                    health,
                    docs_count,
                    store_size,
                    status
                ));
            }

            response.push_str(&format!(
                "📈 Summary: {} indices total\n\n\
                🔧 Index Operations:\n\
                - Stats: rag_index_manage operation=stats index=NAME\n\
                - Delete: rag_index_manage operation=delete index=NAME force=true\n\
                - Health: rag_index_manage operation=health",
                indices_array.len()
            ));

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": response
                }]
            });
            send_response(id, result);
        }
        Err(e) => send_error(id, -1, &format!("❌ Failed to list indices: {e}")),
    }
}
/// Handle index statistics
async fn handle_index_stats(es_client: &ElasticsearchClient, args: &RagIndexManageArgs, id: u64) {
    let index_name = match &args.index {
        Some(name) if !name.trim().is_empty() => name.trim(),
        _ => {
            send_error(id, -1, "❌ Index name is required for stats operation");
            return;
        }
    };

    match es_client.index_stats(index_name).await {
        Ok(stats) => {
            let index_stats = stats
                .get("indices")
                .and_then(|i| i.get(index_name))
                .and_then(|idx| idx.get("total"));

            if let Some(total_stats) = index_stats {
                let docs_count = total_stats
                    .get("docs")
                    .and_then(|d| d.get("count"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                let docs_deleted = total_stats
                    .get("docs")
                    .and_then(|d| d.get("deleted"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                let store_size = total_stats
                    .get("store")
                    .and_then(|s| s.get("size_in_bytes"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                let segments_count = total_stats
                    .get("segments")
                    .and_then(|s| s.get("count"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0);

                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "📊 Index Statistics: {}\n\n\
                            📄 Documents:\n\
                            - Active: {}\n\
                            - Deleted: {}\n\
                            - Total: {}\n\n\
                            💾 Storage:\n\
                            - Size: {} bytes ({:.2} MB)\n\
                            - Segments: {}\n\n\
                            🔍 Detailed Stats:\n{}\n\n\
                            💡 Operations:\n\
                            - Search: rag_search index={}\n\
                            - Add docs: rag_ingest index={}\n\
                            - List all: rag_index_manage operation=list",
                            index_name,
                            docs_count,
                            docs_deleted,
                            docs_count + docs_deleted,
                            store_size,
                            store_size as f64 / 1024.0 / 1024.0,
                            segments_count,
                            serde_json::to_string_pretty(&stats).unwrap_or_else(|_| "Stats unavailable".to_string()),
                            index_name,
                            index_name
                        )
                    }]
                });
                send_response(id, result);
            } else {
                send_error(
                    id,
                    -1,
                    &format!("❌ Index '{index_name}' not found or no stats available"),
                );
            }
        }
        Err(e) => send_error(
            id,
            -1,
            &format!("❌ Failed to get stats for index '{index_name}': {e}"),
        ),
    }
}

/// Handle cluster health check
async fn handle_cluster_health(es_client: &ElasticsearchClient, id: u64) {
    match es_client.cluster_health().await {
        Ok(health) => {
            let status = health
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");

            let cluster_name = health
                .get("cluster_name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");

            let number_of_nodes = health
                .get("number_of_nodes")
                .and_then(|n| n.as_u64())
                .unwrap_or(0);

            let active_primary_shards = health
                .get("active_primary_shards")
                .and_then(|s| s.as_u64())
                .unwrap_or(0);

            let active_shards = health
                .get("active_shards")
                .and_then(|s| s.as_u64())
                .unwrap_or(0);

            let status_emoji = match status {
                "green" => "🟢",
                "yellow" => "🟡",
                "red" => "🔴",
                _ => "⚪",
            };

            let result = json!({
                "content": [{
                    "type": "text",
                    "text": format!(
                        "🏥 Elasticsearch Cluster Health\n\n\
                        {} Status: {}\n\
                        🏷️  Cluster: {}\n\
                        🖥️  Nodes: {}\n\
                        📊 Shards: {} active ({} primary)\n\n\
                        📋 Detailed Health Report:\n{}\n\n\
                        ✅ Cluster is {} for RAG operations\n\n\
                        💡 Next Steps:\n\
                        - List indices: rag_index_manage operation=list\n\
                        - Create index: rag_index_manage operation=create index=NAME\n\
                        - Check specific index: rag_index_manage operation=stats index=NAME",
                        status_emoji,
                        status,
                        cluster_name,
                        number_of_nodes,
                        active_shards,
                        active_primary_shards,
                        serde_json::to_string_pretty(&health).unwrap_or_else(|_| "Health data unavailable".to_string()),
                        if status == "green" || status == "yellow" { "ready" } else { "not ready" }
                    )
                }]
            });
            send_response(id, result);
        }
        Err(e) => send_error(id, -1, &format!("❌ Failed to get cluster health: {e}")),
    }
}

// Register the tool
register_tool!("rag_index_manage", RagIndexManageTool);
