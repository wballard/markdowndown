//! Local file converter for reading markdown files from the filesystem.
//!
//! This converter handles local file paths and file:// URLs by reading markdown content
//! directly from the local filesystem.

use crate::types::{ContentErrorKind, ErrorContext, Markdown, MarkdownError};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, instrument};

/// Converter for reading markdown files from the local filesystem.
///
/// This converter supports both regular file paths and file:// URLs:
/// - `/absolute/path/to/file.md`
/// - `./relative/path.md`
/// - `../parent/relative/path.md`
/// - `file:///absolute/path/to/file.md`
/// - `file://./relative/path.md`
#[derive(Debug, Clone)]
pub struct LocalFileConverter;

impl LocalFileConverter {
    /// Creates a new LocalFileConverter instance.
    pub fn new() -> Self {
        LocalFileConverter
    }

    /// Converts a file path or file:// URL to a standard file path.
    ///
    /// # Arguments
    ///
    /// * `input` - The file path or file:// URL
    ///
    /// # Returns
    ///
    /// Returns the normalized file path as a string.
    fn normalize_path(&self, input: &str) -> String {
        // Handle file:// URLs by stripping the protocol
        if input.starts_with("file://") {
            let path_part = &input[7..]; // Remove "file://"

            // Handle file:///absolute/path case (three slashes for absolute paths)
            if input.starts_with("file:///") {
                format!("/{}", &input[8..]) // Remove "file://" and keep the leading /
            } else {
                // Handle file://./relative or file://../relative
                path_part.to_string()
            }
        } else {
            // Regular file path - use as-is
            input.to_string()
        }
    }

    /// Validates that the file path exists and is readable.
    async fn validate_file_path(&self, path: &str) -> Result<(), MarkdownError> {
        let path_obj = Path::new(path);

        // Check if file exists
        if !path_obj.exists() {
            let context = ErrorContext::new(path, "File validation", "LocalFileConverter")
                .with_info("File does not exist");
            return Err(MarkdownError::ContentError {
                kind: ContentErrorKind::EmptyContent,
                context,
            });
        }

        // Check if it's a file (not a directory)
        if !path_obj.is_file() {
            let context = ErrorContext::new(path, "File validation", "LocalFileConverter")
                .with_info("Path is not a file");
            return Err(MarkdownError::ContentError {
                kind: ContentErrorKind::UnsupportedFormat,
                context,
            });
        }

        Ok(())
    }

    /// Reads the file content as a UTF-8 string.
    async fn read_file_content(&self, path: &str) -> Result<String, MarkdownError> {
        match fs::read_to_string(path).await {
            Ok(content) => Ok(content),
            Err(e) => {
                let context = ErrorContext::new(path, "File reading", "LocalFileConverter")
                    .with_info(format!("IO error: {}", e));

                match e.kind() {
                    std::io::ErrorKind::NotFound => Err(MarkdownError::ContentError {
                        kind: ContentErrorKind::EmptyContent,
                        context,
                    }),
                    std::io::ErrorKind::PermissionDenied => Err(MarkdownError::ContentError {
                        kind: ContentErrorKind::ParsingFailed,
                        context,
                    }),
                    _ => Err(MarkdownError::ContentError {
                        kind: ContentErrorKind::ParsingFailed,
                        context,
                    }),
                }
            }
        }
    }
}

#[async_trait]
impl super::Converter for LocalFileConverter {
    /// Converts a local file path or file:// URL to markdown.
    ///
    /// # Arguments
    ///
    /// * `url` - The file path or file:// URL to read
    ///
    /// # Returns
    ///
    /// Returns the file content as validated Markdown or an error.
    #[instrument(skip(self), fields(file_path))]
    async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        info!("Starting local file conversion for: {}", url);

        // Normalize the path (handle file:// URLs)
        let file_path = self.normalize_path(url);
        tracing::Span::current().record("file_path", &file_path);
        debug!("Normalized file path: {}", file_path);

        // Validate file exists and is readable
        debug!("Validating file path");
        self.validate_file_path(&file_path).await?;

        // Read file content
        debug!("Reading file content");
        let content = self.read_file_content(&file_path).await?;

        // Validate content is not empty
        if content.trim().is_empty() {
            let context = ErrorContext::new(&file_path, "Content validation", "LocalFileConverter")
                .with_info("File content is empty");
            return Err(MarkdownError::ContentError {
                kind: ContentErrorKind::EmptyContent,
                context,
            });
        }

