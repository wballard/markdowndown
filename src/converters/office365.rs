//! Office 365 document to markdown conversion with SharePoint and OneDrive support.
//!
//! This module provides conversion of Office 365 documents (Word, PowerPoint, Excel, PDF)
//! from SharePoint and OneDrive URLs to markdown format. It handles URL parsing,
//! document downloading, and format conversion.
//!
//! # Supported Services
//!
//! - SharePoint Online: `https://{tenant}.sharepoint.com/sites/{site}/...`
//! - OneDrive for Business: `https://{tenant}-my.sharepoint.com/personal/{user}/...`
//! - OneDrive Personal: `https://onedrive.live.com/...`
//! - Office Online: `https://{tenant}.office.com/...`
//!
//! # Supported Document Types
//!
//! - Word documents (.docx) - Requires pandoc for conversion
//! - PowerPoint presentations (.pptx) - Basic text extraction
//! - Excel spreadsheets (.xlsx) - Table conversion to markdown
//! - PDF files (.pdf) - Text extraction
//!
//! # Usage Examples
//!
//! ## Basic Conversion
//!
//! ```rust
//! use markdowndown::converters::Office365Converter;
//!
//! # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
//! let converter = Office365Converter::new();
//! let url = "https://company.sharepoint.com/sites/team/Shared%20Documents/document.docx";
//! let markdown = converter.convert(url).await?;
//! println!("Markdown content: {}", markdown);
//! # Ok(())
//! # }
//! ```
//!
//! # External Dependencies
//!
//! For full document conversion support, the following external tools are recommended:
//!
//! - **pandoc** - Universal document converter (required for Word/PowerPoint)
//! - **python3** with packages:
//!   - `python-pptx` - PowerPoint processing
//!   - `openpyxl` - Excel processing
//!   - `PyPDF2` or `pdfplumber` - PDF text extraction

use crate::client::HttpClient;
use crate::frontmatter::FrontmatterBuilder;
use crate::types::{Markdown, MarkdownError};
use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;
use url::Url as ParsedUrl;

/// Office 365 document types supported for conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentType {
    /// Microsoft Word document (.docx)
    Word,
    /// Microsoft PowerPoint presentation (.pptx)
    PowerPoint,
    /// Microsoft Excel spreadsheet (.xlsx)
    Excel,
    /// PDF document (.pdf)
    Pdf,
    /// Unknown or unsupported document type
    Unknown,
}

impl DocumentType {
    /// Returns the file extension associated with this document type.
    pub fn extension(&self) -> &'static str {
        match self {
            DocumentType::Word => "docx",
            DocumentType::PowerPoint => "pptx",
            DocumentType::Excel => "xlsx",
            DocumentType::Pdf => "pdf",
            DocumentType::Unknown => "",
        }
    }

    /// Returns the MIME type associated with this document type.
    pub fn mime_type(&self) -> &'static str {
        match self {
            DocumentType::Word => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            DocumentType::PowerPoint => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            DocumentType::Excel => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            DocumentType::Pdf => "application/pdf",
            DocumentType::Unknown => "application/octet-stream",
        }
    }

    /// Detects document type from file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "docx" => DocumentType::Word,
            "pptx" => DocumentType::PowerPoint,
            "xlsx" => DocumentType::Excel,
            "pdf" => DocumentType::Pdf,
            _ => DocumentType::Unknown,
        }
    }
}

/// Represents a parsed Office 365 document URL with metadata.
#[derive(Debug, Clone)]
pub struct Office365Document {
    /// The Office 365 tenant name (e.g., "company" from company.sharepoint.com)
    pub tenant: String,
    /// The service type (SharePoint, OneDrive, etc.)
    pub service: Office365Service,
    /// The site path for SharePoint documents
    pub site_path: Option<String>,
    /// The document path within the service
    pub document_path: String,
    /// The detected document type
    pub document_type: DocumentType,
    /// Original URL for reference
    pub original_url: String,
    /// Additional parameters from the URL
    pub parameters: HashMap<String, String>,
}

/// Office 365 service types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Office365Service {
    /// SharePoint Online site
    SharePoint,
    /// OneDrive for Business
    OneDriveBusiness,
    /// OneDrive Personal
    OneDrivePersonal,
    /// Office Online
    OfficeOnline,
}

