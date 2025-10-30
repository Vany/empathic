use empathic::{Config, McpServer, EmpathicResult};
use std::io::Write;
use std::sync::{Arc, Mutex};

/// 📝 TeeWriter - writes to both stderr and a file simultaneously
/// CRITICAL: stdout is reserved exclusively for JSON-RPC protocol messages
/// All logging MUST go to stderr to avoid breaking MCP protocol
struct TeeWriter {
    file: Arc<Mutex<std::fs::File>>,
}

impl TeeWriter {
    fn new(file: std::fs::File) -> Self {
        Self {
            file: Arc::new(Mutex::new(file)),
        }
    }
}

impl Write for TeeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Write to stderr (NOT stdout - stdout is reserved for JSON-RPC)
        std::io::stderr().write_all(buf)?;
        
        // Write to file
        if let Ok(mut file) = self.file.lock() {
            file.write_all(buf)?;
            file.flush()?;
        }
        
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stderr().flush()?;
        if let Ok(mut file) = self.file.lock() {
            file.flush()?;
        }
        Ok(())
    }
}

/// 🔧 Initialize logging with optional file output
fn init_logging() -> EmpathicResult<()> {
    use env_logger::Builder;
    use std::env;
    use std::path::PathBuf;

    let mut builder = Builder::new();
    
    // Parse RUST_LOG or default to info
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    } else {
        builder.filter_level(log::LevelFilter::Info);
    }
    
    // Check if LOGFILE is set and construct path with ROOT_DIR
    if let Ok(logfile_name) = env::var("LOGFILE") {
        // Construct full path: ROOT_DIR/LOGFILE
        let logfile_path = if let Ok(root_dir) = env::var("ROOT_DIR") {
            PathBuf::from(root_dir).join(&logfile_name)
        } else {
            // Fallback to LOGFILE as-is if ROOT_DIR not set
            PathBuf::from(&logfile_name)
        };
        
        // Open file in append mode
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&logfile_path)
            .map_err(|e| empathic::EmpathicError::FileOperationFailed {
                operation: "open logfile".to_string(),
                path: logfile_path.clone(),
                reason: e.to_string(),
            })?;
        
        // Create tee writer
        let tee_writer = TeeWriter::new(file);
        
        // Configure builder to use tee writer
        builder.target(env_logger::Target::Pipe(Box::new(tee_writer)));
        
        // Print to stderr before logger is initialized
        eprintln!("📝 Logging to file: {}", logfile_path.display());
    }
    
    builder.init();
    Ok(())
}

#[tokio::main]
async fn main() -> EmpathicResult<()> {
    // Initialize logging early with optional file output
    init_logging()?;
    
    // Create config with improved error handling
    let config = match Config::from_env() {
        Ok(config) => {
            log::info!("🚀 Configuration loaded: {}", config.summary());
            config
        },
        Err(e) => {
            eprintln!("❌ Configuration error: {}", e);
            
            // Provide helpful hints based on error type
            if e.is_config_error() {
                eprintln!("💡 Hint: Check your environment variables:");
                eprintln!("   export ROOT_DIR=/path/to/your/workspace");
                eprintln!("   export LOGLEVEL=info  # Optional: trace,debug,info,warn,error");
                eprintln!("   export ADD_PATH=/extra/bins  # Optional: additional PATH entries");
            }
            
            std::process::exit(1);
        }
    };
    
    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("❌ Configuration validation failed: {}", e);
        eprintln!("💡 Category: {}", e.category());
        std::process::exit(1);
    }
    
    // Create and run server
    let mut server = McpServer::new(config);
    if let Err(e) = server.run().await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
