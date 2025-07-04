// 🪵 MCP-Compatible Logging System
// Ensures stdout is pure JSON for Claude.ai desktop integration

use std::env;
use std::fmt;
use std::sync::OnceLock;

/// 🪵 Log levels for filtering output
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2, 
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN "),
            LogLevel::Info => write!(f, "INFO "),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

impl LogLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            LogLevel::Error => "❌",
            LogLevel::Warn => "⚠️",
            LogLevel::Info => "ℹ️", 
            LogLevel::Debug => "🐛",
            LogLevel::Trace => "🔍",
        }
    }
}

/// 🛠️ Logger configuration
#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
    enabled: bool,
    use_color: bool,
    use_emoji: bool,
}

impl Logger {
    fn new() -> Self {
        let level = env::var("LOG_LEVEL")
            .ok()
            .and_then(|s| match s.to_uppercase().as_str() {
                "ERROR" => Some(LogLevel::Error),
                "WARN" => Some(LogLevel::Warn),
                "INFO" => Some(LogLevel::Info),
                "DEBUG" => Some(LogLevel::Debug),
                "TRACE" => Some(LogLevel::Trace),
                _ => None,
            })
            .unwrap_or(LogLevel::Warn); // Default to WARN for MCP compatibility

        let enabled = env::var("LOG_DISABLED")
            .map(|s| s != "1" && s.to_uppercase() != "TRUE")
            .unwrap_or(true);

        let use_color = env::var("LOG_COLOR")
            .map(|s| s == "1" || s.to_uppercase() == "TRUE")
            .unwrap_or(false); // Disabled by default for MCP

        let use_emoji = env::var("LOG_EMOJI")
            .map(|s| s == "1" || s.to_uppercase() == "TRUE")
            .unwrap_or(true); // Enabled by default

        Self {
            level,
            enabled,
            use_color,
            use_emoji,
        }
    }

    fn should_log(&self, level: LogLevel) -> bool {
        self.enabled && level <= self.level
    }

    fn format_message(&self, level: LogLevel, component: &str, message: &str) -> String {
        let timestamp = chrono::Utc::now().format("%H:%M:%S%.3f");
        
        let emoji = if self.use_emoji { level.emoji() } else { "" };
        let emoji_space = if self.use_emoji { " " } else { "" };
        
        if self.use_color {
            let color = match level {
                LogLevel::Error => "\x1b[31m", // Red
                LogLevel::Warn => "\x1b[33m",  // Yellow
                LogLevel::Info => "\x1b[36m",  // Cyan
                LogLevel::Debug => "\x1b[32m", // Green
                LogLevel::Trace => "\x1b[35m", // Magenta
            };
            let reset = "\x1b[0m";
            
            format!("[{timestamp}] {color}[{level}]{reset} {emoji}{emoji_space}[{component}] {message}")
        } else {
            format!("[{timestamp}] [{level}] {emoji}{emoji_space}[{component}] {message}")
        }
    }

    pub fn log(&self, level: LogLevel, component: &str, message: &str) {
        if self.should_log(level) {
            let formatted = self.format_message(level, component, message);
            // 🎯 CRITICAL: Use eprintln! to send to stderr, not stdout
            eprintln!("{formatted}");
        }
    }
}

// 🔧 Global logger instance
static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn get_logger() -> &'static Logger {
    LOGGER.get_or_init(Logger::new)
}

/// 🪵 Logging macros for easy use throughout codebase
#[macro_export]
macro_rules! log_error {
    ($component:expr, $($arg:tt)*) => {
        $crate::logging::get_logger().log(
            $crate::logging::LogLevel::Error, 
            $component, 
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($component:expr, $($arg:tt)*) => {
        $crate::logging::get_logger().log(
            $crate::logging::LogLevel::Warn, 
            $component, 
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($component:expr, $($arg:tt)*) => {
        $crate::logging::get_logger().log(
            $crate::logging::LogLevel::Info, 
            $component, 
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($component:expr, $($arg:tt)*) => {
        $crate::logging::get_logger().log(
            $crate::logging::LogLevel::Debug, 
            $component, 
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_trace {
    ($component:expr, $($arg:tt)*) => {
        $crate::logging::get_logger().log(
            $crate::logging::LogLevel::Trace, 
            $component, 
            &format!($($arg)*)
        )
    };
}

/// 🚀 Startup logging for server initialization
pub fn log_startup() {
    log_info!("main", "🤖 Empathic MCP Server v5.3.0 - RAG Auto-Managed + Prompts");
    log_info!("main", "💡 RAG stack will auto-start on first use and auto-stop on exit");
    log_info!("main", "💬 Includes 10 specialized prompts for development workflows");
    log_debug!("main", "📊 Log level: {}, Enabled: {}", 
        get_logger().level, get_logger().enabled);
}

/// 🛑 Shutdown logging
pub fn log_shutdown() {
    log_info!("main", "🛑 Editor shutting down...");
    log_info!("main", "✅ Goodbye!");
}

/// 🧠 RAG engine logging
#[allow(dead_code)]
pub fn log_rag_init(message: &str) {
    log_info!("rag", "{}", message);
}

#[allow(dead_code)]
pub fn log_rag_debug(message: &str) {
    log_debug!("rag", "{}", message);
}

/// 📊 Embeddings logging  
#[allow(dead_code)]
pub fn log_embeddings_info(message: &str) {
    log_info!("embeddings", "{}", message);
}

#[allow(dead_code)]
pub fn log_embeddings_debug(message: &str) {
    log_debug!("embeddings", "{}", message);
}

/// 🔧 Configuration display
#[allow(dead_code)]
pub fn display_config() {
    let logger = get_logger();
    log_debug!("config", "📋 Logging Configuration:");
    log_debug!("config", "   🎚️ Level: {}", logger.level);
    log_debug!("config", "   ✅ Enabled: {}", logger.enabled);
    log_debug!("config", "   🎨 Color: {}", logger.use_color);
    log_debug!("config", "   😀 Emoji: {}", logger.use_emoji);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    #[test]
    fn test_logger_filtering() {
        let logger = Logger {
            level: LogLevel::Info,
            enabled: true,
            use_color: false,
            use_emoji: false,
        };

        assert!(logger.should_log(LogLevel::Error));
        assert!(logger.should_log(LogLevel::Warn));
        assert!(logger.should_log(LogLevel::Info));
        assert!(!logger.should_log(LogLevel::Debug));
        assert!(!logger.should_log(LogLevel::Trace));
    }
}