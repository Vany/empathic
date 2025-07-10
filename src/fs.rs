use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;
use anyhow::{Result, Context};

/// Unicode-aware file operations 🦀
pub struct FileOps;

impl FileOps {
    /// Read entire file content
    pub async fn read_file(path: &Path) -> Result<String> {
        let content = tokio::fs::read_to_string(path).await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        Ok(content)
    }
    
    /// Read file content with line-based chunking
    pub async fn read_file_chunk(path: &Path, line_offset: usize, line_length: Option<usize>) -> Result<String> {
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
    pub async fn write_file(path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        tokio::fs::write(path, content).await
            .with_context(|| format!("Failed to write file: {}", path.display()))?;
        Ok(())
    }
    
    /// Write file content with line-based range replacement
    pub async fn write_file_range(path: &Path, content: &str, start: usize, end: Option<usize>) -> Result<()> {
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
    pub async fn list_files(path: &Path, recursive: bool, show_metadata: bool, pattern: Option<&str>) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        
        if recursive {
            Self::list_files_recursive(path, &mut files, show_metadata, pattern).await?;
        } else {
            Self::list_files_single(path, &mut files, show_metadata, pattern).await?;
        }
        
        Ok(files)
    }
    
    async fn list_files_single(path: &Path, files: &mut Vec<FileInfo>, show_metadata: bool, pattern: Option<&str>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(path).await
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;
        
        while let Some(entry) = entries.next_entry().await? {
            let file_info = Self::create_file_info(&entry, show_metadata).await?;
            
            // Apply pattern filter if specified
            if let Some(pattern) = pattern {
                if !Self::matches_pattern(&file_info.name, pattern)? {
                    continue;
                }
            }
            
            files.push(file_info);
        }
        
        Ok(())
    }
    
    async fn list_files_recursive(path: &Path, files: &mut Vec<FileInfo>, show_metadata: bool, pattern: Option<&str>) -> Result<()> {
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
                .require_git(false)   // Work in non-git directories
                                .standard_filters(true) // Use standard filters for gitignore functionality
                .build();
            
            let mut result = Vec::new();
            for entry in walker {
                match entry {
                    Ok(entry) => result.push(entry),
                    Err(e) => return Err(anyhow::anyhow!("Walk error: {}", e)),
                }
            }
            Ok(result)
        }).await??;
        
        // 🔧 FIX: Manually add .gitignore files that might be filtered out by standard_filters
        let gitignore_path = path.join(".gitignore");
        let mut gitignore_missing = false;
        if gitignore_path.exists() {
            // Check if .gitignore is already in the results
            let has_gitignore = entries.iter().any(|entry| {
                entry.path() == gitignore_path
            });
            
            if !has_gitignore {
                gitignore_missing = true;
            }
        }
        
        
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
            if let Some(pattern) = pattern {
                if !Self::matches_pattern(&file_info.name, pattern)? {
                    continue;
                }
            }
            
            files.push(file_info);
        }
        
        
        // 🔧 FIX: Add .gitignore file if it was filtered out but exists
        if gitignore_missing {
            let gitignore_metadata = std::fs::metadata(&gitignore_path)?;
            let gitignore_info = FileInfo {
                name: ".gitignore".to_string(),
                path: gitignore_path,
                is_dir: false,
                size: if show_metadata { Some(gitignore_metadata.len()) } else { None },
                modified: if show_metadata { gitignore_metadata.modified().ok() } else { None },
                permissions: if cfg!(unix) && show_metadata {
                    Some({
                        use std::os::unix::fs::PermissionsExt;
                        format!("{:o}", gitignore_metadata.permissions().mode() & 0o777)
                    })
                } else {
                    None
                },
            };
            
            // Check if .gitignore matches pattern (if pattern filtering is active)
            if let Some(pattern) = pattern {
                if Self::matches_pattern(".gitignore", pattern)? {
                    files.push(gitignore_info);
                }
            } else {
                files.push(gitignore_info);
            }
        }
        
        Ok(())
    }
    
    async fn create_file_info(entry: &tokio::fs::DirEntry, show_metadata: bool) -> Result<FileInfo> {
        let metadata = if show_metadata {
            Some(entry.metadata().await?)
        } else {
            None
        };
        
        Ok(FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            is_dir: metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
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
    pub async fn delete_file(path: &Path, recursive: bool) -> Result<()> {
        if path.is_dir() {
            if recursive {
                tokio::fs::remove_dir_all(path).await
                    .with_context(|| format!("Failed to remove directory recursively: {}", path.display()))?;
            } else {
                tokio::fs::remove_dir(path).await
                    .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
            }
        } else {
            tokio::fs::remove_file(path).await
                .with_context(|| format!("Failed to remove file: {}", path.display()))?;
        }
        Ok(())
    }
    
    /// Get unicode-aware character position in text
    #[allow(dead_code)]
    pub fn char_position(text: &str, byte_offset: usize) -> usize {
        text.grapheme_indices(true)
            .take_while(|(i, _)| *i < byte_offset)
            .count()
    }
    
    /// Get byte offset from character position
    #[allow(dead_code)]
    pub fn byte_offset(text: &str, char_position: usize) -> usize {
        text.grapheme_indices(true)
            .nth(char_position)
            .map(|(i, _)| i)
            .unwrap_or(text.len())
    }
    
    /// Check if filename matches glob pattern
    fn matches_pattern(filename: &str, pattern: &str) -> Result<bool> {
        use glob::Pattern;
        
        let glob_pattern = Pattern::new(pattern)
            .with_context(|| format!("Invalid glob pattern: {pattern}"))?;
        
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

// Add missing import for unix permissions