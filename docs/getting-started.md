# Getting Started with markdowndown

Welcome to markdowndown! This guide will help you get up and running with the library quickly and efficiently.

## What is markdowndown?

markdowndown is a Rust library that converts URLs to clean markdown format with intelligent handling of different URL types. It supports HTML pages, Google Docs, Office 365 documents, and GitHub issues with specialized converters for each type.

## Installation

### Prerequisites

- Rust 1.70 or later (2021 edition)
- Cargo package manager

### Adding to Your Project

Add markdowndown to your `Cargo.toml`:

```toml
[dependencies]
markdowndown = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

Since markdowndown is fully async, you'll need an async runtime like tokio.

### Verifying Installation

Create a simple test to verify the installation:

```rust
// src/main.rs or examples/test_install.rs
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = convert_url("https://httpbin.org/html").await?;
    println!("Installation successful! Converted {} characters", markdown.as_str().len());
    Ok(())
}
```

Run with:
```bash
cargo run
# or
cargo run --example test_install
```

## Your First Conversion

### Simple URL Conversion

The easiest way to convert a URL is using the `convert_url` function:

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Convert a simple web page
    let markdown = convert_url("https://blog.rust-lang.org/2024/01/15/Rust-1.75.0.html").await?;
    
    // Print the result
    println!("Converted markdown:");
    println!("{}", markdown);
    
    Ok(())
}
```

### Understanding the Output

markdowndown returns a `Markdown` type that includes:

1. **YAML Frontmatter** (by default) with metadata:
   ```yaml
   ---
   source_url: "https://example.com/article"
   exporter: "markdowndown/0.1.0"
   date_downloaded: "2024-01-15T10:30:00Z"
   ---
   ```

2. **Clean Markdown Content** converted from the original HTML

### Working with the Result

The `Markdown` type provides several useful methods:

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = convert_url("https://example.com").await?;
    
    // Get the full content (frontmatter + markdown)
    println!("Full content: {}", markdown.as_str());
    
    // Get only the markdown content (without frontmatter)
    let content_only = markdown.content_only();
    println!("Content only: {}", content_only);
    
    // Check if it has frontmatter
    if let Some(frontmatter) = markdown.frontmatter() {
        println!("Has frontmatter: {} characters", frontmatter.len());
    }
    
    Ok(())
}
```

## URL Type Detection

markdowndown automatically detects URL types and uses appropriate converters. You can also detect types manually:

```rust
use markdowndown::{detect_url_type, types::UrlType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Detect different URL types
    let html_type = detect_url_type("https://example.com/article")?;
    assert_eq!(html_type, UrlType::Html);
    
    let gdocs_type = detect_url_type("https://docs.google.com/document/d/abc123/edit")?;
    assert_eq!(gdocs_type, UrlType::GoogleDocs);
    
    let github_type = detect_url_type("https://github.com/owner/repo/issues/123")?;
    assert_eq!(github_type, UrlType::GitHubIssue);
    
    println!("URL type detection working correctly!");
    Ok(())
}
```

## Error Handling

markdowndown provides comprehensive error handling. Here's how to handle common errors:

```rust
use markdowndown::{convert_url, types::MarkdownError};

