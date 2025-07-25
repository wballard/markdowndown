# URL Types Guide

markdowndown supports four main URL types, each with specialized converters optimized for their specific content format and access patterns. This guide covers each type in detail.

## Overview

markdowndown automatically detects URL types and routes them to appropriate converters:

| URL Type | Detection Pattern | Special Features |
|----------|------------------|------------------|
| **HTML** | Any HTTP/HTTPS URL | Clean HTML-to-markdown conversion |
| **Google Docs** | `docs.google.com/document/` | Direct export API access |
| **Office 365** | `sharepoint.com`, `onedrive.com` | Document download and conversion |
| **GitHub Issues** | `github.com/.../issues/` or `.../pull/` | API-based content extraction |

## HTML Pages

The most common URL type, supporting any web page with HTML content.

### Supported URLs

```rust
// Blog posts and articles
"https://blog.rust-lang.org/2024/01/15/Rust-1.75.0.html"

// Documentation pages  
"https://doc.rust-lang.org/book/ch01-00-getting-started.html"

// News articles
"https://example.com/news/article-title"

// Any HTML page
"https://httpbin.org/html"
```

### Features

- **Content Extraction**: Identifies main content, removes navigation/ads
- **Clean Conversion**: Converts HTML elements to proper markdown
- **Link Preservation**: Maintains all links and references
- **Image Handling**: Converts image tags to markdown format
- **Table Support**: Converts HTML tables to markdown tables

### Configuration

```rust
use markdowndown::{MarkdownDown, Config};
use markdowndown::converters::html::HtmlConverterConfig;

let html_config = HtmlConverterConfig::default(); // Customize as needed

let config = Config::builder()
    .html_config(html_config)
    .timeout_seconds(30)        // Standard timeout for HTML
    .normalize_whitespace(true) // Clean up HTML whitespace
    .build();

let md = MarkdownDown::with_config(config);
```

### Example Usage

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_url = "https://blog.rust-lang.org/2024/01/15/Rust-1.75.0.html";
    let markdown = convert_url(html_url).await?;
    
    println!("Converted HTML page:");
    println!("Length: {} characters", markdown.as_str().len());
    println!("Lines: {}", markdown.as_str().lines().count());
    
    // Extract content without frontmatter
    let content = markdown.content_only();
    println!("Content preview: {}", &content[..200.min(content.len())]);
    
    Ok(())
}
```

### HTML-Specific Considerations

- **JavaScript-heavy sites**: May not render properly (content loaded by JS)
- **Authentication**: Some sites require cookies or login
- **Rate limiting**: Respect robots.txt and implement delays
- **Content quality**: Varies significantly between sites

## Google Docs

Google Docs URLs are detected and converted using specialized export endpoints.

### Supported URLs

```rust
// Standard sharing URLs
"https://docs.google.com/document/d/1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs/edit"

// View-only URLs
"https://docs.google.com/document/d/1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs/view"

// Published URLs
"https://docs.google.com/document/d/1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs/pub"
```

### Features

- **Direct Export**: Uses Google's export API for clean conversion
- **Formatting Preservation**: Maintains headers, lists, and styling
- **Comment Extraction**: Can include comments (if accessible)
- **Collaborative Content**: Handles shared documents
- **Version Handling**: Gets current version of the document

### Configuration

```rust
use markdowndown::{MarkdownDown, Config};

let config = Config::builder()
    .google_api_key(std::env::var("GOOGLE_API_KEY").ok()) // Optional for better access
    .timeout_seconds(60)        // Google Docs can be slower
    .max_retries(3)            // Retry on temporary failures
    .user_agent("MyApp/1.0")   // Identify your application
    .build();

