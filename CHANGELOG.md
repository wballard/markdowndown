# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Additional documentation and examples

### Changed
- Improved error messages and context

### Fixed
- Minor bug fixes and improvements

## [0.1.0] - 2024-01-15

### Added
- **Core Functionality**
  - URL to markdown conversion with automatic type detection
  - Support for 4 URL types: HTML pages, Google Docs, GitHub Issues, Office 365 documents
  - Async/await support with tokio runtime
  - Comprehensive error handling with detailed error types and context

- **Configuration System**
  - Flexible configuration via `Config` builder pattern
  - Environment variable support for common settings
  - Authentication support for GitHub (personal access tokens)
  - HTTP client configuration (timeouts, retries, user agent)
  - Output formatting options (frontmatter, whitespace normalization)

- **URL Type Detection**
  - Automatic detection of URL types with `detect_url_type()` function
  - Specialized converters for each supported URL type
  - Fallback strategies for enhanced reliability

- **Rich Error Handling**
  - Enhanced error system with detailed context and categorization
  - Legacy error compatibility for backward compatibility
  - Error characteristics (retryable, recoverable) for robust error handling
  - User-friendly error suggestions and recovery strategies

- **Metadata and Frontmatter**
  - YAML frontmatter generation with source metadata
  - Custom frontmatter fields support
  - Configurable frontmatter inclusion/exclusion
  - Rich document metadata preservation

- **Performance Features**
  - Configurable HTTP timeouts and retry logic
  - Network connection reuse and optimization
  - Memory-efficient processing for large documents
  - Benchmark suite for performance monitoring

- **Documentation and Examples**
  - Comprehensive API documentation with rustdoc
  - 5 practical usage examples covering different scenarios
  - Detailed guides for configuration, URL types, and error handling
  - Performance optimization guide and troubleshooting documentation

- **Testing Infrastructure**
  - Comprehensive unit test suite (>90% coverage)
  - Integration tests for all supported URL types
  - Property-based testing for validation functions
  - Mock servers for reliable testing

- **Developer Experience**
  - Clear error messages with actionable suggestions
  - Extensive logging and tracing support
  - Development tools and scripts for contributors
  - Continuous integration and automated testing

### Technical Details

- **Supported URL Types:**
  - HTML pages: Clean HTML-to-markdown conversion with content extraction
  - Google Docs: Direct export API integration for authentic document conversion
  - GitHub Issues: API-based extraction including comments and metadata
  - Office 365: Document download and conversion pipeline

- **Core Dependencies:**
  - `tokio` 1.0+ for async runtime
  - `reqwest` 0.11+ for HTTP client functionality
  - `html2text` 0.6+ for HTML to markdown conversion
  - `serde` 1.0+ for configuration and metadata serialization
  - `chrono` 0.4+ for timestamp handling
  - `thiserror` 1.0+ for error handling

- **API Overview:**
  - Main conversion function: `convert_url(url: &str) -> Result<Markdown, MarkdownError>`
  - Configured conversion: `MarkdownDown::with_config(config).convert_url(url)`
  - URL type detection: `detect_url_type(url: &str) -> Result<UrlType, MarkdownError>`
  - Configuration builder: `Config::builder()` with fluent interface

- **Error Types:**
  - `ValidationError` - Input validation failures
  - `EnhancedNetworkError` - Network and connectivity issues
  - `AuthenticationError` - Authentication and authorization failures
  - `ContentError` - Content processing and parsing failures
  - `ConverterError` - Converter-specific processing failures
  - `ConfigurationError` - Setup and configuration failures

- **Performance Characteristics:**
  - HTML pages: 1-5 seconds typical conversion time
  - Google Docs: 2-10 seconds depending on document size
  - GitHub Issues: 1-8 seconds including comment extraction
  - Office 365: 5-60 seconds depending on document size and network
  - Memory usage: 2-5x final markdown size during processing

### Breaking Changes
- N/A (initial release)

### Migration Guide
- N/A (initial release)

### Known Issues
- JavaScript-heavy websites may not convert properly (content loaded dynamically)
- Large Office 365 documents may require extended timeouts
- Some corporate firewalls may block programmatic access to SharePoint
- Rate limiting on GitHub API may affect batch processing of GitHub URLs

### Deprecations
- N/A (initial release)

### Security
- Authentication tokens are handled securely and never logged
- Network requests follow security best practices
- No sensitive information is exposed in error messages
- HTTPS is enforced for all external connections

---

## Version History

- **0.1.0** - Initial release with core functionality and comprehensive documentation

## Upgrade Instructions

### From Pre-release to 0.1.0
This is the initial stable release. No migration needed.

## Compatibility

### Rust Version Compatibility
- **Minimum Supported Rust Version (MSRV)**: 1.70.0
- **Edition**: 2021
- **Tested Versions**: 1.70.0 through 1.75.0

### Platform Support
- **Linux**: Full support (tested on Ubuntu 20.04+)
- **macOS**: Full support (tested on macOS 12+)  
- **Windows**: Full support (tested on Windows 10+)

### Dependency Compatibility
- Compatible with tokio 1.0+ ecosystem
- Compatible with serde 1.0+ ecosystem
- Uses stable Rust features only (no nightly required)

## Contributors

Special thanks to all contributors who made this release possible:

- Core development and architecture
- Documentation and examples
- Testing and quality assurance
- Performance optimization
- Error handling improvements

## Acknowledgments

This project builds upon the excellent work of:
- The Rust community for foundational libraries
- `html2text` for HTML to markdown conversion
- `reqwest` for HTTP client functionality
- `tokio` for async runtime support
- `serde` for serialization support

## Support

- **Documentation**: https://docs.rs/markdowndown
- **Repository**: https://github.com/wballard/markdowndown
- **Issues**: https://github.com/wballard/markdowndown/issues
- **Examples**: See `examples/` directory in the repository

For questions, bug reports, or feature requests, please use the GitHub issue tracker.

---

*This changelog follows the [Keep a Changelog](https://keepachangelog.com/) format and the project adheres to [Semantic Versioning](https://semver.org/).*