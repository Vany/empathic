use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use tokio::io::{self, AsyncBufReadExt, BufReader};

mod common;
mod logging;
mod modules;
mod executor;
mod platform;
mod platform_test;
mod tools;
mod prompts;
mod resources;
mod rag;

use common::{send_error, send_response};
use prompts::{get_prompt, get_prompts_schema};
use resources::{get_resources, read_resource};
use logging::{log_startup, log_shutdown};
use modules::security::SecurityValidator;
use tools::tool_registry::{execute_tool, get_tools_schema};

#[derive(Deserialize)]
#[allow(dead_code)]
struct Request {
    jsonrpc: String,
    id: Option<u64>,
    method: String,
    params: Option<Value>,
}

#[derive(Deserialize)]
struct ToolCallParams {
    name: String,
    arguments: Option<Value>,
}

#[derive(Deserialize)]
struct ResourceReadParams {
    uri: String,
}

#[derive(Deserialize)]
struct PromptGetParams {
    name: String,
    arguments: Option<Value>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 🛤️ Extend PATH if ADD_PATH env var is present
    if let Ok(add_path) = env::var("ADD_PATH") {
        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = if current_path.is_empty() {
            add_path
        } else {
            format!("{add_path}:{current_path}")
        };
        unsafe {
            env::set_var("PATH", new_path);
        }
    }

    // 🪵 Initialize logging system - outputs to stderr, not stdout
    // This ensures stdout remains pure JSON for Claude.ai desktop integration
    log_startup();

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<Request>(line) else {
            continue;
        };

        let id = request.id.unwrap_or(0);
        match request.method.as_str() {
            "initialize" => send_response(
                id,
                json!({ "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {}, "resources": {}, "prompts": {} }, 
                    "serverInfo": {"name": "editor", "version": "5.3.0"} }),
            ),
            "tools/list" => send_response(id, json!({"tools": get_tools_schema()})),
            "prompts/list" => send_response(id, json!({"prompts": get_prompts_schema()})),
            "prompts/get" => match request
                .params
                .and_then(|p| serde_json::from_value::<PromptGetParams>(p).ok())
            {
                Some(p) => handle_prompt_get(id, &p.name, p.arguments),
                None => send_error(id, -1, "Invalid prompt get parameters"),
            },
            "resources/list" => send_response(id, json!({"resources": get_resources()})),
            "resources/read" => match request
                .params
                .and_then(|p| serde_json::from_value::<ResourceReadParams>(p).ok())
            {
                Some(p) => match read_resource(&p.uri) {
                    Ok(result) => send_response(id, result),
                    Err(err) => send_error(id, -1, &err),
                },
                None => send_error(id, -1, "Invalid resource read parameters"),
            },
            "tools/call" => match request
                .params
                .and_then(|p| serde_json::from_value::<ToolCallParams>(p).ok())
            {
                Some(p) => handle_tool_call(id, &p.name, p.arguments),
                None => send_error(id, -1, "Invalid tool call parameters"),
            },
            _ => send_error(id, -5, &format!("Unknown method: {}", request.method)),
        }
    }

    // 🪵 Log shutdown via stderr
    log_shutdown();

    Ok(())
}

/// 🚀 Handle tool calls with security validation
fn handle_tool_call(id: u64, name: &str, arguments: Option<Value>) {
    // 🛡️ Security validation for file operations
    if let Some(args) = &arguments {
        if let Err(err) = SecurityValidator::validate_tool_args(name, args) {
            send_error(id, -3, &format!("🚫 Security violation: {err}"));
            return;
        }
    }

    // 🚀 Execute with registry
    if !execute_tool(name, id, arguments) {
        send_error(id, -4, &format!("❌ Unknown tool: {name}"));
    }
}

/// 💬 Handle prompt get requests
fn handle_prompt_get(id: u64, name: &str, arguments: Option<Value>) {
    let args = arguments.unwrap_or(json!({}));

    match get_prompt(name, &args) {
        Some(prompt) => send_response(id, prompt),
        None => send_error(id, -4, &format!("❌ Unknown prompt: {name}")),
    }
}