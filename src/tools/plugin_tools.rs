use crate::common::{send_error, send_response};
use crate::modules::plugins::{get_plugin_registry, init_plugins};
use super::tool_trait::Tool;
use crate::register_tool;
use serde::Deserialize;
use serde_json::{Value, json};

/// 🔌 Plugin list tool
#[derive(Default)]
pub struct PluginListTool;

impl Tool for PluginListTool {
    fn name(&self) -> &'static str {
        "plugin_list"
    }
    fn description(&self) -> &'static str {
        "List all available plugins"
    }
    fn emoji(&self) -> &'static str {
        "🔌"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        let registry = get_plugin_registry();

        match registry.list() {
            Ok(plugins) => {
                if plugins.is_empty() {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": "🔌 No plugins registered"
                        }]
                    });
                    send_response(id, result);
                } else {
                    let mut output = format!("🔌 Available Plugins ({})\n\n", plugins.len());

                    for plugin in plugins {
                        output.push_str(&format!(
                            "📦 {} v{}\n   {}\n\n",
                            plugin.name, plugin.version, plugin.description
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
            Err(e) => send_error(id, -3, &format!("Failed to list plugins: {e}")),
        }
    }
}

register_tool!("plugin_list", PluginListTool);

/// ⚡ Plugin execution tool
#[derive(Default)]
pub struct PluginExecuteTool;

#[derive(Deserialize)]
pub struct PluginExecuteArgs {
    pub name: String,
    pub args: Option<Value>,
}

impl Tool for PluginExecuteTool {
    fn name(&self) -> &'static str {
        "plugin_execute"
    }
    fn description(&self) -> &'static str {
        "Execute a plugin by name"
    }
    fn emoji(&self) -> &'static str {
        "⚡"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Plugin name to execute"
                },
                "args": {
                    "description": "Arguments to pass to the plugin"
                }
            },
            "required": ["name"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args: PluginExecuteArgs = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for plugin_execute");
                return;
            }
        };

        let registry = get_plugin_registry();

        match registry.execute(&args.name, args.args) {
            Ok(result) => {
                let response = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("⚡ Plugin Result:\n{}",
                            serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string())
                        )
                    }]
                });
                send_response(id, response);
            }
            Err(e) => send_error(id, -3, &format!("Plugin execution failed: {e}")),
        }
    }
}

register_tool!("plugin_execute", PluginExecuteTool);

/// 🔧 Plugin initialization tool
#[derive(Default)]
pub struct PluginInitTool;

impl Tool for PluginInitTool {
    fn name(&self) -> &'static str {
        "plugin_init"
    }
    fn description(&self) -> &'static str {
        "Initialize plugin system and load default plugins"
    }
    fn emoji(&self) -> &'static str {
        "🔧"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn execute_impl(&self, id: u64, _args: Option<Value>) {
        init_plugins();

        let result = json!({
            "content": [{
                "type": "text",
                "text": "🔧 Plugin system initialized and default plugins loaded"
            }]
        });
        send_response(id, result);
    }
}

register_tool!("plugin_init", PluginInitTool);
