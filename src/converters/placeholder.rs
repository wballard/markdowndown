//! Placeholder converters for URL types not yet fully implemented.
//!
//! These converters provide basic functionality and can be extended later
//! with full implementations for their respective services.

use crate::client::HttpClient;
use crate::types::{Markdown, MarkdownError};
use async_trait::async_trait;

use super::Converter;

/// Placeholder converter for Google Docs URLs.
///
/// Currently falls back to HTML conversion but can be extended
/// to use Google Docs API for better results.
#[derive(Debug, Clone)]
pub struct GoogleDocsConverter {
    client: HttpClient,
}

impl GoogleDocsConverter {
    /// Creates a new Google Docs converter.
    pub fn new() -> Self {
        Self {
            client: HttpClient::new(),
        }
    }
}

#[async_trait]
impl Converter for GoogleDocsConverter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // For now, just fetch as HTML and convert
        // TODO: Implement proper Google Docs API integration
        let html_content = self.client.get_text(url).await?;

        // Basic HTML to text conversion
        // This is a simplified implementation - a full implementation would
        // use Google Docs API to export as markdown directly
        let markdown_content = format!(
            "# Converted from Google Docs\n\nSource: {}\n\n{}",
            url,
            html_content.chars().take(1000).collect::<String>()
        );

        Markdown::new(markdown_content)
    }

    fn name(&self) -> &'static str {
        "Google Docs"
    }
}

impl Default for GoogleDocsConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder converter for Office 365 URLs.
///
/// Currently falls back to HTML conversion but can be extended
/// to use Office 365 APIs for better results.
#[derive(Debug, Clone)]
pub struct Office365Converter {
    client: HttpClient,
}

impl Office365Converter {
    /// Creates a new Office 365 converter.
    pub fn new() -> Self {
        Self {
            client: HttpClient::new(),
        }
    }
}

#[async_trait]
impl Converter for Office365Converter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // For now, just fetch as HTML and convert
        // TODO: Implement proper Office 365 API integration
        let html_content = self.client.get_text(url).await?;

        // Basic HTML to text conversion
        // This is a simplified implementation - a full implementation would
        // use Office 365 APIs to export documents properly
        let markdown_content = format!(
            "# Converted from Office 365\n\nSource: {}\n\n{}",
            url,
            html_content.chars().take(1000).collect::<String>()
        );

        Markdown::new(markdown_content)
    }

    fn name(&self) -> &'static str {
        "Office 365"
    }
}

impl Default for Office365Converter {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder converter for GitHub Issues URLs.
///
/// Currently falls back to HTML conversion but can be extended
/// to use GitHub API for better results.
#[derive(Debug, Clone)]
pub struct GitHubIssueConverter {
    client: HttpClient,
}

impl GitHubIssueConverter {
    /// Creates a new GitHub Issue converter.
    pub fn new() -> Self {
        Self {
            client: HttpClient::new(),
        }
    }
}

#[async_trait]
impl Converter for GitHubIssueConverter {
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // For now, just fetch as HTML and convert
        // TODO: Implement proper GitHub API integration
        let html_content = self.client.get_text(url).await?;

        // Basic HTML to text conversion
        // This is a simplified implementation - a full implementation would
        // use GitHub API to get issue content with proper formatting
        let markdown_content = format!(
            "# Converted from GitHub Issue\n\nSource: {}\n\n{}",
            url,
            html_content.chars().take(1000).collect::<String>()
        );

        Markdown::new(markdown_content)
    }

    fn name(&self) -> &'static str {
        "GitHub Issue"
    }
}

impl Default for GitHubIssueConverter {
    fn default() -> Self {
        Self::new()
    }
}