        // Create validated Markdown instance
        debug!("Creating validated markdown instance");
        let markdown = Markdown::new(content).map_err(|e| {
            let context =
                ErrorContext::new(&file_path, "Markdown validation", "LocalFileConverter")
                    .with_info(format!("Validation error: {}", e));
            MarkdownError::ContentError {
                kind: ContentErrorKind::ParsingFailed,
                context,
            }
        })?;

        info!(
            "Successfully converted local file to markdown ({} chars)",
            markdown.as_str().len()
        );
        Ok(markdown)
    }

    fn name(&self) -> &'static str {
        "Local File Converter"
    }
}

impl Default for LocalFileConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::converter::Converter;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_normalize_path_regular_path() {
        let converter = LocalFileConverter::new();

        // Test absolute path
        assert_eq!(
            converter.normalize_path("/path/to/file.md"),
            "/path/to/file.md"
        );

        // Test relative paths
        assert_eq!(converter.normalize_path("./file.md"), "./file.md");
        assert_eq!(converter.normalize_path("../file.md"), "../file.md");
        assert_eq!(
            converter.normalize_path("relative/path.md"),
            "relative/path.md"
        );
    }

    #[test]
    fn test_normalize_path_file_url() {
        let converter = LocalFileConverter::new();

        // Test file:// URLs with absolute paths (three slashes)
        assert_eq!(
            converter.normalize_path("file:///path/to/file.md"),
            "/path/to/file.md"
        );
        assert_eq!(
            converter.normalize_path("file:///home/user/doc.md"),
            "/home/user/doc.md"
        );

        // Test file:// URLs with relative paths (two slashes)
        assert_eq!(
            converter.normalize_path("file://./relative.md"),
            "./relative.md"
        );
        assert_eq!(
            converter.normalize_path("file://../parent.md"),
            "../parent.md"
        );
        assert_eq!(
            converter.normalize_path("file://relative/path.md"),
            "relative/path.md"
        );
    }

    #[tokio::test]
    async fn test_convert_existing_file() {
        let converter = LocalFileConverter::new();

        // Create a temporary file with markdown content
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "# Test Document\n\nThis is a test markdown file.";
        writeln!(temp_file, "{}", content).unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        let result = converter.convert(file_path).await;

        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.as_str().contains("# Test Document"));
        assert!(markdown.as_str().contains("This is a test markdown file."));
    }

    #[tokio::test]
    async fn test_convert_file_url() {
        let converter = LocalFileConverter::new();

        // Create a temporary file with markdown content
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "# File URL Test\n\nTesting file:// URLs.";
        writeln!(temp_file, "{}", content).unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        let file_url = format!("file://{}", file_path);
        let result = converter.convert(&file_url).await;

        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.as_str().contains("# File URL Test"));
        assert!(markdown.as_str().contains("Testing file:// URLs."));
    }

    #[tokio::test]
    async fn test_convert_nonexistent_file() {
        let converter = LocalFileConverter::new();

        let result = converter.convert("/nonexistent/file.md").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ContentError { kind, context } => {
                assert_eq!(kind, ContentErrorKind::EmptyContent);
                assert_eq!(context.url, "/nonexistent/file.md");
                assert!(context
                    .additional_info
                    .unwrap()
                    .contains("File does not exist"));
            }
            _ => panic!("Expected ContentError"),
        }
    }

    #[tokio::test]
    async fn test_convert_empty_file() {
        let converter = LocalFileConverter::new();

        // Create an empty temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let result = converter.convert(file_path).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ContentError { kind, .. } => {
                assert_eq!(kind, ContentErrorKind::EmptyContent);
            }
            _ => panic!("Expected ContentError"),
        }
    }

    #[tokio::test]
    async fn test_convert_directory() {
        let converter = LocalFileConverter::new();

        // Try to convert a directory (should fail)
        let result = converter.convert("/tmp").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ContentError { kind, context } => {
                assert_eq!(kind, ContentErrorKind::UnsupportedFormat);
                assert!(context
                    .additional_info
                    .unwrap()
                    .contains("Path is not a file"));
            }
            _ => panic!("Expected ContentError"),
        }
    }

    #[test]
    fn test_converter_name() {
        let converter = LocalFileConverter::new();
        assert_eq!(converter.name(), "Local File Converter");
    }

    #[test]
    fn test_default_implementation() {
        let converter = LocalFileConverter::default();
        assert_eq!(converter.name(), "Local File Converter");
    }
}