let md = MarkdownDown::with_config(config);
```

### Example Usage

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gdocs_url = "https://docs.google.com/document/d/1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs/edit";
    
    match convert_url(gdocs_url).await {
        Ok(markdown) => {
            println!("✅ Google Docs conversion successful");
            println!("Length: {} characters", markdown.as_str().len());
            
            // Check for frontmatter
            if let Some(frontmatter) = markdown.frontmatter() {
                println!("📋 Frontmatter included");
            }
            
            // Show content preview
            let content = markdown.content_only();
            println!("📝 Content preview:");
            for line in content.lines().take(5) {
                println!("   {}", line);
            }
        }
        Err(e) => {
            eprintln!("❌ Google Docs conversion failed: {}", e);
            
            // Check if it's a permission issue
            if e.to_string().contains("403") || e.to_string().contains("permission") {
                eprintln!("💡 Document may be private - check sharing settings");
            }
        }
    }
    
    Ok(())
}
```

### Google Docs Considerations

- **Permissions**: Document must be publicly accessible or shared with "Anyone with the link"
- **Private Documents**: Require authentication (Google API key)
- **Large Documents**: May take longer to export
- **Complex Formatting**: Some advanced formatting may not convert perfectly
- **Rate Limits**: Google may rate limit requests

### Troubleshooting Google Docs

1. **Permission Denied (403):**
   ```
   Solution: Make document public or "Anyone with the link can view"
   ```

2. **Document Not Found (404):**
   ```
   Solution: Check URL format and document ID
   ```

3. **Slow Conversion:**
   ```
   Solution: Increase timeout for large documents
   ```

## Office 365 Documents

Office 365 documents from SharePoint and OneDrive are supported with specialized handling.

### Supported URLs

```rust
// SharePoint documents
"https://company.sharepoint.com/sites/team/Shared%20Documents/Document.docx"

// OneDrive documents  
"https://company-my.sharepoint.com/personal/user_company_com/Documents/Document.docx"

// Office Online documents
"https://company.sharepoint.com/:w:/r/sites/team/_layouts/15/Doc.aspx?sourcedoc={id}"
```

### Features

- **Document Download**: Downloads Office documents for conversion
- **Format Support**: Handles .docx, .xlsx, .pptx files
- **Authentication**: Supports Office 365 authentication
- **Metadata Extraction**: Preserves document properties
- **Version Handling**: Gets current version

### Configuration

```rust
use markdowndown::{MarkdownDown, Config};

let config = Config::builder()
    .office365_token(std::env::var("OFFICE365_TOKEN").ok()) // Required for private docs
    .timeout_seconds(180)       // Office docs can be very slow
    .max_retries(2)            // Retry on download failures
    .user_agent("DocumentProcessor/1.0")
    .build();

let md = MarkdownDown::with_config(config);
```

### Example Usage

```rust
use markdowndown::convert_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let office_url = "https://company.sharepoint.com/sites/team/Document.docx";
    
    // Note: This may require authentication for private documents
    match convert_url(office_url).await {
        Ok(markdown) => {
            println!("✅ Office 365 conversion successful");
            println!("Document length: {} characters", markdown.as_str().len());
            
            // Office documents often have rich metadata
            if let Some(frontmatter) = markdown.frontmatter() {
                println!("📊 Document metadata available");
                println!("{}", frontmatter);
            }
        }
        Err(e) => {
            eprintln!("❌ Office 365 conversion failed: {}", e);
            
            if e.to_string().contains("401") || e.to_string().contains("authentication") {
                eprintln!("🔐 Authentication required - set OFFICE365_TOKEN environment variable");
            }
        }
    }
    
    Ok(())
}
```

### Office 365 Considerations

- **Authentication**: Most corporate documents require authentication
- **File Size**: Large documents take significantly longer
- **Network Latency**: SharePoint can be slow depending on location
- **Document Types**: .docx works best, .xlsx and .pptx have limited support
- **Corporate Policies**: IT policies may block programmatic access

## GitHub Issues and Pull Requests

GitHub Issues and Pull Requests are converted using the GitHub API for rich content extraction.

### Supported URLs

```rust
// Public repository issues
"https://github.com/rust-lang/rust/issues/100000"

// Public repository pull requests
"https://github.com/microsoft/vscode/pull/12345"

// Private repository issues (requires token)
"https://github.com/company/private-repo/issues/42"
```

### Features

- **Complete Issue Content**: Title, body, labels, assignees
- **Comments Extraction**: All comments with authors and timestamps
- **Rich Formatting**: Preserves code blocks, mentions, references
- **Metadata**: Issue state, creation date, update date
- **API-Based**: Uses GitHub API for reliable access

