# Configuration Guide

This guide covers all configuration options available in markdowndown, including the builder pattern, environment variables, and advanced configuration scenarios.

## Overview

markdowndown uses a hierarchical configuration system with these components:

- **HttpConfig** - HTTP client settings (timeouts, retries, user agent)
- **AuthConfig** - Authentication tokens for various services
- **HtmlConverterConfig** - HTML conversion options
- **PlaceholderSettings** - Placeholder converter settings
- **OutputConfig** - Output formatting options

## Configuration Methods

### 1. Default Configuration

The simplest way to use markdowndown is with default settings:

```rust
use markdowndown::MarkdownDown;

let md = MarkdownDown::new();
// Uses all default settings
```

Default values:
- Timeout: 30 seconds
- Max retries: 3
- User agent: `markdowndown/0.1.0`
- Include frontmatter: `true`
- No authentication tokens

### 2. Builder Pattern

For custom configuration, use the builder pattern:

```rust
use markdowndown::{MarkdownDown, Config};

let config = Config::builder()
    .timeout_seconds(60)
    .max_retries(5)
    .user_agent("MyApp/1.0")
    .github_token("ghp_your_token_here")
    .include_frontmatter(false)
    .build();

let md = MarkdownDown::with_config(config);
```

### 3. Environment Variables

Load configuration from environment variables:

```rust
use markdowndown::{MarkdownDown, Config};

// Reads from environment variables
let config = Config::from_env();
let md = MarkdownDown::with_config(config);
```

Supported environment variables:
- `GITHUB_TOKEN` - GitHub personal access token
- `MARKDOWNDOWN_TIMEOUT` - HTTP timeout in seconds
- `MARKDOWNDOWN_USER_AGENT` - Custom user agent string
- `MARKDOWNDOWN_MAX_RETRIES` - Maximum retry attempts

## HTTP Configuration

### Timeout Settings

Control how long to wait for responses:

```rust
let config = Config::builder()
    .timeout_seconds(60)        // 60 second timeout
    .timeout(Duration::from_secs(120))  // Alternative: using Duration
    .build();
```

**Recommendations:**
- Standard web pages: 30-60 seconds
- Large documents: 120-300 seconds
- Batch processing: 10-30 seconds per URL

### Retry Configuration

Configure retry behavior for failed requests:

```rust
let config = Config::builder()
    .max_retries(5)                                    // Maximum retry attempts
    .retry_delay(Duration::from_millis(500))          // Base delay between retries
    .build();
```

**Retry Behavior:**
- Uses exponential backoff (delay * 2^attempt)
- Only retries on retryable errors (network timeouts, 5xx errors)
- Never retries on client errors (4xx) or validation errors

### User Agent

Set a custom user agent for HTTP requests:

```rust
let config = Config::builder()
    .user_agent("MyApp/1.0 (https://mysite.com)")
    .build();
```

