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

/// YAML frontmatter generation and manipulation utilities
pub mod frontmatter;

/// URL type detection and classification
pub mod detection;

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
    use crate::detection::UrlDetector;
    use crate::converters::GitHubConverter;
    use crate::types::UrlType;

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

    #[test]
    fn test_github_integration_issue_and_pr() {
        // Test integration between URL detection and GitHub converter
        let detector = UrlDetector::new();
        let converter = GitHubConverter::new();

        // Test GitHub issue URL
        let issue_url = "https://github.com/microsoft/vscode/issues/12345";
        let detected_type = detector.detect_type(issue_url).unwrap();
        assert_eq!(detected_type, UrlType::GitHubIssue);

        // Verify GitHub converter can parse the issue URL
        let parsed_issue = converter.parse_github_url(issue_url).unwrap();
        assert_eq!(parsed_issue.owner, "microsoft");
        assert_eq!(parsed_issue.repo, "vscode");
        assert_eq!(parsed_issue.number, 12345);

        // Test GitHub pull request URL
        let pr_url = "https://github.com/rust-lang/rust/pull/98765";
        let detected_type = detector.detect_type(pr_url).unwrap();
        assert_eq!(detected_type, UrlType::GitHubIssue);

        // Verify GitHub converter can parse the PR URL
        let parsed_pr = converter.parse_github_url(pr_url).unwrap();
        assert_eq!(parsed_pr.owner, "rust-lang");
        assert_eq!(parsed_pr.repo, "rust");
        assert_eq!(parsed_pr.number, 98765);
    }
}
