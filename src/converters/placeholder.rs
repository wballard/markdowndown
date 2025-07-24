//! Placeholder converters for URL types not yet fully implemented.
//!
//! These converters provide basic functionality and can be extended later
//! with full implementations for their respective services.

use crate::client::HttpClient;
use crate::types::{Markdown, MarkdownError};
use async_trait::async_trait;

use super::Converter;

/// Configuration for a placeholder converter.
#[derive(Debug, Clone)]
pub struct PlaceholderConfig {
    /// The name of the service (e.g., "Google Docs", "Office 365")
    pub service_name: String,
    /// The converter name for debugging
    pub converter_name: &'static str,
    /// Maximum characters to include from content
    pub max_content_length: usize,
}

/// Generic placeholder converter that can be configured for different services.
///
/// This eliminates code duplication between service-specific placeholder converters
/// by providing a single implementation with configurable metadata.
#[derive(Debug, Clone)]
pub struct PlaceholderConverter {
    client: HttpClient,
    config: PlaceholderConfig,
}

impl PlaceholderConverter {
    /// Creates a new placeholder converter with default HTTP client.
    pub fn new(config: PlaceholderConfig) -> Self {
        Self {
            client: HttpClient::new(),
            config,
        }
    }

    /// Creates a new placeholder converter with configured HTTP client.
    pub fn with_client(client: HttpClient, config: PlaceholderConfig) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl Converter for PlaceholderConverter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // For now, just fetch as HTML and convert
        // TODO: Implement proper API integrations for each service
        let html_content = self.client.get_text(url).await?;

        // Basic HTML to text conversion
        // This is a simplified implementation - full implementations would
        // use service-specific APIs to export content properly
        let truncated_content = if html_content.len() > self.config.max_content_length {
            // Use char_indices to find a safe UTF-8 boundary for truncation
            let mut truncation_point = self.config.max_content_length;
            while !html_content.is_char_boundary(truncation_point) && truncation_point > 0 {
                truncation_point -= 1;
            }
            format!("{}...", &html_content[..truncation_point])
        } else {
            html_content
        };

        let markdown_content = format!(
            "# Converted from {}\n\nSource: {}\n\n{}",
            self.config.service_name, url, truncated_content
        );

        Markdown::new(markdown_content)
    }

    fn name(&self) -> &'static str {
        self.config.converter_name
    }
}

/// Wrapper for Google Docs converter using the generic placeholder implementation.
#[derive(Debug, Clone)]
pub struct GoogleDocsConverter {
    inner: PlaceholderConverter,
}

/// Wrapper for Office 365 converter using the generic placeholder implementation.
#[derive(Debug, Clone)]
pub struct Office365Converter {
    inner: PlaceholderConverter,
}

/// Wrapper for GitHub Issue converter using the generic placeholder implementation.
#[derive(Debug, Clone)]
pub struct GitHubIssueConverter {
    inner: PlaceholderConverter,
}

impl GoogleDocsConverter {
    /// Creates a new Google Docs converter with default HTTP client.
    pub fn new() -> Self {
        let config = PlaceholderConfig {
            service_name: "Google Docs".to_string(),
            converter_name: "Google Docs",
            max_content_length: 1000,
        };
        Self {
            inner: PlaceholderConverter::new(config),
        }
    }

    /// Creates a new Google Docs converter with configured HTTP client.
    pub fn with_client(client: HttpClient) -> Self {
        let config = PlaceholderConfig {
            service_name: "Google Docs".to_string(),
            converter_name: "Google Docs",
            max_content_length: 1000,
        };
        Self {
            inner: PlaceholderConverter::with_client(client, config),
        }
    }

    /// Creates a new Google Docs converter with configured HTTP client and settings.
    pub fn with_client_and_settings(
        client: HttpClient,
        settings: &crate::config::PlaceholderSettings,
    ) -> Self {
        let config = PlaceholderConfig {
            service_name: "Google Docs".to_string(),
            converter_name: "Google Docs",
            max_content_length: settings.max_content_length,
        };
        Self {
            inner: PlaceholderConverter::with_client(client, config),
        }
    }
}

#[async_trait]
impl Converter for GoogleDocsConverter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        self.inner.convert(url).await
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

impl Office365Converter {
    /// Creates a new Office 365 converter with default HTTP client.
    pub fn new() -> Self {
        let config = PlaceholderConfig {
            service_name: "Office 365".to_string(),
            converter_name: "Office 365",
            max_content_length: 1000,
        };
        Self {
            inner: PlaceholderConverter::new(config),
        }
    }

    /// Creates a new Office 365 converter with configured HTTP client.
    pub fn with_client(client: HttpClient) -> Self {
        let config = PlaceholderConfig {
            service_name: "Office 365".to_string(),
            converter_name: "Office 365",
            max_content_length: 1000,
        };
        Self {
            inner: PlaceholderConverter::with_client(client, config),
        }
    }

    /// Creates a new Office 365 converter with configured HTTP client and settings.
    pub fn with_client_and_settings(
        client: HttpClient,
        settings: &crate::config::PlaceholderSettings,
    ) -> Self {
        let config = PlaceholderConfig {
            service_name: "Office 365".to_string(),
            converter_name: "Office 365",
            max_content_length: settings.max_content_length,
        };
        Self {
            inner: PlaceholderConverter::with_client(client, config),
        }
    }
}

#[async_trait]
impl Converter for Office365Converter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        self.inner.convert(url).await
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

impl GitHubIssueConverter {
    /// Creates a new GitHub Issue converter with default HTTP client.
    pub fn new() -> Self {
        let config = PlaceholderConfig {
            service_name: "GitHub Issue".to_string(),
            converter_name: "GitHub Issue",
            max_content_length: 1000,
        };
        Self {
            inner: PlaceholderConverter::new(config),
        }
    }

    /// Creates a new GitHub Issue converter with configured HTTP client.
    pub fn with_client(client: HttpClient) -> Self {
        let config = PlaceholderConfig {
            service_name: "GitHub Issue".to_string(),
            converter_name: "GitHub Issue",
            max_content_length: 1000,
        };
        Self {
            inner: PlaceholderConverter::with_client(client, config),
        }
    }

    /// Creates a new GitHub Issue converter with configured HTTP client and settings.
    pub fn with_client_and_settings(
        client: HttpClient,
        settings: &crate::config::PlaceholderSettings,
    ) -> Self {
        let config = PlaceholderConfig {
            service_name: "GitHub Issue".to_string(),
            converter_name: "GitHub Issue",
            max_content_length: settings.max_content_length,
        };
        Self {
            inner: PlaceholderConverter::with_client(client, config),
        }
    }
}

#[async_trait]
impl Converter for GitHubIssueConverter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        self.inner.convert(url).await
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

impl Default for GoogleDocsConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Office365Converter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GitHubIssueConverter {
    fn default() -> Self {
        Self::new()
    }
}
