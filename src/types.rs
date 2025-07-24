//! Core types and data structures for the markdowndown library.
//!
//! This module provides the fundamental data types used throughout the markdowndown library,
//! including validated wrappers for markdown content and URLs, error handling, and metadata structures.
//!
//! # Usage Examples
//!
//! ## Basic Markdown Processing
//!
//! ```rust
//! use markdowndown::types::{Markdown, MarkdownError};
//!
//! // Create validated markdown content
//! let markdown = Markdown::new("# Hello World\n\nThis is a test document.".to_string())?;
//! println!("Content: {}", markdown);
//! println!("As string: {}", markdown.as_str());
//!
//! // Validation catches empty content
//! let invalid = Markdown::new("   \n\t  ".to_string());
//! assert!(invalid.is_err());
//! # Ok::<(), MarkdownError>(())
//! ```
//!
//! ## URL Validation and Processing
//!
//! ```rust
//! use markdowndown::types::{Url, MarkdownError};
//!
//! // Create validated URLs
//! let url = Url::new("https://docs.google.com/document/d/123".to_string())?;
//! println!("URL: {}", url);
//!
//! // Invalid URLs are rejected
//! let invalid_url = Url::new("not-a-url".to_string());
//! assert!(invalid_url.is_err());
//! # Ok::<(), MarkdownError>(())
//! ```
//!
//! ## Working with Frontmatter
//!
//! ```rust
//! use markdowndown::types::{Frontmatter, Url, UrlType};
//! use chrono::Utc;
//!
//! // Create document metadata
//! let frontmatter = Frontmatter {
//!     source_url: Url::new("https://example.com/document".to_string())?,
//!     exporter: "markdowndown".to_string(),
//!     date_downloaded: Utc::now(),
//! };
//!
//! // Serialize to YAML for document headers
//! let yaml = serde_yaml::to_string(&frontmatter)?;
//! println!("YAML frontmatter:\n{}", yaml);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Complete Document Processing Workflow
//!
//! ```rust
//! use markdowndown::types::{Markdown, Frontmatter, Url, UrlType};
//! use chrono::Utc;
//!
//! // Process a complete document with validation and metadata
//! let source_url = Url::new("https://docs.google.com/document/d/abc123".to_string())?;
//! let content = Markdown::new("# Project Overview\n\nThis document outlines...".to_string())?;
//!
//! let metadata = Frontmatter {
//!     source_url,
//!     exporter: "markdowndown-v1.0".to_string(),
//!     date_downloaded: Utc::now(),
//! };
//!
//! // Generate complete markdown document with frontmatter
//! let yaml_header = serde_yaml::to_string(&metadata)?;
//! let complete_document = format!("---\n{}---\n\n{}", yaml_header, content);
//!
//! println!("Complete document:\n{}", complete_document);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Error Handling Patterns
//!
//! ```rust
//! use markdowndown::types::{Markdown, Url, MarkdownError};
//!
//! fn process_content(url_str: String, content: String) -> Result<(Url, Markdown), MarkdownError> {
//!     let url = Url::new(url_str)?;  // May fail with InvalidUrl
//!     let markdown = Markdown::new(content)?;  // May fail with ParseError
//!     Ok((url, markdown))
//! }
//!
//! match process_content("https://example.com".to_string(), "# Valid content".to_string()) {
//!     Ok((url, markdown)) => {
//!         println!("Successfully processed: {} -> {}", url, markdown);
//!     }
//!     Err(MarkdownError::InvalidUrl { url }) => {
//!         eprintln!("Invalid URL: {}", url);
//!     }
//!     Err(MarkdownError::ParseError { message }) => {
//!         eprintln!("Content validation failed: {}", message);
//!     }
//!     Err(e) => {
//!         eprintln!("Other error: {}", e);
//!     }
//! }
//! # Ok::<(), MarkdownError>(())
//! ```
//!
//! ## URL Type Detection
//!
//! ```rust
//! use markdowndown::types::UrlType;
//!
//! // Different URL types for specialized processing
//! let google_docs = UrlType::GoogleDocs;
//! let github_issue = UrlType::GitHubIssue;
//! let html_page = UrlType::Html;
//!
//! println!("Processing {} content", google_docs);  // "Processing Google Docs content"
//! println!("Processing {} content", github_issue); // "Processing GitHub Issue content"
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A newtype wrapper for markdown content with validation and conversion methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Markdown(String);

impl Markdown {
    /// Creates a new Markdown instance with validation.
    ///
    /// # Errors
    ///
    /// Returns a `MarkdownError::ParseError` if the content is empty or whitespace-only.
    pub fn new(content: String) -> Result<Self, MarkdownError> {
        let markdown = Markdown(content);
        markdown.validate()?;
        Ok(markdown)
    }

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

    /// Creates a new Markdown instance with frontmatter prepended to the content.
    ///
    /// # Arguments
    ///
    /// * `frontmatter` - The YAML frontmatter string (should include delimiters)
    ///
    /// # Returns
    ///
    /// A new `Markdown` instance containing the frontmatter and original content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::types::Markdown;
    ///
    /// let content = Markdown::new("# Hello World".to_string())?;
    /// let frontmatter = "---\nsource_url: \"https://example.com\"\n---\n";
    /// let with_frontmatter = content.with_frontmatter(frontmatter);
    ///
    /// assert!(with_frontmatter.as_str().contains("source_url"));
    /// assert!(with_frontmatter.as_str().contains("# Hello World"));
    /// # Ok::<(), markdowndown::types::MarkdownError>(())
    /// ```
    pub fn with_frontmatter(&self, frontmatter: &str) -> Markdown {
        let combined_content = format!("{}\n{}", frontmatter, self.0);
        Markdown(combined_content)
    }

