//! ⚡ LSP Cache - Response caching for performance optimization
//!
//! Provides in-memory caching for LSP responses with smart invalidation based on
//! file modifications and cache TTL policies.

use crate::lsp::types::{CacheConfig, LspResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 🎯 Cache key for LSP operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    Diagnostics(PathBuf),
    Hover {
        file_path: PathBuf,
        line: u32,
        character: u32,
    },
    Completion {
        file_path: PathBuf,
        line: u32,
        character: u32,
    },
    DocumentSymbols(PathBuf),
    WorkspaceSymbols {
        query: String,
        project_path: PathBuf,
    },
}

impl CacheKey {
    /// Get the TTL for this cache key type
    pub fn ttl(&self, config: &CacheConfig) -> Duration {
        match self {
            CacheKey::Diagnostics(_) => Duration::from_secs(config.diagnostics_ttl_secs),
            CacheKey::Hover { .. } => Duration::from_secs(config.hover_ttl_secs),
            CacheKey::Completion { .. } => Duration::from_secs(config.completion_ttl_secs),
            CacheKey::DocumentSymbols(_) => Duration::from_secs(config.symbols_ttl_secs),
            CacheKey::WorkspaceSymbols { .. } => Duration::from_secs(config.symbols_ttl_secs),
        }
    }

    /// Get the file path associated with this cache key (if any)
    pub fn file_path(&self) -> Option<&Path> {
        match self {
            CacheKey::Diagnostics(path) => Some(path),
            CacheKey::Hover { file_path, .. } => Some(file_path),
            CacheKey::Completion { file_path, .. } => Some(file_path),
            CacheKey::DocumentSymbols(path) => Some(path),
            CacheKey::WorkspaceSymbols { .. } => None,
        }
    }
}

/// 🗃️ Cached entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// 💾 LSP response cache
#[derive(Debug)]
pub struct LspCache {
    /// Cache storage with dynamic values
    storage: RwLock<HashMap<CacheKey, CacheEntry<serde_json::Value>>>,
    /// Cache configuration
    config: CacheConfig,
    /// File modification times for invalidation
    file_mtimes: RwLock<HashMap<PathBuf, std::time::SystemTime>>,
}

impl LspCache {
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            storage: RwLock::new(HashMap::new()),
            config,
            file_mtimes: RwLock::new(HashMap::new()),
        }
    }

    /// 📥 Get a cached value
    pub async fn get<T>(&self, key: &CacheKey) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Check if the cache entry is still valid
        if let Some(entry) = self.get_entry(key).await
            && !entry.is_expired() && !self.is_file_modified(key).await
            && let Ok(value) = serde_json::from_value(entry.value)
        {
            return Some(value);
        }

        None
    }

    /// 📤 Store a value in the cache
    pub async fn set<T>(&self, key: CacheKey, value: T) -> LspResult<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value).map_err(|e| {
            crate::lsp::types::LspError::SerializationError { source: e }
        })?;

        let ttl = key.ttl(&self.config);
        let entry = CacheEntry::new(json_value, ttl);

        // Update file modification time if applicable
        if let Some(file_path) = key.file_path() {
            self.update_file_mtime(file_path).await;
        }

        // Store in cache
        let mut storage = self.storage.write().await;
        storage.insert(key, entry);

        Ok(())
    }

    /// 🗑️ Remove a specific cache entry
    pub async fn remove(&self, key: &CacheKey) {
        let mut storage = self.storage.write().await;
        storage.remove(key);
    }

    /// 🗑️ Remove all cache entries for a specific file
    pub async fn invalidate_file(&self, file_path: &Path) {
        let mut storage = self.storage.write().await;
        storage.retain(|key, _| {
            if let Some(key_file) = key.file_path() {
                key_file != file_path
            } else {
                true
            }
        });

        // Update file modification time
        let mut file_mtimes = self.file_mtimes.write().await;
        if let Ok(metadata) = std::fs::metadata(file_path)
            && let Ok(mtime) = metadata.modified()
        {
            file_mtimes.insert(file_path.to_path_buf(), mtime);
        }
    }

    /// 🗑️ Remove all cache entries for a project
    pub async fn invalidate_project(&self, project_path: &Path) {
        let mut storage = self.storage.write().await;
        storage.retain(|key, _| {
            match key {
                CacheKey::WorkspaceSymbols { project_path: p, .. } => p != project_path,
                _ => {
                    if let Some(file_path) = key.file_path() {
                        !file_path.starts_with(project_path)
                    } else {
                        true
                    }
                }
            }
        });
    }

    /// 🧹 Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut storage = self.storage.write().await;
        storage.retain(|_key, entry| !entry.is_expired());
    }

    /// 📊 Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let storage = self.storage.read().await;
        let total_entries = storage.len();

        let mut expired_count = 0;
        let mut by_type = HashMap::new();

        for (key, entry) in storage.iter() {
            if entry.is_expired() {
                expired_count += 1;
            }

            let key_type = match key {
                CacheKey::Diagnostics(_) => "diagnostics",
                CacheKey::Hover { .. } => "hover",
                CacheKey::Completion { .. } => "completion",
                CacheKey::DocumentSymbols(_) => "document_symbols",
                CacheKey::WorkspaceSymbols { .. } => "workspace_symbols",
            };

            *by_type.entry(key_type.to_string()).or_insert(0) += 1;
        }

        CacheStats {
            total_entries,
            expired_entries: expired_count,
            entries_by_type: by_type,
        }
    }

    /// Get a cache entry (internal)
    async fn get_entry(&self, key: &CacheKey) -> Option<CacheEntry<serde_json::Value>> {
        let storage = self.storage.read().await;
        storage.get(key).cloned()
    }

    /// Check if a file has been modified since caching
    async fn is_file_modified(&self, key: &CacheKey) -> bool {
        if let Some(file_path) = key.file_path() {
            let file_mtimes = self.file_mtimes.read().await;
            if let Some(cached_mtime) = file_mtimes.get(file_path) {
                if let Ok(metadata) = std::fs::metadata(file_path)
                    && let Ok(current_mtime) = metadata.modified()
                {
                    return current_mtime > *cached_mtime;
                }
                // File exists in cache but can't read metadata - assume modified
                return true;
            }
            // File not in cache tracking - not considered modified
            return false;
        }
        false
    }

    /// Update file modification time
    async fn update_file_mtime(&self, file_path: &Path) {
        if let Ok(metadata) = std::fs::metadata(file_path)
            && let Ok(mtime) = metadata.modified()
        {
            let mut file_mtimes = self.file_mtimes.write().await;
            file_mtimes.insert(file_path.to_path_buf(), mtime);
        }
    }
}

impl Default for LspCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 📊 Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub entries_by_type: HashMap<String, usize>,
}

