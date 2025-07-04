use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{OnceLock, RwLock};

use crate::common::{send_error, send_response};
use crate::resources::get_allowed_dir;
use crate::modules::security::get_security_validator;

// Global file cache
static FILE_CACHE: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

fn get_file_cache() -> &'static RwLock<HashMap<String, String>> {
    FILE_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn get_cached_file_content(path: &str) -> Option<String> {
    get_file_cache().read().ok()?.get(path).cloned()
}

#[derive(Deserialize)]
pub struct ReadFileArgs {
    pub path: String,
}

#[derive(Deserialize)]
pub struct WriteFileArgs {
    pub path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ListFilesArgs {
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct EditRangeArgs {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub new_content: String,
}

#[derive(Deserialize)]
pub struct InsertAtLineArgs {
    pub path: String,
    pub line: usize,
    pub content: String,
    pub insert_mode: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchReplaceArgs {
    pub path: String,
    pub search: String,
    pub replace: String,
}

#[derive(Deserialize)]
pub struct SearchFilesArgs {
    pub query: String,
    pub path: Option<String>,
    pub file_pattern: Option<String>,
    pub case_sensitive: Option<bool>,
    pub max_results: Option<usize>,
}

#[derive(Deserialize)]
pub struct FindFilesArgs {
    pub pattern: String,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchSymbolsArgs {
    pub symbol: String,
    pub symbol_type: Option<String>,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchReplaceMultiArgs {
    pub search: String,
    pub replace: String,
    pub file_pattern: Option<String>,
    pub path: Option<String>,
    pub dry_run: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateDirectoryArgs {
    pub path: String,
}

#[derive(Deserialize)]
pub struct DeleteFileArgs {
    pub path: String,
    pub recursive: Option<bool>,
}

#[derive(Deserialize)]
pub struct MoveFileArgs {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
pub struct CacheFilesArgs {
    pub paths: Vec<String>,
}

#[derive(Deserialize)]
pub struct CacheFilesInFoldersArgs {
    pub paths: Vec<String>,
    pub max_depth: Option<u32>,
    pub ignore: Option<Vec<String>>,
}

#[derive(Debug)]
struct SearchResult {
    file_path: String,
    line_number: usize,
    line_content: String,
    match_start: usize,
    match_end: usize,
}

pub fn read_file(id: u64, args: ReadFileArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(file_path) => {
            if file_path.is_file() {
                let path_str = file_path.to_string_lossy().to_string();

                if let Some(cached_content) = get_cached_file_content(&path_str) {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": cached_content
                        }]
                    });
                    send_response(id, result);
                } else {
                    match fs::read_to_string(&file_path) {
                        Ok(content) => {
                            let result = json!({
                                "content": [{
                                    "type": "text",
                                    "text": content
                                }]
                            });
                            send_response(id, result);
                        }
                        Err(e) => send_error(id, -1, &format!("Failed to read file: {e}")),
                    }
                }
            } else {
                send_error(id, -1, "Path is not a file");
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn write_file(id: u64, args: WriteFileArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(file_path) => {
            if let Some(parent) = file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    send_error(id, -3, &format!("Failed to create directory: {e}"));
                    return;
                }
            }

            match fs::write(&file_path, &args.content) {
                Ok(_) => {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!("📝 File written: {}", file_path.display())
                        }]
                    });
                    send_response(id, result);
                }
                Err(e) => send_error(id, -3, &format!("Failed to write file: {e}")),
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn list_files(id: u64, args: ListFilesArgs) {
    let dir_path = args.path.unwrap_or_else(get_allowed_dir);

    match get_security_validator().validate_path(&dir_path) {
        Ok(validated_path) => {
            if validated_path.is_dir() {
                match fs::read_dir(&validated_path) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        let mut dirs = Vec::new();

                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = entry.file_name().to_string_lossy().to_string();

                            if path.is_file() {
                                if let Ok(metadata) = entry.metadata() {
                                    let size = metadata.len();
                                    let ext =
                                        path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                    files.push(format!("📄 {name} ({size} bytes) .{ext}"));
                                } else {
                                    files.push(format!("📄 {name}"));
                                }
                            } else if path.is_dir() && !name.starts_with('.') && name != "target" {
                                dirs.push(format!("📁 {name}/"));
                            }
                        }

                        dirs.sort();
                        files.sort();

                        let mut listing = dirs;
                        listing.extend(files);

                        let result = json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Directory: {}\n\n{}",
                                    validated_path.display(),
                                    if listing.is_empty() {
                                        "(empty directory)".to_string()
                                    } else {
                                        listing.join("\n")
                                    }
                                )
                            }]
                        });
                        send_response(id, result);
                    }
                    Err(e) => send_error(id, -1, &format!("Failed to read directory: {e}")),
                }
            } else {
                send_error(id, -1, "Path is not a directory");
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn edit_file_range(id: u64, args: EditRangeArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(file_path) => {
            if let Ok(content) = fs::read_to_string(&file_path) {
                let lines: Vec<&str> = content.lines().collect();

                if args.start_line > 0
                    && args.end_line <= lines.len()
                    && args.start_line <= args.end_line
                {
                    let mut new_lines = Vec::new();

                    new_lines.extend_from_slice(&lines[0..args.start_line - 1]);

                    if !args.new_content.is_empty() {
                        new_lines.extend(args.new_content.lines());
                    }

                    if args.end_line < lines.len() {
                        new_lines.extend_from_slice(&lines[args.end_line..]);
                    }

                    let new_content = new_lines.join("\n");

                    match fs::write(&file_path, new_content) {
                        Ok(_) => {
                            let result = json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("✅ Edited lines {}-{} in {}",
                                        args.start_line, args.end_line, file_path.display())
                                }]
                            });
                            send_response(id, result);
                        }
                        Err(e) => send_error(id, -3, &format!("Failed to write file: {e}")),
                    }
                } else {
                    send_error(id, -1, "Invalid line range");
                }
            } else {
                send_error(id, -1, "Failed to read file");
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn insert_at_line(id: u64, args: InsertAtLineArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(file_path) => {
            if let Ok(content) = fs::read_to_string(&file_path) {
                let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

                let insert_mode = args.insert_mode.as_deref().unwrap_or("after");
                let line_idx = if args.line > 0 { args.line - 1 } else { 0 };

                match insert_mode {
                    "before" => {
                        if line_idx <= lines.len() {
                            lines.insert(line_idx, args.content);
                        }
                    }
                    "after" => {
                        if line_idx < lines.len() {
                            lines.insert(line_idx + 1, args.content);
                        } else {
                            lines.push(args.content);
                        }
                    }
                    "replace" => {
                        if line_idx < lines.len() {
                            lines[line_idx] = args.content;
                        }
                    }
                    _ => {
                        send_error(id, -1, "Invalid insert mode");
                        return;
                    }
                }

                let new_content = lines.join("\n");

                match fs::write(&file_path, new_content) {
                    Ok(_) => {
                        let result = json!({
                            "content": [{
                                "type": "text",
                                "text": format!("✅ Inserted at line {} in {} (mode: {})",
                                    args.line, file_path.display(), insert_mode)
                            }]
                        });
                        send_response(id, result);
                    }
                    Err(e) => send_error(id, -3, &format!("Failed to write file: {e}")),
                }
            } else {
                send_error(id, -1, "Failed to read file");
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn search_replace(id: u64, args: SearchReplaceArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(file_path) => {
            if let Ok(content) = fs::read_to_string(&file_path) {
                let new_content = content.replace(&args.search, &args.replace);
                let changes = content.matches(&args.search).count();

                if content == new_content {
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!("🔍 No matches found for '{}' in {}",
                                args.search, file_path.display())
                        }]
                    });
                    send_response(id, result);
                } else {
                    match fs::write(&file_path, new_content) {
                        Ok(_) => {
                            let result = json!({
                                "content": [{
                                    "type": "text",
                                    "text": format!("✅ Replaced {} occurrences of '{}' with '{}' in {}",
                                        changes, args.search, args.replace, file_path.display())
                                }]
                            });
                            send_response(id, result);
                        }
                        Err(e) => send_error(id, -3, &format!("Failed to write file: {e}")),
                    }
                }
            } else {
                send_error(id, -1, "Failed to read file");
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn search_files(id: u64, args: SearchFilesArgs) {
    let search_path = args.path.unwrap_or_else(get_allowed_dir);

    match get_security_validator().validate_path(&search_path) {
        Ok(validated_path) => {
            if !validated_path.is_dir() {
                send_error(id, -1, "Search path must be a directory");
                return;
            }

            let file_pattern = args.file_pattern.as_deref().unwrap_or("*");
            let case_sensitive = args.case_sensitive.unwrap_or(false);
            let max_results = args.max_results.unwrap_or(100);

            let mut results = Vec::new();
            let query = if case_sensitive {
                args.query.clone()
            } else {
                args.query.to_lowercase()
            };

            search_directory_recursive(
                &validated_path,
                &query,
                file_pattern,
                case_sensitive,
                &mut results,
                max_results,
            );

            if results.is_empty() {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🔍 No matches found for '{}' in {}", args.query, validated_path.display())
                    }]
                });
                send_response(id, result);
            } else {
                let mut output = format!(
                    "🔍 Found {} matches for '{}':\n\n",
                    results.len(),
                    args.query
                );

                for result in results.iter().take(max_results) {
                    output.push_str(&format!(
                        "📄 {}:{}\n   {}\n   {}{}^\n\n",
                        result.file_path,
                        result.line_number,
                        result.line_content.trim(),
                        " ".repeat(result.match_start),
                        "~".repeat((result.match_end - result.match_start).max(1))
                    ));
                }

                if results.len() > max_results {
                    output.push_str(&format!(
                        "... and {} more matches\n",
                        results.len() - max_results
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
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn find_files(id: u64, args: FindFilesArgs) {
    let search_path = args.path.unwrap_or_else(get_allowed_dir);

    match get_security_validator().validate_path(&search_path) {
        Ok(validated_path) => {
            if !validated_path.is_dir() {
                send_error(id, -1, "Search path must be a directory");
                return;
            }

            let mut found_files = Vec::new();
            find_files_recursive(&validated_path, &args.pattern, &mut found_files);

            if found_files.is_empty() {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🔍 No files found matching pattern '{}'", args.pattern)
                    }]
                });
                send_response(id, result);
            } else {
                let mut output = format!(
                    "📁 Found {} files matching '{}':\n\n",
                    found_files.len(),
                    args.pattern
                );

                for file_path in found_files {
                    output.push_str(&format!("📄 {file_path}\n"));
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
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn search_symbols(id: u64, args: SearchSymbolsArgs) {
    let search_path = args.path.unwrap_or_else(get_allowed_dir);

    match get_security_validator().validate_path(&search_path) {
        Ok(validated_path) => {
            if !validated_path.is_dir() {
                send_error(id, -1, "Search path must be a directory");
                return;
            }

            let symbol_type = args.symbol_type.as_deref().unwrap_or("any");
            let mut symbols = Vec::new();

            search_symbols_recursive(&validated_path, &args.symbol, symbol_type, &mut symbols);

            if symbols.is_empty() {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🔍 No {} symbols found for '{}'", symbol_type, args.symbol)
                    }]
                });
                send_response(id, result);
            } else {
                let mut output = format!(
                    "🔍 Found {} {} symbols for '{}':\n\n",
                    symbols.len(),
                    symbol_type,
                    args.symbol
                );

                for symbol in symbols {
                    output.push_str(&format!(
                        "📄 {}:{}\n   {}\n\n",
                        symbol.file_path,
                        symbol.line_number,
                        symbol.line_content.trim()
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
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn search_replace_multi(id: u64, args: SearchReplaceMultiArgs) {
    let search_path = args.path.unwrap_or_else(get_allowed_dir);
    let file_pattern = args.file_pattern.as_deref().unwrap_or("*");
    let dry_run = args.dry_run.unwrap_or(true);

    match get_security_validator().validate_path(&search_path) {
        Ok(validated_path) => {
            if !validated_path.is_dir() {
                send_error(id, -1, "Search path must be a directory");
                return;
            }

            let mut changes = HashMap::new();
            collect_multi_replace_changes(
                &validated_path,
                &args.search,
                &args.replace,
                file_pattern,
                &mut changes,
            );

            if changes.is_empty() {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("🔍 No matches found for '{}' in files matching '{}'", args.search, file_pattern)
                    }]
                });
                send_response(id, result);
                return;
            }

            let total_files = changes.len();
            let total_changes: usize = changes.values().sum();

            if dry_run {
                let mut output = format!(
                    "🔍 Dry run: Would replace '{}' with '{}' in {} files ({} total changes):\n\n",
                    args.search, args.replace, total_files, total_changes
                );

                for (file_path, count) in &changes {
                    output.push_str(&format!("📄 {file_path}: {count} changes\n"));
                }

                output.push_str("\n💡 Set dry_run: false to apply changes");

                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": output
                    }]
                });
                send_response(id, result);
            } else {
                let mut applied = 0;
                for file_path in changes.keys() {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        let new_content = content.replace(&args.search, &args.replace);
                        if fs::write(file_path, new_content).is_ok() {
                            applied += 1;
                        }
                    }
                }

                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("✅ Applied changes to {} files ({} total replacements)", applied, total_changes)
                    }]
                });
                send_response(id, result);
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn create_directory(id: u64, args: CreateDirectoryArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(dir_path) => match fs::create_dir_all(&dir_path) {
            Ok(_) => {
                let result = json!({
                    "content": [{
                        "type": "text",
                        "text": format!("📁 Directory created: {}", dir_path.display())
                    }]
                });
                send_response(id, result);
            }
            Err(e) => send_error(id, -3, &format!("Failed to create directory: {e}")),
        },
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn delete_file(id: u64, args: DeleteFileArgs) {
    match get_security_validator().validate_path(&args.path) {
        Ok(file_path) => {
            let recursive = args.recursive.unwrap_or(false);

            let result = if file_path.is_file() {
                fs::remove_file(&file_path)
            } else if file_path.is_dir() {
                if recursive {
                    fs::remove_dir_all(&file_path)
                } else {
                    fs::remove_dir(&file_path)
                }
            } else {
                send_error(id, -1, "Path does not exist");
                return;
            };

            match result {
                Ok(_) => {
                    if let Ok(mut cache) = get_file_cache().write() {
                        cache.remove(&file_path.to_string_lossy().to_string());
                    }

                    let file_type = if file_path.is_dir() {
                        "directory"
                    } else {
                        "file"
                    };
                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!("🗑️ Deleted {}: {}", file_type, file_path.display())
                        }]
                    });
                    send_response(id, result);
                }
                Err(e) => send_error(id, -3, &format!("Failed to delete: {e}")),
            }
        }
        Err(err) => send_error(id, -2, &err.to_string()),
    }
}

