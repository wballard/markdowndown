# markdowndown

A Rust library for converting URLs to markdown with intelligent handling of different URL types.

## Features

- 🌐 **Universal URL Support**: Convert any web page to clean markdown  
- 📝 **Smart Conversion**: Specialized handlers for Google Docs, Office 365, GitHub Issues
- 🔧 **Configurable**: Flexible configuration for different use cases
- 🚀 **Fast & Reliable**: Built with performance and reliability in mind
- 📊 **Rich Metadata**: YAML frontmatter with source URL, date, and processing info
- 🔄 **Async Support**: Full async/await support with tokio
- 🛡️ **Robust Error Handling**: Comprehensive error types with recovery strategies

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
markdowndown = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Simple Usage

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = convert_url("https://example.com/article").await?;
    println!("{}", markdown);
    Ok(())
}
```

### With Configuration

```rust
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

## Supported URL Types

| URL Type | Example | Features |
|----------|---------|----------|
| HTML Pages | `https://example.com/article` | Clean HTML to markdown conversion |
| Google Docs | `https://docs.google.com/document/d/{id}/edit` | Direct markdown export |
| Office 365 | `https://company.sharepoint.com/.../document.docx` | Document download and conversion |
| GitHub Issues | `https://github.com/owner/repo/issues/123` | Issue + comments via API |

## API Overview

### Main Functions

- **`convert_url(url)`** - Convert any URL to markdown with default configuration
- **`convert_url_with_config(url, config)`** - Convert with custom configuration
- **`detect_url_type(url)`** - Determine URL type without conversion

### Core Types

- **`MarkdownDown`** - Main library struct with configuration
- **`Config`** - Configuration builder for customizing behavior
- **`Markdown`** - Validated markdown content wrapper
- **`MarkdownError`** - Comprehensive error handling

### Configuration Options

```rust
let config = Config::builder()
    // Authentication
    .github_token("ghp_xxxxxxxxxxxxxxxxxxxx")
    .office365_token("office_token") 
    .google_api_key("google_key")
    
    // HTTP Settings
    .timeout_seconds(60)
    .user_agent("MyApp/1.0")
    .max_retries(5)
    
    // Output Options
    .include_frontmatter(true)
    .custom_frontmatter_field("project", "my-project")
    .max_consecutive_blank_lines(2)
    
    .build();
```

## Error Handling

The library provides comprehensive error handling with specific error types:

```rust
use markdowndown::{convert_url, types::MarkdownError};

match convert_url("https://example.com").await {
    Ok(markdown) => println!("Success: {}", markdown),
    Err(MarkdownError::ValidationError { kind, context }) => {
        eprintln!("Invalid input: {:?}", kind);
    }
    Err(MarkdownError::EnhancedNetworkError { kind, context }) => {
        eprintln!("Network issue: {:?}", kind);
    }
    Err(MarkdownError::AuthenticationError { kind, context }) => {
        eprintln!("Auth problem: {:?}", kind);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Performance

Typical conversion times on modern hardware:

| URL Type | Small Document | Medium Document | Large Document |
|----------|----------------|-----------------|----------------|
| HTML Page | < 1s | 1-3s | 3-10s |
| Google Docs | < 2s | 2-5s | 5-15s |
| GitHub Issue | < 1s | 1-2s | 2-5s |
| Office 365 | 2-5s | 5-15s | 15-60s |

*Note: Performance metrics are hardware and network dependent. Actual conversion times may vary based on your system specifications, network connectivity, and document complexity.*

Memory usage scales linearly with document size. Network latency is typically the limiting factor.

## Examples

The repository includes comprehensive examples in the `examples/` directory:

- **`basic_usage.rs`** - Simple URL conversion
- **`with_configuration.rs`** - Custom configuration usage  
- **`batch_processing.rs`** - Converting multiple URLs
- **`async_usage.rs`** - Async/await patterns
- **`error_handling.rs`** - Comprehensive error handling

Run examples with:

```bash
cargo run --example basic_usage
cargo run --example with_configuration
```

## Environment Configuration

The library can be configured via environment variables:

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
export MARKDOWNDOWN_TIMEOUT=60
export MARKDOWNDOWN_USER_AGENT="MyApp/1.0"
export MARKDOWNDOWN_MAX_RETRIES=5
```

Then use:

```rust
let config = Config::from_env();
let md = MarkdownDown::with_config(config);
```

## Documentation

- **[Getting Started Guide](docs/getting-started.md)** - Installation and first steps
- **[Configuration Reference](docs/configuration.md)** - All configuration options
- **[URL Types Guide](docs/url-types.md)** - Supported URL types and specifics
- **[Error Handling Guide](docs/error-handling.md)** - Error types and recovery
- **[Performance Guide](docs/performance.md)** - Optimization tips and benchmarks
- **[Troubleshooting](docs/troubleshooting.md)** - Common issues and solutions
- **[API Reference](https://docs.rs/markdowndown)** - Complete API documentation

## Development

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Building

```bash
# Check the project
cargo check

# Build the project  
cargo build

# Run tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Generate documentation
cargo doc --open

# Run benchmarks
cargo bench
```

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Development environment setup
- Code style and standards
- Testing requirements
- Submitting pull requests

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and breaking changes.

## Support

- **Issues**: [GitHub Issues](https://github.com/wballard/markdowndown/issues)
- **Documentation**: [docs.rs/markdowndown](https://docs.rs/markdowndown)
- **Examples**: See the `examples/` directory