# Documentation and Examples

Create comprehensive documentation including API docs, usage examples, and getting started guides for the markdowndown library.

## Objectives

- Provide clear, comprehensive API documentation
- Create practical usage examples and tutorials
- Document configuration options and best practices
- Establish contributing guidelines and development setup

## Tasks

1. Create comprehensive README.md:
   - Project overview and key features
   - Installation instructions via crates.io
   - Quick start examples for common use cases
   - Supported URL types and their capabilities
   - Configuration options overview

2. Add rustdoc documentation to all public APIs:
   - Detailed documentation for all public structs, enums, and functions
   - Code examples for each public method
   - Usage patterns and best practices
   - Error handling examples
   - Performance considerations

3. Create examples directory with practical use cases:
   - `examples/basic_usage.rs` - Simple URL conversion
   - `examples/with_configuration.rs` - Custom configuration usage
   - `examples/batch_processing.rs` - Converting multiple URLs
   - `examples/async_usage.rs` - Async/await patterns
   - `examples/error_handling.rs` - Comprehensive error handling

4. Write detailed guides in `docs/` directory:
   - `docs/getting-started.md` - Installation and first steps
   - `docs/configuration.md` - All configuration options explained
   - `docs/url-types.md` - Supported URL types and their specifics
   - `docs/error-handling.md` - Error types and recovery strategies
   - `docs/performance.md` - Performance tips and benchmarks

5. Create API reference documentation:
   - Generate rustdoc with `cargo doc`
   - Include comprehensive examples in doc comments
   - Cross-reference related functions and types
   - Document all feature flags and optional dependencies

6. Add troubleshooting and FAQ documentation:
   - Common issues and their solutions
   - Authentication setup for different services
   - Network and firewall configuration
   - Performance optimization tips

7. Create contributing guidelines:
   - Development environment setup
   - Code style and contribution standards
   - Testing requirements and procedures
   - Release process and versioning

8. Document advanced usage patterns:
   - Custom converter implementation
   - Integration with web frameworks
   - Batch processing and rate limiting
   - Caching and performance optimization

9. Add configuration examples:
   - Environment variable configuration
   - Configuration file examples
   - Service-specific authentication setup
   - Production deployment considerations

10. Create changelog and migration guides:
    - Detailed changelog following semver conventions
    - Migration guides for breaking changes
    - Deprecation notices and alternatives
    - Version compatibility matrix

## Acceptance Criteria

- [ ] README provides clear project overview and quick start
- [ ] All public APIs have comprehensive rustdoc documentation
- [ ] Examples compile and run successfully
- [ ] Documentation covers all major use cases
- [ ] Troubleshooting guide addresses common issues
- [ ] Contributing guidelines are clear and actionable
- [ ] Documentation is spell-checked and well-formatted
- [ ] API docs include performance characteristics where relevant

## Dependencies

- Previous: [000013_integration_tests]
- Requires: Complete implementation of all features

## Documentation Structure

```
/
├── README.md                 # Main project documentation
├── CHANGELOG.md             # Version history and changes
├── CONTRIBUTING.md          # Contribution guidelines
├── docs/
│   ├── getting-started.md   # Installation and basics
│   ├── configuration.md     # Configuration reference  
│   ├── url-types.md         # Supported URL types
│   ├── error-handling.md    # Error handling guide
│   ├── performance.md       # Performance considerations
│   ├── troubleshooting.md   # Common issues and solutions
│   └── api-reference.md     # Generated API reference
├── examples/
│   ├── basic_usage.rs       # Simple conversion examples
│   ├── with_configuration.rs # Configuration examples
│   ├── batch_processing.rs  # Multiple URL processing
│   ├── async_usage.rs       # Async patterns
│   └── error_handling.rs    # Error handling patterns
└── src/
    └── lib.rs               # API documentation in rustdoc
```

## README.md Template

```markdown
# markdowndown

A Rust library for converting URLs to markdown with intelligent handling of different URL types.

## Features

- 🌐 **Universal URL Support**: Convert any web page to clean markdown
- 📝 **Smart Conversion**: Specialized handlers for Google Docs, Office 365, GitHub Issues
- 🔧 **Configurable**: Flexible configuration for different use cases
- 🚀 **Fast & Reliable**: Built with performance and reliability in mind
- 📊 **Rich Metadata**: YAML frontmatter with source URL, date, and processing info

## Quick Start

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = convert_url("https://example.com/article").await?;
    println!("{}", markdown);
    Ok(())
}
```

