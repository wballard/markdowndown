# Troubleshooting Guide

This guide covers common issues, their causes, and solutions when using markdowndown. It includes diagnostic steps, workarounds, and prevention strategies.

## Common Issues

### 1. Authentication Errors

#### GitHub Authentication Failed

**Symptoms:**
```
Error: Authentication error: Invalid token
Error: Authentication error: Permission denied
```

**Causes:**
- Invalid or expired GitHub token
- Insufficient token permissions
- Token not properly set in environment

**Solutions:**

1. **Verify Token Format:**
   ```bash
   # GitHub tokens should start with ghp_ or github_pat_
   echo $GITHUB_TOKEN | head -c 4
   # Should output: ghp_ or gith
   ```

2. **Generate New Token:**
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Click "Generate new token (classic)"
   - Select appropriate scopes:
     - `repo` for private repositories
     - `public_repo` for public repositories only

3. **Set Environment Variable:**
   ```bash
   export GITHUB_TOKEN=ghp_your_token_here
   # Or in your application:
   ```
   ```rust
   let config = Config::builder()
       .github_token("ghp_your_token_here")
       .build();
   ```

4. **Check Token Permissions:**
   ```bash
   # Test token with GitHub API
   curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
   ```

#### Office 365 Authentication Issues

**Symptoms:**
```
Error: Authentication error: Missing token
Error: Access denied to Office 365 document
```

**Solutions:**
- Contact your IT administrator for proper access
- Ensure document is shared appropriately
- Check corporate firewall settings

### 2. Network and Connection Issues

#### Timeout Errors

**Symptoms:**
```
Error: Network error: Timeout
Error: Request timed out after 30 seconds
```

**Causes:**
- Server is slow or overloaded
- Large document taking long to process
- Network connectivity issues
- Timeout setting too aggressive

**Solutions:**

1. **Increase Timeout:**
   ```rust
   let config = Config::builder()
       .timeout_seconds(120)  // Increase from default 30s
       .build();
   ```

2. **Check Network Connectivity:**
   ```bash
   # Test basic connectivity
   ping google.com
   
   # Test specific domain
   curl -I https://docs.google.com
   ```

3. **Retry with Exponential Backoff:**
   ```rust
   async fn retry_with_backoff(url: &str) -> Result<String, MarkdownError> {
       for attempt in 1..=3 {
           match convert_url(url).await {
               Ok(result) => return Ok(result.as_str().to_string()),
               Err(e) if e.is_retryable() && attempt < 3 => {
                   let delay = Duration::from_secs(2_u64.pow(attempt));
                   tokio::time::sleep(delay).await;
                   continue;
               }
               Err(e) => return Err(e),
           }
       }
       unreachable!()
   }
   ```

#### DNS Resolution Failures

**Symptoms:**
```
Error: Network error: DNS resolution failed
Error: Could not resolve hostname
```

**Solutions:**
1. **Check DNS Settings:**
   ```bash
   nslookup docs.google.com
   dig github.com
   ```

2. **Try Alternative DNS:**
   ```bash
   # Temporarily use Google DNS
   export RESOLV_CONF=/dev/null
   # Or configure your system to use 8.8.8.8
   ```

3. **Verify Domain Spelling:**
   - Double-check URL for typos
   - Ensure domain exists and is accessible

#### Rate Limiting

**Symptoms:**
```
Error: Network error: Rate limited (HTTP 429)
Error: Too many requests
```

**Solutions:**

1. **Implement Rate Limiting:**
   ```rust
   use std::time::Duration;
   use tokio::time::sleep;
   
   async fn rate_limited_convert(urls: Vec<&str>) -> Vec<Result<String, MarkdownError>> {
       let mut results = Vec::new();
       
       for url in urls {
           // Wait between requests
           sleep(Duration::from_millis(500)).await;
           
           let result = convert_url(url).await;
           results.push(result.map(|m| m.as_str().to_string()));
       }
       
       results
   }
   ```

