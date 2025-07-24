//! Core types and data structures for the markdowndown library.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A newtype wrapper for markdown content with validation and conversion methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Markdown(String);

impl Markdown {
    /// Returns the markdown content as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates that the markdown content is not empty or whitespace-only.
    pub fn validate(&self) -> Result<(), MarkdownError> {
        if self.0.trim().is_empty() {
            Err(MarkdownError::ParseError {
                message: "Markdown content cannot be empty or whitespace-only".to_string(),
            })
        } else {
            Ok(())
        }
    }
}

impl From<String> for Markdown {
    fn from(content: String) -> Self {
        Markdown(content)
    }
}

impl From<Markdown> for String {
    fn from(val: Markdown) -> Self {
        val.0
    }
}

impl AsRef<str> for Markdown {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Markdown {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Markdown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Enumeration of supported URL types for content extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrlType {
    /// Generic HTML pages
    Html,
    /// Google Docs documents
    GoogleDocs,
    /// Office 365 documents
    Office365,
    /// GitHub issues
    GitHubIssue,
}

/// Error types for the markdowndown library.
#[derive(Debug, Error)]
pub enum MarkdownError {
    /// Network-related errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Parsing errors
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Invalid URL errors
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    /// Authentication errors
    #[error("Authentication error: {message}")]
    AuthError { message: String },
}

/// Frontmatter structure for document metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frontmatter {
    /// The source URL of the document
    pub source_url: String,
    /// The exporter that generated this markdown
    pub exporter: String,
    /// The date and time when the document was downloaded
    pub date_downloaded: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_creation_from_string() {
        let content = "# Hello World";
        let markdown = Markdown::from(content.to_string());
        assert_eq!(markdown.as_str(), content);
    }

    #[test]
    fn test_markdown_display() {
        let content = "# Hello World\n\nThis is a test.";
        let markdown = Markdown::from(content.to_string());
        assert_eq!(format!("{}", markdown), content);
    }

    #[test]
    fn test_markdown_validation_valid() {
        let markdown = Markdown::from("# Valid markdown".to_string());
        assert!(markdown.validate().is_ok());
    }

    #[test]
    fn test_markdown_validation_empty() {
        let markdown = Markdown::from("".to_string());
        assert!(markdown.validate().is_err());
    }

    #[test]
    fn test_markdown_validation_whitespace_only() {
        let markdown = Markdown::from("   \n\t  ".to_string());
        assert!(markdown.validate().is_err());
    }

    #[test]
    fn test_urltype_serialization() {
        let url_type = UrlType::GoogleDocs;
        let serialized = serde_yaml::to_string(&url_type).unwrap();
        let deserialized: UrlType = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(url_type, deserialized);
    }

    #[test]
    fn test_markdown_error_display() {
        let error = MarkdownError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert_eq!(error.to_string(), "Network error: Connection timeout");
    }

    #[test]
    fn test_frontmatter_yaml_serialization() {
        let frontmatter = Frontmatter {
            source_url: "https://example.com".to_string(),
            exporter: "markdowndown".to_string(),
            date_downloaded: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let yaml = serde_yaml::to_string(&frontmatter).unwrap();
        let deserialized: Frontmatter = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(frontmatter, deserialized);
    }
}
