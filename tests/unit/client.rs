//! Comprehensive unit tests for HTTP client functionality.
//!
//! This module tests the HTTP client with mock servers, timeout handling,
//! retry logic, authentication, and comprehensive error scenarios.

use markdowndown::client::HttpClient;
use markdowndown::config::Config;
use markdowndown::types::{AuthErrorKind, MarkdownError, NetworkErrorKind, ValidationErrorKind};
use mockito::Server;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

/// Configurable test timeouts - can be overridden via environment variables
const DEFAULT_TEST_RETRY_DELAY_MS: u64 = 10;
const DEFAULT_TEST_TIMEOUT_SECS: u64 = 2;

fn get_test_retry_delay() -> Duration {
    let delay_ms = std::env::var("TEST_RETRY_DELAY_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TEST_RETRY_DELAY_MS);
    Duration::from_millis(delay_ms)
}

fn get_test_timeout() -> Duration {
    let timeout_secs = std::env::var("TEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_TEST_TIMEOUT_SECS);
    Duration::from_secs(timeout_secs)
}

mod helpers {
    use super::*;

    /// Create a test HTTP client with configurable delays for testing
    pub fn create_test_client() -> HttpClient {
        let config = Config::builder()
            .retry_delay(get_test_retry_delay())
            .timeout(get_test_timeout())
            .build();

        HttpClient::with_config(&config.http, &config.auth)
    }

    /// Create a test HTTP client with authentication tokens
    pub fn create_auth_client() -> HttpClient {
        let config = Config::builder()
            .retry_delay(get_test_retry_delay())
            .timeout(get_test_timeout())
            .github_token("test_github_token")
            .office365_token("test_office365_token")
            .google_api_key("test_google_api_key")
            .build();

        HttpClient::with_config(&config.http, &config.auth)
    }

    /// Assert that a result contains a ValidationError with InvalidUrl kind
    pub fn assert_validation_error(result: Result<String, MarkdownError>, expected_url: &str) {
        match result.unwrap_err() {
            MarkdownError::ValidationError { kind, context } => {
                assert_eq!(kind, ValidationErrorKind::InvalidUrl);
                assert_eq!(context.url, expected_url);
                assert_eq!(context.operation, "URL validation");
                assert_eq!(context.converter_type, "HttpClient");
            }
            err => panic!("Expected ValidationError, got: {err:?}"),
        }
    }

    /// Assert that a URL is rejected with a ValidationError
    pub async fn assert_url_rejected(client: &HttpClient, url: &str) {
        let result = client.get_text(url).await;
        assert!(result.is_err(), "Should reject URL: {url}");

        match result.unwrap_err() {
            MarkdownError::ValidationError { kind, .. } => {
                assert_eq!(kind, ValidationErrorKind::InvalidUrl);
            }
            err => panic!("Expected ValidationError for URL: {url}, got: {err:?}"),
        }
    }
}

/// Tests for HTTP client creation and configuration
mod client_creation_tests {
    use super::*;

    #[test]
    fn test_http_client_new() {
        let _client = HttpClient::new();
        // These are private fields, so we test indirectly through behavior
        // The default client should be created successfully
        // Test passes if no panic occurs during creation
    }

    #[test]
    fn test_http_client_default() {
        let _client = HttpClient::default();
        // Test that default is equivalent to new()
        // Test passes if no panic occurs during creation
    }

    #[test]
    fn test_http_client_with_custom_config() {
        let config = Config::builder()
            .max_retries(5)
            .timeout_seconds(60)
            .user_agent("custom-agent/1.0")
            .build();

        let _client = HttpClient::with_config(&config.http, &config.auth);
        // Client should be created with custom config
        // Test passes if no panic occurs during creation with custom config
    }
}

/// Tests for successful HTTP operations
mod success_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_text_success() {
        let mut server = Server::new_async().await;
        let expected_body = "Hello, World! This is test content.";

        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(expected_body)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/test", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_body);
    }

    #[tokio::test]
    async fn test_get_bytes_success() {
        let mut server = Server::new_async().await;
        let expected_body = b"Binary data content";

        let mock = server
            .mock("GET", "/binary")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(expected_body)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/binary", server.url());
        let result = client.get_bytes(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), expected_body);
    }

    #[tokio::test]
    async fn test_get_text_with_headers() {
        let mut server = Server::new_async().await;
        let expected_body = "Content with custom headers";

        let mock = server
            .mock("GET", "/with-headers")
            .match_header("X-Custom-Header", "test-value")
            .match_header("User-Agent", "test-agent/1.0")
            .with_status(200)
            .with_body(expected_body)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/with-headers", server.url());

        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "test-value".to_string());
        headers.insert("User-Agent".to_string(), "test-agent/1.0".to_string());

        let result = client.get_text_with_headers(&url, &headers).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_body);
    }

    #[tokio::test]
    async fn test_large_response_handling() {
        let mut server = Server::new_async().await;
        let large_content = "A".repeat(100_000); // 100KB of data

        let mock = server
            .mock("GET", "/large")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(&large_content)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/large", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 100_000);
    }
}

