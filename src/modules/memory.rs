use serde_json::Value;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 🧠 Memory entry with metadata
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: Value,
    pub created_at: u64,
    pub accessed_at: u64,
    pub access_count: u64,
    pub tags: Vec<String>,
}

impl MemoryEntry {
    pub fn new(key: String, value: Value, tags: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            key,
            value,
            created_at: now,
            accessed_at: now,
            access_count: 1,
            tags,
        }
    }

    pub fn access(&mut self) {
        self.accessed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.access_count += 1;
    }
}

/// 🧠 Global memory store
pub struct MemoryStore {
    entries: RwLock<HashMap<String, MemoryEntry>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Store value with key and optional tags
    pub fn store(&self, key: String, value: Value, tags: Vec<String>) -> Result<(), String> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| "Failed to acquire write lock")?;

        entries.insert(key.clone(), MemoryEntry::new(key, value, tags));
        Ok(())
    }

    /// Retrieve value by key
    pub fn retrieve(&self, key: &str) -> Result<Option<Value>, String> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| "Failed to acquire write lock")?;

        if let Some(entry) = entries.get_mut(key) {
            entry.access();
            Ok(Some(entry.value.clone()))
        } else {
            Ok(None)
        }
    }

    /// Search entries by tag or key pattern
    pub fn search(&self, query: &str, search_tags: bool) -> Result<Vec<MemoryEntry>, String> {
        let entries = self
            .entries
            .read()
            .map_err(|_| "Failed to acquire read lock")?;

        let results: Vec<MemoryEntry> = entries
            .values()
            .filter(|entry| {
                if search_tags {
                    entry.tags.iter().any(|tag| tag.contains(query))
                } else {
                    entry.key.contains(query)
                }
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// List all entries with optional limit
    pub fn list(&self, limit: Option<usize>) -> Result<Vec<MemoryEntry>, String> {
        let entries = self
            .entries
            .read()
            .map_err(|_| "Failed to acquire read lock")?;

        let mut results: Vec<MemoryEntry> = entries.values().cloned().collect();
        results.sort_by(|a, b| b.accessed_at.cmp(&a.accessed_at)); // Most recent first

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Delete entry by key
    pub fn delete(&self, key: &str) -> Result<bool, String> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| "Failed to acquire write lock")?;

        Ok(entries.remove(key).is_some())
    }

    /// Clear all entries
    pub fn clear(&self) -> Result<usize, String> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| "Failed to acquire write lock")?;

        let count = entries.len();
        entries.clear();
        Ok(count)
    }

    /// Get memory statistics
    pub fn stats(&self) -> Result<MemoryStats, String> {
        let entries = self
            .entries
            .read()
            .map_err(|_| "Failed to acquire read lock")?;

        let total_entries = entries.len();
        let total_access_count: u64 = entries.values().map(|e| e.access_count).sum();

        let most_accessed = entries
            .values()
            .max_by_key(|e| e.access_count)
            .map(|e| (e.key.clone(), e.access_count));

        let oldest_entry = entries
            .values()
            .min_by_key(|e| e.created_at)
            .map(|e| e.key.clone());

        let total_tags: std::collections::HashSet<String> = entries
            .values()
            .flat_map(|e| e.tags.iter().cloned())
            .collect();

        Ok(MemoryStats {
            total_entries,
            total_access_count,
            unique_tags: total_tags.len(),
            most_accessed,
            oldest_entry,
        })
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub total_access_count: u64,
    pub unique_tags: usize,
    pub most_accessed: Option<(String, u64)>,
    pub oldest_entry: Option<String>,
}

/// Global memory store instance
static MEMORY_STORE: OnceLock<MemoryStore> = OnceLock::new();

pub fn get_memory_store() -> &'static MemoryStore {
    MEMORY_STORE.get_or_init(MemoryStore::new)
}
