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

use crate::client::HttpClient;
use crate::converters::ConverterRegistry;
use crate::detection::UrlDetector;
use crate::types::{Markdown, MarkdownError, UrlType};
use tracing::{info, warn, error, debug, instrument};

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
        // Create configured HTTP client
        let http_client = HttpClient::with_config(&config.http, &config.auth);

        // Create registry with configured HTTP client, HTML config, and placeholder settings
        let registry =
            ConverterRegistry::with_config(http_client, config.html.clone(), &config.placeholder);

        Self {
            config,
            detector: UrlDetector::new(),
            registry,
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
    /// * `MarkdownError::ConfigurationError` - If no converter is available for the URL type
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
    #[instrument(skip(self), fields(url_type))]
    pub async fn convert_url(&self, url: &str) -> Result<Markdown, MarkdownError> {
        info!("Starting URL conversion for: {}", url);
        
        // Step 1: Normalize the URL
        debug!("Normalizing URL");
        let normalized_url = self.detector.normalize_url(url)?;
        debug!("Normalized URL: {}", normalized_url);

        // Step 2: Detect URL type
        debug!("Detecting URL type");
        let url_type = self.detector.detect_type(&normalized_url)?;
        tracing::Span::current().record("url_type", &format!("{}", url_type));
        info!("Detected URL type: {}", url_type);

        // Step 3: Get appropriate converter
        debug!("Looking up converter for type: {}", url_type);
        let converter = self.registry.get_converter(&url_type).ok_or_else(|| {
            error!("No converter available for URL type: {}", url_type);
            MarkdownError::LegacyConfigurationError {
                message: format!("No converter available for URL type: {url_type}"),
            }
        })?;
        debug!("Found converter for type: {}", url_type);

        // Step 4: Convert using the selected converter
        info!("Starting conversion with {} converter", url_type);
        match converter.convert(&normalized_url).await {
            Ok(result) => {
                info!("Successfully converted URL to markdown ({} chars)", result.as_str().len());
                Ok(result)
            }
            Err(e) => {
                error!("Primary converter failed: {}", e);
                
                // Step 5: Attempt fallback strategies for recoverable errors
                if e.is_recoverable() && url_type != UrlType::Html {
                    warn!("Attempting HTML fallback conversion for recoverable error");
                    
                    // Try HTML converter as fallback
                    if let Some(html_converter) = self.registry.get_converter(&UrlType::Html) {
                        match html_converter.convert(&normalized_url).await {
                            Ok(fallback_result) => {
                                warn!("Fallback HTML conversion succeeded ({} chars)", fallback_result.as_str().len());
                                return Ok(fallback_result);
                            }
                            Err(fallback_error) => {
                                error!("Fallback HTML conversion also failed: {}", fallback_error);
                            }
                        }
                    }
                }
                
                Err(e)
            }
        }
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
pub use types::{Frontmatter, Url};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::GitHubConverter;
    use crate::detection::UrlDetector;
    use crate::types::UrlType;
    use std::time::Duration;

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
    fn test_markdowndown_with_default_config() {
        // Test that MarkdownDown can be created with default configuration
        let md = MarkdownDown::new();

        // Verify config is stored and accessible
        let config = md.config();
        assert_eq!(config.http.timeout, Duration::from_secs(30));
        assert_eq!(config.http.max_retries, 3);
        assert_eq!(config.http.retry_delay, Duration::from_secs(1));
        assert_eq!(config.http.max_redirects, 10);
        assert!(config.auth.github_token.is_none());
        assert!(config.auth.office365_token.is_none());
        assert!(config.auth.google_api_key.is_none());
        assert_eq!(config.placeholder.max_content_length, 1000);
        assert!(config.output.include_frontmatter);
        assert_eq!(config.output.max_consecutive_blank_lines, 2);
    }

    #[test]
    fn test_markdowndown_with_custom_config() {
        // Test that MarkdownDown respects custom configuration
        let config = Config::builder()
            .timeout_seconds(60)
            .user_agent("TestApp/1.0")
            .max_retries(5)
            .github_token("test_token")
            .placeholder_max_content_length(2000)
            .include_frontmatter(false)
            .max_consecutive_blank_lines(1)
            .build();

        let md = MarkdownDown::with_config(config);

        // Verify custom config is stored
        let stored_config = md.config();
        assert_eq!(stored_config.http.timeout, Duration::from_secs(60));
        assert_eq!(stored_config.http.user_agent, "TestApp/1.0");
        assert_eq!(stored_config.http.max_retries, 5);
        assert_eq!(
            stored_config.auth.github_token,
            Some("test_token".to_string())
        );
        assert_eq!(stored_config.placeholder.max_content_length, 2000);
        assert!(!stored_config.output.include_frontmatter);
        assert_eq!(stored_config.output.max_consecutive_blank_lines, 1);
    }

    #[test]
    fn test_config_builder_fluent_interface() {
        // Test that the config builder's fluent interface works correctly
        let config = Config::builder()
            .github_token("ghp_test_token")
            .office365_token("office_token")
            .google_api_key("google_key")
            .timeout_seconds(45)
            .user_agent("IntegrationTest/2.0")
            .max_retries(3)
            .placeholder_max_content_length(1500)
            .include_frontmatter(true)
            .custom_frontmatter_field("project", "markdowndown")
            .custom_frontmatter_field("version", "test")
            .normalize_whitespace(false)
            .max_consecutive_blank_lines(3)
            .build();

        // Verify all custom settings
        assert_eq!(config.auth.github_token, Some("ghp_test_token".to_string()));
        assert_eq!(
            config.auth.office365_token,
            Some("office_token".to_string())
        );
        assert_eq!(config.auth.google_api_key, Some("google_key".to_string()));
        assert_eq!(config.http.timeout, Duration::from_secs(45));
        assert_eq!(config.http.user_agent, "IntegrationTest/2.0");
        assert_eq!(config.http.max_retries, 3);
        assert_eq!(config.placeholder.max_content_length, 1500);
        assert!(config.output.include_frontmatter);
        assert_eq!(config.output.custom_frontmatter_fields.len(), 2);
        assert_eq!(
            config.output.custom_frontmatter_fields[0],
            ("project".to_string(), "markdowndown".to_string())
        );
        assert_eq!(
            config.output.custom_frontmatter_fields[1],
            ("version".to_string(), "test".to_string())
        );
        assert!(!config.output.normalize_whitespace);
        assert_eq!(config.output.max_consecutive_blank_lines, 3);
    }

    #[test]
    fn test_config_from_default() {
        // Test that Config::default() produces expected defaults
        let config = Config::default();

        // HTTP config defaults
        assert_eq!(config.http.timeout, Duration::from_secs(30));
        assert!(config.http.user_agent.starts_with("markdowndown/"));
        assert_eq!(config.http.max_retries, 3);
        assert_eq!(config.http.retry_delay, Duration::from_secs(1));
        assert_eq!(config.http.max_redirects, 10);

        // Auth config defaults
        assert!(config.auth.github_token.is_none());
        assert!(config.auth.office365_token.is_none());
        assert!(config.auth.google_api_key.is_none());

        // Placeholder config defaults
        assert_eq!(config.placeholder.max_content_length, 1000);

        // Output config defaults
        assert!(config.output.include_frontmatter);
        assert!(config.output.custom_frontmatter_fields.is_empty());
        assert!(config.output.normalize_whitespace);
        assert_eq!(config.output.max_consecutive_blank_lines, 2);
    }

    #[test]
    fn test_supported_url_types() {
        // Test that MarkdownDown reports supported URL types correctly
        let md = MarkdownDown::new();
        let supported_types = md.supported_types();

        // Should support at least these URL types
        assert!(supported_types.contains(&crate::types::UrlType::Html));
        assert!(supported_types.contains(&crate::types::UrlType::GoogleDocs));
        assert!(supported_types.contains(&crate::types::UrlType::Office365));
        assert!(supported_types.contains(&crate::types::UrlType::GitHubIssue));

        // Should have exactly 4 supported types
        assert_eq!(supported_types.len(), 4);
    }

    #[test]
    fn test_detect_url_type_integration() {
        // Test that URL type detection works through the main API

        // Test HTML URL
        let html_result = detect_url_type("https://example.com/article.html");
        assert!(html_result.is_ok());
        assert_eq!(html_result.unwrap(), crate::types::UrlType::Html);

        // Test Google Docs URL
        let gdocs_result = detect_url_type("https://docs.google.com/document/d/abc123/edit");
        assert!(gdocs_result.is_ok());
        assert_eq!(gdocs_result.unwrap(), crate::types::UrlType::GoogleDocs);

        // Test Office 365 URL
        let office_result =
            detect_url_type("https://company.sharepoint.com/sites/team/Document.docx");
        assert!(office_result.is_ok());
        assert_eq!(office_result.unwrap(), crate::types::UrlType::Office365);

        // Test GitHub Issue URL
        let github_result = detect_url_type("https://github.com/owner/repo/issues/123");
        assert!(github_result.is_ok());
        assert_eq!(github_result.unwrap(), crate::types::UrlType::GitHubIssue);

        // Test invalid URL
        let invalid_result = detect_url_type("not-a-url");
        assert!(invalid_result.is_err());
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
