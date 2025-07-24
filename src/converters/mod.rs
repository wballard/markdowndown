//! Content converters for transforming various formats to markdown.
//!
//! This module provides converters for different content types, enabling
//! the transformation of HTML, documents, and other formats into clean markdown.

/// HTML to markdown converter
pub mod html;

/// Office 365 to markdown converter
pub mod office365;

// Re-export main converter types for convenience
pub use html::HtmlConverter;
pub use office365::Office365Converter;