/// Configuration for external tools used by the Office 365 converter.
#[derive(Debug, Clone)]
pub struct Office365Config {
    /// Path to the pandoc executable for Word document conversion
    pub pandoc_path: String,
    /// Additional arguments to pass to pandoc
    pub pandoc_args: Vec<String>,
    /// Path to python3 executable for PowerPoint/Excel conversion
    pub python_path: String,
    /// Whether to enable external tool conversion
    pub enable_external_tools: bool,
    /// Whether to extract media files during conversion
    pub extract_media: bool,
    /// Directory to extract media files to (relative to current directory)
    pub media_dir: String,
}

impl Default for Office365Config {
    fn default() -> Self {
        Self {
            pandoc_path: "pandoc".to_string(),
            pandoc_args: vec!["--wrap=none".to_string(), "--extract-media=./".to_string()],
            python_path: "python3".to_string(),
            enable_external_tools: false,
            extract_media: true,
            media_dir: "./".to_string(),
        }
    }
}

impl Office365Config {
    /// Creates a new Office365Config with external tools enabled and default paths.
    pub fn with_external_tools() -> Self {
        Self {
            enable_external_tools: true,
            ..Default::default()
        }
    }

    /// Sets a custom pandoc executable path.
    pub fn with_pandoc_path(mut self, path: impl Into<String>) -> Self {
        self.pandoc_path = path.into();
        self
    }

    /// Sets custom pandoc arguments.
    pub fn with_pandoc_args(mut self, args: Vec<String>) -> Self {
        self.pandoc_args = args;
        self
    }

    /// Sets a custom python executable path.
    pub fn with_python_path(mut self, path: impl Into<String>) -> Self {
        self.python_path = path.into();
        self
    }

    /// Sets the media extraction directory.
    pub fn with_media_dir(mut self, dir: impl Into<String>) -> Self {
        self.media_dir = dir.into();
        // Update pandoc args to use the new media directory
        for arg in &mut self.pandoc_args {
            if arg.starts_with("--extract-media=") {
                *arg = format!("--extract-media={}", self.media_dir);
            }
        }
        self
    }

    /// Disables media extraction during conversion.
    pub fn without_media_extraction(mut self) -> Self {
        self.extract_media = false;
        // Remove extract-media argument from pandoc args
        self.pandoc_args
            .retain(|arg| !arg.starts_with("--extract-media="));
        self
    }
}

/// Office 365 to markdown converter with intelligent URL handling and document processing.
///
/// This converter handles various Office 365 URL formats and converts documents
/// to markdown. It provides robust error handling for authentication issues
/// and unsupported formats.
#[derive(Debug, Clone)]
pub struct Office365Converter {
    /// HTTP client for making requests to Office 365 services
    client: HttpClient,
    /// Configuration for external tools and options
    config: Office365Config,
}

