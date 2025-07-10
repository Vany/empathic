use anyhow::Result;

mod config;
mod fs;
mod mcp;
mod tools;

use config::Config;
use mcp::McpServer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    
    let mut server = McpServer::new(config);
    server.run().await?;
    
    Ok(())
}
