use std::env;
use std::path::{Path, PathBuf};

/// 🛡️ Enhanced security validation
pub struct SecurityValidator {
    allowed_root: PathBuf,
    max_file_size: u64,
    blocked_extensions: &'static [&'static str],
}

impl SecurityValidator {
    pub fn new() -> Result<Self, String> {
        let root = env::var("ROOT_DIR")
            .or_else(|_| env::var("PROJECT_DIR"))
            .map_err(|_| "❌ No ROOT_DIR/PROJECT_DIR environment variable set")?;

        let allowed_root = Path::new(&root)
            .canonicalize()
            .map_err(|e| format!("❌ Invalid root directory: {e}"))?;

        Ok(Self {
            allowed_root,
            max_file_size: 100 * 1024 * 1024, // 100MB limit
            blocked_extensions: &["exe", "dll", "so", "dylib", "bin"],
        })
    }

    /// Validate path with enhanced security checks
    pub fn validate_path(&self, path: &str) -> Result<PathBuf, SecurityError> {
        // 🚫 Block null bytes and control characters
        if path.contains('\0')
            || path
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err(SecurityError::InvalidCharacters);
        }

        // 🚫 Block path traversal attempts
        if path.contains("..") || path.contains("//") {
            return Err(SecurityError::PathTraversal);
        }

        let target_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.allowed_root.join(path)
        };

        // 🔍 Canonicalize and validate containment
        let canonical = target_path
            .canonicalize()
            .or_else(|_| {
                // For non-existent files, validate parent
                if let Some(parent) = target_path.parent() {
                    parent
                        .canonicalize()
                        .map(|p| p.join(target_path.file_name().unwrap()))
                } else {
                    Ok(target_path)
                }
            })
            .map_err(|_| SecurityError::InvalidPath)?;

        if !canonical.starts_with(&self.allowed_root) {
            return Err(SecurityError::OutsideRoot);
        }

        // 🚫 Check blocked extensions
        if let Some(ext) = canonical.extension().and_then(|e| e.to_str()) {
            if self
                .blocked_extensions
                .contains(&ext.to_lowercase().as_str())
            {
                return Err(SecurityError::BlockedExtension(ext.to_string()));
            }
        }

        Ok(canonical)
    }

    /// Validate file size for operations
    pub fn validate_file_size(&self, path: &Path) -> Result<(), SecurityError> {
        if path.is_file() {
            let size = path
                .metadata()
                .map_err(|_| SecurityError::InvalidPath)?
                .len();

            if size > self.max_file_size {
                return Err(SecurityError::FileTooLarge(size));
            }
        }
        Ok(())
    }

    /// 🛡️ Validate tool arguments for security
    pub fn validate_tool_args(name: &str, args: &serde_json::Value) -> Result<(), String> {
        let validator = get_security_validator();

        // File operation tools requiring path validation
        if matches!(
            name,
            "read_file"
                | "write_file"
                | "edit_file_range"
                | "insert_at_line"
                | "search_replace"
                | "delete_file"
                | "move_file"
        ) {
            if let Some(path) = args.get("path").and_then(|p| p.as_str()) {
                validator.validate_path(path).map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            allowed_root: PathBuf::from("/tmp"),
            max_file_size: 100 * 1024 * 1024,
            blocked_extensions: &["exe", "dll", "so", "dylib", "bin"],
        })
    }
}

#[derive(Debug)]
pub enum SecurityError {
    InvalidCharacters,
    PathTraversal,
    InvalidPath,
    OutsideRoot,
    BlockedExtension(String),
    FileTooLarge(u64),
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InvalidCharacters => write!(f, "🚫 Invalid characters in path"),
            SecurityError::PathTraversal => write!(f, "🚫 Path traversal attempt detected"),
            SecurityError::InvalidPath => write!(f, "🚫 Invalid file path"),
            SecurityError::OutsideRoot => {
                write!(f, "🛡️ Access denied: path outside allowed directory")
            }
            SecurityError::BlockedExtension(ext) => write!(f, "🚫 Blocked file extension: .{ext}"),
            SecurityError::FileTooLarge(size) => write!(f, "🚫 File too large: {size} bytes"),
        }
    }
}

/// Global security validator instance
use std::sync::OnceLock;
static SECURITY_VALIDATOR: OnceLock<SecurityValidator> = OnceLock::new();

pub fn get_security_validator() -> &'static SecurityValidator {
    SECURITY_VALIDATOR.get_or_init(SecurityValidator::default)
}

/// Enhanced validate_path function for backward compatibility
#[allow(dead_code)] // Reserved for enhanced security features
pub fn validate_path_secure(path: &str) -> Result<PathBuf, String> {
    get_security_validator()
        .validate_path(path)
        .map_err(|e| e.to_string())
}