impl Office365Converter {
    /// Creates a new Office 365 converter with default configuration.
    ///
    /// Default configuration includes:
    /// - HTTP client with retry logic and timeouts
    /// - External tools disabled (requires manual setup)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::Office365Converter;
    ///
    /// let converter = Office365Converter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            client: HttpClient::new(),
            config: Office365Config::default(),
        }
    }

    /// Creates a new Office 365 converter with external tools enabled.
    ///
    /// This enables conversion using external tools like pandoc. Requires
    /// these tools to be installed and available in the system PATH.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::Office365Converter;
    ///
    /// let converter = Office365Converter::with_external_tools();
    /// ```
    pub fn with_external_tools() -> Self {
        Self {
            client: HttpClient::new(),
            config: Office365Config::with_external_tools(),
        }
    }

    /// Creates a new Office 365 converter with custom configuration.
    ///
    /// This allows full control over external tool paths, arguments, and options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::{Office365Converter, Office365Config};
    ///
    /// let config = Office365Config::with_external_tools()
    ///     .with_pandoc_path("/usr/local/bin/pandoc")
    ///     .with_media_dir("./media");
    /// let converter = Office365Converter::with_config(config);
    /// ```
    pub fn with_config(config: Office365Config) -> Self {
        Self {
            client: HttpClient::new(),
            config,
        }
    }

    /// Converts an Office 365 document URL to markdown with frontmatter.
    ///
    /// This method performs the complete conversion workflow:
    /// 1. Parse and validate the Office 365 URL
    /// 2. Detect document type from URL or content
    /// 3. Construct appropriate download URL
    /// 4. Download document content
    /// 5. Convert to markdown using appropriate method
    /// 6. Generate frontmatter with metadata
    /// 7. Combine frontmatter with content
    ///
    /// # Arguments
    ///
    /// * `url` - The Office 365 document URL to convert
    ///
    /// # Returns
    ///
    /// Returns a `Markdown` instance containing the document content with frontmatter,
    /// or a `MarkdownError` on failure.
    ///
    /// # Errors
    ///
    /// * `MarkdownError::InvalidUrl` - If the URL format is invalid
    /// * `MarkdownError::AuthError` - If the document requires authentication
    /// * `MarkdownError::NetworkError` - For network-related failures
    /// * `MarkdownError::ParseError` - If document conversion fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::Office365Converter;
    ///
    /// # async fn example() -> Result<(), markdowndown::types::MarkdownError> {
    /// let converter = Office365Converter::new();
    /// let url = "https://company.sharepoint.com/sites/team/Shared%20Documents/doc.docx";
    /// let markdown = converter.convert(url).await?;
    ///
    /// // The result includes frontmatter with metadata
    /// assert!(markdown.as_str().contains("source_url:"));
    /// assert!(markdown.as_str().contains("document_type:"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn convert(&self, url: &str) -> Result<Markdown, MarkdownError> {
        // Step 1: Parse and validate the Office 365 URL
        let document = self.parse_office365_url(url)?;

        // Step 2: Construct download URL
        let download_url = self.build_download_url(&document)?;

        // Step 3: Download document content
        let content_data = self.download_document(&download_url).await?;

        // Step 4: Convert document to markdown
        let markdown_content = self
            .convert_document(&content_data, &document.document_type)
            .await?;

        // Step 5: Generate frontmatter
        let now = Utc::now();
        let title = self.extract_title_from_filename(&document.document_path);
        let frontmatter = FrontmatterBuilder::new(url.to_string())
            .exporter(format!(
                "markdowndown-office365-{}",
                env!("CARGO_PKG_VERSION")
            ))
            .download_date(now)
            .additional_field("title".to_string(), title)
            .additional_field("url".to_string(), url.to_string())
            .additional_field("converter".to_string(), "Office365Converter".to_string())
            .additional_field("converted_at".to_string(), now.to_rfc3339())
            .additional_field("conversion_type".to_string(), "office365".to_string())
            .additional_field(
                "document_type".to_string(),
                self.document_type_string(&document.document_type),
            )
            .additional_field(
                "service".to_string(),
                self.service_string(&document.service),
            )
            .additional_field("tenant".to_string(), document.tenant.clone())
            .build()?;

        // Step 6: Combine frontmatter with content
        let markdown_with_frontmatter = format!("{frontmatter}\n{markdown_content}");

        Markdown::new(markdown_with_frontmatter)
    }

    /// Parses an Office 365 URL and extracts document information.
    ///
    /// # Arguments
    ///
    /// * `url` - The Office 365 URL to parse
    ///
    /// # Returns
    ///
    /// Returns an `Office365Document` with parsed information, or a `MarkdownError` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use markdowndown::converters::Office365Converter;
    ///
    /// let converter = Office365Converter::new();
    /// let url = "https://company.sharepoint.com/sites/team/Shared%20Documents/doc.docx";
    /// let doc = converter.parse_office365_url(url)?;
    /// assert_eq!(doc.tenant, "company");
    /// # Ok::<(), markdowndown::types::MarkdownError>(())
    /// ```
    pub fn parse_office365_url(&self, url: &str) -> Result<Office365Document, MarkdownError> {
        let parsed_url = ParsedUrl::parse(url.trim()).map_err(|_| MarkdownError::InvalidUrl {
            url: url.to_string(),
        })?;

        let host = parsed_url
            .host_str()
            .ok_or_else(|| MarkdownError::InvalidUrl {
                url: url.to_string(),
            })?;

        // Parse different Office 365 service patterns
        if let Some(sharepoint_doc) = self.parse_sharepoint_url(&parsed_url, host)? {
            return Ok(sharepoint_doc);
        }

        if let Some(onedrive_doc) = self.parse_onedrive_url(&parsed_url, host)? {
            return Ok(onedrive_doc);
        }

        if let Some(office_doc) = self.parse_office_online_url(&parsed_url, host)? {
            return Ok(office_doc);
        }

        Err(MarkdownError::InvalidUrl {
            url: url.to_string(),
        })
    }

    /// Constructs a download URL for an Office 365 document.
    ///
    /// # Arguments
    ///
    /// * `document` - The parsed Office 365 document information
    ///
    /// # Returns
    ///
    /// Returns a download URL string, or a `MarkdownError` if construction fails.
    pub fn build_download_url(
        &self,
        document: &Office365Document,
    ) -> Result<String, MarkdownError> {
        match document.service {
            Office365Service::SharePoint => self.build_sharepoint_download_url(document),
            Office365Service::OneDriveBusiness => {
                self.build_onedrive_business_download_url(document)
            }
            Office365Service::OneDrivePersonal => {
                self.build_onedrive_personal_download_url(document)
            }
            Office365Service::OfficeOnline => self.build_office_online_download_url(document),
        }
    }

    /// Downloads document content from an Office 365 URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The download URL for the document
    ///
    /// # Returns
    ///
    /// Returns the document content as bytes, or a `MarkdownError` on failure.
    pub async fn download_document(&self, url: &str) -> Result<Vec<u8>, MarkdownError> {
        let bytes = self.client.get_bytes(url).await?;
        Ok(bytes.to_vec())
    }

    /// Converts document data to markdown based on document type.
    ///
    /// This method uses different strategies based on the document type and
    /// whether external tools are enabled.
    ///
    /// # Arguments
    ///
    /// * `data` - The document content as bytes
    /// * `doc_type` - The detected document type
    ///
    /// # Returns
    ///
    /// Returns markdown content as a string, or a `MarkdownError` on failure.
    pub async fn convert_document(
        &self,
        data: &[u8],
        doc_type: &DocumentType,
    ) -> Result<String, MarkdownError> {
        if data.is_empty() {
            return Err(MarkdownError::ParseError {
                message: "Document content is empty".to_string(),
            });
        }

        match doc_type {
            DocumentType::Word => self.convert_word_document(data).await,
            DocumentType::PowerPoint => self.convert_powerpoint_document(data).await,
            DocumentType::Excel => self.convert_excel_document(data).await,
            DocumentType::Pdf => self.convert_pdf_document(data).await,
            DocumentType::Unknown => Err(MarkdownError::ParseError {
                message: "Cannot convert document of unknown type".to_string(),
            }),
        }
    }

    /// Detects document type from file path.
    pub fn detect_document_type(&self, path: &str) -> DocumentType {
        if let Some(extension) = path.split('.').next_back() {
            DocumentType::from_extension(extension)
        } else {
            DocumentType::Unknown
        }
    }

    // Private helper methods for URL parsing

    /// Parses SharePoint URLs.
    fn parse_sharepoint_url(
        &self,
        parsed_url: &ParsedUrl,
        host: &str,
    ) -> Result<Option<Office365Document>, MarkdownError> {
        if !host.contains(".sharepoint.com") || host.contains("-my.sharepoint.com") {
            return Ok(None);
        }

        let tenant = host.split('.').next().unwrap_or("").to_string();
        let path = parsed_url.path();

        // Extract site path and document path
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if path_segments.len() < 3 {
            return Ok(None);
        }

        let site_path = if path_segments[0] == "sites" && path_segments.len() > 1 {
            Some(path_segments[1].to_string())
        } else {
            None
        };

        let document_path = path.to_string();
        let document_type = self.detect_document_type(path);

        let parameters = self.extract_url_parameters(parsed_url);

        Ok(Some(Office365Document {
            tenant,
            service: Office365Service::SharePoint,
            site_path,
            document_path,
            document_type,
            original_url: parsed_url.to_string(),
            parameters,
        }))
    }

    /// Parses OneDrive URLs (both business and personal).
    fn parse_onedrive_url(
        &self,
        parsed_url: &ParsedUrl,
        host: &str,
    ) -> Result<Option<Office365Document>, MarkdownError> {
        let service = if host.contains("-my.sharepoint.com") {
            Office365Service::OneDriveBusiness
        } else if host == "onedrive.live.com" {
            Office365Service::OneDrivePersonal
        } else {
            return Ok(None);
        };

        let tenant = if service == Office365Service::OneDriveBusiness {
            host.split('-').next().unwrap_or("").to_string()
        } else {
            "live".to_string()
        };

        let path = parsed_url.path();
        let document_path = path.to_string();
        let document_type = self.detect_document_type(path);
        let parameters = self.extract_url_parameters(parsed_url);

        Ok(Some(Office365Document {
            tenant,
            service,
            site_path: None,
            document_path,
            document_type,
            original_url: parsed_url.to_string(),
            parameters,
        }))
    }

    /// Parses Office Online URLs.
    fn parse_office_online_url(
        &self,
        parsed_url: &ParsedUrl,
        host: &str,
    ) -> Result<Option<Office365Document>, MarkdownError> {
        if !host.contains(".office.com") {
            return Ok(None);
        }

        let tenant = host.split('.').next().unwrap_or("").to_string();
        let path = parsed_url.path();
        let document_path = path.to_string();
        let document_type = self.detect_document_type(path);
        let parameters = self.extract_url_parameters(parsed_url);

        Ok(Some(Office365Document {
            tenant,
            service: Office365Service::OfficeOnline,
            site_path: None,
            document_path,
            document_type,
            original_url: parsed_url.to_string(),
            parameters,
        }))
    }

    /// Extracts URL parameters into a HashMap.
    fn extract_url_parameters(&self, parsed_url: &ParsedUrl) -> HashMap<String, String> {
        parsed_url.query_pairs().into_owned().collect()
    }

    // Private helper methods for download URL construction

    /// Builds SharePoint download URL.
    fn build_sharepoint_download_url(
        &self,
        document: &Office365Document,
    ) -> Result<String, MarkdownError> {
        // SharePoint download URLs typically use the format:
        // https://{tenant}.sharepoint.com/{path}?download=1
        let base_url = format!(
            "https://{}.sharepoint.com{}",
            document.tenant, document.document_path
        );
        Ok(format!("{base_url}?download=1"))
    }

    /// Builds OneDrive for Business download URL.
    fn build_onedrive_business_download_url(
        &self,
        document: &Office365Document,
    ) -> Result<String, MarkdownError> {
        // OneDrive for Business download URLs typically use:
        // https://{tenant}-my.sharepoint.com/{path}?download=1
        let base_url = format!(
            "https://{}-my.sharepoint.com{}",
            document.tenant, document.document_path
        );
        Ok(format!("{base_url}?download=1"))
    }

    /// Builds OneDrive Personal download URL.
    fn build_onedrive_personal_download_url(
        &self,
        document: &Office365Document,
    ) -> Result<String, MarkdownError> {
        // OneDrive Personal URLs are more complex and often require API calls
        // For now, return the original URL and let the HTTP client handle redirects
        Ok(document.original_url.clone())
    }

    /// Builds Office Online download URL.
    fn build_office_online_download_url(
        &self,
        document: &Office365Document,
    ) -> Result<String, MarkdownError> {
        // Office Online download URLs vary by document type
        Ok(document.original_url.clone())
    }

    // Private helper methods for document conversion

    /// Converts Word documents to markdown using pandoc.
    async fn convert_word_document(&self, data: &[u8]) -> Result<String, MarkdownError> {
        if !self.config.enable_external_tools {
            return Err(MarkdownError::ParseError {
                message: "Word document conversion requires external tools. Use with_external_tools() and install pandoc.".to_string(),
            });
        }

        // Create a temporary file for the Word document
        let mut temp_docx = NamedTempFile::new().map_err(|e| MarkdownError::ParseError {
            message: format!("Failed to create temporary file for Word document: {e}"),
        })?;

        // Write the document data to the temporary file
        temp_docx
            .write_all(data)
            .map_err(|e| MarkdownError::ParseError {
                message: format!("Failed to write Word document data: {e}"),
            })?;

        // Build pandoc command using configuration
        let mut cmd = Command::new(&self.config.pandoc_path);
        cmd.arg("--from=docx").arg("--to=markdown");

        // Add custom pandoc arguments from configuration
        for arg in &self.config.pandoc_args {
            cmd.arg(arg);
        }

        // Add the input file
        cmd.arg(temp_docx.path());

        let output = cmd.output().map_err(|e| MarkdownError::ParseError {
            message: format!(
                "Failed to execute pandoc at '{}' (ensure pandoc is installed): {e}",
                self.config.pandoc_path
            ),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MarkdownError::ParseError {
                message: format!("Pandoc conversion failed: {stderr}"),
            });
        }

        let markdown_content =
            String::from_utf8(output.stdout).map_err(|e| MarkdownError::ParseError {
                message: format!("Failed to parse pandoc output as UTF-8: {e}"),
            })?;

        // Clean up any excessive whitespace
        let cleaned_content = markdown_content
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        if cleaned_content.is_empty() {
            return Err(MarkdownError::ParseError {
                message: "Pandoc conversion resulted in empty content".to_string(),
            });
        }

        Ok(cleaned_content)
    }

    /// Converts PowerPoint documents to markdown.
    async fn convert_powerpoint_document(&self, _data: &[u8]) -> Result<String, MarkdownError> {
        if self.config.enable_external_tools {
            // TODO: Implement PowerPoint text extraction using python-pptx
            Err(MarkdownError::ParseError {
                message: format!("PowerPoint conversion requires python-pptx (not yet implemented). Python path: {}", self.config.python_path),
            })
        } else {
            Err(MarkdownError::ParseError {
                message: "PowerPoint conversion requires external tools. Use with_external_tools() and install python-pptx.".to_string(),
            })
        }
    }

    /// Converts Excel documents to markdown.
    async fn convert_excel_document(&self, _data: &[u8]) -> Result<String, MarkdownError> {
        if self.config.enable_external_tools {
            // TODO: Implement Excel table extraction using openpyxl
            Err(MarkdownError::ParseError {
                message: format!(
                    "Excel conversion requires openpyxl (not yet implemented). Python path: {}",
                    self.config.python_path
                ),
            })
        } else {
            Err(MarkdownError::ParseError {
                message: "Excel conversion requires external tools. Use with_external_tools() and install openpyxl.".to_string(),
            })
        }
    }

    /// Converts PDF documents to markdown.
    async fn convert_pdf_document(&self, _data: &[u8]) -> Result<String, MarkdownError> {
        if self.config.enable_external_tools {
            // TODO: Implement PDF text extraction using PyPDF2 or pdfplumber
            Err(MarkdownError::ParseError {
                message: format!("PDF conversion requires PyPDF2 or pdfplumber (not yet implemented). Python path: {}", self.config.python_path),
            })
        } else {
            Err(MarkdownError::ParseError {
                message: "PDF conversion requires external tools. Use with_external_tools() and install PyPDF2 or pdfplumber.".to_string(),
            })
        }
    }

    // Private helper methods for metadata

    /// Converts document type to string for frontmatter.
    fn document_type_string(&self, doc_type: &DocumentType) -> String {
        match doc_type {
            DocumentType::Word => "word".to_string(),
            DocumentType::PowerPoint => "powerpoint".to_string(),
            DocumentType::Excel => "excel".to_string(),
            DocumentType::Pdf => "pdf".to_string(),
            DocumentType::Unknown => "unknown".to_string(),
        }
    }

    /// Converts service type to string for frontmatter.
    fn service_string(&self, service: &Office365Service) -> String {
        match service {
            Office365Service::SharePoint => "sharepoint".to_string(),
            Office365Service::OneDriveBusiness => "onedrive_business".to_string(),
            Office365Service::OneDrivePersonal => "onedrive_personal".to_string(),
            Office365Service::OfficeOnline => "office_online".to_string(),
        }
    }

    /// Extracts a title from the filename by removing extension and URL decoding.
    fn extract_title_from_filename(&self, filename: &str) -> String {
        // URL decode the filename - simple replacement of common encoded characters
        let decoded = filename
            .replace("%20", " ")
            .replace("%28", "(")
            .replace("%29", ")")
            .replace("%2C", ",");

        // Remove file extension
        if let Some(stem) = std::path::Path::new(&decoded).file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            decoded
        }
    }
}