2. **Add Authentication:**
   ```rust
   // GitHub: Authenticated requests have higher limits
   let config = Config::builder()
       .github_token(std::env::var("GITHUB_TOKEN").ok())
       .build();
   ```

3. **Handle Rate Limit Responses:**
   ```rust
   match convert_url(url).await {
       Err(MarkdownError::EnhancedNetworkError { 
           kind: NetworkErrorKind::RateLimited, 
           .. 
       }) => {
           println!("Rate limited, waiting 60 seconds...");
           tokio::time::sleep(Duration::from_secs(60)).await;
           // Retry the request
       }
       result => result,
   }
   ```

### 3. Content Processing Issues

#### Empty Content Error

**Symptoms:**
```
Error: Content error: Empty content
Error: No content found
```

**Causes:**
- URL returns empty page
- Content is loaded dynamically by JavaScript
- Page requires authentication
- Content is behind a paywall

**Solutions:**

1. **Verify URL Content:**
   ```bash
   # Check if URL returns content
   curl -s https://example.com | head -20
   ```

2. **Check for JavaScript-Heavy Sites:**
   - Many modern sites load content with JavaScript
   - markdowndown processes static HTML only
   - Consider alternative approaches for SPA sites

3. **Try Different URL Format:**
   ```rust
   // For Google Docs, try different URL formats
   let urls = vec![
       "https://docs.google.com/document/d/ID/edit",
       "https://docs.google.com/document/d/ID/pub",
       "https://docs.google.com/document/d/ID/export?format=html",
   ];
   ```

#### Parsing Failures

**Symptoms:**
```
Error: Content error: Parsing failed
Error: Could not parse document content
```

**Solutions:**

1. **Check Content Type:**
   ```bash
   curl -I https://example.com
   # Look for Content-Type header
   ```

2. **Verify HTML Structure:**
   - Malformed HTML can cause parsing issues
   - Try viewing source to check for corruption

3. **Use Fallback Strategy:**
   ```rust
   async fn parse_with_fallback(url: &str) -> Result<String, MarkdownError> {
       // Try primary parsing
       match convert_url(url).await {
           Ok(result) => Ok(result.as_str().to_string()),
           Err(MarkdownError::ContentError { kind: ContentErrorKind::ParsingFailed, .. }) => {
               // Try with different configuration
               let config = Config::builder()
                   .normalize_whitespace(false)  // Disable normalization
                   .build();
               
               let md = MarkdownDown::with_config(config);
               let result = md.convert_url(url).await?;
               Ok(result.as_str().to_string())
           }
           Err(e) => Err(e),
       }
   }
   ```

#### Unsupported Format

**Symptoms:**
```
Error: Content error: Unsupported format
Error: Cannot process this content type
```

**Solutions:**

1. **Check Supported URL Types:**
   ```rust
   use markdowndown::{detect_url_type, types::UrlType};
   
   match detect_url_type(url) {
       Ok(UrlType::Html) => println!("HTML page - supported"),
       Ok(UrlType::GoogleDocs) => println!("Google Docs - supported"),
       Ok(UrlType::GitHubIssue) => println!("GitHub Issue - supported"), 
       Ok(UrlType::Office365) => println!("Office 365 - supported"),
       Err(e) => println!("Unsupported URL type: {}", e),
   }
   ```

2. **Convert to Supported Format:**
   - For PDF files: Convert to HTML first
   - For Word docs: Upload to Google Docs or Office 365
   - For other formats: Check if there's a web version

### 4. Configuration Issues

#### Invalid Configuration Values

**Symptoms:**
```
Error: Configuration error: Invalid value
Error: Timeout must be greater than 0
```

**Solutions:**

1. **Validate Configuration:**
   ```rust
   fn validate_config() -> Result<Config, String> {
       let timeout = std::env::var("MARKDOWNDOWN_TIMEOUT")
           .unwrap_or_else(|_| "30".to_string())
           .parse::<u64>()
           .map_err(|_| "Invalid timeout value")?;
       
       if timeout < 5 || timeout > 600 {
           return Err("Timeout must be between 5 and 600 seconds".to_string());
       }
       
       Ok(Config::builder()
           .timeout_seconds(timeout)
           .build())
   }
   ```

