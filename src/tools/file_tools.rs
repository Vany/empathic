use super::tool_trait::Tool;
use crate::modules::file_ops;
use crate::common::send_error;
use crate::register_tool;
use serde_json::{Value, json};

/// 📁 File reading tool
#[derive(Default)]
pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &'static str {
        "read_file"
    }
    fn description(&self) -> &'static str {
        "Read content of a file"
    }
    fn emoji(&self) -> &'static str {
        "📖"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for read_file");
                return;
            }
        };
        file_ops::read_file(id, args);
    }
}

register_tool!("read_file", ReadFileTool);

/// ✍️ File writing tool
#[derive(Default)]
pub struct WriteFileTool;

impl Tool for WriteFileTool {
    fn name(&self) -> &'static str {
        "write_file"
    }
    fn description(&self) -> &'static str {
        "Write content to a file"
    }
    fn emoji(&self) -> &'static str {
        "✍️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for write_file");
                return;
            }
        };
        file_ops::write_file(id, args);
    }
}

register_tool!("write_file", WriteFileTool);

/// 📂 Directory listing tool
#[derive(Default)]
pub struct ListFilesTool;

impl Tool for ListFilesTool {
    fn name(&self) -> &'static str {
        "list_files"
    }
    fn description(&self) -> &'static str {
        "List files in a directory"
    }
    fn emoji(&self) -> &'static str {
        "📂"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list (optional, defaults to allowed directory)"
                }
            }
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = args
            .and_then(|a| serde_json::from_value(a).ok())
            .unwrap_or(file_ops::ListFilesArgs { path: None });
        file_ops::list_files(id, args);
    }
}

register_tool!("list_files", ListFilesTool);

/// ✂️ File range editing tool
#[derive(Default)]
pub struct EditFileRangeTool;

impl Tool for EditFileRangeTool {
    fn name(&self) -> &'static str {
        "edit_file_range"
    }
    fn description(&self) -> &'static str {
        "Edit specific lines in a file"
    }
    fn emoji(&self) -> &'static str {
        "✂️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "start_line": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Starting line number (1-indexed)"
                },
                "end_line": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Ending line number (1-indexed, inclusive)"
                },
                "new_content": {
                    "type": "string",
                    "description": "New content to replace the specified lines"
                }
            },
            "required": ["path", "start_line", "end_line", "new_content"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for edit_file_range");
                return;
            }
        };
        file_ops::edit_file_range(id, args);
    }
}

register_tool!("edit_file_range", EditFileRangeTool);

/// 📍 Line insertion tool
#[derive(Default)]
pub struct InsertAtLineTool;

impl Tool for InsertAtLineTool {
    fn name(&self) -> &'static str {
        "insert_at_line"
    }
    fn description(&self) -> &'static str {
        "Insert content at specific line"
    }
    fn emoji(&self) -> &'static str {
        "📍"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "line": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Line number (1-indexed)"
                },
                "content": {
                    "type": "string",
                    "description": "Content to insert"
                },
                "insert_mode": {
                    "type": "string",
                    "enum": ["before", "after", "replace"],
                    "default": "after",
                    "description": "Where to insert relative to the line"
                }
            },
            "required": ["path", "line", "content"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for insert_at_line");
                return;
            }
        };
        file_ops::insert_at_line(id, args);
    }
}

register_tool!("insert_at_line", InsertAtLineTool);

/// 🔍 Search and replace tool
#[derive(Default)]
pub struct SearchReplaceTool;

impl Tool for SearchReplaceTool {
    fn name(&self) -> &'static str {
        "search_replace"
    }
    fn description(&self) -> &'static str {
        "Search and replace text in file"
    }
    fn emoji(&self) -> &'static str {
        "🔍"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "search": {
                    "type": "string",
                    "description": "Text to search for"
                },
                "replace": {
                    "type": "string",
                    "description": "Text to replace with"
                },
                "global": {
                    "type": "boolean",
                    "default": true,
                    "description": "Replace all occurrences (true) or just first (false)"
                }
            },
            "required": ["path", "search", "replace"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for search_replace");
                return;
            }
        };
        file_ops::search_replace(id, args);
    }
}

register_tool!("search_replace", SearchReplaceTool);

/// 🔎 Multi-file search tool
#[derive(Default)]
pub struct SearchFilesTool;

impl Tool for SearchFilesTool {
    fn name(&self) -> &'static str {
        "search_files"
    }
    fn description(&self) -> &'static str {
        "Search for text content across multiple files"
    }
    fn emoji(&self) -> &'static str {
        "🔎"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Text to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (optional, defaults to project root)"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "File pattern to include (e.g., '*.rs', '*.toml')"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "default": false,
                    "description": "Case sensitive search"
                },
                "max_results": {
                    "type": "integer",
                    "default": 100,
                    "description": "Maximum number of results to return"
                }
            },
            "required": ["query"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for search_files");
                return;
            }
        };
        file_ops::search_files(id, args);
    }
}

register_tool!("search_files", SearchFilesTool);

/// 🗂️ File finding tool
#[derive(Default)]
pub struct FindFilesTool;

impl Tool for FindFilesTool {
    fn name(&self) -> &'static str {
        "find_files"
    }
    fn description(&self) -> &'static str {
        "Find files by name pattern"
    }
    fn emoji(&self) -> &'static str {
        "🗂️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "File name pattern (supports wildcards like *.rs)"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (optional)"
                }
            },
            "required": ["pattern"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for find_files");
                return;
            }
        };
        file_ops::find_files(id, args);
    }
}

