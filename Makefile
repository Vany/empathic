# 🚀 empathic MCP Server Makefile

# Environment variables
export ROOT_DIR ?= $(shell pwd)
export ADD_PATH ?= 

# Rust environment
CARGO = cargo
RUSTFLAGS = 

# Build configurations
DEBUG_TARGET = target/debug/empathic
RELEASE_TARGET = target/release/empathic

.PHONY: all build release clean test run dev check fmt clippy install

all: build

# 🏗️ Build debug version
build:
	$(CARGO) build

# 🚀 Build release version
release:
	$(CARGO) build --release

# 🧹 Clean build artifacts
clean:
	$(CARGO) clean

# 🧪 Run tests
test:
	$(CARGO) test

# 🏃 Run the server (debug mode)
run: build
	$(DEBUG_TARGET)

# 🔧 Development mode with auto-reload
dev:
	$(CARGO) watch -x run

# ✅ Check code without building
check:
	$(CARGO) check

# 🎨 Format code
fmt:
	$(CARGO) fmt

# 📎 Lint code
clippy:
	$(CARGO) clippy -- -D warnings

# 📦 Install locally
install:
	$(CARGO) install --path .

# 🔍 Show environment
env:
	@echo "ROOT_DIR: $(ROOT_DIR)"
	@echo "ADD_PATH: $(ADD_PATH)"
	@echo "Current directory: $(shell pwd)"

# 📝 Show help
help:
	@echo "🚀 empathic MCP Server - Available commands:"
	@echo "  build    - Build debug version"
	@echo "  release  - Build release version"
	@echo "  clean    - Clean build artifacts"
	@echo "  test     - Run tests"
	@echo "  run      - Run the server (debug mode)"
	@echo "  dev      - Development mode with auto-reload"
	@echo "  check    - Check code without building"
	@echo "  fmt      - Format code"
	@echo "  clippy   - Lint code"
	@echo "  install  - Install locally"
	@echo "  env      - Show environment variables"
	@echo "  help     - Show this help"
