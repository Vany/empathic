# Tasks to be done

## ✅ COMPLETED: LSP Implementation Planning
 - ✅ Architecture analysis (3 options evaluated)
 - ✅ Recommended approach: Embedded LSP Server
 - ✅ Detailed technical design with code examples
 - ✅ Phase-by-phase implementation roadmap
 - ✅ Performance and testing strategies
 - ✅ **Decision**: Dual-protocol server with tree-sitter AST support

## ✅ COMPLETED: Project Cleanup and Critical Bug Fixes
 - ✅ Removed debug artifacts (debug_path_test.rs, debug_test*, test_replace.txt)
 - ✅ Fixed failing replace tool test (schema validation issue)
 - ✅ Fixed gitignore functionality (.gitignore files now properly included)
 - ✅ Resolved syntax errors and duplicate code
 - ✅ **CRITICAL**: Fixed Unicode boundary violations in replace tool causing crashes on 中文 content
 - ✅ **CRITICAL**: Fixed MCP format violation - early return bypassed format_json_response wrapper
 - ✅ Restored Unicode-safe string operations using regex crate built-ins
 - ✅ **Status**: 14/14 test suites passing ✅, production ready

## [ ] IMPLEMENT: LSP Foundation (Phase 1)
 - [ ] Create dual protocol message router in `main.rs`
 - [ ] Add `LspHandler` struct and basic capabilities
 - [ ] Implement protocol detection logic (MCP vs LSP messages)
 - [ ] Set up shared state infrastructure for file management
 - [ ] Add tree-sitter dependencies to `Cargo.toml`

## [ ] IMPLEMENT: Core LSP Features (Phase 2)  
 - [ ] LSP initialization and server capabilities
 - [ ] textDocument lifecycle (didOpen, didChange, didClose)
 - [ ] Basic Rust syntax tree parsing with tree-sitter
 - [ ] Diagnostic integration with existing logging

## [ ] IMPLEMENT: AST Tools (Phase 3)
 - [ ] Add MCP tools: `ast_parse`, `ast_query`, `ast_edit`
 - [ ] XPath-like AST node selector engine
 - [ ] Basic edit operations (insert, replace, delete)
 - [ ] Text edit generation and file application

## ✅ COMPLETED: Fix Major Issues
 - ✅ Fixed Unicode boundary violations causing crashes on multi-byte characters
 - ✅ Fixed MCP format violations in replace tool response format
 - ✅ All other critical issues resolved
