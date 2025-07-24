# MarkdownDown

A Rust library for acquiring markdown from URLs with smart handling.

## Overview

MarkdownDown provides a unified interface for extracting and converting content from various URL sources into clean markdown format. It intelligently detects the URL type and applies the appropriate conversion strategy.

## Supported URL Types

- **HTML Pages**: Direct HTML to markdown conversion with content cleaning
- **Google Docs**: Smart extraction from Google Docs sharing URLs
- **Office 365**: Handler for Office 365 document URLs
- **GitHub**: Issues, pull requests, and other GitHub content

## Architecture

The library follows a modular architecture:

- **Core Types**: Extensible traits and types for URL handling
- **HTTP Client**: Consistent network operations wrapper
- **URL Detection**: Automatic handler selection based on URL patterns
- **Specialized Handlers**: Type-specific conversion implementations
- **Unified API**: Simple integration interface

## Usage

```rust
use markdowndown::*;

// Basic usage example (will be implemented in future iterations)
// let markdown = fetch_markdown("https://example.com").await?;
```

## Development Setup

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

# Generate documentation
cargo doc --open
```

### Project Structure

```
markdowndown/
├── src/
│   ├── lib.rs              # Library root and module declarations
│   ├── types/              # Core types and traits
│   ├── client/             # HTTP client wrapper
│   ├── detection/          # URL type detection
│   ├── handlers/           # URL-specific handlers
│   │   ├── html.rs        # HTML page handler
│   │   ├── google_docs.rs # Google Docs handler
│   │   ├── office365.rs   # Office 365 handler
│   │   └── github.rs      # GitHub handler
│   └── api/               # Public API
├── tests/                  # Integration tests
└── examples/              # Usage examples
```

## License

MIT

## Contributing

This project follows Test-Driven Development (TDD) practices. All contributions should include comprehensive tests.