    /// Extracts the frontmatter from the markdown content if present.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the YAML frontmatter (including delimiters)
    /// if found, or `None` if no frontmatter is present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::types::Markdown;
    ///
    /// let content = "---\nsource_url: \"https://example.com\"\n---\n\n# Hello World";
    /// let markdown = Markdown::from(content.to_string());
    /// let frontmatter = markdown.frontmatter();
    ///
    /// assert!(frontmatter.is_some());
    /// assert!(frontmatter.unwrap().contains("source_url"));
    /// ```
    pub fn frontmatter(&self) -> Option<String> {
        // Check if content starts with frontmatter delimiters
        if !self.0.starts_with("---\n") {
            return None;
        }

        // Find the closing delimiter
        let content_after_start = &self.0[4..]; // Skip "---\n"
        if let Some(end_pos) = content_after_start.find("\n---\n") {
            let full_frontmatter = &self.0[..4 + end_pos + 5]; // Include both delimiters
            Some(full_frontmatter.to_string())
        } else {
            None
        }
    }

    /// Returns only the content portion of the markdown, stripping any frontmatter.
    ///
    /// # Returns
    ///
    /// A `String` containing only the markdown content without frontmatter
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::types::Markdown;
    ///
    /// let content = "---\nsource_url: \"https://example.com\"\n---\n\n# Hello World\n\nContent here.";
    /// let markdown = Markdown::from(content.to_string());
    /// let content_only = markdown.content_only();
    ///
    /// assert_eq!(content_only, "# Hello World\n\nContent here.");
    /// ```
    pub fn content_only(&self) -> String {
        // Check if content starts with frontmatter delimiters
        if !self.0.starts_with("---\n") {
            return self.0.clone();
        }

        // Find the closing delimiter
        let content_after_start = &self.0[4..]; // Skip "---\n"
        if let Some(end_pos) = content_after_start.find("\n---\n") {
            let content_start = 4 + end_pos + 5; // Skip "---\n" + frontmatter + "\n---\n"
            if content_start < self.0.len() {
                // Skip any leading newlines from the combination
                let remaining = &self.0[content_start..];
                if let Some(stripped) = remaining.strip_prefix('\n') {
                    stripped.to_string()
                } else {
                    remaining.to_string()
                }
            } else {
                String::new()
            }
        } else {
            // No closing delimiter found, return original content
            self.0.clone()
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

/// A newtype wrapper for URLs with validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Url(String);

impl Url {
    /// Creates a new URL instance with basic validation.
    ///
    /// # Errors
    ///
    /// Returns a `MarkdownError::InvalidUrl` if the URL format is invalid.
    pub fn new(url: String) -> Result<Self, MarkdownError> {
        // Basic URL validation - must start with http:// or https:// and have content after
        if (url.starts_with("http://") && url.len() > 7)
            || (url.starts_with("https://") && url.len() > 8)
        {
            Ok(Url(url))
        } else {
            Err(MarkdownError::InvalidUrl { url })
        }
    }

    /// Returns the URL as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Enumeration of supported URL types for content extraction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl fmt::Display for UrlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlType::Html => write!(f, "HTML"),
            UrlType::GoogleDocs => write!(f, "Google Docs"),
            UrlType::Office365 => write!(f, "Office 365"),
            UrlType::GitHubIssue => write!(f, "GitHub Issue"),
        }
    }
}

/// Error context providing detailed information about where and how an error occurred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorContext {
    /// The URL being processed when the error occurred
    pub url: String,
    /// The operation being performed (e.g., "URL detection", "Document download")  
    pub operation: String,
    /// The converter type being used (e.g., "GoogleDocsConverter", "GitHubConverter")
    pub converter_type: String,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Additional contextual information
    pub additional_info: Option<String>,
}

impl ErrorContext {
    /// Creates a new error context with the specified details.
    pub fn new(
        url: impl Into<String>,
        operation: impl Into<String>, 
        converter_type: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            operation: operation.into(),
            converter_type: converter_type.into(),
            timestamp: Utc::now(),
            additional_info: None,
        }
    }

    /// Adds additional contextual information to the error context.
    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.additional_info = Some(info.into());
        self
    }
}

/// Validation error kinds for input validation failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationErrorKind {
    InvalidUrl,
    InvalidFormat,
    MissingParameter,
}

/// Network error kinds for connection and communication failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkErrorKind {
    Timeout,
    ConnectionFailed,
    DnsResolution,
    RateLimited,
    ServerError(u16),
}

/// Authentication error kinds for authorization failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthErrorKind {
    MissingToken,
    InvalidToken,
    PermissionDenied,
    TokenExpired,
}

/// Content error kinds for data processing failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentErrorKind {
    EmptyContent,
    UnsupportedFormat,
    ParsingFailed,
}

/// Converter error kinds for external tool and processing failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConverterErrorKind {
    ExternalToolFailed,
    ProcessingError,
    UnsupportedOperation,
}

/// Configuration error kinds for setup and configuration failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigErrorKind {
    InvalidConfig,
    MissingDependency,
    InvalidValue,
}

/// Error types for the markdowndown library.
#[derive(Debug, Error)]
pub enum MarkdownError {
    /// Validation errors for invalid input
    #[error("Validation error: {kind:?} - {context:?}")]
    ValidationError {
        kind: ValidationErrorKind,
        context: ErrorContext,
    },

    /// Enhanced network-related errors with detailed context
    #[error("Network error: {kind:?} - {context:?}")]
    EnhancedNetworkError {
        kind: NetworkErrorKind,
        context: ErrorContext,
    },