2. **Check Environment Variables:**
   ```bash
   # Verify environment variables are set correctly
   env | grep MARKDOWNDOWN
   env | grep GITHUB_TOKEN
   ```

#### Missing Dependencies

**Symptoms:**
```
Error: Configuration error: Missing dependency
Error: Required tool not found
```

**Solutions:**

1. **Check System Dependencies:**
   ```bash
   # Verify required tools are installed
   which curl
   which git
   ```

2. **Install Missing Dependencies:**
   ```bash
   # On macOS
   brew install curl git
   
   # On Ubuntu/Debian
   sudo apt-get install curl git
   
   # On CentOS/RHEL
   sudo yum install curl git
   ```

### 5. Performance Issues

#### Slow Conversions

**Symptoms:**
- Conversions taking longer than expected
- Timeouts on normally responsive sites
- High CPU usage

**Diagnostic Steps:**

1. **Measure Performance:**
   ```rust
   use std::time::Instant;
   
   let start = Instant::now();
   let result = convert_url(url).await;
   let duration = start.elapsed();
   
   println!("Conversion took: {:?}", duration);
   ```

2. **Check Network Latency:**
   ```bash
   # Test latency to target server
   ping -c 4 docs.google.com
   
   # Test full HTTP round trip
   time curl -s -o /dev/null https://docs.google.com
   ```

**Solutions:**

1. **Optimize Configuration:**
   ```rust
   let fast_config = Config::builder()
       .timeout_seconds(15)       // Shorter timeout
       .max_retries(1)           // Fewer retries
       .normalize_whitespace(false) // Skip normalization
       .include_frontmatter(false)  // Skip frontmatter
       .build();
   ```

2. **Use Parallel Processing:**
   ```rust
   use futures::future::join_all;
   
   let futures: Vec<_> = urls.iter()
       .map(|url| convert_url(url))
       .collect();
   
   let results = join_all(futures).await;
   ```

#### High Memory Usage

**Symptoms:**
- Application using excessive memory
- Out of memory errors
- Slow garbage collection

**Solutions:**

1. **Process in Batches:**
   ```rust
   async fn process_urls_in_batches(urls: Vec<&str>, batch_size: usize) {
       for batch in urls.chunks(batch_size) {
           let batch_results: Vec<_> = batch.iter()
               .map(|url| convert_url(url))
               .collect();
           
           let results = futures::future::join_all(batch_results).await;
           
           // Process results immediately
           for result in results {
               match result {
                   Ok(markdown) => {
                       // Process and discard immediately
                       process_markdown(markdown.as_str());
                   }
                   Err(e) => eprintln!("Error: {}", e),
               }
           }
           
           // Memory is freed between batches
       }
   }
   ```

2. **Limit Content Size:**
   ```rust
   let config = Config::builder()
       .placeholder_max_content_length(1000)  // Limit placeholder size
       .build();
   ```

### 6. URL-Specific Issues

#### Google Docs Permission Denied

**Symptoms:**
```
Error: Permission denied accessing Google Doc
HTTP 403 Forbidden
```

**Solutions:**

1. **Check Sharing Settings:**
   - Document must be "Public" or "Anyone with the link can view"
   - Private documents require authentication

2. **Try Different URL Formats:**
   ```rust
   let doc_id = "1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs";
   let urls = vec![
       format!("https://docs.google.com/document/d/{}/edit", doc_id),
       format!("https://docs.google.com/document/d/{}/pub", doc_id),
       format!("https://docs.google.com/document/d/{}/export?format=html", doc_id),
   ];
   ```

3. **Add Google API Key:**
   ```rust
   let config = Config::builder()
       .google_api_key(std::env::var("GOOGLE_API_KEY").ok())
       .build();
   ```

#### GitHub Issues Not Found

