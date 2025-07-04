//! 📚 Resources module - MCP resource management and file access
//! 
//! This module handles the resource system for the MCP server, providing
//! access to project files and directories through the MCP protocol.

use crate::modules::security::get_security_validator;
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the base directory for the project
pub fn get_base_dir() -> String {
    env::var("ROOT_DIR")
        .or_else(|_| env::var("PROJECT_DIR"))
        .unwrap_or_else(|_| env::current_dir()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
        )
}

/// Get the allowed directory for file operations
pub fn get_allowed_dir() -> String {
    env::var("ALLOWED_DIR").unwrap_or_else(|_| get_base_dir())
}

/// Enhanced path validation using security module
pub fn validate_path(file_path: &str) -> Result<PathBuf, String> {
    get_security_validator()
        .validate_path(file_path)
        .map_err(|e| e.to_string())
        .and_then(|path| {
            get_security_validator()
                .validate_file_size(&path)
                .map_err(|e| e.to_string())
                .map(|_| path)
        })
}

/// 🎯 Get MIME type for a file based on its extension
pub fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "text/x-rust",
        Some("toml") => "text/x-toml",
        Some("json") => "application/json",
        Some("md") => "text/markdown",
        Some("js") => "text/javascript",
        Some("ts") => "text/typescript",
        Some("py") => "text/x-python",
        Some("yml") | Some("yaml") => "text/yaml",
        _ => "text/plain",
    }
}

/// 📚 Get list of available resources for MCP client
pub fn get_resources() -> Value {
    let allowed_dir = get_allowed_dir();
    let allowed_path = Path::new(&allowed_dir);

    let mut resources = vec![json!({
        "uri": format!("file://{}", allowed_path.display()),
        "name": "Project Root",
        "description": "Root directory of the project",
        "mimeType": "text/plain"
    })];

    if let Ok(entries) = fs::read_dir(allowed_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if ["Cargo.toml", "README.md", "main.rs"].contains(&name)
                        || name.ends_with(".rs")
                        || name.ends_with(".toml")
                    {
                        resources.push(json!({
                            "uri": format!("file://{}", path.display()),
                            "name": name,
                            "description": format!("File: {}", name),
                            "mimeType": get_mime_type(&path)
                        }));
                    }
                }
            }
        }
    }

    let src_path = allowed_path.join("src");
    if src_path.is_dir() {
        resources.push(json!({
            "uri": format!("file://{}", src_path.display()),
            "name": "src/",
            "description": "Source directory",
            "mimeType": "text/plain"
        }));
    }

    json!(resources)
}

/// 📖 Read a specific resource by URI
pub fn read_resource(uri: &str) -> Result<Value, String> {
    if !uri.starts_with("file://") {
        return Err("Only file:// URIs are supported".to_string());
    }

    let path = &uri[7..];

    match validate_path(path) {
        Ok(file_path) => {
            if file_path.is_file() {
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        let result = json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": get_mime_type(&file_path),
                                "text": content
                            }]
                        });
                        Ok(result)
                    }
                    Err(e) => Err(format!("Failed to read file: {e}")),
                }
            } else if file_path.is_dir() {
                match fs::read_dir(&file_path) {
                    Ok(entries) => {
                        let mut listing = String::new();
                        for entry in entries.flatten() {
                            listing.push_str(&format!("{}\n", entry.path().display()));
                        }

                        let result = json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": "text/plain",
                                "text": listing
                            }]
                        });
                        Ok(result)
                    }
                    Err(e) => Err(format!("Failed to read directory: {e}")),
                }
            } else {
                Err("Path not found".to_string())
            }
        }
        Err(err) => Err(err),
    }
}
