use std::path::Path;
use crate::error::{EmpathicResult, EmpathicError};

/// Unicode-aware file operations 🦀
pub struct FileOps;

impl FileOps {
    /// Read entire file content
    pub async fn read_file(path: &Path) -> EmpathicResult<String> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| EmpathicError::FileOperationFailed {
                operation: "read".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;
        Ok(content)
    }
    
    /// Read file content with line-based chunking
    pub async fn read_file_chunk(path: &Path, line_offset: usize, line_length: Option<usize>) -> EmpathicResult<String> {
        let content = Self::read_file(path).await?;
        let lines: Vec<&str> = content.lines().collect();
        
        if line_offset >= lines.len() {
            return Ok(String::new());
        }
        
        let end_line = match line_length {
            Some(len) => (line_offset + len).min(lines.len()),
            None => lines.len(),
        };
        
        let chunk_lines = &lines[line_offset..end_line];
        Ok(chunk_lines.join("\n"))
    }
    
    /// Write entire file content
    pub async fn write_file(path: &Path, content: &str) -> EmpathicResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| EmpathicError::DirectoryCreationFailed {
                    path: parent.to_path_buf(),
                    reason: e.to_string(),
                })?;
        }
        
        tokio::fs::write(path, content).await
            .map_err(|e| EmpathicError::FileOperationFailed {
                operation: "write".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;
        Ok(())
    }
    
    /// Write file content with line-based range replacement
    pub async fn write_file_range(path: &Path, content: &str, start: usize, end: Option<usize>) -> EmpathicResult<()> {
        let existing_content = Self::read_file(path).await.unwrap_or_default();
        let mut lines: Vec<&str> = existing_content.lines().collect();
        
        // Extend lines if needed
        while lines.len() <= start {
            lines.push("");
        }
        
        let new_lines: Vec<&str> = content.lines().collect();
        
        match end {
            Some(end_line) => {
                // Replace specific range [start, end)
                if end_line <= lines.len() {
                    lines.splice(start..end_line, new_lines);
                } else {
                    lines.splice(start.., new_lines);
                }
            }
            None => {
                // Replace from start to end of file
                lines.splice(start.., new_lines);
            }
        }
        
        let final_content = lines.join("\n");
        Self::write_file(path, &final_content).await
    }
    
    /// List directory contents with metadata and optional pattern matching
    pub async fn list_files(path: &Path, recursive: bool, show_metadata: bool, pattern: Option<&str>) -> EmpathicResult<Vec<FileInfo>> {
        let mut files = Vec::new();
        
        if recursive {
            Self::list_files_recursive(path, &mut files, show_metadata, pattern).await?;
        } else {
            Self::list_files_single(path, &mut files, show_metadata, pattern).await?;
        }
        
        Ok(files)
    }
    
    async fn list_files_single(path: &Path, files: &mut Vec<FileInfo>, show_metadata: bool, pattern: Option<&str>) -> EmpathicResult<()> {
        let mut entries = tokio::fs::read_dir(path).await
            .map_err(|e| EmpathicError::FileOperationFailed {
                operation: "read directory".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;
        
        while let Some(entry) = entries.next_entry().await? {
            let file_info = Self::create_file_info(&entry, show_metadata).await?;
            
            // Apply pattern filter if specified
            if let Some(pattern) = pattern
                && !Self::matches_pattern(&file_info.name, pattern)?
            {
                continue;
            }
            
            files.push(file_info);
        }
        
        Ok(())
    }
    
    async fn list_files_recursive(path: &Path, files: &mut Vec<FileInfo>, show_metadata: bool, pattern: Option<&str>) -> EmpathicResult<()> {
        let path_owned = path.to_owned();
        let entries = tokio::task::spawn_blocking(move || {
            // Use ignore crate for .gitignore support 🎯
            let walker = ignore::WalkBuilder::new(&path_owned)
                .hidden(false)        // Show hidden files by default
                .ignore(true)         // Respect .ignore files
                .git_ignore(true)     // Respect .gitignore files 
                .git_global(false)    // Don't use global git config
                .git_exclude(false)   // Don't use .git/info/exclude
                .require_git(false)   // Work in non-git directories
                                .standard_filters(true) // Use standard filters for gitignore functionality
                .build();
            
            let mut result = Vec::new();
            for entry in walker {
                match entry {
                    Ok(entry) => result.push(entry),
                    Err(e) => return Err(EmpathicError::FileOperationFailed {
                operation: "directory walk".to_string(),
                path: std::path::PathBuf::from("unknown"),
                reason: e.to_string(),
            }),
                }
            }
            Ok(result)
        }).await??;
        
        for entry in entries {
            let metadata = if show_metadata {
                Some(entry.metadata()?)
            } else {
                None
            };
            
            let file_info = FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_path_buf(),
                is_dir: entry.file_type().is_some_and(|ft| ft.is_dir()),
                size: metadata.as_ref().map(|m| m.len()),
                modified: metadata.as_ref().and_then(|m| m.modified().ok()),
                permissions: if cfg!(unix) {
                    metadata.as_ref().map(|m| {
                        use std::os::unix::fs::PermissionsExt;
                        format!("{:o}", m.permissions().mode())
                    })
                } else {
                    None
                },
            };
            
            // Apply pattern filter if specified
            if let Some(pattern) = pattern
                && !Self::matches_pattern(&file_info.name, pattern)?
            {
                continue;
            }
            
            files.push(file_info);
        }
        
        Ok(())
    }
    
    async fn create_file_info(entry: &tokio::fs::DirEntry, show_metadata: bool) -> EmpathicResult<FileInfo> {
        let metadata = if show_metadata {
            Some(entry.metadata().await?)
        } else {
            None
        };
        
        // 🎯 AI Enhancement: Always check if it's a directory, regardless of show_metadata flag
        let file_type = entry.file_type().await?;
        
        Ok(FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            is_dir: file_type.is_dir(),  // ✅ Always determine directory status correctly
            size: metadata.as_ref().map(|m| m.len()),
            modified: metadata.as_ref().and_then(|m| m.modified().ok()),
            permissions: if cfg!(unix) {
                metadata.as_ref().map(|m| {
                    use std::os::unix::fs::PermissionsExt;
                    format!("{:o}", m.permissions().mode())
                })
            } else {
                None
            },
        })
    }
    
    /// Delete file or directory
    pub async fn delete_file(path: &Path, recursive: bool) -> EmpathicResult<()> {
        if path.is_dir() {
            if recursive {
                tokio::fs::remove_dir_all(path).await
                    .map_err(|e| EmpathicError::FileOperationFailed {
                        operation: "remove directory recursively".to_string(),
                        path: path.to_path_buf(),
                        reason: e.to_string(),
                    })?;
            } else {
                tokio::fs::remove_dir(path).await
                    .map_err(|e| EmpathicError::FileOperationFailed {
                        operation: "remove directory".to_string(),
                        path: path.to_path_buf(),
                        reason: e.to_string(),
                    })?;
            }
        } else {
            tokio::fs::remove_file(path).await
                .map_err(|e| EmpathicError::FileOperationFailed {
                    operation: "remove file".to_string(),
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                })?;
        }
        Ok(())
    }

    
    /// Check if filename matches glob pattern
    fn matches_pattern(filename: &str, pattern: &str) -> EmpathicResult<bool> {
        use glob::Pattern;
        
        let glob_pattern = Pattern::new(pattern)
            .map_err(|e| EmpathicError::InvalidRegexPattern {
                pattern: pattern.to_string(),
                reason: format!("Invalid glob pattern: {}", e),
            })?;
        
        Ok(glob_pattern.matches(filename))
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<std::time::SystemTime>,
    pub permissions: Option<String>,
}