**Symptoms:**
```
Error: GitHub issue not found (404)
Error: Repository not accessible
```

**Solutions:**

1. **Verify URL Format:**
   ```rust
   // Correct formats:
   // https://github.com/owner/repo/issues/123
   // https://github.com/owner/repo/pull/456
   
   // Incorrect formats:
   // https://github.com/owner/repo/issue/123 (missing 's')
   // https://github.com/owner/repo/123 (missing 'issues/')
   ```

2. **Check Repository Access:**
   ```bash
   # Test repository access
   curl -H "Authorization: token $GITHUB_TOKEN" \
        https://api.github.com/repos/owner/repo
   ```

3. **Verify Issue Number:**
   - Issue number must exist
   - Issue might be in a different repository

#### Office 365 Access Issues

**Symptoms:**
```
Error: Cannot access SharePoint document
Error: Authentication required
```

**Solutions:**

1. **Check Corporate Policies:**
   - IT policies may block programmatic access
   - May require VPN or specific network access

2. **Try Alternative URLs:**
   ```rust
   // Try different SharePoint URL formats
   let urls = vec![
       "https://company.sharepoint.com/sites/team/Document.docx",
       "https://company-my.sharepoint.com/personal/user_company_com/Document.docx",
   ];
   ```

3. **Contact IT Administrator:**
   - May need special permissions for API access
   - Might need to register application

## Diagnostic Tools

### Basic Connectivity Test

```rust
use markdowndown::{detect_url_type, convert_url};

async fn diagnose_url(url: &str) {
    println!("🔍 Diagnosing URL: {}", url);
    
    // Step 1: URL type detection
    match detect_url_type(url) {
        Ok(url_type) => println!("✅ URL type: {}", url_type),
        Err(e) => {
            println!("❌ URL type detection failed: {}", e);
            return;
        }
    }
    
    // Step 2: Basic HTTP test
    match reqwest::get(url).await {
        Ok(response) => {
            println!("✅ HTTP response: {}", response.status());
            println!("   Content-Type: {:?}", response.headers().get("content-type"));
            println!("   Content-Length: {:?}", response.headers().get("content-length"));
        }
        Err(e) => {
            println!("❌ HTTP request failed: {}", e);
            return;
        }
    }
    
    // Step 3: Conversion test
    let start = std::time::Instant::now();
    match convert_url(url).await {
        Ok(markdown) => {
            let duration = start.elapsed();
            println!("✅ Conversion successful in {:?}", duration);
            println!("   Content length: {} characters", markdown.as_str().len());
            println!("   Line count: {}", markdown.as_str().lines().count());
            
            if let Some(frontmatter) = markdown.frontmatter() {
                println!("   Has frontmatter: {} characters", frontmatter.len());
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            println!("❌ Conversion failed in {:?}: {}", duration, e);
            
            if let Some(context) = e.context() {
                println!("   Context: {} in {}", context.operation, context.converter_type);
            }
            
            let suggestions = e.suggestions();
            if !suggestions.is_empty() {
                println!("   Suggestions:");
                for suggestion in suggestions.iter().take(3) {
                    println!("     - {}", suggestion);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let test_urls = vec![
        "https://httpbin.org/html",
        "https://docs.google.com/document/d/1ZzWTwAmWe0QE24qRV9_xL8B7q8i3rCtO2tVJx8VrIHs/edit",
        "https://github.com/rust-lang/rust/issues/1",
        "invalid-url-for-testing",
    ];
    
    for url in test_urls {
        diagnose_url(url).await;
        println!();
    }
}
```

### Configuration Validator

