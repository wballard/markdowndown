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

/// Configuration system
pub mod config;

use crate::converters::ConverterRegistry;
use crate::detection::UrlDetector;
use crate::types::{Markdown, MarkdownError};

/// Main library struct providing unified URL to markdown conversion.
///
/// This struct integrates URL detection, converter routing, and configuration
/// to provide a simple, unified API for converting any supported URL to markdown.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use markdowndown::MarkdownDown;
///
/// # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
/// let md = MarkdownDown::new();
/// let result = md.convert_url("https://example.com/article.html").await?;
/// println!("{}", result);
/// # Ok(())
/// # }
/// ```
///
/// ## With Custom Configuration
///
/// ```rust
/// use markdowndown::{MarkdownDown, Config};
///
/// # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
/// let config = Config::builder()
///     .timeout_seconds(60)
///     .user_agent("MyApp/1.0")
///     .build();
///
/// let md = MarkdownDown::with_config(config);
/// let result = md.convert_url("https://docs.google.com/document/d/abc123/edit").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct MarkdownDown {
    config: crate::config::Config,
    detector: UrlDetector,
    registry: ConverterRegistry,
}

impl MarkdownDown {
    /// Creates a new MarkdownDown instance with default configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::MarkdownDown;
    ///
    /// let md = MarkdownDown::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: crate::config::Config::default(),
            detector: UrlDetector::new(),
            registry: ConverterRegistry::new(),
        }
    }

    /// Creates a new MarkdownDown instance with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::{MarkdownDown, Config};
    ///
    /// let config = Config::builder()
    ///     .timeout_seconds(45)
    ///     .build();
    ///
    /// let md = MarkdownDown::with_config(config);
    /// ```
    pub fn with_config(config: crate::config::Config) -> Self {
        Self {
            config,
            detector: UrlDetector::new(),
            registry: ConverterRegistry::new(),
        }
    }

    /// Converts content from a URL to markdown.
    ///
    /// This method automatically detects the URL type and routes it to the
    /// appropriate converter for processing.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch and convert
    ///
    /// # Returns
    ///
    /// Returns the converted markdown content or an error.
    ///
    /// # Errors
    ///
    /// * `MarkdownError::InvalidUrl` - If the URL format is invalid
    /// * `MarkdownError::NetworkError` - For network-related failures
    /// * `MarkdownError::ParseError` - If content conversion fails
    /// * `MarkdownError::AuthError` - For authentication failures
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::MarkdownDown;
    ///
    /// # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
    /// let md = MarkdownDown::new();
    /// let result = md.convert_url("https://example.com/page.html").await?;
    /// println!("Converted markdown: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn convert_url(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // Step 1: Normalize the URL
        let normalized_url = self.detector.normalize_url(url)?;

        // Step 2: Detect URL type
        let url_type = self.detector.detect_type(&normalized_url)?;

        // Step 3: Get appropriate converter
        let converter =
            self.registry
                .get_converter(&url_type)
                .ok_or_else(|| MarkdownError::ParseError {
                    message: format!("No converter available for URL type: {url_type}"),
                })?;

        // Step 4: Convert using the selected converter
        converter.convert(&normalized_url).await
    }

    /// Returns the configuration being used by this instance.
    pub fn config(&self) -> &crate::config::Config {
        &self.config
    }

    /// Returns the URL detector being used by this instance.
    pub fn detector(&self) -> &UrlDetector {
        &self.detector
    }

    /// Returns the converter registry being used by this instance.
    pub fn registry(&self) -> &ConverterRegistry {
        &self.registry
    }

    /// Lists all supported URL types.
    pub fn supported_types(&self) -> Vec<crate::types::UrlType> {
        self.registry.supported_types()
    }
}

impl Default for MarkdownDown {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for converting a URL to markdown with default configuration.
///
/// This is equivalent to calling `MarkdownDown::new().convert_url(url).await`.
///
/// # Arguments
///
/// * `url` - The URL to fetch and convert
///
/// # Returns
///
/// Returns the converted markdown content or an error.
///
/// # Examples
///
/// ```rust
/// use markdowndown::convert_url;
///
/// # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
/// let result = convert_url("https://example.com/article.html").await?;
/// println!("{}", result);
/// # Ok(())
/// # }
/// ```
pub async fn convert_url(url: &str) -> Result<Markdown, MarkdownError> {
    MarkdownDown::new().convert_url(url).await
}

/// Convenience function for converting a URL to markdown with custom configuration.
///
/// # Arguments
///
/// * `url` - The URL to fetch and convert
/// * `config` - The configuration to use
///
/// # Returns
///
/// Returns the converted markdown content or an error.
///
/// # Examples
///
/// ```rust
/// use markdowndown::{convert_url_with_config, Config};
///
/// # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
/// let config = Config::builder()
///     .timeout_seconds(60)
///     .build();
///
/// let result = convert_url_with_config("https://example.com/article.html", config).await?;
/// println!("{}", result);
/// # Ok(())
/// # }
/// ```
pub async fn convert_url_with_config(
    url: &str,
    config: crate::config::Config,
) -> Result<Markdown, MarkdownError> {
    MarkdownDown::with_config(config).convert_url(url).await
}

/// Utility function to detect the type of a URL without converting it.
///
/// # Arguments
///
/// * `url` - The URL to analyze
///
/// # Returns
///
/// Returns the detected URL type or an error.
///
/// # Examples
///
/// ```rust
/// use markdowndown::{detect_url_type, types::UrlType};
///
/// # fn example() -> Result<(), markdowndown::types::MarkdownError> {
/// let url_type = detect_url_type("https://docs.google.com/document/d/123/edit")?;
/// assert_eq!(url_type, UrlType::GoogleDocs);
/// # Ok(())
/// # }
/// ```
pub fn detect_url_type(url: &str) -> Result<crate::types::UrlType, MarkdownError> {
    let detector = UrlDetector::new();
    detector.detect_type(url)
}

// Re-export main API items for convenience
pub use config::Config;
pub use converters::{Converter, HtmlConverter};
pub use types::{Frontmatter, Url, UrlType};

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