### Configuration

```rust
use markdowndown::{MarkdownDown, Config};

let config = Config::builder()
    .github_token(std::env::var("GITHUB_TOKEN").unwrap()) // Required for private repos
    .timeout_seconds(60)        // API calls can be slower
    .max_retries(3)            // Retry on API failures
    .user_agent("MyApp/1.0")   // Required by GitHub API
    .build();

let md = MarkdownDown::with_config(config);
```

### GitHub Token Setup

1. **Generate Token:**
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Click "Generate new token (classic)"
   - Select scopes:
     - `repo` for private repositories
     - `public_repo` for public repositories only

2. **Set Environment Variable:**
   ```bash
   export GITHUB_TOKEN=ghp_your_personal_access_token_here
   ```

### Example Usage

```rust
use markdowndown::{MarkdownDown, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure with GitHub token
    let config = Config::builder()
        .github_token(std::env::var("GITHUB_TOKEN")?)
        .build();
    
    let md = MarkdownDown::with_config(config);
    
    // Convert GitHub issue
    let github_url = "https://github.com/rust-lang/rust/issues/100000";
    
    match md.convert_url(github_url).await {
        Ok(markdown) => {
            println!("✅ GitHub issue conversion successful");
            
            // GitHub issues have rich frontmatter
            if let Some(frontmatter) = markdown.frontmatter() {
                println!("📋 Issue metadata:");
                println!("{}", frontmatter);
            }
            
            // Show content structure
            let content = markdown.content_only();
            let lines: Vec<&str> = content.lines().collect();
            println!("📝 Content structure:");
            println!("   Total lines: {}", lines.len());
            
            // Look for common GitHub issue sections
            for (i, line) in lines.iter().enumerate() {
                if line.starts_with("# ") || line.starts_with("## ") {
                    println!("   Header at line {}: {}", i + 1, line);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ GitHub conversion failed: {}", e);
            
            if e.to_string().contains("401") {
                eprintln!("🔐 Authentication failed - check GITHUB_TOKEN");
            } else if e.to_string().contains("404") {
                eprintln!("📭 Issue not found - check URL and repository access");
            } else if e.to_string().contains("403") {
                eprintln!("🚫 Rate limited or insufficient permissions");
            }
        }
    }
    
    Ok(())
}
```

### GitHub-Specific Features

#### Comment Extraction
```rust
// Comments are included in the markdown output
// Each comment is formatted with author and timestamp
```

#### Rich Metadata
```yaml
---
source_url: "https://github.com/rust-lang/rust/issues/100000"
issue_number: 100000
repository: "rust-lang/rust"  
title: "Issue Title"
state: "open"
author: "username"
created_at: "2024-01-15T10:30:00Z"
updated_at: "2024-01-16T15:45:00Z"
labels: ["bug", "P-high"]
assignees: ["assignee1", "assignee2"]
comments_count: 5
---
```

### GitHub Considerations

- **Rate Limits**: 
  - Authenticated: 5,000 requests/hour
  - Unauthenticated: 60 requests/hour
- **Private Repositories**: Require appropriate token scopes
- **Large Issues**: Issues with many comments take longer
- **API Changes**: GitHub API may change, affecting functionality

### Rate Limit Handling

```rust
use markdowndown::{MarkdownDown, Config, types::MarkdownError};

async fn convert_github_with_rate_limit_handling(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .github_token(std::env::var("GITHUB_TOKEN")?)
        .max_retries(3)
        .build();
    
    let md = MarkdownDown::with_config(config);
    
    match md.convert_url(url).await {
        Ok(markdown) => Ok(markdown.as_str().to_string()),
        Err(MarkdownError::EnhancedNetworkError { kind, context }) => {
            if let markdowndown::types::NetworkErrorKind::RateLimited = kind {
                eprintln!("⏳ Rate limited, waiting before retry...");
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                
                // Retry once after rate limit
                let retry_result = md.convert_url(url).await?;
                Ok(retry_result.as_str().to_string())
            } else {
                Err(Box::new(MarkdownError::EnhancedNetworkError { kind, context }))
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}
```

