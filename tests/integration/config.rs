//! Integration test configuration module
//!
//! Provides configuration management for integration tests with external services.

use std::env;
use std::time::Duration;

/// Configuration for integration tests with external services
#[derive(Debug, Clone)]
pub struct IntegrationTestConfig {
    // Rate limiting
    pub requests_per_minute: u32,
    pub request_delay_ms: u64,
    
    // Timeouts
    pub default_timeout_secs: u64,
    pub large_document_timeout_secs: u64,
    
    // Authentication
    pub github_token: Option<String>,
    pub office365_credentials: Option<Office365Credentials>,
    pub google_api_key: Option<String>,
    
    // Test control
    pub skip_slow_tests: bool,
    pub skip_external_services: bool,
    pub skip_network_tests: bool,
}

/// Office 365 authentication credentials
#[derive(Debug, Clone)]
pub struct Office365Credentials {
    pub username: String,
    pub password: String,
}

impl IntegrationTestConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            requests_per_minute: env::var("INTEGRATION_REQUESTS_PER_MINUTE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            request_delay_ms: env::var("INTEGRATION_REQUEST_DELAY_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2000), // 2 seconds between requests
            default_timeout_secs: env::var("INTEGRATION_DEFAULT_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            large_document_timeout_secs: env::var("INTEGRATION_LARGE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120),
            github_token: env::var("GITHUB_TOKEN").ok(),
            office365_credentials: Self::parse_office365_credentials(),
            google_api_key: env::var("GOOGLE_API_KEY").ok(),
            skip_slow_tests: env::var("SKIP_SLOW_TESTS")
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
            skip_external_services: env::var("SKIP_EXTERNAL_SERVICES")
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
            skip_network_tests: env::var("SKIP_NETWORK_TESTS")
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
        }
    }

    /// Create a test configuration with defaults for local testing
    pub fn for_local_testing() -> Self {
        Self {
            requests_per_minute: 10, // Conservative for local testing
            request_delay_ms: 6000,  // 6 seconds between requests
            default_timeout_secs: 30,
            large_document_timeout_secs: 120,
            github_token: env::var("GITHUB_TOKEN").ok(),
            office365_credentials: None, // Usually not available locally
            google_api_key: env::var("GOOGLE_API_KEY").ok(),
            skip_slow_tests: false,
            skip_external_services: false,
            skip_network_tests: false,
        }
    }

    /// Create a CI-friendly configuration that skips tests requiring credentials
    pub fn for_ci() -> Self {
        Self {
            requests_per_minute: 60, // Higher rate for CI
            request_delay_ms: 1000,  // 1 second between requests
            default_timeout_secs: 60,
            large_document_timeout_secs: 180,
            github_token: env::var("GITHUB_TOKEN").ok(),
            office365_credentials: Self::parse_office365_credentials(),
            google_api_key: env::var("GOOGLE_API_KEY").ok(),
            skip_slow_tests: env::var("SKIP_SLOW_TESTS")
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(true), // Skip slow tests in CI by default
            skip_external_services: env::var("SKIP_EXTERNAL_SERVICES")
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
            skip_network_tests: false,
        }
    }

    /// Get the delay duration between requests
    pub fn request_delay(&self) -> Duration {
        Duration::from_millis(self.request_delay_ms)
    }

    /// Get the default timeout duration
    pub fn default_timeout(&self) -> Duration {
        Duration::from_secs(self.default_timeout_secs)
    }

    /// Get the large document timeout duration
    pub fn large_document_timeout(&self) -> Duration {
        Duration::from_secs(self.large_document_timeout_secs)
    }

    /// Check if GitHub tests can be run (token available)
    pub fn can_test_github(&self) -> bool {
        !self.skip_external_services && self.github_token.is_some()
    }

    /// Check if Office 365 tests can be run (credentials available)
    pub fn can_test_office365(&self) -> bool {
        !self.skip_external_services && self.office365_credentials.is_some()
    }

    /// Check if Google Docs tests can be run
    pub fn can_test_google_docs(&self) -> bool {
        !self.skip_external_services
    }

    /// Check if HTML tests can be run
    pub fn can_test_html(&self) -> bool {
        !self.skip_external_services && !self.skip_network_tests
    }

    /// Parse Office 365 credentials from environment variables
    fn parse_office365_credentials() -> Option<Office365Credentials> {
        let username = env::var("OFFICE365_USERNAME").ok()?;
        let password = env::var("OFFICE365_PASSWORD").ok()?;
        Some(Office365Credentials { username, password })
    }
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Test URL collections for different services
pub struct TestUrls;