impl Default for Office365Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_type_from_extension() {
        assert_eq!(DocumentType::from_extension("docx"), DocumentType::Word);
        assert_eq!(DocumentType::from_extension("DOCX"), DocumentType::Word);
        assert_eq!(
            DocumentType::from_extension("pptx"),
            DocumentType::PowerPoint
        );
        assert_eq!(DocumentType::from_extension("xlsx"), DocumentType::Excel);
        assert_eq!(DocumentType::from_extension("pdf"), DocumentType::Pdf);
        assert_eq!(DocumentType::from_extension("txt"), DocumentType::Unknown);
    }

    #[test]
    fn test_document_type_properties() {
        assert_eq!(DocumentType::Word.extension(), "docx");
        assert_eq!(DocumentType::PowerPoint.extension(), "pptx");
        assert_eq!(DocumentType::Excel.extension(), "xlsx");
        assert_eq!(DocumentType::Pdf.extension(), "pdf");

        assert_eq!(
            DocumentType::Word.mime_type(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        );
        assert_eq!(DocumentType::Pdf.mime_type(), "application/pdf");
    }

    #[test]
    fn test_office365_converter_new() {
        let converter = Office365Converter::new();
        assert!(!converter.config.enable_external_tools);
        assert_eq!(converter.config.pandoc_path, "pandoc");
        assert_eq!(converter.config.python_path, "python3");
    }

    #[test]
    fn test_office365_converter_with_external_tools() {
        let converter = Office365Converter::with_external_tools();
        assert!(converter.config.enable_external_tools);
        assert_eq!(converter.config.pandoc_path, "pandoc");
    }

    #[test]
    fn test_office365_converter_with_config() {
        let config = Office365Config::with_external_tools()
            .with_pandoc_path("/custom/pandoc")
            .with_python_path("/custom/python")
            .with_media_dir("./custom_media");

        let converter = Office365Converter::with_config(config);
        assert!(converter.config.enable_external_tools);
        assert_eq!(converter.config.pandoc_path, "/custom/pandoc");
        assert_eq!(converter.config.python_path, "/custom/python");
        assert_eq!(converter.config.media_dir, "./custom_media");
    }

    #[test]
    fn test_office365_config_default() {
        let config = Office365Config::default();
        assert!(!config.enable_external_tools);
        assert_eq!(config.pandoc_path, "pandoc");
        assert_eq!(config.python_path, "python3");
        assert!(config.extract_media);
        assert_eq!(config.media_dir, "./");
        assert!(config.pandoc_args.contains(&"--wrap=none".to_string()));
        assert!(config
            .pandoc_args
            .iter()
            .any(|arg| arg.starts_with("--extract-media=")));
    }

    #[test]
    fn test_office365_config_fluent_interface() {
        let config = Office365Config::default()
            .with_pandoc_path("/usr/bin/pandoc")
            .with_python_path("/usr/bin/python3")
            .with_media_dir("./documents")
            .without_media_extraction();

        assert_eq!(config.pandoc_path, "/usr/bin/pandoc");
        assert_eq!(config.python_path, "/usr/bin/python3");
        assert_eq!(config.media_dir, "./documents");
        assert!(!config.extract_media);
        assert!(!config
            .pandoc_args
            .iter()
            .any(|arg| arg.starts_with("--extract-media=")));
    }

    #[test]
    fn test_parse_sharepoint_url() {
        let converter = Office365Converter::new();
        let url = "https://company.sharepoint.com/sites/team/Shared%20Documents/document.docx";
        let result = converter.parse_office365_url(url).unwrap();

        assert_eq!(result.tenant, "company");
        assert_eq!(result.service, Office365Service::SharePoint);
        assert_eq!(result.site_path, Some("team".to_string()));
        assert_eq!(result.document_type, DocumentType::Word);
        assert_eq!(result.original_url, url);
    }

    #[test]
    fn test_parse_onedrive_business_url() {
        let converter = Office365Converter::new();
        let url =
            "https://company-my.sharepoint.com/personal/user_company_com/Documents/document.xlsx";
        let result = converter.parse_office365_url(url).unwrap();

        assert_eq!(result.tenant, "company");
        assert_eq!(result.service, Office365Service::OneDriveBusiness);
        assert_eq!(result.site_path, None);
        assert_eq!(result.document_type, DocumentType::Excel);
    }

    #[test]
    fn test_parse_onedrive_personal_url() {
        let converter = Office365Converter::new();
        let url = "https://onedrive.live.com/redir?resid=123&authkey=456&file=presentation.pptx";
        let result = converter.parse_office365_url(url).unwrap();

        assert_eq!(result.tenant, "live");
        assert_eq!(result.service, Office365Service::OneDrivePersonal);
        assert!(result.parameters.contains_key("resid"));
        assert!(result.parameters.contains_key("authkey"));
    }

    #[test]
    fn test_parse_office_online_url() {
        let converter = Office365Converter::new();
        let url = "https://company.office.com/documents/report.pdf";
        let result = converter.parse_office365_url(url).unwrap();

        assert_eq!(result.tenant, "company");
        assert_eq!(result.service, Office365Service::OfficeOnline);
        assert_eq!(result.document_type, DocumentType::Pdf);
    }

    #[test]
    fn test_parse_invalid_url() {
        let converter = Office365Converter::new();
        let url = "https://example.com/not-office365";
        let result = converter.parse_office365_url(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_sharepoint_download_url() {
        let converter = Office365Converter::new();
        let document = Office365Document {
            tenant: "company".to_string(),
            service: Office365Service::SharePoint,
            site_path: Some("team".to_string()),
            document_path: "/sites/team/Shared%20Documents/doc.docx".to_string(),
            document_type: DocumentType::Word,
            original_url: "https://company.sharepoint.com/sites/team/Shared%20Documents/doc.docx"
                .to_string(),
            parameters: HashMap::new(),
        };

        let download_url = converter.build_download_url(&document).unwrap();
        assert!(download_url.contains("company.sharepoint.com"));
        assert!(download_url.contains("download=1"));
    }

    #[test]
    fn test_detect_document_type() {
        let converter = Office365Converter::new();

        assert_eq!(
            converter.detect_document_type("/path/to/document.docx"),
            DocumentType::Word
        );
        assert_eq!(
            converter.detect_document_type("/path/to/presentation.pptx"),
            DocumentType::PowerPoint
        );
        assert_eq!(
            converter.detect_document_type("/path/to/spreadsheet.xlsx"),
            DocumentType::Excel
        );
        assert_eq!(
            converter.detect_document_type("/path/to/document.pdf"),
            DocumentType::Pdf
        );
        assert_eq!(
            converter.detect_document_type("/path/to/file"),
            DocumentType::Unknown
        );
    }

    #[test]
    fn test_document_conversion_errors() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let converter = Office365Converter::new();

        // Test that conversion fails appropriately when external tools are disabled
        rt.block_on(async {
            let data = b"fake document data";

            let result = converter.convert_word_document(data).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("external tools"));

            let result = converter.convert_powerpoint_document(data).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("external tools"));

            let result = converter.convert_excel_document(data).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("external tools"));

            let result = converter.convert_pdf_document(data).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("external tools"));
        });
    }

    #[test]
    fn test_document_conversion_with_external_tools() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let converter = Office365Converter::with_external_tools();

        rt.block_on(async {
            let data = b"fake document data";

            // Should fail with pandoc-related error when external tools are enabled
            // (either pandoc not found or conversion failure with fake data)
            let result = converter.convert_word_document(data).await;
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string().to_lowercase();
            assert!(
                error_msg.contains("pandoc") || error_msg.contains("failed to execute"),
                "Expected pandoc-related error, got: {error_msg}"
            );
        });
    }

    #[test]
    fn test_empty_document_data() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let converter = Office365Converter::new();

        rt.block_on(async {
            let result = converter.convert_document(&[], &DocumentType::Word).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("empty"));
        });
    }

    #[test]
    fn test_unknown_document_type_conversion() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let converter = Office365Converter::new();

        rt.block_on(async {
            let data = b"some data";
            let result = converter
                .convert_document(data, &DocumentType::Unknown)
                .await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("unknown type"));
        });
    }

    #[test]
    fn test_default_implementation() {
        let converter = Office365Converter::default();
        assert!(!converter.config.enable_external_tools);
    }

    // Edge case tests for URL parsing
    #[test]
    fn test_url_parsing_edge_cases() {
        let converter = Office365Converter::new();

        // Test URLs with query parameters
        let url_with_params =
            "https://company.sharepoint.com/sites/team/doc.docx?param1=value1&param2=value2";
        let result = converter.parse_office365_url(url_with_params).unwrap();
        assert_eq!(result.parameters.get("param1"), Some(&"value1".to_string()));
        assert_eq!(result.parameters.get("param2"), Some(&"value2".to_string()));

        // Test URL with no file extension
        let url_no_ext = "https://company.sharepoint.com/sites/team/document";
        let result = converter.parse_office365_url(url_no_ext).unwrap();
        assert_eq!(result.document_type, DocumentType::Unknown);

        // Test malformed URLs
        let bad_urls = [
            "",
            "not-a-url",
            "https://example.com",
            "https://not-office365.com/document.docx",
        ];

        for bad_url in &bad_urls {
            let result = converter.parse_office365_url(bad_url);
            assert!(result.is_err(), "Should fail for URL: {bad_url}");
        }
    }
}
