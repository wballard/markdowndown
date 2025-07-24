# Integration Tests with Real URLs

Create integration tests that validate the library functionality with real URLs and external services, providing confidence in production behavior.

## Objectives

- Test real-world URL conversion scenarios
- Validate integration with external services (Google, GitHub, Office 365)
- Create comprehensive test suites for different URL types
- Implement rate-limited testing to avoid service abuse

## Tasks

1. Create integration test structure in `tests/integration/`:
   - `tests/integration/html_sites.rs` - Real HTML website testing
   - `tests/integration/google_docs.rs` - Public Google Docs testing
   - `tests/integration/github_issues.rs` - GitHub API integration testing
   - `tests/integration/office365.rs` - Office 365 document testing
   - `tests/integration/end_to_end.rs` - Full workflow testing

2. Implement HTML site integration tests:
   - Test with popular websites (Wikipedia, news sites, blogs)
   - Validate content extraction quality
   - Test various HTML structures and complexity levels
   - Measure conversion speed and output quality

3. Create Google Docs integration tests:
   - Set up public test documents with various content types
   - Test different Google Docs URL formats
   - Validate markdown export quality
   - Test error handling for private/deleted documents

4. Add GitHub integration tests:
   - Test with public repositories and issues
   - Validate API response parsing
   - Test comment and reaction handling
   - Include pull request testing
   - Test rate limiting behavior

5. Implement Office 365 integration tests:
   - Create test documents in SharePoint/OneDrive
   - Test various document types (Word, PowerPoint, Excel)
   - Validate download and conversion processes
   - Test authentication scenarios

6. Create rate limiting and retry logic:
   - Implement test throttling to avoid service abuse
   - Add configurable delays between requests
   - Test retry behavior with real network failures
   - Validate circuit breaker functionality

7. Add performance and reliability testing:
   - Measure conversion times for different document sizes
   - Test concurrent request handling
   - Validate memory usage with large documents
   - Test timeout behavior with slow services

8. Create test data management:
   - Maintain list of stable test URLs
   - Regular validation of test document availability
   - Backup test content for offline testing
   - Version control for test document changes

9. Implement comprehensive validation:
   - Content quality checks (completeness, formatting)
   - Frontmatter validation (required fields, accuracy)
   - Error message validation for failure cases
   - Cross-platform testing (Windows, macOS, Linux)

10. Add reporting and monitoring:
    - Test result reporting with detailed metrics
    - Performance trend tracking over time
    - Service availability monitoring
    - Regression detection and alerting

## Acceptance Criteria

- [ ] All URL types tested with real external services
- [ ] Rate limiting prevents service abuse
- [ ] Tests are reliable and don't depend on changing content
- [ ] Performance benchmarks establish realistic expectations
- [ ] Error scenarios are properly tested
- [ ] Tests run in CI/CD with proper service credentials
- [ ] Documentation includes test URL maintenance procedures
- [ ] Offline testing capabilities for development

## Dependencies

- Previous: [000012_unit_tests]
- Requires: All implemented functionality, external service access
- Environment: Test credentials and configuration

## Test Configuration

```rust
// Integration test configuration
#[derive(Debug)]
struct IntegrationTestConfig {
    // Rate limiting
    pub requests_per_minute: u32,
    pub request_delay_ms: u64,
    
    // Timeouts
    pub default_timeout_secs: u64,
    pub large_document_timeout_secs: u64,
    
    // Authentication
    pub github_token: Option<String>,
    pub office365_credentials: Option<Office365Credentials>,
    
    // Test control
    pub skip_slow_tests: bool,
    pub skip_external_services: bool,
}
```

## Test URL Collections

### HTML Sites
```rust
const HTML_TEST_URLS: &[(&str, &str)] = &[
    ("https://en.wikipedia.org/wiki/Rust_(programming_language)", "Complex Wikipedia page"),
    ("https://blog.rust-lang.org/2024/01/15/Rust-1.75.0.html", "Rust blog post"),
    ("https://doc.rust-lang.org/book/ch01-00-getting-started.html", "Rust book chapter"),
    ("https://github.com/rust-lang/rust/blob/master/README.md", "GitHub README"),
];
```