**Best Practices:**
- Include your app name and version
- Add contact information for rate limiting discussions
- Follow [RFC 7231](https://tools.ietf.org/html/rfc7231#section-5.5.3) format

### Redirect Handling

Control how many redirects to follow:

```rust
let config = Config::builder()
    .max_redirects(10)  // Default is 10
    .build();
```

## Authentication Configuration

### GitHub Token

Required for GitHub private repositories and higher rate limits:

```rust
let config = Config::builder()
    .github_token("ghp_your_personal_access_token")
    .build();
```

**GitHub Token Setup:**
1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate new token with appropriate scopes:
   - `repo` for private repositories
   - `public_repo` for public repositories only
3. Copy the token (starts with `ghp_`)

**Token Scopes:**
- **Public repos only**: `public_repo`
- **Private repos**: `repo`
- **Organizations**: May need `read:org`


### Google API Key

For enhanced Google Docs access (future feature):

```rust
let config = Config::builder()
    .google_api_key("your_google_api_key")
    .build();
```

## Output Configuration

### Frontmatter Settings

Control YAML frontmatter generation:

```rust
let config = Config::builder()
    .include_frontmatter(true)              // Enable/disable frontmatter
    .custom_frontmatter_field("project", "my-project")  // Add custom fields
    .custom_frontmatter_field("version", "1.0.0")
    .custom_frontmatter_field("author", "Your Name")
    .build();
```

**Standard Frontmatter Fields:**
```yaml
---
source_url: "https://example.com/document"
exporter: "markdowndown/0.1.0"
date_downloaded: "2024-01-15T10:30:00Z"
# Plus any custom fields you add
---
```

### Content Formatting

Control markdown output formatting:

```rust
let config = Config::builder()
    .normalize_whitespace(true)            // Clean up whitespace
    .max_consecutive_blank_lines(2)        // Limit blank lines
    .build();
```

**Whitespace Normalization:**
- Removes excessive whitespace
- Standardizes line endings
- Cleans up malformed HTML spacing

## HTML Converter Configuration

Configure HTML-to-markdown conversion:

```rust
use markdowndown::converters::html::HtmlConverterConfig;

let html_config = HtmlConverterConfig::default(); // Customize as needed

let config = Config::builder()
    .html_config(html_config)
    .build();
```

## Placeholder Settings

Configure placeholder converters (for unsupported content):

```rust
let config = Config::builder()
    .placeholder_max_content_length(2000)  // Max chars to include
    .build();
```

## Complete Configuration Example

Here's a comprehensive configuration example:

```rust
use markdowndown::{MarkdownDown, Config};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        // HTTP settings
        .timeout_seconds(90)
        .user_agent("DocumentProcessor/2.0 (support@example.com)")
        .max_retries(3)
        .retry_delay(Duration::from_secs(1))
        .max_redirects(5)
        
        // Authentication
        .github_token(std::env::var("GITHUB_TOKEN").ok())
        
        // Output formatting
        .include_frontmatter(true)
        .custom_frontmatter_field("processor", "document-pipeline")
        .custom_frontmatter_field("version", env!("CARGO_PKG_VERSION"))
        .custom_frontmatter_field("environment", "production")
        .normalize_whitespace(true)
        .max_consecutive_blank_lines(1)
        
        // Content limits
        .placeholder_max_content_length(5000)
        
        .build();
    
    let md = MarkdownDown::with_config(config);
    
    // Test the configuration
    let result = md.convert_url("https://example.com").await?;
    println!("Configured conversion successful: {} chars", result.as_str().len());
    
    Ok(())
}
```

## Environment Variable Configuration

### Setting Environment Variables

**Linux/macOS:**
```bash
export GITHUB_TOKEN=ghp_your_token_here
export MARKDOWNDOWN_TIMEOUT=120
export MARKDOWNDOWN_USER_AGENT="MyApp/2.0"
export MARKDOWNDOWN_MAX_RETRIES=5
```

**Windows:**
```cmd
set GITHUB_TOKEN=ghp_your_token_here
set MARKDOWNDOWN_TIMEOUT=120
set MARKDOWNDOWN_USER_AGENT=MyApp/2.0
set MARKDOWNDOWN_MAX_RETRIES=5
```

**Docker:**
```dockerfile
ENV GITHUB_TOKEN=ghp_your_token_here
ENV MARKDOWNDOWN_TIMEOUT=120
ENV MARKDOWNDOWN_USER_AGENT="MyApp/2.0"
ENV MARKDOWNDOWN_MAX_RETRIES=5
```

### Loading Environment Configuration

```rust
use markdowndown::{MarkdownDown, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load from environment with fallback to defaults
    let config = Config::from_env();
    
    // Inspect loaded configuration
    println!("Timeout: {:?}", config.http.timeout);
    println!("User Agent: {}", config.http.user_agent);
    println!("Max Retries: {}", config.http.max_retries);
    println!("GitHub Token: {}", 
        if config.auth.github_token.is_some() { "configured" } else { "not set" });
    
    let md = MarkdownDown::with_config(config);
    
    // Use configured instance
    let result = md.convert_url("https://github.com/rust-lang/rust/issues/1").await?;
    println!("Environment configuration working!");
    
    Ok(())
}
```

## Configuration Patterns

### Development vs Production

**Development Configuration:**
```rust
let dev_config = Config::builder()
    .timeout_seconds(10)           // Faster feedback
    .max_retries(1)               // Fail fast
    .include_frontmatter(false)   // Cleaner output
    .normalize_whitespace(false)  // Preserve original formatting
    .build();
```

**Production Configuration:**
```rust
let prod_config = Config::builder()
    .timeout_seconds(120)         // Handle slow responses
    .max_retries(5)              // Be resilient
    .include_frontmatter(true)   // Full metadata
    .custom_frontmatter_field("environment", "production")
    .custom_frontmatter_field("service", "document-processor")
    .user_agent("DocProcessor/1.0 (ops@company.com)")
    .build();
```

### URL-Specific Configuration

```rust
use markdowndown::{MarkdownDown, Config, detect_url_type, types::UrlType};

async fn convert_with_appropriate_config(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url_type = detect_url_type(url)?;
    
    let config = match url_type {
        UrlType::GitHubIssue => {
            Config::builder()
                .github_token(std::env::var("GITHUB_TOKEN").ok())
                .timeout_seconds(60)
                .build()
        }
        UrlType::GoogleDocs => {
            Config::builder()
                .timeout_seconds(120)  // Google Docs can be slow
                .max_retries(2)
                .build()
        }
        UrlType::Html => {
            Config::builder()
                .timeout_seconds(30)   // Standard timeout for HTML
                .build()
        }
    };
    
    let md = MarkdownDown::with_config(config);
    let result = md.convert_url(url).await?;
    Ok(result.as_str().to_string())
}
```

### Batch Processing Configuration

```rust
let batch_config = Config::builder()
    .timeout_seconds(15)          // Shorter timeout for batch
    .max_retries(2)              // Fewer retries
    .user_agent("BatchProcessor/1.0")
    .include_frontmatter(true)   // Include metadata for tracking
    .custom_frontmatter_field("batch_id", "batch_001")
    .build();
```

## Configuration Validation

Always validate critical configuration:

```rust
use markdowndown::{MarkdownDown, Config};

fn create_validated_config() -> Result<Config, String> {
    let mut builder = Config::builder();
    
    // Validate GitHub token format
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.starts_with("ghp_") && !token.starts_with("github_pat_") {
            return Err("Invalid GitHub token format".to_string());
        }
        builder = builder.github_token(token);
    }
    
    // Validate timeout range
    let timeout = std::env::var("MARKDOWNDOWN_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()
        .map_err(|_| "Invalid timeout value")?;
    
    if timeout < 5 || timeout > 600 {
        return Err("Timeout must be between 5 and 600 seconds".to_string());
    }
    
    Ok(builder.timeout_seconds(timeout).build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = create_validated_config()
        .map_err(|e| format!("Configuration error: {}", e))?;
    
    let md = MarkdownDown::with_config(config);
    println!("Configuration validated successfully!");
    
    Ok(())
}
```

## Advanced Configuration

### Custom Configuration Struct

For complex applications, create your own configuration layer:

```rust
use markdowndown::{MarkdownDown, Config};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub github_token: Option<String>,
    pub user_agent: String,
    pub output_format: OutputFormat,
    pub batch_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    WithFrontmatter,
    ContentOnly,
    Custom { template: String },
}

impl AppConfig {
    pub fn to_markdowndown_config(&self) -> Config {
        let mut builder = Config::builder()
            .timeout_seconds(self.timeout_seconds)
            .max_retries(self.max_retries)
            .user_agent(&self.user_agent);
        
        if let Some(token) = &self.github_token {
            builder = builder.github_token(token);
        }
        
        match self.output_format {
            OutputFormat::WithFrontmatter => {
                builder = builder.include_frontmatter(true);
            }
            OutputFormat::ContentOnly => {
                builder = builder.include_frontmatter(false);
            }
            OutputFormat::Custom { .. } => {
                // Custom template handling would go here
                builder = builder.include_frontmatter(true);
            }
        }
        
        builder.build()
    }
}
```

## Configuration Best Practices

1. **Use Environment Variables** for sensitive data (tokens, credentials)
2. **Set Appropriate Timeouts** based on expected response times
3. **Configure User Agents** to identify your application
4. **Validate Configuration** before using it
5. **Use Different Configs** for different environments
6. **Document Custom Fields** in frontmatter
7. **Monitor Configuration** in production logs

## Troubleshooting Configuration

### Common Issues

1. **Invalid GitHub Token:**
   ```
   Error: Authentication error: Invalid token
   Solution: Check token format (should start with ghp_ or github_pat_)
   ```

2. **Timeout Too Short:**
   ```
   Error: Network error: Timeout
   Solution: Increase timeout_seconds for large documents
   ```

3. **Environment Variables Not Loading:**
   ```
   Solution: Check variable names and restart application
   ```

### Configuration Debugging

```rust
let config = Config::from_env();

// Debug configuration
println!("Configuration Debug:");
println!("  Timeout: {:?}", config.http.timeout);
println!("  Max Retries: {}", config.http.max_retries);
println!("  User Agent: {}", config.http.user_agent);
println!("  GitHub Token: {}", 
    if config.auth.github_token.is_some() { "SET" } else { "NOT SET" });
println!("  Include Frontmatter: {}", config.output.include_frontmatter);
```

## Next Steps

- Review the [URL Types Guide](url-types.md) for URL-specific configuration
- Check the [Error Handling Guide](error-handling.md) for error-related configuration
- See the [Performance Guide](performance.md) for performance-oriented configuration
- Explore [examples/with_configuration.rs](../examples/with_configuration.rs) for practical examples