## Supported URL Types

| URL Type | Example | Features |
|----------|---------|----------|
| HTML Pages | `https://example.com/article` | Clean HTML to markdown conversion |
| Google Docs | `https://docs.google.com/document/d/{id}/edit` | Direct markdown export |
| Office 365 | `https://company.sharepoint.com/.../document.docx` | Document download and conversion |
| GitHub Issues | `https://github.com/owner/repo/issues/123` | Issue + comments via API |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
markdowndown = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```
```

## Rustdoc Examples

```rust
/// Converts a URL to markdown with automatic type detection.
///
/// This is the main entry point for the library. It automatically detects
/// the URL type and routes to the appropriate converter.  
///
/// # Arguments
///
/// * `url` - The URL to convert to markdown
///
/// # Returns
///
/// Returns a `Result` containing the converted `Markdown` or a `MarkdownError`.
///
/// # Examples
///
/// ```rust
/// use markdowndown::convert_url;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Convert a simple HTML page
/// let markdown = convert_url("https://example.com/article").await?;
/// println!("{}", markdown);
///
/// // Convert a Google Docs document
/// let markdown = convert_url("https://docs.google.com/document/d/abc123/edit").await?;
/// println!("{}", markdown);
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// This function can return various error types:
/// - `ValidationError` for invalid URLs
/// - `NetworkError` for connection issues  
/// - `AuthenticationError` for access denied
/// - `ContentError` for processing failures
///
/// ```rust
/// use markdowndown::{convert_url, MarkdownError};
///
/// # #[tokio::main]  
/// # async fn main() {
/// match convert_url("https://invalid-url").await {
///     Ok(markdown) => println!("Success: {}", markdown),
///     Err(MarkdownError::ValidationError(_, _)) => {
///         eprintln!("Invalid URL format");
///     }
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// # }
/// ```
pub async fn convert_url(url: &str) -> Result<Markdown, MarkdownError> {
    // Implementation
}
```

## Example Files

### Basic Usage Example
```rust
// examples/basic_usage.rs
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://blog.rust-lang.org/2024/01/15/Rust-1.75.0.html",
        "https://doc.rust-lang.org/book/ch01-00-getting-started.html",
        "https://github.com/rust-lang/rust/issues/100000",
    ];

    for url in urls {
        match convert_url(url).await {
            Ok(markdown) => {
                println!("✅ Successfully converted: {}", url);
                println!("Content length: {} characters", markdown.as_str().len());
                println!("---");
            }
            Err(e) => {
                eprintln!("❌ Failed to convert {}: {}", url, e);
            }
        }
    }

    Ok(())
}
```

### Configuration Example
```rust
// examples/with_configuration.rs
use markdowndown::{MarkdownDown, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .github_token("ghp_your_token_here")
        .timeout_seconds(60)
        .user_agent("my-app/1.0")
        .build();

    let md = MarkdownDown::with_config(config);
    
    let result = md.convert_url("https://github.com/rust-lang/rust/issues/1").await?;
    println!("{}", result);

    Ok(())
}
```

## Performance Documentation

Include performance characteristics and benchmarks:

```markdown
## Performance

Typical conversion times on modern hardware:

| URL Type | Small Document | Medium Document | Large Document |
|----------|----------------|-----------------|----------------|
| HTML Page | < 1s | 1-3s | 3-10s |
| Google Docs | < 2s | 2-5s | 5-15s |
| GitHub Issue | < 1s | 1-2s | 2-5s |
| Office 365 | 2-5s | 5-15s | 15-60s |

Memory usage scales linearly with document size. Network latency is typically the limiting factor.
```

## Contributing Guidelines

```markdown
# Contributing to markdowndown

## Development Setup

1. Clone the repository
2. Install Rust toolchain
3. Run tests: `cargo test`
4. Run integration tests: `cargo test --test integration`

## Code Style

- Follow rustfmt formatting
- Use clippy for linting
- Add rustdoc comments to all public APIs
- Include examples in documentation

## Testing

- Add unit tests for all new functionality
- Add integration tests for new URL types
- Ensure all tests pass before submitting PR
- Include performance considerations in reviews
```