    /// Authentication and authorization errors
    #[error("Authentication error: {kind:?} - {context:?}")]
    AuthenticationError {
        kind: AuthErrorKind,
        context: ErrorContext,
    },

    /// Content processing and parsing errors
    #[error("Content error: {kind:?} - {context:?}")]
    ContentError {
        kind: ContentErrorKind,
        context: ErrorContext,
    },

    /// Converter-specific processing errors
    #[error("Converter error: {kind:?} - {context:?}")]
    ConverterError {
        kind: ConverterErrorKind,
        context: ErrorContext,
    },

    /// Configuration and system setup errors
    #[error("Configuration error: {kind:?} - {context:?}")]  
    ConfigurationError {
        kind: ConfigErrorKind,
        context: ErrorContext,
    },

    // Legacy error types for backward compatibility - keep the exact same names and structures
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

    /// Legacy configuration errors - renamed to avoid conflicts
    #[error("Configuration error: {message}")]
    LegacyConfigurationError { message: String },
}

impl MarkdownError {
    /// Returns the error context if available.
    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            MarkdownError::ValidationError { context, .. } => Some(context),
            MarkdownError::EnhancedNetworkError { context, .. } => Some(context),
            MarkdownError::AuthenticationError { context, .. } => Some(context),
            MarkdownError::ContentError { context, .. } => Some(context),
            MarkdownError::ConverterError { context, .. } => Some(context),
            MarkdownError::ConfigurationError { context, .. } => Some(context),
            _ => None,
        }
    }

    /// Returns true if this error is potentially retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            MarkdownError::EnhancedNetworkError { kind, .. } => match kind {
                NetworkErrorKind::Timeout => true,
                NetworkErrorKind::ConnectionFailed => true,
                NetworkErrorKind::DnsResolution => false,
                NetworkErrorKind::RateLimited => true,
                NetworkErrorKind::ServerError(status) => *status >= 500,
            },
            MarkdownError::AuthenticationError { kind, .. } => match kind {
                AuthErrorKind::TokenExpired => true,
                _ => false,
            },
            // Legacy network errors - use simple heuristics based on message content
            MarkdownError::NetworkError { message } => {
                message.contains("timeout") || message.contains("connection") || message.contains("rate limit")
            },
            _ => false,
        }
    }

    /// Returns true if recovery strategies should be attempted.
    pub fn is_recoverable(&self) -> bool {
        match self {
            MarkdownError::EnhancedNetworkError { .. } => true,
            MarkdownError::AuthenticationError { .. } => true,
            MarkdownError::ConverterError { .. } => true,
            MarkdownError::ContentError { kind, .. } => match kind {
                ContentErrorKind::UnsupportedFormat => true,
                _ => false,
            },
            // Legacy errors are considered recoverable for network and auth issues
            MarkdownError::NetworkError { .. } => true,
            MarkdownError::AuthError { .. } => true,
            _ => false,
        }
    }

    /// Returns user-friendly suggestions for resolving this error.
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            MarkdownError::ValidationError { kind, .. } => match kind {
                ValidationErrorKind::InvalidUrl => vec![
                    "Ensure the URL starts with http:// or https://".to_string(),
                    "Check that the URL is complete and properly formatted".to_string(),
                    "Try copying the URL directly from your browser".to_string(),
                ],
                ValidationErrorKind::InvalidFormat => vec![
                    "Verify the input format matches the expected pattern".to_string(),
                ],
                ValidationErrorKind::MissingParameter => vec![
                    "Check that all required parameters are provided".to_string(),
                ],
            },
            MarkdownError::EnhancedNetworkError { kind, .. } => match kind {
                NetworkErrorKind::Timeout => vec![
                    "Check your internet connection".to_string(),
                    "Try again in a few minutes".to_string(),
                    "Consider increasing the timeout in configuration".to_string(),
                ],
                NetworkErrorKind::ConnectionFailed => vec![
                    "Verify the server is accessible".to_string(),
                    "Check if you're behind a firewall or proxy".to_string(),
                ],
                NetworkErrorKind::DnsResolution => vec![
                    "Check that the domain name is correct".to_string(),
                    "Try using a different DNS server".to_string(),
                ],
                NetworkErrorKind::RateLimited => vec![
                    "Wait before making additional requests".to_string(),
                    "Consider authenticating to increase rate limits".to_string(),
                ],
                NetworkErrorKind::ServerError(_) => vec![
                    "The server is experiencing issues".to_string(),
                    "Try again later".to_string(),
                ],
            },
            MarkdownError::AuthenticationError { kind, .. } => match kind {
                AuthErrorKind::MissingToken => vec![
                    "Set up authentication credentials".to_string(),
                    "Check the documentation for authentication requirements".to_string(),
                ],
                AuthErrorKind::InvalidToken => vec![
                    "Verify your authentication token is correct".to_string(),
                    "Generate a new token if the current one is invalid".to_string(),
                ],
                AuthErrorKind::PermissionDenied => vec![
                    "Ensure you have permission to access this resource".to_string(),
                    "Check that your token has the required scopes".to_string(),
                ],
                AuthErrorKind::TokenExpired => vec![
                    "Refresh or regenerate your authentication token".to_string(),
                ],
            },
            MarkdownError::ContentError { kind, .. } => match kind {
                ContentErrorKind::EmptyContent => vec![
                    "Verify the source contains actual content".to_string(),
                    "Check if the URL is publicly accessible".to_string(),
                ],
                ContentErrorKind::UnsupportedFormat => vec![
                    "Try using a different converter for this content type".to_string(),
                    "Check if the content format is supported".to_string(),
                ],
                ContentErrorKind::ParsingFailed => vec![
                    "The content format may be corrupted or unsupported".to_string(),
                    "Try accessing the content directly to verify it's valid".to_string(),
                ],
            },
            MarkdownError::ConverterError { kind, .. } => match kind {
                ConverterErrorKind::ExternalToolFailed => vec![
                    "Check that required external tools are installed".to_string(),
                    "Verify tool dependencies and PATH configuration".to_string(),
                ],
                ConverterErrorKind::ProcessingError => vec![
                    "Try again with different converter settings".to_string(),
                    "Check if the input format is supported".to_string(),
                ],
                ConverterErrorKind::UnsupportedOperation => vec![
                    "This operation is not supported for this content type".to_string(),
                    "Try using a different converter or approach".to_string(),
                ],
            },
            MarkdownError::ConfigurationError { kind, .. } => match kind {
                ConfigErrorKind::InvalidConfig => vec![
                    "Check your configuration file for syntax errors".to_string(),
                    "Verify all configuration values are valid".to_string(),
                ],
                ConfigErrorKind::MissingDependency => vec![
                    "Install the required dependencies".to_string(),
                    "Check the documentation for setup requirements".to_string(),
                ],
                ConfigErrorKind::InvalidValue => vec![
                    "Check that configuration values are within valid ranges".to_string(),
                    "Refer to documentation for valid configuration options".to_string(),
                ],
            },
            // Legacy error suggestions
            MarkdownError::NetworkError { .. } => vec![
                "Check your internet connection".to_string(),
                "Try again in a few minutes".to_string(),
                "The server may be experiencing issues".to_string(),
            ],
            MarkdownError::ParseError { .. } => vec![
                "Verify the content format is supported".to_string(),
                "Check if the source content is valid".to_string(),
            ],
            MarkdownError::InvalidUrl { .. } => vec![
                "Ensure the URL starts with http:// or https://".to_string(),
                "Check that the URL is complete and properly formatted".to_string(),
                "Try copying the URL directly from your browser".to_string(),
            ],
            MarkdownError::AuthError { .. } => vec![
                "Check your authentication credentials".to_string(),
                "Verify that your token has the required permissions".to_string(),
                "Consider regenerating your authentication token".to_string(),
            ],
            MarkdownError::LegacyConfigurationError { .. } => vec![
                "Check your configuration file for errors".to_string(),
                "Verify all configuration values are valid".to_string(),
            ],
        }
    }
}

