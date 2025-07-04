use std::env;
use std::path::PathBuf;
use crate::log_debug;

/// 🛡️ Get secure RAG data directory using ROOT_DIR
pub fn get_rag_data_dir() -> Result<PathBuf, String> {
    let root = env::var("ROOT_DIR")
        .or_else(|_| env::var("PROJECT_DIR"))
        .map_err(|_| "❌ No ROOT_DIR/PROJECT_DIR environment variable set. Set with: export ROOT_DIR=\"$(pwd)\"")?;
    
    let rag_path = PathBuf::from(root).join("rag_data");
    
    // Ensure directory exists
    std::fs::create_dir_all(&rag_path)
        .map_err(|e| format!("Failed to create RAG data directory at {}: {e}", rag_path.display()))?;
    
    log_debug!("rag", "🗂️ RAG data directory: {}", rag_path.display());
    Ok(rag_path)
}

/// 🔍 Check if RAG data directory exists and is accessible
pub fn rag_data_dir_exists() -> bool {
    get_rag_data_dir()
        .map(|path| path.exists())
        .unwrap_or(false)
}
