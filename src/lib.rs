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
pub mod types {
    //! Core types and trait definitions for the MarkdownDown library
}

/// HTTP client wrapper for network operations
pub mod client {
    //! HTTP client wrapper providing consistent network operations
}

/// URL type detection and classification
pub mod detection {
    //! URL type detection and classification utilities
}

/// Handler implementations for different URL types
pub mod handlers {
    //! Specific handlers for different URL types
    
    /// HTML page handler
    pub mod html {
        //! Handler for converting HTML pages to markdown
    }
    
    /// Google Docs handler
    pub mod google_docs {
        //! Handler for Google Docs URLs
    }
    
    /// Office 365 handler  
    pub mod office365 {
        //! Handler for Office 365 document URLs
    }
    
    /// GitHub handler
    pub mod github {
        //! Handler for GitHub issues, PRs, and other content
    }
}

/// Main library API
pub mod api {
    //! Unified public API for the MarkdownDown library
}

// Re-export main API items for convenience
pub use api::*;

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_available() {
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION, "0.1.0");
    }
}