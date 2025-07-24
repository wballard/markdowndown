//! Content converters for transforming various formats to markdown.
//!
//! This module provides converters for different content types, enabling
//! the transformation of HTML, documents, and other formats into clean markdown.

/// Configuration options for HTML conversion
pub mod config;

/// HTML preprocessing utilities
pub mod preprocessor;

/// Markdown postprocessing utilities
pub mod postprocessor;

/// HTML to markdown converter
pub mod html;

/// Google Docs to markdown converter
pub mod google_docs;

/// Placeholder converters for services not yet fully implemented
pub mod placeholder;

// Re-export main converter types for convenience
pub use config::HtmlConverterConfig;
pub use html::HtmlConverter;
pub use google_docs::GoogleDocsConverter;
pub use placeholder::{GitHubIssueConverter, Office365Converter};
