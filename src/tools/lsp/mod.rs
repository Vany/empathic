//! 🧠 LSP Tools - Language Server Protocol integration tools for empathic
//!
//! Provides semantic code analysis capabilities through external LSP servers

pub mod base;
pub mod completion;
pub mod diagnostics;
pub mod document_symbols;
pub mod find_references;
pub mod goto_definition;
pub mod hover;
pub mod workspace_symbols;

pub use completion::LspCompletionTool;
pub use diagnostics::LspDiagnosticsTool;
pub use document_symbols::LspDocumentSymbolsTool;
pub use find_references::LspFindReferencesTool;
pub use goto_definition::LspGotoDefinitionTool;
pub use hover::LspHoverTool;
pub use workspace_symbols::LspWorkspaceSymbolsTool;