pub fn move_file(id: u64, args: MoveFileArgs) {
    match (
        get_security_validator().validate_path(&args.from),
        get_security_validator().validate_path(&args.to),
    ) {
        (Ok(from_path), Ok(to_path)) => {
            if !from_path.exists() {
                send_error(id, -1, "Source path does not exist");
                return;
            }

            if let Some(parent) = to_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    send_error(id, -3, &format!("Failed to create parent directory: {e}"));
                    return;
                }
            }

            match fs::rename(&from_path, &to_path) {
                Ok(_) => {
                    if let Ok(mut cache) = get_file_cache().write() {
                        let from_str = from_path.to_string_lossy().to_string();
                        let to_str = to_path.to_string_lossy().to_string();

                        if let Some(content) = cache.remove(&from_str) {
                            cache.insert(to_str, content);
                        }
                    }

                    let result = json!({
                        "content": [{
                            "type": "text",
                            "text": format!("🔄 Moved: {} → {}", from_path.display(), to_path.display())
                        }]
                    });
                    send_response(id, result);
                }
                Err(e) => send_error(id, -3, &format!("Failed to move: {e}")),
            }
        }
        (Err(msg), _) => send_error(id, -2, &format!("Invalid source path: {msg}")),
        (_, Err(msg)) => send_error(id, -2, &format!("Invalid destination path: {msg}")),
    }
}

