//! Content converters for transforming various formats to markdown.
//!
//! This module provides converters for different content types, enabling
//! the transformation of HTML, documents, and other formats into clean markdown.

/// HTML to markdown converter
pub mod html;

/// Google Docs to markdown converter
pub mod google_docs;

// Re-export main converter types for convenience
pub use html::HtmlConverter;
pub use google_docs::GoogleDocsConverter;
