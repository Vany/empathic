use super::tool_trait::ToolRegistry;
use serde_json::Value;
// Import modules to trigger inventory registration 🏭
#[allow(unused_imports)]
use super::{
    comm_tools, file_tools, memory_tools, rag_tools,
};

#[allow(unused_imports)]
use crate::rag::{
    rag_indices, rag_ingestion, rag_search, rag_vectors,
};

// Import our new executor tools to trigger registration
#[allow(unused_imports)]
use crate::tools::executor;
/// 🏭 Create the global tool registry with all available tools
pub fn create_tool_registry() -> ToolRegistry {
    // Auto-registers all tools via inventory
    ToolRegistry::new()
}

/// Get tools schema for MCP protocol
pub fn get_tools_schema() -> Value {
    create_tool_registry().get_schema()
}

/// Execute tool by name
pub fn execute_tool(name: &str, id: u64, args: Option<Value>) -> bool {
    create_tool_registry().execute(name, id, args)
}