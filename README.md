# Empathic MCP Server

A production-ready [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server providing comprehensive file management and command execution tools for AI assistants.

## Overview

Empathic is designed to integrate with Claude Desktop, providing 14 specialized tools that enable sophisticated file operations and command execution within your development workspace. It implements the full MCP specification over JSON-RPC 2.0, ensuring reliable communication and proper error handling.

## Features

### File System Operations (8 tools)
- **Environment access** - Read environment variables with PATH enhancement
- **File reading** - Unicode-safe file reading with optional chunking
- **File writing** - Atomic file writing with line-range replacement support
- **Directory listing** - Recursive directory traversal with glob patterns and .gitignore support
- **File deletion** - Safe file and directory removal with recursive capabilities
- **Text replacement** - Advanced search and replace with regex and fuzzy matching
- **Directory creation** - Create directories with automatic parent directory creation
- **Symbolic links** - Cross-platform symbolic link creation and management

### Command Execution (6 tools)
- **Shell commands** - Execute arbitrary shell commands with full bash feature support
- **Git operations** - Complete git command execution with working directory control
- **Rust projects** - Cargo-based Rust project management and build operations
- **Build automation** - Make-based build system execution and target management
- **Java/JVM projects** - Gradle-based project management and dependency handling
- **Node.js projects** - npm package management and script execution

## Installation

### Build from Source

```bash
# Clone repository
git clone <repository-url>
cd empathic

# Build release binary
make release

# Binary will be available at target/release/empathic
```

### System Requirements

- **Development**: macOS with Rust 1.87+
- **Deployment**: Ubuntu 24.10+ or equivalent Linux distribution
- **Dependencies**: Self-contained binary with no external runtime requirements

## Configuration

### Environment Variables

```bash
# Required
ROOT_DIR=/path/to/your/workspace

# Optional
ADD_PATH=/additional/bin/paths  # Colon-separated additional PATH entries
LOGLEVEL=warn                   # Log level: debug, info, warn, error
```

### Claude Desktop Integration

Add to your Claude Desktop configuration file:

```json
{
  "mcpServers": {
    "empathic": {
      "command": "/path/to/empathic",
      "env": {
        "ROOT_DIR": "/Users/username/projects",
        "LOGLEVEL": "warn"
      }
    }
  }
}
```

## Usage

Once configured, empathic runs automatically when Claude Desktop starts. The server provides tools that Claude can invoke to:

- Read and write files in your workspace
- Execute development commands (git, cargo, npm, etc.)
- Navigate and search through project directories
- Perform text transformations and replacements
- Manage project build processes

All operations are restricted to the configured `ROOT_DIR` for security.

## Development

### Build Commands

```bash
make build    # Debug build for development
make release  # Optimized production build
make test     # Run full test suite
make check    # Run linting and formatting checks
make clean    # Clean build artifacts
```

### Project Structure

```
src/
├── main.rs           # Entry point and JSON-RPC server
├── lib.rs            # Library exports
├── config.rs         # Configuration management
├── mcp.rs            # MCP protocol implementation
├── fs.rs             # Filesystem utilities
└── tools/            # MCP tool implementations
    ├── mod.rs        # Tool registry and common utilities
    ├── env.rs        # Environment variable access
    ├── read_file.rs  # File reading operations
    ├── write_file.rs # File writing operations
    ├── list_files.rs # Directory listing
    ├── delete_file.rs # File deletion
    ├── replace.rs    # Text search and replace
    ├── mkdir.rs      # Directory creation
    ├── symlink.rs    # Symbolic link management
    └── executor.rs   # Command execution tools

tests/
├── common/           # Shared test utilities
└── *.rs             # Per-tool test files
```

### Testing

The project includes comprehensive tests covering:

- All 14 MCP tools with edge cases
- Unicode handling and international text support
- Cross-platform compatibility
- Error conditions and recovery
- Integration scenarios

Run tests with `make test` or `cargo test`.

## Technical Details

### Protocol Compliance
- **JSON-RPC 2.0**: Full specification compliance with proper error codes
- **MCP v1.0**: Complete Model Context Protocol implementation
- **Unicode Support**: Proper grapheme cluster handling for international text
- **Error Handling**: Structured error responses with contextual information

### Performance
- Optimized for typical development workflows
- Memory efficient with minimal runtime overhead
- Fast startup time for responsive tool execution
- Atomic file operations where possible

### Security
- All operations restricted to configured workspace directory
- No network access or external system modification
- Safe handling of user input and file paths
- Proper error isolation and recovery

## Logging

Empathic provides structured logging with configurable levels:

- **ERROR**: Tool failures and protocol errors
- **WARN**: Performance issues and fallback operations
- **INFO**: Tool executions and file operations
- **DEBUG**: Detailed protocol messages and internal state

Configure logging level with the `LOGLEVEL` environment variable.

## License

[Insert appropriate license information]

## Contributing

[Insert contribution guidelines if open source]

## Support

For issues and questions:
- Check the [troubleshooting guide](REQUIREMENTS.md)
- Review the [technical documentation](MEMO.md)
- [Insert contact information or issue tracker]

---

*Empathic MCP Server v1.0.0 - Production-ready file management and command execution for AI assistants.*