pub fn cache_files(id: u64, args: CacheFilesArgs) {
    let mut cached_count = 0;
    let mut failed_files = Vec::new();

    for path_str in &args.paths {
        match get_security_validator().validate_path(path_str) {
            Ok(file_path) => {
                if file_path.is_file() {
                    match fs::read_to_string(&file_path) {
                        Ok(content) => {
                            if let Ok(mut cache) = get_file_cache().write() {
                                cache.insert(file_path.to_string_lossy().to_string(), content);
                                cached_count += 1;
                            }
                        }
                        Err(_) => failed_files.push(path_str.clone()),
                    }
                } else {
                    failed_files.push(format!("{path_str} (not a file)"));
                }
            }
            Err(_) => failed_files.push(format!("{path_str} (invalid path)")),
        }
    }

    let mut message = format!("📦 Cached {cached_count} files successfully");
    if !failed_files.is_empty() {
        message.push_str(&format!(
            "\n❌ Failed to cache {} files: {}",
            failed_files.len(),
            failed_files.join(", ")
        ));
    }

    let result = json!({
        "content": [{
            "type": "text",
            "text": message
        }]
    });
    send_response(id, result);
}

pub fn cache_files_in_folders(id: u64, args: CacheFilesInFoldersArgs) {
    let max_depth = args.max_depth.unwrap_or(10);
    let ignore_patterns = args.ignore.unwrap_or_else(|| {
        vec![
            "*.log".to_string(),
            "target".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
            "*.tmp".to_string(),
        ]
    });

    let mut total_cached = 0;
    let mut failed_folders = Vec::new();

    for folder_str in &args.paths {
        match get_security_validator().validate_path(folder_str) {
            Ok(folder_path) => {
                if folder_path.is_dir() {
                    let cached_in_folder =
                        cache_directory_recursive(&folder_path, 0, max_depth, &ignore_patterns);
                    total_cached += cached_in_folder;
                } else {
                    failed_folders.push(format!("{folder_str} (not a directory)"));
                }
            }
            Err(_) => failed_folders.push(format!("{folder_str} (invalid path)")),
        }
    }

    let mut message = format!(
        "📦 Cached {} files from {} folders",
        total_cached,
        args.paths.len()
    );
    if !failed_folders.is_empty() {
        message.push_str(&format!(
            "\n❌ Failed folders: {}",
            failed_folders.join(", ")
        ));
    }

    let result = json!({
        "content": [{
            "type": "text",
            "text": message
        }]
    });
    send_response(id, result);
}

