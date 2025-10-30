//! 🔬 LSP Tools Comprehensive Test Suite
//!
//! Individual comprehensive tests for each LSP tool including:
//! - Diagnostics: Error detection and classification testing
//! - Hover: Type information and documentation testing
//! - Completion: Context-aware autocomplete testing
//! - Goto Definition: Symbol navigation testing
//! - Find References: Reference discovery testing
//! - Document Symbols: File structure testing
//! - Workspace Symbols: Project-wide symbol search testing

pub mod diagnostics;
pub mod hover;
pub mod completion;
pub mod goto_definition;
pub mod find_references;
pub mod document_symbols;
pub mod workspace_symbols;