```rust
use markdowndown::{Config, MarkdownDown};

fn validate_environment() {
    println!("🔧 Environment Validation:");
    
    // Check GitHub token
    match std::env::var("GITHUB_TOKEN") {
        Ok(token) => {
            if token.starts_with("ghp_") || token.starts_with("github_pat_") {
                println!("✅ GitHub token format valid");
            } else {
                println!("⚠️  GitHub token format suspicious");
            }
        }
        Err(_) => {
            println!("ℹ️  GitHub token not set (optional)");
        }
    }
    
    // Check timeout setting
    match std::env::var("MARKDOWNDOWN_TIMEOUT") {
        Ok(timeout_str) => {
            match timeout_str.parse::<u64>() {
                Ok(timeout) if timeout >= 5 && timeout <= 600 => {
                    println!("✅ Timeout setting valid: {}s", timeout);
                }
                Ok(timeout) => {
                    println!("⚠️  Timeout out of recommended range: {}s", timeout);
                }
                Err(_) => {
                    println!("❌ Invalid timeout format: {}", timeout_str);
                }
            }
        }
        Err(_) => {
            println!("ℹ️  Using default timeout: 30s");
        }
    }
    
    // Test configuration creation
    match std::panic::catch_unwind(|| Config::from_env()) {
        Ok(config) => {
            println!("✅ Configuration creation successful");
            println!("   Timeout: {:?}", config.http.timeout);
            println!("   User Agent: {}", config.http.user_agent);
            println!("   Max Retries: {}", config.http.max_retries);
        }
        Err(_) => {
            println!("❌ Configuration creation failed");
        }
    }
}
```

## Getting Help

### Before Asking for Help

1. **Check Error Messages Carefully:**
   - Read the full error message
   - Look for specific error codes or types
   - Check if error suggestions are provided

2. **Test with Simple Cases:**
   ```rust
   // Test with a simple, known-working URL
   let result = convert_url("https://httpbin.org/html").await;
   ```

3. **Check Your Configuration:**
   ```rust
   let config = Config::from_env();
   println!("Debug config: {:?}", config);
   ```

4. **Test Network Connectivity:**
   ```bash
   curl -I https://docs.google.com
   ping github.com
   ```

### What to Include in Bug Reports

1. **Environment Information:**
   - Operating system and version
   - Rust version (`rustc --version`)
   - markdowndown version
   - Network configuration (proxy, firewall, etc.)

2. **Reproduction Case:**
   ```rust
   // Minimal example that reproduces the issue
   use markdowndown::convert_url;
   
   #[tokio::main]
   async fn main() {
       let result = convert_url("https://problem-url.com").await;
       println!("Result: {:?}", result);
   }
   ```

3. **Error Details:**
   - Full error message
   - Error context if available
   - Steps leading to the error

4. **Expected vs. Actual Behavior:**
   - What you expected to happen
   - What actually happened
   - Any workarounds you've tried

### Community Resources

- **GitHub Issues**: [https://github.com/wballard/markdowndown/issues](https://github.com/wballard/markdowndown/issues)
- **Documentation**: [https://docs.rs/markdowndown](https://docs.rs/markdowndown)
- **Examples**: Check the `examples/` directory in the repository

### Reporting Security Issues

For security-related issues:
- **DO NOT** open public issues
- Email security concerns privately
- Include full details for reproduction
- Allow time for fix before public disclosure

## Prevention Tips

1. **Use Appropriate Timeouts:**
   - Set timeouts based on expected document sizes
   - Allow extra time for first requests to new domains

2. **Implement Proper Error Handling:**
   - Always handle errors explicitly
   - Check if errors are retryable
   - Implement exponential backoff for retries

3. **Monitor Performance:**
   - Track conversion times and success rates
   - Set up alerts for high error rates
   - Monitor memory usage in production

4. **Test Thoroughly:**
   - Test with various URL types
   - Test error conditions
   - Test with your specific configuration

5. **Keep Dependencies Updated:**
   - Regularly update markdowndown
   - Update Rust toolchain
   - Monitor security advisories

6. **Use Appropriate Configuration:**
   - Configure based on your use case
   - Use different configs for different environments
   - Validate configuration at startup

This troubleshooting guide should help you resolve most common issues with markdowndown. If you encounter issues not covered here, please refer to the [GitHub Issues](https://github.com/wballard/markdowndown/issues) page or create a new issue with detailed information about your problem.