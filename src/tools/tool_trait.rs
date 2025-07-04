use serde_json::Value;
use std::collections::HashMap;

/// 🛠️ Core trait for all editor tools
pub trait Tool: Send + Sync {
    /// Tool identifier for MCP protocol
    fn name(&self) -> &'static str;

    /// Tool description for schema
    fn description(&self) -> &'static str;

    /// Tool category emoji for display
    fn emoji(&self) -> &'static str {
        "🔧"
    }

    /// JSON schema for tool parameters
    fn schema(&self) -> Value;

    /// Execute tool with given arguments
    fn execute(&self, id: u64, args: Option<Value>) {
        self.execute_impl(id, args);
    }

    /// Actual execution implementation (override this)
    fn execute_impl(&self, id: u64, args: Option<Value>);
}

/// 📦 Tool registration entry for inventory
/// 📦 Tool registration entry for inventory
pub struct ToolEntry {
    pub name: &'static str,
    pub factory: fn() -> Box<dyn Tool>,
}

impl ToolEntry {
    pub const fn new(name: &'static str, factory: fn() -> Box<dyn Tool>) -> Self {
        Self { name, factory }
    }
}

inventory::collect!(ToolEntry);

/// 📦 Registry for all available tools
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // 🏭 Auto-register all tools from inventory
        for entry in inventory::iter::<ToolEntry> {
            let tool = (entry.factory)();
            registry.tools.insert(entry.name.to_string(), tool);
        }

        registry
    }

    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Get all tools schema for MCP
    pub fn get_schema(&self) -> Value {
        let schemas: Vec<Value> = self
            .tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": format!("{} {}", tool.emoji(), tool.description()),
                    "inputSchema": tool.schema()
                })
            })
            .collect();

        serde_json::json!(schemas)
    }

    /// Execute tool by name
    pub fn execute(&self, name: &str, id: u64, args: Option<Value>) -> bool {
        if let Some(tool) = self.get_tool(name) {
            tool.execute(id, args);
            true
        } else {
            false
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 🔧 Macro for easy tool registration
#[macro_export]
macro_rules! register_tool {
    ($name:literal, $tool_type:ty) => {
        inventory::submit! {
            $crate::tools::tool_trait::ToolEntry::new(
                $name,
                || Box::new(<$tool_type>::default())
            )
        }
    };
}