// Helper functions
fn search_directory_recursive(
    dir: &Path,
    query: &str,
    file_pattern: &str,
    case_sensitive: bool,
    results: &mut Vec<SearchResult>,
    max_results: usize,
) {
    if results.len() >= max_results {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        if results.len() >= max_results {
            break;
        }

        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" && name != "node_modules" {
                search_directory_recursive(
                    &path,
                    query,
                    file_pattern,
                    case_sensitive,
                    results,
                    max_results,
                );
            }
        } else if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            if matches_pattern(&file_name, file_pattern) {
                let path_str = path.to_string_lossy().to_string();

                let content =
                    get_cached_file_content(&path_str).or_else(|| fs::read_to_string(&path).ok());

                if let Some(content) = content {
                    search_file_content(&path, &content, query, case_sensitive, results);
                }
            }
        }
    }
}

fn search_file_content(
    file_path: &Path,
    content: &str,
    query: &str,
    case_sensitive: bool,
    results: &mut Vec<SearchResult>,
) {
    for (line_num, line) in content.lines().enumerate() {
        let search_line = if case_sensitive {
            line
        } else {
            &line.to_lowercase()
        };

        if let Some(match_start) = search_line.find(query) {
            results.push(SearchResult {
                file_path: file_path.to_string_lossy().to_string(),
                line_number: line_num + 1,
                line_content: line.to_string(),
                match_start,
                match_end: match_start + query.len(),
            });
        }
    }
}

