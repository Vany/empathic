//! 🗂️ Filesystem helpers for test scenarios

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// 📁 Create a directory structure from a map of relative paths to content
pub async fn create_structure(
    root: &Path,
    structure: &HashMap<&str, &str>,
) -> Result<Vec<PathBuf>> {
    let mut created_files = Vec::new();
    
    for (relative_path, content) in structure {
        let full_path = root.join(relative_path);
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Write file content
        fs::write(&full_path, content).await?;
        created_files.push(full_path);
    }
    
    Ok(created_files)
}

/// 🎯 Create test files for specific scenarios
pub async fn create_test_files(
    root: &Path,
    files: &[(&str, &str)],
) -> Result<Vec<PathBuf>> {
    let mut created_paths = Vec::new();
    
    for (path, content) in files {
        let full_path = root.join(path);
        
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        fs::write(&full_path, content).await?;
        created_paths.push(full_path);
    }
    
    Ok(created_paths)
}

/// 🔍 Verify file content matches expected
pub async fn verify_file_content(path: &Path, expected: &str) -> Result<bool> {
    let actual = fs::read_to_string(path).await?;
    Ok(actual == expected)
}

/// 📊 Count files in directory (recursive)
pub fn count_files_recursive(dir: &Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize>> + '_>> {
    Box::pin(async move {
        let mut count = 0;
        let mut entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_files_recursive(&path).await?;
            }
        }
        
        Ok(count)
    })
}

/// 🎲 Generate random test directory name
pub fn random_test_dir() -> PathBuf {
    std::env::temp_dir().join(format!("ee_test_{}", rand::random::<u32>()))
}

/// 🧹 Cleanup multiple paths
pub async fn cleanup_paths(paths: &[&Path]) -> Result<()> {
    for path in paths {
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(path).await?;
            } else {
                fs::remove_file(path).await?;
            }
        }
    }
    Ok(())
}

/// 📏 Get file size
pub async fn file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path).await?;
    Ok(metadata.len())
}

/// 🔗 Create symlink helper
pub async fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        fs::symlink(target, link).await?;
    }
    #[cfg(windows)]
    {
        if target.is_dir() {
            fs::symlink_dir(target, link).await?;
        } else {
            fs::symlink_file(target, link).await?;
        }
    }
    Ok(())
}

/// 📂 Get relative path from root to target
pub fn get_relative_path(root: &Path, target: &Path) -> Option<PathBuf> {
    target.strip_prefix(root).ok().map(|p| p.to_path_buf())
}

/// 🔄 Copy directory structure (for testing copy operations)
pub fn copy_dir_recursive<'a>(src: &'a Path, dst: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(dst).await?;
        
        let mut entries = fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name();
            let dest_path = dst.join(name);
            
            if path.is_dir() {
                copy_dir_recursive(&path, &dest_path).await?;
            } else {
                fs::copy(&path, &dest_path).await?;
            }
        }
        
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_structure() {
        let temp_dir = TempDir::new().unwrap();
        let structure = [
            ("file1.txt", "content1"),
            ("dir/file2.txt", "content2"),
        ].into_iter().collect::<HashMap<_, _>>();
        
        let files = create_structure(temp_dir.path(), &structure).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let content = fs::read_to_string(&files[0]).await.unwrap();
        assert!(content == "content1" || content == "content2");
    }

    #[tokio::test]
    async fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        fs::write(&test_file, "hello").await.unwrap();
        
        assert!(verify_file_content(&test_file, "hello").await.unwrap());
        assert!(!verify_file_content(&test_file, "world").await.unwrap());
        
        let size = file_size(&test_file).await.unwrap();
        assert_eq!(size, 5);
    }

    #[tokio::test]
    async fn test_count_files() {
        let temp_dir = TempDir::new().unwrap();
        let files = [
            ("a.txt", "1"),
            ("b.txt", "2"),
            ("dir/c.txt", "3"),
        ];
        
        create_test_files(temp_dir.path(), &files).await.unwrap();
        
        let count = count_files_recursive(temp_dir.path()).await.unwrap();
        assert_eq!(count, 3);
    }
}