#[tokio::main]
async fn main() {
    match convert_url("https://invalid-domain.nonexistent").await {
        Ok(markdown) => {
            println!("Success: {}", markdown);
        }
        Err(error) => {
            // Print the error
            eprintln!("Conversion failed: {}", error);
            
            // Get suggestions for fixing the error
            let suggestions = error.suggestions();
            if !suggestions.is_empty() {
                eprintln!("Suggestions:");
                for suggestion in suggestions {
                    eprintln!("  - {}", suggestion);
                }
            }
            
            // Check if the error is retryable
            if error.is_retryable() {
                eprintln!("This error might succeed if retried");
            }
        }
    }
}
```

## Basic Configuration

For more control over the conversion process, use the configuration system:

```rust
use markdowndown::{MarkdownDown, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = Config::builder()
        .timeout_seconds(60)                    // Longer timeout
        .user_agent("MyApp/1.0")               // Custom user agent
        .max_retries(3)                        // Retry failed requests
        .include_frontmatter(true)             // Include metadata
        .build();
    
    // Create configured instance
    let md = MarkdownDown::with_config(config);
    
    // Use it
    let result = md.convert_url("https://example.com").await?;
    println!("Configured conversion: {} chars", result.as_str().len());
    
    Ok(())
}
```

## Supported URL Types

markdowndown supports four main URL types:

### 1. HTML Pages (Default)
```rust
// Any regular web page
convert_url("https://blog.rust-lang.org/2024/01/15/Rust-1.75.0.html").await?;
```

### 2. Google Docs
```rust
// Google Docs sharing URLs
convert_url("https://docs.google.com/document/d/1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs/edit").await?;
```

### 3. GitHub Issues
```rust
// Requires GitHub token for private repos
let config = Config::builder()
    .github_token("ghp_your_token_here")
    .build();
let md = MarkdownDown::with_config(config);
md.convert_url("https://github.com/rust-lang/rust/issues/100000").await?;
```

### 4. Office 365 Documents
```rust
// Office 365 documents (requires authentication for most cases)
convert_url("https://company.sharepoint.com/sites/team/Document.docx").await?;
```

## Environment Configuration

For convenience, you can configure markdowndown using environment variables:

```bash
# Set environment variables
export GITHUB_TOKEN=ghp_your_github_token_here
export MARKDOWNDOWN_TIMEOUT=60
export MARKDOWNDOWN_USER_AGENT="MyApp/1.0"
export MARKDOWNDOWN_MAX_RETRIES=5
```

Then use them in your code:

```rust
use markdowndown::{MarkdownDown, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = Config::from_env();
    let md = MarkdownDown::with_config(config);
    
    // This will use the environment-configured settings
    let result = md.convert_url("https://github.com/rust-lang/rust/issues/1").await?;
    println!("Environment-configured conversion successful!");
    
    Ok(())
}
```

## Performance Considerations

For best performance:

1. **Reuse MarkdownDown instances** when possible:
   ```rust
   let md = MarkdownDown::new();
   for url in urls {
       let result = md.convert_url(url).await?;
       // Process result...
   }
   ```

2. **Use appropriate timeouts** for your use case:
   ```rust
   let config = Config::builder()
       .timeout_seconds(30)  // Adjust based on expected response times
       .build();
   ```

3. **Consider parallel processing** for multiple URLs:
   ```rust
   let futures: Vec<_> = urls.iter()
       .map(|url| convert_url(url))
       .collect();
   let results = futures::future::join_all(futures).await;
   ```

## Common Patterns

### Converting Multiple URLs
```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://example.com/page1",
        "https://example.com/page2",
        "https://example.com/page3",
    ];
    
    for url in urls {
        match convert_url(url).await {
            Ok(markdown) => {
                println!("✅ {}: {} chars", url, markdown.as_str().len());
            }
            Err(e) => {
                eprintln!("❌ {}: {}", url, e);
            }
        }
    }
    
    Ok(())
}
```

### Saving Results to Files
```rust
use markdowndown::convert_url;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let markdown = convert_url("https://example.com").await?;
    
    // Save to file
    let mut file = File::create("output.md")?;
    file.write_all(markdown.as_str().as_bytes())?;
    
    println!("Saved to output.md");
    Ok(())
}
```

## Next Steps

Now that you have the basics working:

1. **Read the [Configuration Guide](configuration.md)** to learn about all available options
2. **Check the [URL Types Guide](url-types.md)** for specific details about each URL type
3. **Review the [Error Handling Guide](error-handling.md)** for robust error handling patterns
4. **Explore the [examples/](../examples/)** directory for more comprehensive examples
5. **Check the [API Reference](https://docs.rs/markdowndown)** for complete documentation

## Getting Help

If you run into issues:

1. Check the [Troubleshooting Guide](troubleshooting.md)
2. Review the [examples/](../examples/) for similar use cases
3. Open an issue on [GitHub](https://github.com/wballard/markdowndown/issues)

Happy converting! 🚀