fn matches_pattern(filename: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(ext) = pattern.strip_prefix("*.") {
        return filename.ends_with(&format!(".{ext}"));
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return filename.starts_with(parts[0]) && filename.ends_with(parts[1]);
        }
    }

    filename == pattern
}

fn find_files_recursive(dir: &Path, pattern: &str, found_files: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" && name != "node_modules" {
                find_files_recursive(&path, pattern, found_files);
            }
        } else if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if matches_pattern(&file_name, pattern) {
                found_files.push(path.to_string_lossy().to_string());
            }
        }
    }
}

fn search_symbols_recursive(
    dir: &Path,
    symbol: &str,
    symbol_type: &str,
    symbols: &mut Vec<SearchResult>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" {
                search_symbols_recursive(&path, symbol, symbol_type, symbols);
            }
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            let path_str = path.to_string_lossy().to_string();

            let content =
                get_cached_file_content(&path_str).or_else(|| fs::read_to_string(&path).ok());

            if let Some(content) = content {
                search_rust_symbols(&path, &content, symbol, symbol_type, symbols);
            }
        }
    }
}

fn search_rust_symbols(
    file_path: &Path,
    content: &str,
    symbol: &str,
    symbol_type: &str,
    symbols: &mut Vec<SearchResult>,
) {
    for (line_num, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        let matches = match symbol_type {
            "function" | "fn" => {
                trimmed.starts_with("fn ") && trimmed.contains(&format!("fn {symbol}"))
            }
            "struct" => {
                trimmed.starts_with("struct ") && trimmed.contains(&format!("struct {symbol}"))
            }
            "enum" => trimmed.starts_with("enum ") && trimmed.contains(&format!("enum {symbol}")),
            "trait" => {
                trimmed.starts_with("trait ") && trimmed.contains(&format!("trait {symbol}"))
            }
            "impl" => trimmed.starts_with("impl ") && trimmed.contains(symbol),
            "mod" => trimmed.starts_with("mod ") && trimmed.contains(&format!("mod {symbol}")),
            "any" => {
                trimmed.starts_with("fn ") && trimmed.contains(&format!("fn {symbol}"))
                    || trimmed.starts_with("struct ")
                        && trimmed.contains(&format!("struct {symbol}"))
                    || trimmed.starts_with("enum ") && trimmed.contains(&format!("enum {symbol}"))
                    || trimmed.starts_with("trait ") && trimmed.contains(&format!("trait {symbol}"))
                    || trimmed.starts_with("impl ") && trimmed.contains(symbol)
                    || trimmed.starts_with("mod ") && trimmed.contains(&format!("mod {symbol}"))
            }
            _ => {
                trimmed.starts_with("fn ") && trimmed.contains(&format!("fn {symbol}"))
                    || trimmed.starts_with("struct ")
                        && trimmed.contains(&format!("struct {symbol}"))
                    || trimmed.starts_with("enum ") && trimmed.contains(&format!("enum {symbol}"))
                    || trimmed.starts_with("trait ") && trimmed.contains(&format!("trait {symbol}"))
                    || trimmed.starts_with("impl ") && trimmed.contains(symbol)
                    || trimmed.starts_with("mod ") && trimmed.contains(&format!("mod {symbol}"))
            }
        };

        if matches {
            symbols.push(SearchResult {
                file_path: file_path.to_string_lossy().to_string(),
                line_number: line_num + 1,
                line_content: line.to_string(),
                match_start: 0,
                match_end: line.len(),
            });
        }
    }
}

