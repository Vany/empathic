use crate::common::{send_error, send_response};
use crate::modules::memory::get_memory_store;
use super::tool_trait::Tool;
use crate::register_tool;
use serde::Deserialize;
use serde_json::{Value, json};

/// 💾 Memory store tool
#[derive(Default)]
pub struct MemoryStoreTool;

#[derive(Deserialize)]
pub struct MemoryStoreArgs {
    pub key: String,
    pub value: Value,
    pub tags: Option<Vec<String>>,
}

impl Tool for MemoryStoreTool {
    fn name(&self) -> &'static str {
        "memory_store"
    }
    fn description(&self) -> &'static str {
        "Store data in memory with key and optional tags"
    }
    fn emoji(&self) -> &'static str {
        "💾"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Unique key for the memory entry"
                },
                "value": {
                    "description": "Value to store (any JSON type)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional tags for categorization"
                }
            },
            "required": ["key", "value"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args: MemoryStoreArgs = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for memory_store");
                return;
            }
        };

        let tags = args.tags.unwrap_or_default();
        let store = get_memory_store();

        match store.store(args.key.clone(), args.value, tags.clone()) {
            Ok(_) => {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("💾 Stored '{}' in memory{}",
                            args.key,
                            if tags.is_empty() { String::new() } else { format!(" with tags: {}", tags.join(", ")) }
                        )
                    }]
                });
                send_response(id, result);
            }
            Err(e) => send_error(id, -3, &format!("Failed to store memory: {e}")),
        }
    }
}

register_tool!("memory_store", MemoryStoreTool);

/// 🔍 Memory retrieve tool
#[derive(Default)]
pub struct MemoryRetrieveTool;

#[derive(Deserialize)]
pub struct MemoryRetrieveArgs {
    pub key: String,
}

impl Tool for MemoryRetrieveTool {
    fn name(&self) -> &'static str {
        "memory_retrieve"
    }
    fn description(&self) -> &'static str {
        "Retrieve data from memory by key"
    }
    fn emoji(&self) -> &'static str {
        "🔍"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Key of the memory entry to retrieve"
                }
            },
            "required": ["key"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args: MemoryRetrieveArgs = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for memory_retrieve");
                return;
            }
        };

        let store = get_memory_store();

        match store.retrieve(&args.key) {
            Ok(Some(value)) => {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🔍 Retrieved '{}': {}", args.key, serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()))
                    }]
                });
                send_response(id, result);
            }
            Ok(None) => {
                send_error(id, -1, &format!("Memory key '{}' not found", args.key));
            }
            Err(e) => send_error(id, -3, &format!("Failed to retrieve memory: {e}")),
        }
    }
}

register_tool!("memory_retrieve", MemoryRetrieveTool);

/// 🔎 Memory search tool
#[derive(Default)]
pub struct MemorySearchTool;

#[derive(Deserialize)]
pub struct MemorySearchArgs {
    pub query: String,
    pub search_tags: Option<bool>,
    pub limit: Option<usize>,
}

impl Tool for MemorySearchTool {
    fn name(&self) -> &'static str {
        "memory_search"
    }
    fn description(&self) -> &'static str {
        "Search memory entries by key or tags"
    }
    fn emoji(&self) -> &'static str {
        "🔎"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "search_tags": {
                    "type": "boolean",
                    "default": false,
                    "description": "Search in tags instead of keys"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results"
                }
            },
            "required": ["query"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args: MemorySearchArgs = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for memory_search");
                return;
            }
        };

        let store = get_memory_store();
        let search_tags = args.search_tags.unwrap_or(false);

        match store.search(&args.query, search_tags) {
            Ok(mut results) => {
                if let Some(limit) = args.limit {
                    results.truncate(limit);
                }

                if results.is_empty() {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!("🔎 No memory entries found for '{}' in {}",
                                args.query,
                                if search_tags { "tags" } else { "keys" }
                            )
                        }]
                    });
                    send_response(id, result);
                } else {
                    let mut output = format!(
                        "🔎 Found {} memory entries for '{}':\n\n",
                        results.len(),
                        args.query
                    );

                    for entry in results {
                        output.push_str(&format!(
                            "🔑 {}\n📊 Accessed {} times\n🏷️ Tags: {}\n💾 Value: {}\n\n",
                            entry.key,
                            entry.access_count,
                            if entry.tags.is_empty() {
                                "none".to_string()
                            } else {
                                entry.tags.join(", ")
                            },
                            serde_json::to_string_pretty(&entry.value)
                                .unwrap_or_else(|_| entry.value.to_string())
                        ));
                    }

                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": output
                        }]
                    });
                    send_response(id, result);
                }
            }
            Err(e) => send_error(id, -3, &format!("Failed to search memory: {e}")),
        }
    }
}