/// Frontmatter structure for document metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frontmatter {
    /// The source URL of the document
    pub source_url: Url,
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
    fn test_markdown_new_valid() {
        let content = "# Hello World";
        let markdown = Markdown::new(content.to_string()).unwrap();
        assert_eq!(markdown.as_str(), content);
    }

    #[test]
    fn test_markdown_new_invalid_empty() {
        let result = Markdown::new("".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ParseError { message } => {
                assert_eq!(
                    message,
                    "Markdown content cannot be empty or whitespace-only"
                );
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_markdown_new_invalid_whitespace() {
        let result = Markdown::new("   \n\t  ".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ParseError { message } => {
                assert_eq!(
                    message,
                    "Markdown content cannot be empty or whitespace-only"
                );
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_markdown_display() {
        let content = "# Hello World\n\nThis is a test.";
        let markdown = Markdown::from(content.to_string());
        assert_eq!(format!("{markdown}"), content);
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
    fn test_markdown_with_frontmatter() {
        let content = Markdown::from("# Hello World\n\nThis is content.".to_string());
        let frontmatter = "---\nsource_url: \"https://example.com\"\nexporter: markdowndown\n---\n";
        let result = content.with_frontmatter(frontmatter);

        assert!(result
            .as_str()
            .contains("source_url: \"https://example.com\""));
        assert!(result.as_str().contains("# Hello World"));
        assert!(result.as_str().starts_with("---\n"));
    }

    #[test]
    fn test_markdown_frontmatter_extraction() {
        let content =
            "---\nsource_url: https://example.com\nexporter: markdowndown\n---\n\n# Hello World";
        let markdown = Markdown::from(content.to_string());
        let frontmatter = markdown.frontmatter();

        assert!(frontmatter.is_some());
        let fm = frontmatter.unwrap();
        assert!(fm.contains("source_url: https://example.com"));
        assert!(fm.starts_with("---\n"));
        assert!(fm.ends_with("---\n"));
    }

    #[test]
    fn test_markdown_frontmatter_extraction_none() {
        let content = "# Hello World\n\nNo frontmatter here.";
        let markdown = Markdown::from(content.to_string());
        let frontmatter = markdown.frontmatter();

        assert!(frontmatter.is_none());
    }

    #[test]
    fn test_markdown_content_only() {
        let content = "---\nsource_url: https://example.com\nexporter: markdowndown\n---\n\n# Hello World\n\nContent here.";
        let markdown = Markdown::from(content.to_string());
        let content_only = markdown.content_only();

        assert_eq!(content_only, "# Hello World\n\nContent here.");
        assert!(!content_only.contains("source_url"));
    }

    #[test]
    fn test_markdown_content_only_no_frontmatter() {
        let content = "# Hello World\n\nNo frontmatter here.";
        let markdown = Markdown::from(content.to_string());
        let content_only = markdown.content_only();

        assert_eq!(content_only, content);
    }

    #[test]
    fn test_markdown_frontmatter_roundtrip() {
        let original_content = "# Test Document\n\nThis is test content.";
        let frontmatter = "---\nsource_url: https://example.com\nexporter: markdowndown\n---\n";

        let markdown = Markdown::from(original_content.to_string());
        let with_frontmatter = markdown.with_frontmatter(frontmatter);

        // Extract frontmatter back
        let extracted_frontmatter = with_frontmatter.frontmatter();
        assert!(extracted_frontmatter.is_some());
        assert!(extracted_frontmatter.unwrap().contains("source_url"));

        // Extract content back
        let extracted_content = with_frontmatter.content_only();
        assert_eq!(extracted_content, original_content);
    }

    #[test]
    fn test_urltype_serialization() {
        let url_type = UrlType::GoogleDocs;
        let serialized = serde_yaml::to_string(&url_type).unwrap();
        let deserialized: UrlType = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(url_type, deserialized);
    }

    #[test]
    fn test_urltype_display() {
        assert_eq!(format!("{}", UrlType::Html), "HTML");
        assert_eq!(format!("{}", UrlType::GoogleDocs), "Google Docs");
        assert_eq!(format!("{}", UrlType::Office365), "Office 365");
        assert_eq!(format!("{}", UrlType::GitHubIssue), "GitHub Issue");
    }

    #[test]
    fn test_markdown_error_display() {
        let error = MarkdownError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert_eq!(error.to_string(), "Network error: Connection timeout");
    }

    #[test]
    fn test_url_new_valid_https() {
        let url = Url::new("https://example.com".to_string()).unwrap();
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn test_url_new_valid_http() {
        let url = Url::new("http://example.com".to_string()).unwrap();
        assert_eq!(url.as_str(), "http://example.com");
    }

    #[test]
    fn test_url_new_invalid() {
        let result = Url::new("not-a-url".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::InvalidUrl { url } => {
                assert_eq!(url, "not-a-url");
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_url_display() {
        let url = Url::new("https://example.com".to_string()).unwrap();
        assert_eq!(format!("{url}"), "https://example.com");
    }

    #[test]
    fn test_frontmatter_yaml_serialization() {
        let frontmatter = Frontmatter {
            source_url: Url::new("https://example.com".to_string()).unwrap(),
            exporter: "markdowndown".to_string(),
            date_downloaded: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let yaml = serde_yaml::to_string(&frontmatter).unwrap();
        let deserialized: Frontmatter = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(frontmatter, deserialized);
    }

    /// Integration tests for type interactions
    mod integration_tests {
        use super::*;

        #[test]
        fn test_markdown_with_frontmatter_complete_workflow() {
            // Test complete workflow: create validated types and serialize together
            let markdown =
                Markdown::new("# Hello World\n\nThis is a test document.".to_string()).unwrap();
            let url = Url::new("https://docs.google.com/document/d/123".to_string()).unwrap();
            let frontmatter = Frontmatter {
                source_url: url,
                exporter: "markdowndown".to_string(),
                date_downloaded: Utc::now(),
            };

            // Test that all components work together
            let yaml_frontmatter = serde_yaml::to_string(&frontmatter).unwrap();
            let full_document = format!("---\n{yaml_frontmatter}---\n\n{markdown}");

            assert!(full_document.contains("# Hello World"));
            assert!(full_document.contains("https://docs.google.com"));
            assert!(full_document.contains("markdowndown"));
        }

        #[test]
        fn test_error_propagation_url_to_frontmatter() {
            // Test that URL validation errors propagate correctly
            let invalid_url_result = Url::new("not-a-valid-url".to_string());
            assert!(invalid_url_result.is_err());

            match invalid_url_result.unwrap_err() {
                MarkdownError::InvalidUrl { url } => {
                    assert_eq!(url, "not-a-valid-url");
                }
                _ => panic!("Expected InvalidUrl error"),
            }
        }

        #[test]
        fn test_error_propagation_markdown_validation() {
            // Test that Markdown validation errors propagate correctly
            let invalid_markdown_result = Markdown::new("   \n\t  ".to_string());
            assert!(invalid_markdown_result.is_err());

            match invalid_markdown_result.unwrap_err() {
                MarkdownError::ParseError { message } => {
                    assert_eq!(
                        message,
                        "Markdown content cannot be empty or whitespace-only"
                    );
                }
                _ => panic!("Expected ParseError"),
            }
        }

        #[test]
        fn test_combined_type_serialization_roundtrip() {
            // Test that complex nested structures serialize and deserialize correctly
            let original_frontmatter = Frontmatter {
                source_url: Url::new("https://github.com/user/repo/issues/123".to_string())
                    .unwrap(),
                exporter: "markdowndown".to_string(),
                date_downloaded: DateTime::parse_from_rfc3339("2023-12-01T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            };

            // Serialize to YAML
            let yaml = serde_yaml::to_string(&original_frontmatter).unwrap();

            // Deserialize back
            let deserialized: Frontmatter = serde_yaml::from_str(&yaml).unwrap();

            // Verify all fields match exactly
            assert_eq!(original_frontmatter.source_url, deserialized.source_url);
            assert_eq!(original_frontmatter.exporter, deserialized.exporter);
            assert_eq!(
                original_frontmatter.date_downloaded,
                deserialized.date_downloaded
            );
        }
    }

    /// Property-like tests for validation edge cases
    mod validation_edge_cases {
        use super::*;

        #[test]
        fn test_markdown_validation_unicode_whitespace() {
            // Test various Unicode whitespace characters
            let unicode_whitespace_chars = [
                "\u{0009}", // TAB
                "\u{000A}", // LINE FEED
                "\u{000B}", // LINE TABULATION
                "\u{000C}", // FORM FEED
                "\u{000D}", // CARRIAGE RETURN
                "\u{0020}", // SPACE
                "\u{0085}", // NEXT LINE
                "\u{00A0}", // NO-BREAK SPACE
                "\u{1680}", // OGHAM SPACE MARK
                "\u{2000}", // EN QUAD
                "\u{2001}", // EM QUAD
                "\u{2002}", // EN SPACE
                "\u{2003}", // EM SPACE
                "\u{2004}", // THREE-PER-EM SPACE
                "\u{2005}", // FOUR-PER-EM SPACE
                "\u{2006}", // SIX-PER-EM SPACE
                "\u{2007}", // FIGURE SPACE
                "\u{2008}", // PUNCTUATION SPACE
                "\u{2009}", // THIN SPACE
                "\u{200A}", // HAIR SPACE
                "\u{2028}", // LINE SEPARATOR
                "\u{2029}", // PARAGRAPH SEPARATOR
                "\u{202F}", // NARROW NO-BREAK SPACE
                "\u{205F}", // MEDIUM MATHEMATICAL SPACE
                "\u{3000}", // IDEOGRAPHIC SPACE
            ];

            for whitespace in unicode_whitespace_chars {
                let only_whitespace = whitespace.repeat(5);
                let markdown_result = Markdown::new(only_whitespace);
                assert!(
                    markdown_result.is_err(),
                    "Should reject Unicode whitespace: {whitespace:?}"
                );
            }

            // Test combinations of different whitespace characters
            let mixed_whitespace = "  \u{2000}\u{3000}\t\n\u{00A0}  ".to_string();
            let markdown_result = Markdown::new(mixed_whitespace);
            assert!(
                markdown_result.is_err(),
                "Should reject mixed Unicode whitespace"
            );
        }

        #[test]
        fn test_markdown_validation_minimal_valid_content() {
            // Test minimal valid content that should pass validation
            let minimal_valid_cases = [
                "a",         // Single character
                "1",         // Single digit
                ".",         // Single punctuation
                "🚀",        // Single emoji
                " a ",       // Valid content with surrounding whitespace
                "\n\na\n\n", // Valid content with surrounding newlines
                "\t\ta\t\t", // Valid content with surrounding tabs
            ];

            for case in minimal_valid_cases {
                let markdown_result = Markdown::new(case.to_string());
                assert!(
                    markdown_result.is_ok(),
                    "Should accept minimal valid content: {case:?}"
                );
            }
        }

        #[test]
        fn test_url_validation_edge_cases() {
            // Test URL validation with various edge cases
            let valid_url_cases = [
                "http://example.com",
                "https://example.com",
                "http://localhost",
                "https://localhost:8080",
                "http://192.168.1.1",
                "https://sub.domain.com/path?query=value#fragment",
                "http://user:pass@example.com",
                "https://example.com:443/very/long/path/with/many/segments",
            ];

            for url_case in valid_url_cases {
                let url_result = Url::new(url_case.to_string());
                assert!(url_result.is_ok(), "Should accept valid URL: {url_case}");
            }

            let invalid_url_cases = [
                "ftp://example.com",       // Wrong protocol
                "example.com",             // Missing protocol
                "www.example.com",         // Missing protocol
                "mailto:test@example.com", // Wrong protocol
                "file:///path/to/file",    // Wrong protocol
                "",                        // Empty string
                "http://",                 // Incomplete
                "https://",                // Incomplete
            ];

            for url_case in invalid_url_cases {
                let url_result = Url::new(url_case.to_string());
                assert!(url_result.is_err(), "Should reject invalid URL: {url_case}");
            }
        }

        #[test]
        fn test_roundtrip_serialization_properties() {
            // Test that serialization roundtrips preserve all data exactly
            let test_cases = [
                (
                    "https://docs.google.com/document/d/1234567890abcdef",
                    "markdowndown-v1.0",
                    "2023-01-15T14:30:45Z",
                ),
                (
                    "http://localhost:3000/api/docs/12345",
                    "custom-exporter",
                    "2023-12-31T23:59:59Z",
                ),
                (
                    "https://github.com/user/repo/issues/999",
                    "github-markdown-exporter",
                    "2023-06-15T09:15:30Z",
                ),
            ];

            for (url_str, exporter_str, date_str) in test_cases {
                let original = Frontmatter {
                    source_url: Url::new(url_str.to_string()).unwrap(),
                    exporter: exporter_str.to_string(),
                    date_downloaded: DateTime::parse_from_rfc3339(date_str)
                        .unwrap()
                        .with_timezone(&Utc),
                };

                // Test YAML roundtrip
                let yaml = serde_yaml::to_string(&original).unwrap();
                let from_yaml: Frontmatter = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(
                    original, from_yaml,
                    "YAML roundtrip should preserve all data"
                );

                // Verify specific field preservation
                assert_eq!(original.source_url.as_str(), from_yaml.source_url.as_str());
                assert_eq!(original.exporter, from_yaml.exporter);
                assert_eq!(original.date_downloaded, from_yaml.date_downloaded);
            }
        }

        #[test]
        fn test_markdown_content_preservation() {
            // Test that various markdown content is preserved exactly
            let markdown_cases = [
                "# Simple Header",
                "## Header with *emphasis* and **bold**",
                "- List item 1\n- List item 2\n  - Nested item",
                "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```",
                "Link: [Example](https://example.com)",
                "![Image](https://example.com/image.png)",
                "> Blockquote with **bold** text",
                "Table | Header\n------|-------\nCell1 | Cell2",
                "Mixed content with 🚀 emoji and Unicode: café résumé naïve",
                "Line 1\n\nLine 3 (with blank line)\n\n\nLine 6 (multiple blanks)",
            ];

            for content in markdown_cases {
                let markdown = Markdown::new(content.to_string()).unwrap();
                assert_eq!(
                    markdown.as_str(),
                    content,
                    "Content should be preserved exactly"
                );
                assert_eq!(
                    format!("{markdown}"),
                    content,
                    "Display should match original"
                );
            }
        }

        /// Tests for the enhanced error handling system
        mod enhanced_error_handling_tests {
            use super::*;

            #[test]
            fn test_error_context_creation() {
                let context = ErrorContext::new(
                    "https://example.com/test",
                    "URL validation",
                    "TestConverter"
                );

                assert_eq!(context.url, "https://example.com/test");
                assert_eq!(context.operation, "URL validation");
                assert_eq!(context.converter_type, "TestConverter");
                assert!(context.additional_info.is_none());
                // Timestamp should be recent (within last few seconds)
                let now = Utc::now();
                let diff = (now - context.timestamp).num_seconds();
                assert!(diff >= 0 && diff < 5);
            }

            #[test]
            fn test_error_context_with_info() {
                let context = ErrorContext::new(
                    "https://example.com/test",
                    "URL validation",
                    "TestConverter"
                ).with_info("Additional debugging info");

                assert_eq!(context.additional_info, Some("Additional debugging info".to_string()));
            }

            #[test]
            fn test_validation_error_creation() {
                let context = ErrorContext::new(
                    "invalid-url",
                    "URL parsing",
                    "UrlValidator"
                );
                
                let error = MarkdownError::ValidationError {
                    kind: ValidationErrorKind::InvalidUrl,
                    context: context.clone(),
                };

                assert_eq!(error.context(), Some(&context));
                assert!(!error.is_retryable());
                assert!(!error.is_recoverable());
                
                let suggestions = error.suggestions();
                assert!(suggestions.contains(&"Ensure the URL starts with http:// or https://".to_string()));
            }

            #[test]
            fn test_network_error_retryable() {
                let context = ErrorContext::new(
                    "https://example.com",
                    "HTTP request",
                    "HttpClient"
                );

                // Test retryable network errors
                let timeout_error = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::Timeout,
                    context: context.clone(),
                };
                assert!(timeout_error.is_retryable());
                assert!(timeout_error.is_recoverable());

                let connection_error = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::ConnectionFailed,
                    context: context.clone(),
                };
                assert!(connection_error.is_retryable());

                let rate_limit_error = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::RateLimited,
                    context: context.clone(),
                };
                assert!(rate_limit_error.is_retryable());

                // Test non-retryable network errors
                let dns_error = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::DnsResolution,
                    context: context.clone(),
                };
                assert!(!dns_error.is_retryable());

                // Test server errors - 5xx should be retryable, 4xx should not
                let server_error_500 = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::ServerError(500),
                    context: context.clone(),
                };
                assert!(server_error_500.is_retryable());

                let client_error_404 = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::ServerError(404),
                    context,
                };
                assert!(!client_error_404.is_retryable());
            }

            #[test]
            fn test_auth_error_retryable() {
                let context = ErrorContext::new(
                    "https://api.example.com",
                    "API request",
                    "ApiClient"
                );

                // Only expired tokens should be retryable
                let expired_token_error = MarkdownError::AuthenticationError {
                    kind: AuthErrorKind::TokenExpired,
                    context: context.clone(),
                };
                assert!(expired_token_error.is_retryable());
                assert!(expired_token_error.is_recoverable());

                let missing_token_error = MarkdownError::AuthenticationError {
                    kind: AuthErrorKind::MissingToken,
                    context: context.clone(),
                };
                assert!(!missing_token_error.is_retryable());
                assert!(missing_token_error.is_recoverable());

                let invalid_token_error = MarkdownError::AuthenticationError {
                    kind: AuthErrorKind::InvalidToken,
                    context: context.clone(),
                };
                assert!(!invalid_token_error.is_retryable());

                let permission_denied_error = MarkdownError::AuthenticationError {
                    kind: AuthErrorKind::PermissionDenied,
                    context,
                };
                assert!(!permission_denied_error.is_retryable());
            }

            #[test]
            fn test_content_error_recovery() {
                let context = ErrorContext::new(
                    "https://example.com/document",
                    "Content parsing",
                    "ContentParser"
                );

                // Unsupported format should be recoverable
                let unsupported_format_error = MarkdownError::ContentError {
                    kind: ContentErrorKind::UnsupportedFormat,
                    context: context.clone(),
                };
                assert!(unsupported_format_error.is_recoverable());
                assert!(!unsupported_format_error.is_retryable());

                // Empty content and parsing failed should not be recoverable
                let empty_content_error = MarkdownError::ContentError {
                    kind: ContentErrorKind::EmptyContent,
                    context: context.clone(),
                };
                assert!(!empty_content_error.is_recoverable());

                let parsing_failed_error = MarkdownError::ContentError {
                    kind: ContentErrorKind::ParsingFailed,
                    context,
                };
                assert!(!parsing_failed_error.is_recoverable());
            }

            #[test]
            fn test_converter_error_recovery() {
                let context = ErrorContext::new(
                    "https://example.com/document",
                    "Document conversion",
                    "PandocConverter"
                );

                let converter_error = MarkdownError::ConverterError {
                    kind: ConverterErrorKind::ExternalToolFailed,
                    context,
                };
                assert!(converter_error.is_recoverable());
                assert!(!converter_error.is_retryable());
            }

            #[test]
            fn test_configuration_error_recovery() {
                let context = ErrorContext::new(
                    "file://config.yaml",
                    "Configuration loading",
                    "ConfigLoader"
                );

                let config_error = MarkdownError::ConfigurationError {
                    kind: ConfigErrorKind::InvalidConfig,
                    context,
                };
                assert!(!config_error.is_recoverable());
                assert!(!config_error.is_retryable());
            }

            #[test]
            fn test_error_suggestions_comprehensive() {
                let context = ErrorContext::new(
                    "https://example.com",
                    "Test operation",
                    "TestConverter"
                );

                // Test validation error suggestions
                let validation_error = MarkdownError::ValidationError {
                    kind: ValidationErrorKind::InvalidUrl,
                    context: context.clone(),
                };
                let suggestions = validation_error.suggestions();
                assert!(!suggestions.is_empty());
                assert!(suggestions.iter().any(|s| s.contains("http")));

                // Test network error suggestions  
                let network_error = MarkdownError::EnhancedNetworkError {
                    kind: NetworkErrorKind::Timeout,
                    context: context.clone(),
                };
                let suggestions = network_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("internet connection")));

                // Test auth error suggestions
                let auth_error = MarkdownError::AuthenticationError {
                    kind: AuthErrorKind::MissingToken,
                    context: context.clone(),
                };
                let suggestions = auth_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("authentication")));

                // Test content error suggestions
                let content_error = MarkdownError::ContentError {
                    kind: ContentErrorKind::EmptyContent,
                    context: context.clone(),
                };
                let suggestions = content_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("content")));

                // Test converter error suggestions
                let converter_error = MarkdownError::ConverterError {
                    kind: ConverterErrorKind::ExternalToolFailed,
                    context: context.clone(),
                };
                let suggestions = converter_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("external tools")));

                // Test configuration error suggestions
                let config_error = MarkdownError::ConfigurationError {
                    kind: ConfigErrorKind::InvalidConfig,
                    context,
                };
                let suggestions = config_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("configuration")));
            }

            #[test]
            fn test_legacy_error_compatibility() {
                // Test that legacy errors still work but don't have enhanced features
                let legacy_parse_error = MarkdownError::ParseError {
                    message: "Legacy parsing failed".to_string(),
                };
                
                assert!(legacy_parse_error.context().is_none());
                assert!(!legacy_parse_error.is_retryable());
                assert!(!legacy_parse_error.is_recoverable());
                
                let suggestions = legacy_parse_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("content format")));

                // Test legacy network error
                let legacy_network_error = MarkdownError::NetworkError {
                    message: "Connection timeout occurred".to_string(),
                };
                
                assert!(legacy_network_error.context().is_none());
                assert!(legacy_network_error.is_retryable()); // Should detect "timeout" in message
                assert!(legacy_network_error.is_recoverable());
                
                let suggestions = legacy_network_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("internet connection")));

                // Test legacy invalid URL error
                let legacy_url_error = MarkdownError::InvalidUrl {
                    url: "not-a-url".to_string(),
                };
                
                assert!(legacy_url_error.context().is_none());
                assert!(!legacy_url_error.is_retryable());
                assert!(!legacy_url_error.is_recoverable());
                
                let suggestions = legacy_url_error.suggestions();
                assert!(suggestions.iter().any(|s| s.contains("http")));
            }

            #[test]
            fn test_error_display_format() {
                let context = ErrorContext::new(
                    "https://example.com/test",
                    "Test operation",
                    "TestConverter"
                );

                let error = MarkdownError::ValidationError {
                    kind: ValidationErrorKind::InvalidUrl,
                    context,
                };

                let error_string = format!("{}", error);
                assert!(error_string.contains("Validation error"));
                assert!(error_string.contains("InvalidUrl"));
            }

            #[test]
            fn test_error_context_serialization() {
                let context = ErrorContext::new(
                    "https://example.com/test",
                    "Test operation", 
                    "TestConverter"
                ).with_info("Additional context");

                // Test that ErrorContext can be serialized/deserialized
                let yaml = serde_yaml::to_string(&context).unwrap();
                let deserialized: ErrorContext = serde_yaml::from_str(&yaml).unwrap();
                
                assert_eq!(context.url, deserialized.url);
                assert_eq!(context.operation, deserialized.operation);
                assert_eq!(context.converter_type, deserialized.converter_type);
                assert_eq!(context.additional_info, deserialized.additional_info);
            }

            #[test]
            fn test_error_kind_serialization() {
                // Test that all error kinds can be serialized/deserialized
                let validation_kind = ValidationErrorKind::InvalidUrl;
                let yaml = serde_yaml::to_string(&validation_kind).unwrap();
                let deserialized: ValidationErrorKind = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(validation_kind, deserialized);

                let network_kind = NetworkErrorKind::ServerError(500);
                let yaml = serde_yaml::to_string(&network_kind).unwrap();
                let deserialized: NetworkErrorKind = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(network_kind, deserialized);

                let auth_kind = AuthErrorKind::TokenExpired;
                let yaml = serde_yaml::to_string(&auth_kind).unwrap();
                let deserialized: AuthErrorKind = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(auth_kind, deserialized);

                let content_kind = ContentErrorKind::ParsingFailed;
                let yaml = serde_yaml::to_string(&content_kind).unwrap();
                let deserialized: ContentErrorKind = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(content_kind, deserialized);

                let converter_kind = ConverterErrorKind::ExternalToolFailed;
                let yaml = serde_yaml::to_string(&converter_kind).unwrap();
                let deserialized: ConverterErrorKind = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(converter_kind, deserialized);

                let config_kind = ConfigErrorKind::MissingDependency;
                let yaml = serde_yaml::to_string(&config_kind).unwrap();
                let deserialized: ConfigErrorKind = serde_yaml::from_str(&yaml).unwrap();
                assert_eq!(config_kind, deserialized);
            }
        }
    }
}
