use crate::modules::communication;
use super::tool_trait::Tool;
use crate::register_tool;
use serde_json::{Value, json};

/// 🔊 Text-to-speech tool
#[derive(Default)]
pub struct SayTool;

impl Tool for SayTool {
    fn name(&self) -> &'static str {
        "say"
    }
    fn description(&self) -> &'static str {
        "Cross-platform text-to-speech synthesis"
    }
    fn emoji(&self) -> &'static str {
        "🔊"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Text to speak"
                },
                "voice": {
                    "type": "string",
                    "description": "Voice to use (optional, e.g., 'Alex', 'Samantha')"
                }
            },
            "required": ["text"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                crate::common::send_error(id, -1, "Invalid arguments for say");
                return;
            }
        };
        communication::say(id, args);
    }
}

register_tool!("say", SayTool);
