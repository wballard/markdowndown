//! # MarkdownDown
//!
//! A Rust library for acquiring markdown from URLs with smart handling.
//!
//! This library provides a unified interface for extracting and converting content
//! from various URL sources (HTML pages, Google Docs, Office 365, GitHub) into
//! clean markdown format.
//!
//! ## Architecture
//!
//! The library follows a modular architecture:
//! - Core types and traits for extensible URL handling
//! - HTTP client wrapper for consistent network operations
//! - URL type detection for automatic handler selection
//! - Specific handlers for each supported URL type
//! - Unified public API for simple integration

/// Core types, traits, and error definitions
pub mod types;

/// HTTP client wrapper for network operations
pub mod client;

/// Content converters for different formats
pub mod converters;

/// URL type detection and classification
pub mod detection {}

/// Handler implementations for different URL types
pub mod handlers {

    /// HTML page handler
    pub mod html {}

    /// Google Docs handler
    pub mod google_docs {}

    /// Office 365 handler  
    pub mod office365 {}

    /// GitHub handler
    pub mod github {}
}

/// Main library API
pub mod api {}

// Re-export main API items for convenience
// TODO: Re-enable when api module has exports
// pub use api::*;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_available() {
        // Verify version follows semantic versioning pattern (major.minor.patch)
        assert!(VERSION.chars().any(|c| c.is_ascii_digit()));
        assert!(VERSION.contains('.'));
        // Basic format validation - should have at least one dot for major.minor
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "Version should have at least major.minor format"
        );
    }
}