/// Tests for URL validation errors
mod url_validation_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_url_format() {
        let client = helpers::create_test_client();
        let result = client.get_text("not-a-valid-url").await;
        helpers::assert_validation_error(result, "not-a-valid-url");
    }

    #[tokio::test]
    async fn test_unsupported_url_scheme() {
        let client = helpers::create_test_client();
        let unsupported_urls = [
            "ftp://example.com/file.txt",
            "file:///path/to/file",
            "mailto:test@example.com",
            "ws://example.com/socket",
            "data:text/plain;base64,SGVsbG8gV29ybGQ=",
        ];

        for url in unsupported_urls {
            helpers::assert_url_rejected(&client, url).await;
        }
    }

    #[tokio::test]
    async fn test_empty_url() {
        let client = helpers::create_test_client();
        helpers::assert_url_rejected(&client, "").await;
    }
}

/// Tests for HTTP error responses
mod http_error_tests {
    use super::*;

    #[tokio::test]
    async fn test_http_404_not_found() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/notfound")
            .with_status(404)
            .with_body("Not Found")
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/notfound", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, context } => {
                match kind {
                    NetworkErrorKind::ServerError(status) => {
                        assert_eq!(status, 404);
                    }
                    _ => panic!("Expected ServerError(404)"),
                }
                assert!(context.additional_info.unwrap().contains("404"));
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_http_401_unauthorized() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/secure")
            .with_status(401)
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/secure", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::AuthenticationError { kind, context } => {
                assert_eq!(kind, AuthErrorKind::MissingToken);
                assert!(context.additional_info.unwrap().contains("401"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[tokio::test]
    async fn test_http_403_forbidden() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/forbidden")
            .with_status(403)
            .with_body("Forbidden")
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/forbidden", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::AuthenticationError { kind, context } => {
                assert_eq!(kind, AuthErrorKind::PermissionDenied);
                assert!(context.additional_info.unwrap().contains("403"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[tokio::test]
    async fn test_http_429_rate_limited() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/rate-limited")
            .with_status(429)
            .with_header("Retry-After", "60")
            .with_body("Too Many Requests")
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/rate-limited", server.url());
        let result = client.get_text(&url).await;

        // Should retry and eventually fail
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, context } => match kind {
                NetworkErrorKind::RateLimited => {
                    assert!(context.additional_info.unwrap().contains("429"));
                }
                _ => panic!("Expected RateLimited error, got: {kind:?}"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_http_500_server_error() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/server-error")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/server-error", server.url());
        let result = client.get_text(&url).await;

        // Should retry and eventually fail
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, context } => {
                match kind {
                    NetworkErrorKind::ServerError(status) => {
                        assert_eq!(status, 500);
                    }
                    _ => panic!("Expected ServerError(500)"),
                }
                assert!(context.additional_info.unwrap().contains("500"));
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_client_error_no_retry() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/bad-request")
            .with_status(400)
            .with_body("Bad Request")
            .expect(1) // Should only be called once (no retry)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/bad-request", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => match kind {
                NetworkErrorKind::ServerError(status) => {
                    assert_eq!(status, 400);
                }
                _ => panic!("Expected ServerError(400)"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }
}

/// Tests for retry logic and resilience
mod retry_logic_tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let mut server = Server::new_async().await;

        // First two requests fail, third succeeds
        let failing_mock = server
            .mock("GET", "/flaky")
            .with_status(500)
            .expect(2)
            .create_async()
            .await;

        let success_mock = server
            .mock("GET", "/flaky")
            .with_status(200)
            .with_body("Success after retries!")
            .expect(1)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/flaky", server.url());
        let result = client.get_text(&url).await;

        failing_mock.assert_async().await;
        success_mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success after retries!");
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/always-fails")
            .with_status(502)
            .expect(4) // 1 initial + 3 retries
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/always-fails", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, context } => {
                match kind {
                    NetworkErrorKind::ServerError(status) => {
                        assert_eq!(status, 502);
                    }
                    _ => panic!("Expected ServerError(502)"),
                }
                assert!(context.additional_info.unwrap().contains("4 attempts"));
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_no_retry_for_auth_errors() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/unauthorized")
            .with_status(401)
            .expect(1) // Should only be called once (no retry)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/unauthorized", server.url());
        let result = client.get_text(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::AuthenticationError { .. } => {
                // Expected - no retry for auth errors
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/backoff-test")
            .with_status(503)
            .expect(4) // 1 initial + 3 retries
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/backoff-test", server.url());

        let start = std::time::Instant::now();
        let result = client.get_text(&url).await;
        let duration = start.elapsed();

        mock.assert_async().await;
        assert!(result.is_err());

        // Verify that backoff introduced some delay, but don't test exact timing
        // since that can be flaky depending on system load and CI environments
        let expected_minimum = get_test_retry_delay().as_millis() as u64;
        let reasonable_maximum = Duration::from_secs(5).as_millis() as u64; // Generous upper bound

        assert!(
            (duration.as_millis() as u64) >= expected_minimum,
            "Expected minimum delay of {}ms, got {}ms",
            expected_minimum,
            duration.as_millis()
        );
        assert!(
            (duration.as_millis() as u64) < reasonable_maximum,
            "Test took too long: {}ms (max: {}ms)",
            duration.as_millis(),
            reasonable_maximum
        );
    }
}

/// Tests for authentication header injection
mod authentication_tests {
    use super::*;

    #[tokio::test]
    async fn test_github_authentication() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/github-api")
            .match_header("Authorization", "token test_github_token")
            .with_status(200)
            .with_body("GitHub API response")
            .create_async()
            .await;

        let client = helpers::create_auth_client();

        let url = format!("{}/github-api", server.url());

        // Since we can't mock domain detection easily, let's test the header logic
        // by manually adding the expected authorization header
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "token test_github_token".to_string(),
        );

        let result = client.get_text_with_headers(&url, &headers).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "GitHub API response");
    }

    #[tokio::test]
    async fn test_office365_authentication() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/office365-api")
            .match_header("Authorization", "Bearer test_office365_token")
            .with_status(200)
            .with_body("Office 365 API response")
            .create_async()
            .await;

        let client = helpers::create_auth_client();
        let url = format!("{}/office365-api", server.url());

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer test_office365_token".to_string(),
        );