## URL Type Detection

### Automatic Detection

markdowndown automatically detects URL types:

```rust
use markdowndown::{convert_url, detect_url_type};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://example.com/article",
        "https://docs.google.com/document/d/abc123/edit", 
        "https://github.com/owner/repo/issues/123",
        "https://company.sharepoint.com/Document.docx",
    ];
    
    for url in urls {
        // Detect type first
        let url_type = detect_url_type(url)?;
        println!("🔍 {}: {}", url, url_type);
        
        // Then convert
        let markdown = convert_url(url).await?;
        println!("   ✅ Converted: {} chars\n", markdown.as_str().len());
    }
    
    Ok(())
}
```

### Manual Type Handling

```rust
use markdowndown::{MarkdownDown, Config, detect_url_type, types::UrlType};

async fn convert_with_type_specific_config(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url_type = detect_url_type(url)?;
    
    let config = match url_type {
        UrlType::Html => {
            Config::builder()
                .timeout_seconds(30)
                .build()
        }
        UrlType::GoogleDocs => {
            Config::builder()
                .timeout_seconds(120)
                .max_retries(2)
                .build()
        }
        UrlType::GitHubIssue => {
            Config::builder()
                .github_token(std::env::var("GITHUB_TOKEN").ok())
                .timeout_seconds(60)
                .build()
        }
        UrlType::Office365 => {
            Config::builder()
                .office365_token(std::env::var("OFFICE365_TOKEN").ok())
                .timeout_seconds(180)
                .build()
        }
    };
    
    let md = MarkdownDown::with_config(config);
    let result = md.convert_url(url).await?;
    Ok(result.as_str().to_string())
}
```

## Best Practices by URL Type

### HTML Pages
- Use reasonable timeouts (30-60 seconds)
- Implement retry logic for unreliable sites
- Check robots.txt before bulk processing
- Handle JavaScript-heavy sites gracefully

### Google Docs
- Make documents publicly accessible when possible
- Use longer timeouts for large documents
- Implement authentication for private documents
- Cache results when appropriate

### GitHub Issues
- Always use authentication tokens
- Handle rate limits gracefully
- Be aware of repository access permissions
- Consider pagination for issues with many comments

### Office 365
- Expect longer processing times
- Implement proper authentication
- Handle corporate firewall restrictions
- Test with different document types

## Performance Comparison

| URL Type | Typical Speed | Factors Affecting Speed |
|----------|---------------|------------------------|
| **HTML** | 1-5 seconds | Page size, server speed, complexity |
| **Google Docs** | 2-10 seconds | Document size, formatting complexity |
| **GitHub Issues** | 1-8 seconds | Comment count, API rate limits |
| **Office 365** | 5-60 seconds | Document size, network latency |

## Error Handling by Type

Each URL type has specific error patterns:

```rust
use markdowndown::{convert_url, types::MarkdownError};

async fn handle_type_specific_errors(url: &str) {
    match convert_url(url).await {
        Ok(markdown) => println!("✅ Success: {}", markdown.as_str().len()),
        Err(e) => {
            match &e {
                MarkdownError::AuthenticationError { .. } => {
                    eprintln!("🔐 Authentication issue - check tokens");
                }
                MarkdownError::EnhancedNetworkError { kind, .. } => {
                    use markdowndown::types::NetworkErrorKind;
                    match kind {
                        NetworkErrorKind::RateLimited => {
                            eprintln!("🐌 Rate limited - wait before retrying");
                        }
                        NetworkErrorKind::ServerError(404) => {
                            eprintln!("📭 Resource not found - check URL");
                        }
                        _ => {
                            eprintln!("🌐 Network error: {:?}", kind);
                        }
                    }
                }
                _ => {
                    eprintln!("❌ Other error: {}", e);
                }
            }
        }
    }
}
```

## Next Steps

- Review the [Configuration Guide](configuration.md) for detailed configuration options
- Check the [Error Handling Guide](error-handling.md) for robust error handling
- See the [Performance Guide](performance.md) for optimization tips
- Explore [examples/](../examples/) for practical usage examples