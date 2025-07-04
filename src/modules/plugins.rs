use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// 🔌 Plugin trait for external tools
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    fn execute(&self, args: Option<Value>) -> Result<Value, String>;
}

/// 🔌 Plugin registry for dynamic loading
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin
    pub fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), String> {
        let name = plugin.name().to_string();
        let mut plugins = self
            .plugins
            .write()
            .map_err(|_| "Failed to acquire write lock")?;

        if plugins.contains_key(&name) {
            return Err(format!("Plugin '{name}' already registered"));
        }

        plugins.insert(name, plugin);
        Ok(())
    }

    /// Execute plugin by name
    pub fn execute(&self, name: &str, args: Option<Value>) -> Result<Value, String> {
        let plugins = self
            .plugins
            .read()
            .map_err(|_| "Failed to acquire read lock")?;

        match plugins.get(name) {
            Some(plugin) => plugin.execute(args),
            None => Err(format!("Plugin '{name}' not found")),
        }
    }

    /// List all plugins
    pub fn list(&self) -> Result<Vec<PluginInfo>, String> {
        let plugins = self
            .plugins
            .read()
            .map_err(|_| "Failed to acquire read lock")?;

        let info: Vec<PluginInfo> = plugins
            .values()
            .map(|p| PluginInfo {
                name: p.name().to_string(),
                version: p.version().to_string(),
                description: p.description().to_string(),
                schema: p.schema(),
            })
            .collect();

        Ok(info)
    }
}

#[derive(Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    #[allow(dead_code)] // Used in plugin system JSON serialization
    pub schema: Value,
}

/// Global plugin registry
static PLUGIN_REGISTRY: OnceLock<PluginRegistry> = OnceLock::new();

pub fn get_plugin_registry() -> &'static PluginRegistry {
    PLUGIN_REGISTRY.get_or_init(PluginRegistry::new)
}

/// 🔌 Example plugin implementation
pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str {
        "example"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn description(&self) -> &'static str {
        "Example plugin for testing"
    }

    fn schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo back"
                }
            },
            "required": ["message"]
        })
    }

    fn execute(&self, args: Option<Value>) -> Result<Value, String> {
        let message = args
            .as_ref()
            .and_then(|a| a.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Hello from plugin!");

        Ok(serde_json::json!({
            "response": format!("🔌 Plugin says: {}", message),
            "plugin": self.name(),
            "version": self.version()
        }))
    }
}

/// Auto-register example plugin
pub fn init_plugins() {
    let registry = get_plugin_registry();
    let _ = registry.register(Box::new(ExamplePlugin));
}
