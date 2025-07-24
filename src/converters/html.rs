//! HTML to markdown conversion with preprocessing and cleanup.
//!
//! This module provides robust HTML to markdown conversion using html2text
//! with intelligent preprocessing to remove unwanted elements and postprocessing
//! to clean up the markdown output.

use crate::client::HttpClient;
use crate::types::{Markdown, MarkdownError};
use async_trait::async_trait;
use html2text::from_read;
use std::io::Cursor;

pub use super::config::HtmlConverterConfig;
use super::converter::Converter;
use super::postprocessor::MarkdownPostprocessor;
use super::preprocessor::HtmlPreprocessor;

/// HTML to markdown converter with intelligent preprocessing and cleanup.
#[derive(Debug, Clone)]
pub struct HtmlConverter {
    config: HtmlConverterConfig,
    client: HttpClient,
}

impl HtmlConverter {
    /// Creates a new HTML converter with default configuration.
    ///
    /// # Returns
    ///
    /// A new `HtmlConverter` instance with sensible defaults for most use cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::HtmlConverter;
    ///
    /// let converter = HtmlConverter::new();
    /// // Use converter.convert(url) to convert HTML from URL to markdown
    /// ```
    pub fn new() -> Self {
        Self {
            config: HtmlConverterConfig::default(),
            client: HttpClient::new(),
        }
    }

    /// Creates a new HTML converter with custom configuration and HTTP client.
    ///
    /// # Arguments
    ///
    /// * `client` - Configured HTTP client to use for requests
    /// * `config` - Custom configuration options for the converter
    ///
    /// # Returns
    ///
    /// A new `HtmlConverter` instance with the specified configuration.
    pub fn with_config(client: HttpClient, config: HtmlConverterConfig) -> Self {
        Self { config, client }
    }

    /// Creates a new HTML converter with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration options for the converter
    ///
    /// # Returns
    ///
    /// A new `HtmlConverter` instance with the specified configuration and default HTTP client.
    pub fn with_config_only(config: HtmlConverterConfig) -> Self {
        Self {
            config,
            client: HttpClient::new(),
        }
    }

    /// Converts HTML to clean markdown with preprocessing and postprocessing.
    ///
    /// This method implements a complete pipeline:
    /// 1. Preprocess HTML to remove unwanted elements
    /// 2. Convert HTML to markdown using html2text
    /// 3. Postprocess markdown to clean up formatting
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to convert
    ///
    /// # Returns
    ///
    /// Returns clean markdown content on success, or a `MarkdownError` on failure.
    ///
    /// # Errors
    ///
    /// * `MarkdownError::ParseError` - If HTML parsing or conversion fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::HtmlConverter;
    ///
    /// let converter = HtmlConverter::new();
    /// let html = "<h1>Hello World</h1><p>This is a test.</p>";
    /// let markdown = converter.convert_html(html)?;
    /// assert!(markdown.contains("# Hello World"));
    /// # Ok::<(), markdowndown::types::MarkdownError>(())
    /// ```
    pub fn convert_html(&self, html: &str) -> Result<String, MarkdownError> {
        // Validate input
        if html.trim().is_empty() {
            return Err(MarkdownError::ParseError {
                message: format!(
                    "HTML content cannot be empty (received {} characters of whitespace/empty content)",
                    html.len()
                ),
            });
        }

        // Step 1: Preprocess HTML
        let preprocessor = HtmlPreprocessor::new(&self.config);
        let cleaned_html = preprocessor.preprocess(html);

        // Step 2: Convert to markdown
        let markdown = self.html_to_markdown(&cleaned_html).map_err(|e| {
            if let MarkdownError::ParseError { message } = e {
                MarkdownError::ParseError {
                    message: format!(
                        "Failed to convert HTML to markdown (HTML length: {} chars): {}",
                        cleaned_html.len(),
                        message
                    ),
                }
            } else {
                e
            }
        })?;

        // Step 3: Postprocess markdown
        let postprocessor = MarkdownPostprocessor::new(&self.config);
        let cleaned_markdown = postprocessor.postprocess(&markdown);

        Ok(cleaned_markdown)
    }

    /// Converts preprocessed HTML to markdown using html2text.
    fn html_to_markdown(&self, html: &str) -> Result<String, MarkdownError> {
        let cursor = Cursor::new(html.as_bytes());
        let markdown = from_read(cursor, self.config.max_line_width);
        Ok(markdown)
    }
}

#[async_trait]
impl Converter for HtmlConverter {
    /// Converts content from a URL to markdown by fetching HTML and converting it.
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // Fetch HTML content from URL with HTML-specific headers
        let headers = std::collections::HashMap::from([(
            "Accept".to_string(),
            "text/html,application/xhtml+xml".to_string(),
        )]);
        let html_content = self.client.get_text_with_headers(url, &headers).await?;

        // Convert HTML to markdown string
        let markdown_string = self.convert_html(&html_content)?;

        // Handle empty content case - provide minimal markdown for empty HTML
        let final_markdown = if markdown_string.trim().is_empty() {
            "<!-- Empty HTML document -->".to_string()
        } else {
            markdown_string
        };

        // Wrap in Markdown type with validation
        Markdown::new(final_markdown)
    }

    /// Returns the name of this converter.
    fn name(&self) -> &'static str {
        "HTML"
    }
}

impl Default for HtmlConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_converter_new() {
        let converter = HtmlConverter::new();
        assert_eq!(converter.config.max_line_width, 120);
        assert!(converter.config.remove_scripts_styles);
    }

    #[test]
    fn test_html_converter_with_config() {
        let config = HtmlConverterConfig {
            max_line_width: 80,
            remove_scripts_styles: false,
            ..Default::default()
        };

        let converter = HtmlConverter::with_config_only(config);
        assert_eq!(converter.config.max_line_width, 80);
        assert!(!converter.config.remove_scripts_styles);
    }

    #[test]
    fn test_convert_empty_html_error() {
        let converter = HtmlConverter::new();
        let result = converter.convert_html("");
        assert!(result.is_err());

        if let Err(MarkdownError::ParseError { message }) = result {
            assert!(message.contains("HTML content cannot be empty"));
        } else {
            panic!("Expected ParseError with specific message");
        }
    }

    #[test]
    fn test_convert_whitespace_only_html_error() {
        let converter = HtmlConverter::new();
        let result = converter.convert_html("   \n\t  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_basic_html_success() {
        let converter = HtmlConverter::new();
        let html = "<p>Hello, world!</p>";
        let result = converter.convert_html(html);
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("Hello, world!"));
    }

    #[test]
    fn test_default_implementation() {
        let converter1 = HtmlConverter::new();
        let converter2 = HtmlConverter::default();
        assert_eq!(
            converter1.config.max_line_width,
            converter2.config.max_line_width
        );
    }
}