### Google Docs
```rust
const GOOGLE_DOCS_TEST_URLS: &[(&str, &str)] = &[
    ("https://docs.google.com/document/d/test-simple-doc/edit", "Simple text document"),
    ("https://docs.google.com/document/d/test-formatted-doc/edit", "Document with formatting"),
    ("https://docs.google.com/document/d/test-tables-doc/edit", "Document with tables"),
    ("https://docs.google.com/document/d/test-images-doc/edit", "Document with images"),
];
```

### GitHub Issues
```rust
const GITHUB_TEST_URLS: &[(&str, &str)] = &[
    ("https://github.com/rust-lang/rust/issues/1", "Historic issue #1"),
    ("https://github.com/tokio-rs/tokio/issues/100", "Issue with many comments"),
    ("https://github.com/serde-rs/serde/pull/1000", "Pull request example"),
];
```

## Test Implementation Example

```rust
#[tokio::test]
async fn test_wikipedia_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    if config.skip_external_services {
        return Ok(());
    }
    
    // Rate limiting
    tokio::time::sleep(Duration::from_millis(config.request_delay_ms)).await;
    
    let md = MarkdownDown::new();
    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    
    let result = md.convert_url(url).await?;
    
    // Validate output quality
    assert!(result.as_str().contains("# Rust (programming language)"));
    assert!(result.as_str().len() > 1000);
    assert!(result.frontmatter().is_some());
    
    // Validate frontmatter
    let frontmatter = result.frontmatter().unwrap();
    assert!(frontmatter.contains("source_url"));
    assert!(frontmatter.contains("html2markdown"));
    
    Ok(())
}
```

## Performance Testing

```rust
#[tokio::test]
async fn benchmark_conversion_performance() -> Result<(), Box<dyn std::error::Error>> {
    let md = MarkdownDown::new();
    let test_urls = vec![
        "https://doc.rust-lang.org/book/",
        "https://docs.rs/tokio/latest/tokio/",
        "https://github.com/rust-lang/rust/issues/100000",
    ];
    
    for url in test_urls {
        let start = Instant::now();
        let result = md.convert_url(url).await?;
        let duration = start.elapsed();
        
        println!("URL: {}", url);
        println!("Duration: {:?}", duration);
        println!("Content length: {}", result.as_str().len());
        println!("---");
        
        // Performance assertions
        assert!(duration < Duration::from_secs(30));
        assert!(result.as_str().len() > 100);
    }
    
    Ok(())
}
```

## Error Scenario Testing

```rust
#[tokio::test]
async fn test_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let md = MarkdownDown::new();
    
    // Test various error conditions
    let error_cases = vec![
        ("https://docs.google.com/document/d/nonexistent/edit", "Private/deleted document"),
        ("https://github.com/nonexistent/repo/issues/1", "Non-existent repository"),
        ("https://invalid-domain-12345.com/page", "DNS resolution failure"),
    ];
    
    for (url, description) in error_cases {
        let result = md.convert_url(url).await;
        assert!(result.is_err(), "Expected error for: {}", description);
        
        let error = result.unwrap_err();
        println!("Error for {}: {}", description, error);
        
        // Validate error message quality
        assert!(!error.to_string().is_empty());
        assert!(error.to_string().len() > 10);
    }
    
    Ok(())
}
```

## CI/CD Integration

### Environment Variables
- `GITHUB_TOKEN` - GitHub personal access token for testing
- `OFFICE365_USERNAME` / `OFFICE365_PASSWORD` - Office 365 credentials
- `SKIP_INTEGRATION_TESTS` - Skip tests in environments without credentials
- `INTEGRATION_TEST_DELAY_MS` - Rate limiting delay

### Test Execution Strategy
- Run integration tests nightly (not on every commit)
- Use different credentials for CI vs development
- Implement test result caching to reduce external requests
- Monitor external service availability and adjust tests accordingly

## Maintenance Procedures

1. **Monthly URL Validation**: Verify all test URLs are still accessible
2. **Performance Baseline Updates**: Update performance expectations quarterly
3. **Service Changes**: Monitor external services for API changes
4. **Test Data Refresh**: Update test documents and expected outputs
5. **Rate Limit Monitoring**: Ensure tests don't exceed service limits