impl TestUrls {
    /// Stable HTML test URLs that should remain accessible
    pub const HTML_TEST_URLS: &'static [(&'static str, &'static str)] = &[
        ("https://httpbin.org/html", "Simple HTML test page"),
        ("https://en.wikipedia.org/wiki/Rust_(programming_language)", "Complex Wikipedia page"),
        ("https://doc.rust-lang.org/book/ch01-00-getting-started.html", "Rust book chapter"),
        ("https://github.com/rust-lang/rust/blob/master/README.md", "GitHub README"),
    ];

    /// Google Docs test URLs (public documents)
    pub const GOOGLE_DOCS_TEST_URLS: &'static [(&'static str, &'static str)] = &[
        // Note: These would need to be real public Google Docs URLs
        // For now, using placeholder that should be replaced with actual test documents
        ("https://docs.google.com/document/d/1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms/edit", "Example public Google Sheet (as placeholder)"),
    ];

    /// GitHub test URLs for issues and pull requests
    pub const GITHUB_TEST_URLS: &'static [(&'static str, &'static str)] = &[
        ("https://github.com/rust-lang/rust/issues/1", "Historic issue #1"),
        ("https://github.com/tokio-rs/tokio/issues/1000", "Issue with discussions"),
        ("https://github.com/serde-rs/serde/pull/2000", "Pull request example"),
    ];

    /// Office 365 test URLs (publicly accessible documents)
    pub const OFFICE365_TEST_URLS: &'static [(&'static str, &'static str)] = &[
        // Note: These would need to be real public SharePoint documents
        // For now, using placeholders that should be replaced with actual test documents
    ];

    /// Error test URLs for testing failure scenarios
    pub const ERROR_TEST_URLS: &'static [(&'static str, &'static str)] = &[
        ("https://docs.google.com/document/d/nonexistent/edit", "Private/deleted Google document"),
        ("https://github.com/nonexistent/repo/issues/1", "Non-existent GitHub repository"),
        ("https://invalid-domain-12345.com/page", "DNS resolution failure"),
        ("https://httpbin.org/status/404", "HTTP 404 error"),
        ("https://httpbin.org/status/500", "HTTP 500 error"),
    ];
}

/// Utility functions for integration tests
pub struct TestUtils;

impl TestUtils {
    /// Apply rate limiting delay if configured
    pub async fn apply_rate_limit(config: &IntegrationTestConfig) {
        if config.request_delay_ms > 0 {
            tokio::time::sleep(config.request_delay()).await;
        }
    }

    /// Check if content looks like valid markdown
    pub fn validate_markdown_quality(content: &str) -> bool {
        // Basic quality checks
        !content.is_empty() &&
        content.len() > 50 && // Should have meaningful content
        !content.trim().starts_with("Error") && // Should not be an error message
        content.lines().count() > 3 // Should have multiple lines
    }

    /// Validate that frontmatter contains expected fields
    pub fn validate_frontmatter(frontmatter: &str) -> bool {
        frontmatter.contains("source_url") &&
        frontmatter.contains("converted_at") &&
        frontmatter.contains("conversion_type")
    }

    /// Get a user agent string for testing
    pub fn test_user_agent() -> String {
        format!("markdowndown-integration-tests/{}", env!("CARGO_PKG_VERSION"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_with_defaults() {
        // Test that config creation works even without environment variables
        let config = IntegrationTestConfig::from_env();
        
        assert_eq!(config.requests_per_minute, 30);
        assert_eq!(config.request_delay_ms, 2000);
        assert_eq!(config.default_timeout_secs, 30);
        assert_eq!(config.large_document_timeout_secs, 120);
        assert!(!config.skip_slow_tests || env::var("SKIP_SLOW_TESTS").is_ok());
    }

    #[test]
    fn test_local_testing_config() {
        let config = IntegrationTestConfig::for_local_testing();
        
        assert_eq!(config.requests_per_minute, 10);
        assert_eq!(config.request_delay_ms, 6000);
        assert!(!config.skip_slow_tests);
        assert!(!config.skip_external_services);
    }

    #[test]
    fn test_ci_config() {
        let config = IntegrationTestConfig::for_ci();
        
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.request_delay_ms, 1000);
        // CI should skip slow tests by default unless overridden
        assert!(config.skip_slow_tests || env::var("SKIP_SLOW_TESTS").map(|s| s == "false").unwrap_or(false));
    }

    #[test]
    fn test_duration_helpers() {
        let config = IntegrationTestConfig::for_local_testing();
        
        assert_eq!(config.request_delay(), Duration::from_millis(6000));
        assert_eq!(config.default_timeout(), Duration::from_secs(30));
        assert_eq!(config.large_document_timeout(), Duration::from_secs(120));
    }

    #[test]
    fn test_capability_checks() {
        let config = IntegrationTestConfig::for_local_testing();
        
        // These depend on environment variables, so we just test the logic
        assert_eq!(config.can_test_github(), config.github_token.is_some());
        assert_eq!(config.can_test_office365(), config.office365_credentials.is_some());
        assert!(config.can_test_google_docs()); // Should be true for local testing
        assert!(config.can_test_html()); // Should be true for local testing
    }

    #[test]
    fn test_validation_helpers() {
        // Test markdown quality validation
        assert!(TestUtils::validate_markdown_quality("# Title\n\nThis is a substantial piece of content that should pass validation."));
        assert!(!TestUtils::validate_markdown_quality(""));
        assert!(!TestUtils::validate_markdown_quality("Short"));
        assert!(!TestUtils::validate_markdown_quality("Error: Something went wrong"));

        // Test frontmatter validation
        assert!(TestUtils::validate_frontmatter("source_url: test\nconverted_at: now\nconversion_type: html"));
        assert!(!TestUtils::validate_frontmatter("missing_fields: true"));
    }

    #[test]
    fn test_user_agent() {
        let ua = TestUtils::test_user_agent();
        assert!(ua.starts_with("markdowndown-integration-tests/"));
        assert!(ua.contains(env!("CARGO_PKG_VERSION")));
    }
}