register_tool!("find_files", FindFilesTool);

/// 🎯 Symbol search tool  
#[derive(Default)]
pub struct SearchSymbolsTool;

impl Tool for SearchSymbolsTool {
    fn name(&self) -> &'static str {
        "search_symbols"
    }
    fn description(&self) -> &'static str {
        "Search for code symbols (functions, structs, etc.)"
    }
    fn emoji(&self) -> &'static str {
        "🎯"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Symbol name to search for"
                },
                "symbol_type": {
                    "type": "string",
                    "enum": ["function", "struct", "enum", "trait", "impl", "mod", "any"],
                    "default": "any",
                    "description": "Type of symbol to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (optional)"
                }
            },
            "required": ["symbol"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for search_symbols");
                return;
            }
        };
        file_ops::search_symbols(id, args);
    }
}

register_tool!("search_symbols", SearchSymbolsTool);

/// 🔄 Multi-file search & replace tool
#[derive(Default)]
pub struct SearchReplaceMultiTool;

impl Tool for SearchReplaceMultiTool {
    fn name(&self) -> &'static str {
        "search_replace_multi"
    }
    fn description(&self) -> &'static str {
        "Search and replace across multiple files"
    }
    fn emoji(&self) -> &'static str {
        "🔄"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "search": {
                    "type": "string",
                    "description": "Text to search for"
                },
                "replace": {
                    "type": "string",
                    "description": "Text to replace with"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "File pattern to include (e.g., '*.rs')"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (optional)"
                },
                "dry_run": {
                    "type": "boolean",
                    "default": true,
                    "description": "Preview changes without applying them"
                }
            },
            "required": ["search", "replace"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for search_replace_multi");
                return;
            }
        };
        file_ops::search_replace_multi(id, args);
    }
}

register_tool!("search_replace_multi", SearchReplaceMultiTool);

/// 📁 Directory creation tool
#[derive(Default)]
pub struct CreateDirectoryTool;

impl Tool for CreateDirectoryTool {
    fn name(&self) -> &'static str {
        "create_directory"
    }
    fn description(&self) -> &'static str {
        "Create a directory and its parent directories"
    }
    fn emoji(&self) -> &'static str {
        "📁"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to create"
                }
            },
            "required": ["path"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for create_directory");
                return;
            }
        };
        file_ops::create_directory(id, args);
    }
}

register_tool!("create_directory", CreateDirectoryTool);

/// 🗑️ File deletion tool
#[derive(Default)]
pub struct DeleteFileTool;

impl Tool for DeleteFileTool {
    fn name(&self) -> &'static str {
        "delete_file"
    }
    fn description(&self) -> &'static str {
        "Delete a file or directory"
    }
    fn emoji(&self) -> &'static str {
        "🗑️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to file or directory to delete"
                },
                "recursive": {
                    "type": "boolean",
                    "default": false,
                    "description": "Delete directories recursively"
                }
            },
            "required": ["path"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for delete_file");
                return;
            }
        };
        file_ops::delete_file(id, args);
    }
}

register_tool!("delete_file", DeleteFileTool);

/// 📦 File move tool
#[derive(Default)]
pub struct MoveFileTool;

impl Tool for MoveFileTool {
    fn name(&self) -> &'static str {
        "move_file"
    }
    fn description(&self) -> &'static str {
        "Move or rename a file or directory"
    }
    fn emoji(&self) -> &'static str {
        "📦"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from": {
                    "type": "string",
                    "description": "Source path"
                },
                "to": {
                    "type": "string",
                    "description": "Destination path"
                }
            },
            "required": ["from", "to"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for move_file");
                return;
            }
        };
        file_ops::move_file(id, args);
    }
}

register_tool!("move_file", MoveFileTool);

/// 💾 File caching tool
#[derive(Default)]
pub struct CacheFilesTool;

impl Tool for CacheFilesTool {
    fn name(&self) -> &'static str {
        "cache_files"
    }
    fn description(&self) -> &'static str {
        "Cache file contents for efficient reuse in future API calls"
    }
    fn emoji(&self) -> &'static str {
        "💾"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Array of absolute file paths to cache"
                }
            },
            "required": ["paths"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for cache_files");
                return;
            }
        };
        file_ops::cache_files(id, args);
    }
}

register_tool!("cache_files", CacheFilesTool);

/// 🗂️ Folder caching tool
#[derive(Default)]
pub struct CacheFilesInFoldersTool;

impl Tool for CacheFilesInFoldersTool {
    fn name(&self) -> &'static str {
        "cache_files_in_folders"
    }
    fn description(&self) -> &'static str {
        "Recursively cache all files in specified folders"
    }
    fn emoji(&self) -> &'static str {
        "🗂️"
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Array of absolute folder paths to cache recursively"
                },
                "max_depth": {
                    "type": "integer",
                    "default": 10,
                    "description": "Maximum recursion depth"
                },
                "ignore": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Patterns to ignore (e.g., '*.log', 'node_modules')"
                }
            },
            "required": ["paths"]
        })
    }

    fn execute_impl(&self, id: u64, args: Option<Value>) {
        let args = match args.and_then(|a| serde_json::from_value(a).ok()) {
            Some(args) => args,
            None => {
                send_error(id, -1, "Invalid arguments for cache_files_in_folders");
                return;
            }
        };
        file_ops::cache_files_in_folders(id, args);
    }
}

register_tool!("cache_files_in_folders", CacheFilesInFoldersTool);