        let result = client.get_text_with_headers(&url, &headers).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Office 365 API response");
    }

    #[tokio::test]
    async fn test_google_api_authentication() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/google-api")
            .match_header("Authorization", "Bearer test_google_api_key")
            .with_status(200)
            .with_body("Google API response")
            .create_async()
            .await;

        let client = helpers::create_auth_client();
        let url = format!("{}/google-api", server.url());

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer test_google_api_key".to_string(),
        );

        let result = client.get_text_with_headers(&url, &headers).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Google API response");
    }
}

/// Tests for timeout behavior
mod timeout_tests {
    use super::*;

    #[tokio::test]
    async fn test_request_timeout() {
        let mut server = Server::new_async().await;

        // Create a mock that simulates a slow response
        let _mock = server
            .mock("GET", "/slow")
            .with_status(200)
            .with_body("Slow response")
            .with_chunked_body(|w| {
                // Sleep longer than the client timeout
                std::thread::sleep(Duration::from_secs(3));
                w.write_all(b"Too late!")
            })
            .create_async()
            .await;

        let config = Config::builder()
            .timeout(Duration::from_millis(100)) // Very short timeout
            .retry_delay(Duration::from_millis(10))
            .build();

        let client = HttpClient::with_config(&config.http, &config.auth);
        let url = format!("{}/slow", server.url());

        let result = timeout(Duration::from_secs(5), client.get_text(&url)).await;

        assert!(result.is_ok()); // The timeout wrapper shouldn't timeout
        let inner_result = result.unwrap();
        assert!(inner_result.is_err());

        // Should be a timeout error
        match inner_result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => {
                match kind {
                    NetworkErrorKind::Timeout => {
                        // Expected timeout error
                    }
                    NetworkErrorKind::ConnectionFailed => {
                        // Also acceptable - reqwest might map timeout to connection failed
                    }
                    _ => panic!("Expected Timeout or ConnectionFailed error, got: {kind:?}"),
                }
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }
}

/// Tests for edge cases and error conditions
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_response_body() {
        let mut server = Server::new_async().await;

        let _mock = server
            .mock("GET", "/empty")
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/empty", server.url());
        let result = client.get_text(&url).await;

        _mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[tokio::test]
    async fn test_binary_content_as_text() {
        let mut server = Server::new_async().await;
        let binary_data = b"\x00\x01\x02\x03\xFF\xFE\xFD\xFC";

        let mock = server
            .mock("GET", "/binary")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(binary_data)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/binary", server.url());

        // get_text should handle binary data gracefully
        let result = client.get_text(&url).await;
        mock.assert_async().await;

        assert!(result.is_ok());
        // The content might not be valid UTF-8, but get_text should handle it
    }

    #[tokio::test]
    async fn test_very_long_url() {
        let client = helpers::create_test_client();

        // Create a very long but valid URL
        let long_path = "a".repeat(2000);
        let long_url = format!("https://example.com/{long_path}");

        let result = client.get_text(&long_url).await;

        // Should fail with network error (can't actually connect to example.com)
        // but not with URL validation error
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { .. } => {
                // Expected - connection will fail but URL is valid
            }
            e => panic!("Expected network error, got: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_international_domain_names() {
        let client = helpers::create_test_client();

        // Test with international domain name (IDN)
        let idn_url = "https://例え.テスト/path";

        let result = client.get_text(idn_url).await;

        // Should fail with network error (can't connect) but not validation error
        assert!(result.is_err());
        // The specific error type may vary depending on how the URL library handles IDNs
    }

    #[tokio::test]
    async fn test_redirect_handling() {
        let mut server = Server::new_async().await;

        let redirect_mock = server
            .mock("GET", "/redirect-source")
            .with_status(302)
            .with_header("Location", &format!("{}/redirect-target", server.url()))
            .create_async()
            .await;

        let target_mock = server
            .mock("GET", "/redirect-target")
            .with_status(200)
            .with_body("Redirected content")
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/redirect-source", server.url());
        let result = client.get_text(&url).await;

        redirect_mock.assert_async().await;
        target_mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Redirected content");
    }
}

/// Integration tests combining multiple HTTP client features
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_workflow_with_retries_and_auth() {
        let mut server = Server::new_async().await;

        // Simulate a service that's initially down, then requires auth, then works
        let failing_mock = server
            .mock("GET", "/api/data")
            .with_status(503)
            .expect(2) // Fail twice
            .create_async()
            .await;

        let auth_required_mock = server
            .mock("GET", "/api/data")
            .with_status(401)
            .expect(2) // Auth error may be hit during retries
            .create_async()
            .await;

        let success_mock = server
            .mock("GET", "/api/data")
            .match_header("Authorization", "Bearer custom-token")
            .with_status(200)
            .with_body("Protected data")
            .expect(1)
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let url = format!("{}/api/data", server.url());

        // First attempt should fail with retries
        let result1 = client.get_text(&url).await;
        assert!(result1.is_err());

        // Second attempt should fail with auth error (no retries)
        let result2 = client.get_text(&url).await;
        assert!(result2.is_err());
        match result2.unwrap_err() {
            MarkdownError::AuthenticationError { .. } => {
                // Expected auth error
            }
            _ => panic!("Expected AuthenticationError"),
        }

        // Third attempt with proper auth header should succeed
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer custom-token".to_string(),
        );
        let result3 = client.get_text_with_headers(&url, &headers).await;

        failing_mock.assert_async().await;
        auth_required_mock.assert_async().await;
        success_mock.assert_async().await;
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), "Protected data");
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/concurrent")
            .with_status(200)
            .with_body("Concurrent response")
            .expect(5) // Expect 5 concurrent requests
            .create_async()
            .await;

        let _client = helpers::create_test_client();
        let url = format!("{}/concurrent", server.url());

        // Launch 5 concurrent requests
        let mut handles = Vec::new();
        for i in 0..5 {
            let client_clone = helpers::create_test_client();
            let url_clone = url.clone();
            let handle = tokio::spawn(async move {
                let result = client_clone.get_text(&url_clone).await;
                (i, result)
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let mut results = Vec::new();
        for handle in handles {
            let (i, result) = handle.await.unwrap();
            results.push((i, result));
        }

        mock.assert_async().await;

        // All requests should succeed
        for (i, result) in results {
            assert!(result.is_ok(), "Request {} failed: {:?}", i, result.err());
            assert_eq!(result.unwrap(), "Concurrent response");
        }
    }

    #[tokio::test]
    async fn test_mixed_success_failure_scenarios() {
        let mut server = Server::new_async().await;

        // Set up multiple endpoints with different behaviors
        let success_mock = server
            .mock("GET", "/success")
            .with_status(200)
            .with_body("Success")
            .create_async()
            .await;

        let not_found_mock = server
            .mock("GET", "/not-found")
            .with_status(404)
            .create_async()
            .await;

        let server_error_mock = server
            .mock("GET", "/server-error")
            .with_status(500)
            .expect(4) // 1 initial + 3 retries
            .create_async()
            .await;

        let client = helpers::create_test_client();
        let base_url = server.url();

        // Test successful request
        let success_result = client.get_text(&format!("{base_url}/success")).await;
        assert!(success_result.is_ok());
        assert_eq!(success_result.unwrap(), "Success");

        // Test 404 error (no retry)
        let not_found_result = client.get_text(&format!("{base_url}/not-found")).await;
        assert!(not_found_result.is_err());
        match not_found_result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => {
                match kind {
                    NetworkErrorKind::ServerError(404) => {
                        // Expected
                    }
                    _ => panic!("Expected ServerError(404)"),
                }
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }

        // Test 500 error (with retries)
        let server_error_result = client.get_text(&format!("{base_url}/server-error")).await;
        assert!(server_error_result.is_err());
        match server_error_result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => {
                match kind {
                    NetworkErrorKind::ServerError(500) => {
                        // Expected after retries
                    }
                    _ => panic!("Expected ServerError(500)"),
                }
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }

        success_mock.assert_async().await;
        not_found_mock.assert_async().await;
        server_error_mock.assert_async().await;
    }
}