register_tool!("memory_search", MemorySearchTool);

/// 📋 Memory list tool
#[derive(Default)]
pub struct MemoryListTool;

#[derive(Deserialize)]
pub struct MemoryListArgs {
    pub limit: Option<usize>,
}

impl Tool for MemoryListTool {
    fn name(&self) -> &'static str {
        "memory_list"
    }
    fn description(&self) -> &'static str {
        "List all memory entries"
    }
    fn emoji(&self) -> &'static str {
        "📋"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of entries to list"
                }
            }
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args: MemoryListArgs = args
            .and_then(|a| serde_json::from_value(a).ok())
            .unwrap_or(MemoryListArgs { limit: None });

        let store = get_memory_store();

        match store.list(args.limit) {
            Ok(entries) => {
                if entries.is_empty() {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": "📋 Memory is empty"
                        }]
                    });
                    send_response(id, result);
                } else {
                    let mut output = format!("📋 Memory entries ({} total):\n\n", entries.len());

                    for entry in entries {
                        output.push_str(&format!(
                            "🔑 {} (accessed {} times)\n🏷️ {}\n\n",
                            entry.key,
                            entry.access_count,
                            if entry.tags.is_empty() {
                                "No tags".to_string()
                            } else {
                                entry.tags.join(", ")
                            }
                        ));
                    }

                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": output
                        }]
                    });
                    send_response(id, result);
                }
            }
            Err(e) => send_error(id, -3, &format!("Failed to list memory: {e}")),
        }
    }
}

register_tool!("memory_list", MemoryListTool);

/// 🗑️ Memory delete tool
#[derive(Default)]
pub struct MemoryDeleteTool;

#[derive(Deserialize)]
pub struct MemoryDeleteArgs {
    pub key: String,
}

impl Tool for MemoryDeleteTool {
    fn name(&self) -> &'static str {
        "memory_delete"
    }
    fn description(&self) -> &'static str {
        "Delete memory entry by key"
    }
    fn emoji(&self) -> &'static str {
        "🗑️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "Key of the memory entry to delete"
                }
            },
            "required": ["key"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args: MemoryDeleteArgs = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for memory_delete");
                return;
            }
        };

        let store = get_memory_store();

        match store.delete(&args.key) {
            Ok(true) => {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🗑️ Deleted memory entry '{}'", args.key)
                    }]
                });
                send_response(id, result);
            }
            Ok(false) => {
                send_error(id, -1, &format!("Memory key '{}' not found", args.key));
            }
            Err(e) => send_error(id, -3, &format!("Failed to delete memory: {e}")),
        }
    }
}

register_tool!("memory_delete", MemoryDeleteTool);

/// 🧹 Memory clear tool
#[derive(Default)]
pub struct MemoryClearTool;

impl Tool for MemoryClearTool {
    fn name(&self) -> &'static str {
        "memory_clear"
    }
    fn description(&self) -> &'static str {
        "Clear all memory entries"
    }
    fn emoji(&self) -> &'static str {
        "🧹"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        let store = get_memory_store();

        match store.clear() {
            Ok(count) => {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🧹 Cleared {} memory entries", count)
                    }]
                });
                send_response(id, result);
            }
            Err(e) => send_error(id, -3, &format!("Failed to clear memory: {e}")),
        }
    }
}

register_tool!("memory_clear", MemoryClearTool);

/// 📊 Memory stats tool
#[derive(Default)]
pub struct MemoryStatsTool;

impl Tool for MemoryStatsTool {
    fn name(&self) -> &'static str {
        "memory_stats"
    }
    fn description(&self) -> &'static str {
        "Get memory usage statistics"
    }
    fn emoji(&self) -> &'static str {
        "📊"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        let store = get_memory_store();

        match store.stats() {
            Ok(stats) => {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "📊 Memory Statistics\n\
                            ===================\n\
                            📦 Total entries: {}\n\
                            👁️ Total accesses: {}\n\
                            🏷️ Unique tags: {}\n\
                            🔥 Most accessed: {}\n\
                            ⏰ Oldest entry: {}",
                            stats.total_entries,
                            stats.total_access_count,
                            stats.unique_tags,
                            stats.most_accessed.map_or("none".to_string(), |(key, count)| format!("{key} ({count} accesses)")),
                            stats.oldest_entry.unwrap_or_else(|| "none".to_string())
                        )
                    }]
                });
                send_response(id, result);
            }
            Err(e) => send_error(id, -3, &format!("Failed to get memory stats: {e}")),
        }
    }
}

register_tool!("memory_stats", MemoryStatsTool);
