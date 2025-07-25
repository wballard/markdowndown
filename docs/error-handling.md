# Error Handling Guide

markdowndown provides comprehensive error handling with detailed error types, context information, and recovery strategies. This guide covers all error types and best practices for robust error handling.

## Error System Overview

markdowndown uses two error handling systems:

1. **Enhanced Error System** - Modern errors with rich context and categorization
2. **Legacy Error System** - Backward-compatible simple errors

### Enhanced Error Types

```rust
pub enum MarkdownError {
    ValidationError { kind: ValidationErrorKind, context: ErrorContext },
    EnhancedNetworkError { kind: NetworkErrorKind, context: ErrorContext },
    AuthenticationError { kind: AuthErrorKind, context: ErrorContext },
    ContentError { kind: ContentErrorKind, context: ErrorContext },
    ConverterError { kind: ConverterErrorKind, context: ErrorContext },
    ConfigurationError { kind: ConfigErrorKind, context: ErrorContext },
    
    // Legacy error types (for backward compatibility)
    NetworkError { message: String },
    ParseError { message: String },
    InvalidUrl { url: String },
    AuthError { message: String },
    LegacyConfigurationError { message: String },
}
```

## Error Context

Enhanced errors include rich context information:

```rust
pub struct ErrorContext {
    pub url: String,              // URL being processed
    pub operation: String,        // Operation being performed
    pub converter_type: String,   // Converter being used
    pub timestamp: DateTime<Utc>, // When the error occurred
    pub additional_info: Option<String>, // Extra context
}
```

## Validation Errors

Input validation failures with specific error kinds.

### ValidationErrorKind Types

```rust
pub enum ValidationErrorKind {
    InvalidUrl,        // Malformed URLs
    InvalidFormat,     // Wrong input format
    MissingParameter,  // Required parameter missing
}
```

### Example Handling

```rust
use markdowndown::{convert_url, types::{MarkdownError, ValidationErrorKind}};

#[tokio::main]
async fn main() {
    match convert_url("not-a-valid-url").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(MarkdownError::ValidationError { kind, context }) => {
            match kind {
                ValidationErrorKind::InvalidUrl => {
                    eprintln!("❌ Invalid URL: {}", context.url);
                    eprintln!("💡 Ensure URL starts with http:// or https://");
                }
                ValidationErrorKind::InvalidFormat => {
                    eprintln!("❌ Invalid format for operation: {}", context.operation);
                }
                ValidationErrorKind::MissingParameter => {
                    eprintln!("❌ Missing required parameter for: {}", context.operation);
                }
            }
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

### Common Validation Errors

1. **Invalid URL Format:**
   ```rust
   // ❌ These will cause ValidationError::InvalidUrl
   convert_url("example.com").await;           // Missing protocol
   convert_url("ftp://example.com").await;     // Wrong protocol
   convert_url("").await;                      // Empty string
   ```

2. **Missing Parameters:**
   ```rust
   // ❌ These might cause ValidationError::MissingParameter  
   // (in specific converter contexts)
   ```

## Network Errors

Network-related failures with detailed categorization.

### NetworkErrorKind Types

```rust
pub enum NetworkErrorKind {
    Timeout,                    // Request timed out
    ConnectionFailed,          // Could not establish connection
    DnsResolution,            // DNS lookup failed
    RateLimited,              // Too many requests (HTTP 429)
    ServerError(u16),         // Server errors (HTTP status codes)
}
```

### Example Handling

```rust
use markdowndown::{convert_url, types::{MarkdownError, NetworkErrorKind}};

