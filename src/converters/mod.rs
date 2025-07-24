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

/// Placeholder converters for services not yet fully implemented
pub mod placeholder;

// Re-export main converter types for convenience
pub use config::HtmlConverterConfig;
pub use html::HtmlConverter;
pub use placeholder::{GitHubIssueConverter, GoogleDocsConverter, Office365Converter};
