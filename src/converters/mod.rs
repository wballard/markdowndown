//! Content converters for transforming various formats to markdown.
//!
//! This module provides converters for different content types, enabling
//! the transformation of HTML, documents, and other formats into clean markdown.

use crate::types::{Markdown, MarkdownError, UrlType};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for converting content from URLs to markdown.
#[async_trait]
pub trait Converter: Send + Sync + std::fmt::Debug {
    /// Converts content from the given URL to markdown.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch and convert
    ///
    /// # Returns
    ///
    /// Returns the converted markdown content or an error.
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError>;

    /// Returns the name of this converter for debugging and error messages.
    fn name(&self) -> &'static str;
}

/// Registry for managing URL converters and routing requests to appropriate handlers.
#[derive(Debug, Clone)]
pub struct ConverterRegistry {
    converters: HashMap<UrlType, Arc<dyn Converter>>,
}

impl ConverterRegistry {
    /// Creates a new converter registry with default converters registered.
    pub fn new() -> Self {
        let mut registry = Self {
            converters: HashMap::new(),
        };

        // Register default converters
        registry.register(UrlType::Html, Arc::new(html::HtmlConverter::new()));
        registry.register(
            UrlType::GoogleDocs,
            Arc::new(placeholder::GoogleDocsConverter::new()),
        );
        registry.register(
            UrlType::Office365,
            Arc::new(placeholder::Office365Converter::new()),
        );
        registry.register(
            UrlType::GitHubIssue,
            Arc::new(placeholder::GitHubIssueConverter::new()),
        );

        registry
    }

    /// Registers a converter for a specific URL type.
    ///
    /// # Arguments
    ///
    /// * `url_type` - The URL type to register the converter for
    /// * `converter` - The converter implementation
    pub fn register(&mut self, url_type: UrlType, converter: Arc<dyn Converter>) {
        self.converters.insert(url_type, converter);
    }

    /// Gets a converter for the specified URL type.
    ///
    /// # Arguments
    ///
    /// * `url_type` - The URL type to get a converter for
    ///
    /// # Returns
    ///
    /// Returns a reference to the converter if one is registered, None otherwise.
    pub fn get_converter(&self, url_type: &UrlType) -> Option<&Arc<dyn Converter>> {
        self.converters.get(url_type)
    }

    /// Lists all registered URL types.
    pub fn supported_types(&self) -> Vec<UrlType> {
        self.converters.keys().cloned().collect()
    }
}

impl Default for ConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// HTML to markdown converter
pub mod html;

/// Placeholder converters for services not yet fully implemented
pub mod placeholder;

// Re-export main converter types for convenience
pub use html::HtmlConverter;
pub use placeholder::{GitHubIssueConverter, GoogleDocsConverter, Office365Converter};