#[tokio::main]
async fn main() {
    match convert_url("https://httpbin.org/status/503").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(MarkdownError::EnhancedNetworkError { kind, context }) => {
            match kind {
                NetworkErrorKind::Timeout => {
                    eprintln!("⏰ Request timed out for: {}", context.url);
                    eprintln!("💡 Try increasing timeout or check connection");
                }
                NetworkErrorKind::ConnectionFailed => {
                    eprintln!("🔌 Connection failed to: {}", context.url);
                    eprintln!("💡 Check network connectivity and firewall");
                }
                NetworkErrorKind::DnsResolution => {
                    eprintln!("🌐 DNS resolution failed for: {}", context.url);
                    eprintln!("💡 Check domain name and DNS settings");
                }
                NetworkErrorKind::RateLimited => {
                    eprintln!("🐌 Rate limited by: {}", context.url);
                    eprintln!("💡 Wait before retrying or authenticate");
                }
                NetworkErrorKind::ServerError(status) => {
                    eprintln!("🖥️ Server error {} from: {}", status, context.url);
                    match status {
                        500..=503 => eprintln!("💡 Server issue - retry later"),
                        404 => eprintln!("💡 Resource not found"),
                        401 => eprintln!("💡 Authentication required"),
                        403 => eprintln!("💡 Access forbidden"),
                        _ => eprintln!("💡 Check server documentation"),
                    }
                }
            }
            eprintln!("🕐 Error occurred at: {}", context.timestamp);
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

### Network Error Recovery

```rust
use markdowndown::{convert_url, types::{MarkdownError, NetworkErrorKind}};
use std::time::Duration;

async fn convert_with_network_retry(url: &str, max_attempts: usize) -> Result<String, MarkdownError> {
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match convert_url(url).await {
            Ok(markdown) => return Ok(markdown.as_str().to_string()),
            Err(e) => {
                match &e {
                    MarkdownError::EnhancedNetworkError { kind, .. } => {
                        let should_retry = match kind {
                            NetworkErrorKind::Timeout => true,
                            NetworkErrorKind::ConnectionFailed => true, 
                            NetworkErrorKind::RateLimited => true,
                            NetworkErrorKind::ServerError(status) => *status >= 500,
                            NetworkErrorKind::DnsResolution => false, // Don't retry DNS failures
                        };
                        
                        if should_retry && attempt < max_attempts {
                            let delay = Duration::from_millis(1000 * 2_u64.pow(attempt as u32 - 1));
                            println!("🔄 Attempt {} failed, retrying in {:?}...", attempt, delay);
                            tokio::time::sleep(delay).await;
                            last_error = Some(e);
                            continue;
                        }
                    }
                    _ => {} // Don't retry non-network errors
                }
                
                return Err(e);
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

## Authentication Errors

Authentication and authorization failures.

### AuthErrorKind Types

```rust
pub enum AuthErrorKind {
    MissingToken,      // No authentication token provided
    InvalidToken,      // Token format is invalid
    PermissionDenied,  // Token lacks required permissions
    TokenExpired,      // Token has expired
}
```

### Example Handling

```rust
use markdowndown::{MarkdownDown, Config, types::{MarkdownError, AuthErrorKind}};

#[tokio::main]
async fn main() {
    let config = Config::builder()
        .github_token("invalid_token_format") // This will cause auth errors
        .build();
    
    let md = MarkdownDown::with_config(config);
    
    match md.convert_url("https://github.com/private/repo/issues/1").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(MarkdownError::AuthenticationError { kind, context }) => {
            match kind {
                AuthErrorKind::MissingToken => {
                    eprintln!("🔑 No authentication token for: {}", context.url);
                    eprintln!("💡 Set GITHUB_TOKEN environment variable");
                }
                AuthErrorKind::InvalidToken => {
                    eprintln!("❌ Invalid token format for: {}", context.converter_type);
                    eprintln!("💡 Check token format (GitHub tokens start with ghp_)");
                }
                AuthErrorKind::PermissionDenied => {
                    eprintln!("🚫 Permission denied for: {}", context.url);
                    eprintln!("💡 Check token scopes and repository access");
                }
                AuthErrorKind::TokenExpired => {
                    eprintln!("⏰ Token expired for: {}", context.converter_type);
                    eprintln!("💡 Generate a new authentication token");
                }
            }
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

### Authentication Recovery

```rust
use markdowndown::{MarkdownDown, Config, types::{MarkdownError, AuthErrorKind}};

async fn convert_with_auth_fallback(url: &str) -> Result<String, MarkdownError> {
    // Try with environment token first
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let config = Config::builder()
            .github_token(token)
            .build();
        
        let md = MarkdownDown::with_config(config);
        
        match md.convert_url(url).await {
            Ok(markdown) => return Ok(markdown.as_str().to_string()),
            Err(MarkdownError::AuthenticationError { kind, .. }) => {
                match kind {
                    AuthErrorKind::MissingToken | AuthErrorKind::InvalidToken => {
                        eprintln!("🔄 Auth failed, trying without authentication...");
                        // Fall through to unauthenticated attempt
                    }
                    AuthErrorKind::PermissionDenied | AuthErrorKind::TokenExpired => {
                        eprintln!("🔄 Auth issue, trying without authentication...");
                        // Fall through to unauthenticated attempt  
                    }
                }
            }
            Err(e) => return Err(e), // Non-auth error, don't retry
        }
    }
    
    // Try without authentication
    let md = MarkdownDown::new();
    let result = md.convert_url(url).await?;
    Ok(result.as_str().to_string())
}
```

## Content Errors

Content processing and parsing failures.

### ContentErrorKind Types

```rust
pub enum ContentErrorKind {
    EmptyContent,       // No content found
    UnsupportedFormat, // Content format not supported
    ParsingFailed,     // Content parsing failed
}
```

### Example Handling

```rust
use markdowndown::{convert_url, types::{MarkdownError, ContentErrorKind}};

#[tokio::main]
async fn main() {
    match convert_url("https://example.com/empty-page").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(MarkdownError::ContentError { kind, context }) => {
            match kind {
                ContentErrorKind::EmptyContent => {
                    eprintln!("📄 No content found at: {}", context.url);
                    eprintln!("💡 Check if URL contains actual content");
                }
                ContentErrorKind::UnsupportedFormat => {
                    eprintln!("📝 Unsupported format for: {}", context.converter_type);
                    eprintln!("💡 Try a different converter or check content type");
                }
                ContentErrorKind::ParsingFailed => {
                    eprintln!("🔧 Parsing failed for: {}", context.url);
                    eprintln!("💡 Content may be corrupted or malformed");
                }
            }
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## Converter Errors

External tool and processing failures.

### ConverterErrorKind Types

```rust
pub enum ConverterErrorKind {
    ExternalToolFailed,    // External dependency failed
    ProcessingError,       // Internal processing error
    UnsupportedOperation,  // Operation not supported
}
```

### Example Handling

```rust
use markdowndown::{convert_url, types::{MarkdownError, ConverterErrorKind}};

#[tokio::main]
async fn main() {
    match convert_url("https://complex-document.example.com").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(MarkdownError::ConverterError { kind, context }) => {
            match kind {
                ConverterErrorKind::ExternalToolFailed => {
                    eprintln!("🔨 External tool failed for: {}", context.converter_type);
                    eprintln!("💡 Check dependencies and PATH configuration");
                }
                ConverterErrorKind::ProcessingError => {
                    eprintln!("⚙️ Processing error in: {}", context.converter_type);
                    eprintln!("💡 Try different converter settings");
                }
                ConverterErrorKind::UnsupportedOperation => {
                    eprintln!("🚫 Unsupported operation for: {}", context.converter_type);
                    eprintln!("💡 Use different converter or approach");
                }
            }
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## Configuration Errors

Setup and configuration failures.

### ConfigErrorKind Types

```rust
pub enum ConfigErrorKind {
    InvalidConfig,      // Configuration is invalid
    MissingDependency, // Required dependency missing
    InvalidValue,      // Configuration value invalid
}
```

### Example Handling

```rust
use markdowndown::{MarkdownDown, Config, types::{MarkdownError, ConfigErrorKind}};

fn create_configured_instance() -> Result<MarkdownDown, MarkdownError> {
    let config = Config::builder()
        .timeout_seconds(0) // This might cause InvalidValue error
        .build();
    
    Ok(MarkdownDown::with_config(config))
}

#[tokio::main]
async fn main() {
    match create_configured_instance() {
        Ok(md) => {
            // Use the configured instance
            match md.convert_url("https://example.com").await {
                Ok(markdown) => println!("Success: {}", markdown),
                Err(e) => eprintln!("Conversion error: {}", e),
            }
        }
        Err(MarkdownError::ConfigurationError { kind, context }) => {
            match kind {
                ConfigErrorKind::InvalidConfig => {
                    eprintln!("⚙️ Invalid configuration: {}", context.operation);
                    eprintln!("💡 Check configuration file syntax");
                }
                ConfigErrorKind::MissingDependency => {
                    eprintln!("📦 Missing dependency for: {}", context.converter_type);
                    eprintln!("💡 Install required dependencies");
                }
                ConfigErrorKind::InvalidValue => {
                    eprintln!("❌ Invalid configuration value: {}", context.operation);
                    eprintln!("💡 Check valid ranges and formats");
                }
            }
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
}
```

## Error Characteristics

markdowndown errors have useful characteristics for handling:

### Retryable Errors

```rust
use markdowndown::{convert_url, types::MarkdownError};

#[tokio::main]
async fn main() {
    match convert_url("https://unreliable-server.example.com").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(error) => {
            if error.is_retryable() {
                println!("🔄 Error is retryable, implementing retry logic...");
                // Implement retry with backoff
            } else {
                println!("🛑 Error is not retryable, failing permanently");
            }
        }
    }
}
```

### Recoverable Errors

```rust
use markdowndown::{convert_url, types::MarkdownError};

#[tokio::main]
async fn main() {
    match convert_url("https://difficult-site.example.com").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(error) => {
            if error.is_recoverable() {
                println!("🔄 Error is recoverable, trying fallback strategies...");
                // Try alternative approaches
            } else {
                println!("💀 Error is not recoverable, permanent failure");
            }
        }
    }
}
```

### Error Suggestions

```rust
use markdowndown::{convert_url, types::MarkdownError};

#[tokio::main]
async fn main() {
    match convert_url("invalid-url").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(error) => {
            eprintln!("❌ Error: {}", error);
            
            let suggestions = error.suggestions();
            if !suggestions.is_empty() {
                eprintln!("💡 Suggestions:");
                for suggestion in suggestions {
                    eprintln!("   - {}", suggestion);
                }
            }
        }
    }
}
```

## Error Context Usage

Access rich error context for debugging:

```rust
use markdowndown::{convert_url, types::MarkdownError};

#[tokio::main]
async fn main() {
    match convert_url("https://problematic-site.example.com").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(error) => {
            eprintln!("❌ Error: {}", error);
            
            if let Some(context) = error.context() {
                eprintln!("🔍 Context:");
                eprintln!("   URL: {}", context.url);
                eprintln!("   Operation: {}", context.operation);
                eprintln!("   Converter: {}", context.converter_type);
                eprintln!("   Timestamp: {}", context.timestamp);
                
                if let Some(additional_info) = &context.additional_info {
                    eprintln!("   Additional: {}", additional_info);
                }
            }
        }
    }
}
```

## Legacy Error Handling

For backward compatibility, legacy errors are still supported:

```rust
use markdowndown::{convert_url, types::MarkdownError};

#[tokio::main]
async fn main() {
    match convert_url("https://example.com").await {
        Ok(markdown) => println!("Success: {}", markdown),
        Err(error) => {
            match error {
                // Enhanced errors (preferred)
                MarkdownError::ValidationError { kind, context } => {
                    eprintln!("Enhanced validation error: {:?}", kind);
                }
                MarkdownError::EnhancedNetworkError { kind, context } => {
                    eprintln!("Enhanced network error: {:?}", kind);
                }
                
                // Legacy errors (backward compatibility)
                MarkdownError::NetworkError { message } => {
                    eprintln!("Legacy network error: {}", message);
                }
                MarkdownError::ParseError { message } => {
                    eprintln!("Legacy parse error: {}", message);
                }
                MarkdownError::InvalidUrl { url } => {
                    eprintln!("Legacy invalid URL: {}", url);
                }
                MarkdownError::AuthError { message } => {
                    eprintln!("Legacy auth error: {}", message);
                }
                MarkdownError::LegacyConfigurationError { message } => {
                    eprintln!("Legacy config error: {}", message);
                }
                
                _ => {
                    eprintln!("Other error: {}", error);
                }
            }
        }
    }
}
```

## Comprehensive Error Handling Pattern

Here's a complete error handling pattern:

```rust
use markdowndown::{convert_url, types::MarkdownError};
use std::time::Duration;

async fn robust_convert_url(url: &str) -> Result<String, String> {
    let max_attempts = 3;
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match convert_url(url).await {
            Ok(markdown) => {
                return Ok(markdown.as_str().to_string());
            }
            Err(error) => {
                // Log error with context
                eprintln!("Attempt {}/{} failed: {}", attempt, max_attempts, error);
                
                if let Some(context) = error.context() {
                    eprintln!("Context: {} in {}", context.operation, context.converter_type);
                }
                
                // Check if we should retry
                let should_retry = attempt < max_attempts && 
                    (error.is_retryable() || error.is_recoverable());
                
                if should_retry {
                    // Show suggestions on last retry
                    if attempt == max_attempts - 1 {
                        let suggestions = error.suggestions();
                        if !suggestions.is_empty() {
                            eprintln!("Suggestions for next attempt:");
                            for suggestion in suggestions.iter().take(2) {
                                eprintln!("  - {}", suggestion);
                            }
                        }
                    }
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(1000 * 2_u64.pow(attempt as u32 - 1));
                    eprintln!("Retrying in {:?}...", delay);
                    tokio::time::sleep(delay).await;
                    
                    last_error = Some(error);
                    continue;
                } else {
                    return Err(format!("Permanent failure: {}", error));
                }
            }
        }
    }
    
    Err(format!("Failed after {} attempts: {}", max_attempts, 
        last_error.unwrap()))
}

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://example.com",
        "https://invalid-domain.nonexistent",
        "https://httpbin.org/status/503",
    ];
    
    for url in urls {
        println!("🔄 Processing: {}", url);
        match robust_convert_url(url).await {
            Ok(content) => {
                println!("✅ Success: {} characters", content.len());
            }
            Err(error) => {
                println!("❌ Failed: {}", error);
            }
        }
        println!();
    }
}
```

## Error Monitoring and Alerting

For production applications, implement error monitoring:

```rust
use markdowndown::{convert_url, types::MarkdownError};
use std::collections::HashMap;

struct ErrorMetrics {
    total_requests: u64,
    error_count: u64,
    error_types: HashMap<String, u64>,
}

impl ErrorMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            error_count: 0,
            error_types: HashMap::new(),
        }
    }
    
    fn record_request(&mut self) {
        self.total_requests += 1;
    }
    
    fn record_error(&mut self, error: &MarkdownError) {
        self.error_count += 1;
        
        let error_type = match error {
            MarkdownError::ValidationError { .. } => "validation",
            MarkdownError::EnhancedNetworkError { .. } => "network",
            MarkdownError::AuthenticationError { .. } => "authentication",
            MarkdownError::ContentError { .. } => "content",
            MarkdownError::ConverterError { .. } => "converter",
            MarkdownError::ConfigurationError { .. } => "configuration",
            _ => "legacy",
        };
        
        *self.error_types.entry(error_type.to_string()).or_insert(0) += 1;
    }
    
    fn should_alert(&self) -> bool {
        let error_rate = self.error_count as f64 / self.total_requests as f64;
        error_rate > 0.1 // Alert if error rate > 10%
    }
    
    fn report(&self) {
        println!("📊 Error Metrics:");
        println!("   Total requests: {}", self.total_requests);
        println!("   Total errors: {}", self.error_count);
        println!("   Error rate: {:.2}%", 
            (self.error_count as f64 / self.total_requests as f64) * 100.0);
        
        for (error_type, count) in &self.error_types {
            println!("   {}: {}", error_type, count);
        }
        
        if self.should_alert() {
            println!("🚨 HIGH ERROR RATE - Investigation needed!");
        }
    }
}

async fn monitored_convert_url(url: &str, metrics: &mut ErrorMetrics) -> Result<String, MarkdownError> {
    metrics.record_request();
    
    match convert_url(url).await {
        Ok(markdown) => Ok(markdown.as_str().to_string()),
        Err(error) => {
            metrics.record_error(&error);
            
            // Log detailed error information
            eprintln!("🚨 Error processing {}: {}", url, error);
            
            if let Some(context) = error.context() {
                eprintln!("   Context: {} at {}", context.operation, context.timestamp);
            }
            
            Err(error)
        }
    }
}
```

## Best Practices

1. **Always Handle Errors Explicitly**: Don't use `.unwrap()` in production code
2. **Use Error Characteristics**: Check `is_retryable()` and `is_recoverable()`
3. **Implement Retry Logic**: Use exponential backoff for retryable errors
4. **Log Error Context**: Use the context information for debugging
5. **Show User-Friendly Messages**: Use error suggestions for user guidance
6. **Monitor Error Patterns**: Track error types and rates in production
7. **Handle Legacy Errors**: Support both enhanced and legacy error types
8. **Implement Fallbacks**: Have backup strategies for recoverable errors

## Next Steps

- Review the [Configuration Guide](configuration.md) for error-related configuration
- Check the [Performance Guide](performance.md) for performance-related error handling
- See [examples/error_handling.rs](../examples/error_handling.rs) for practical examples
- Explore the [Troubleshooting Guide](troubleshooting.md) for common issues and solutions