fn collect_multi_replace_changes(
    dir: &Path,
    search: &str,
    _replace: &str,
    file_pattern: &str,
    changes: &mut HashMap<String, usize>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "target" {
                collect_multi_replace_changes(&path, search, _replace, file_pattern, changes);
            }
        } else if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if matches_pattern(&file_name, file_pattern) {
                let path_str = path.to_string_lossy().to_string();

                let content =
                    get_cached_file_content(&path_str).or_else(|| fs::read_to_string(&path).ok());

                if let Some(content) = content {
                    let count = content.matches(search).count();
                    if count > 0 {
                        changes.insert(path_str, count);
                    }
                }
            }
        }
    }
}

fn cache_directory_recursive(
    dir: &Path,
    depth: u32,
    max_depth: u32,
    ignore_patterns: &[String],
) -> usize {
    if depth > max_depth {
        return 0;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut cached_count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        if ignore_patterns
            .iter()
            .any(|pattern| matches_ignore_pattern(&name, pattern))
        {
            continue;
        }

        if path.is_file() && is_text_file(&path) {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut cache) = get_file_cache().write() {
                    cache.insert(path.to_string_lossy().to_string(), content);
                    cached_count += 1;
                }
            }
        } else if path.is_dir() {
            cached_count += cache_directory_recursive(&path, depth + 1, max_depth, ignore_patterns);
        }
    }

    cached_count
}

fn matches_ignore_pattern(name: &str, pattern: &str) -> bool {
    if let Some(ext) = pattern.strip_prefix("*.") {
        return name.ends_with(&format!(".{ext}"));
    }
    name == pattern
}

fn is_text_file(path: &Path) -> bool {
    let text_extensions = [
        "rs", "toml", "json", "md", "txt", "yml", "yaml", "js", "ts", "py", "go", "java", "cpp",
        "h", "c",
    ];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| text_extensions.contains(&ext))
        .unwrap_or(false)
}
