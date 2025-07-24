//! Content converters for transforming various formats to markdown.
//!
//! This module provides converters for different content types, enabling
//! the transformation of HTML, documents, and other formats into clean markdown.

/// HTML to markdown converter
pub mod html;

/// GitHub Issues to markdown converter
pub mod github;

// Re-export main converter types for convenience
pub use html::HtmlConverter;
pub use github::GitHubConverter;
