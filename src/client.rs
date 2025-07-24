//! HTTP client wrapper for network operations.
//!
//! This module provides a robust HTTP client with retry logic, timeout handling,
//! and proper error mapping for the markdowndown library.

use crate::config::{AuthConfig, HttpConfig};
use crate::types::{
    AuthErrorKind, ErrorContext, MarkdownError, NetworkErrorKind, ValidationErrorKind,
};
use bytes::Bytes;
use reqwest::{Client, Response};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument};
use url::Url;

/// HTTP client configuration with retry logic and error handling.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    max_retries: u32,
    base_delay: Duration,
    auth: AuthConfig,
}

impl HttpClient {
    /// Creates a new HTTP client with sensible defaults.
    ///
    /// Default configuration:
    /// - Timeout: 30 seconds
    /// - Max redirects: 10
    /// - User agent: "markdowndown/0.1.0"
    /// - Max retries: 3
    /// - Base delay: 1 second (with exponential backoff)
    pub fn new() -> Self {
        let config = crate::config::Config::default();
        Self::with_config(&config.http, &config.auth)
    }

    /// Creates a new HTTP client with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `http_config` - HTTP client configuration options
    /// * `auth_config` - Authentication configuration
    ///
    /// # Returns
    ///
    /// A new `HttpClient` instance configured with the provided settings.
    ///
    pub fn with_config(http_config: &HttpConfig, auth_config: &AuthConfig) -> Self {
        let client = Client::builder()
            .timeout(http_config.timeout)
            .redirect(reqwest::redirect::Policy::limited(
                http_config.max_redirects as usize,
            ))
            .user_agent(&http_config.user_agent)
            .build()
            .expect("Failed to create HTTP client");

        HttpClient {
            client,
            max_retries: http_config.max_retries,
            base_delay: http_config.retry_delay,
            auth: auth_config.clone(),
        }
    }

    /// Fetches text content from a URL with retry logic.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch content from
    ///
    /// # Returns
    ///
    /// Returns the response body as a String on success, or a MarkdownError on failure.
    ///
    /// # Errors
    ///
    /// * `MarkdownError::InvalidUrl` - If the URL is malformed
    /// * `MarkdownError::NetworkError` - For network-related failures
    /// * `MarkdownError::AuthError` - For authentication failures (401, 403)
    #[instrument(skip(self))]
    pub async fn get_text(&self, url: &str) -> Result<String, MarkdownError> {
        debug!("Fetching text content from URL");
        let response = self.retry_request(url).await?;

        debug!("Reading response body as text");
        let text = response.text().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            let context = ErrorContext::new(url, "Read response body", "HttpClient")
                .with_info(format!("Error: {e}"));
            MarkdownError::EnhancedNetworkError {
                kind: NetworkErrorKind::ConnectionFailed,
                context,
            }
        })?;

        info!("Successfully fetched text content ({} chars)", text.len());
        Ok(text)
    }

    /// Fetches binary content from a URL with retry logic.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch content from
    ///
    /// # Returns
    ///
    /// Returns the response body as Bytes on success, or a MarkdownError on failure.
    ///
    /// # Errors
    ///
    /// * `MarkdownError::InvalidUrl` - If the URL is malformed
    /// * `MarkdownError::NetworkError` - For network-related failures
    /// * `MarkdownError::AuthError` - For authentication failures (401, 403)
    pub async fn get_bytes(&self, url: &str) -> Result<Bytes, MarkdownError> {
        let response = self.retry_request(url).await?;
        let bytes = response.bytes().await.map_err(|e| {
            let context = ErrorContext::new(url, "Read response body", "HttpClient")
                .with_info(format!("Error: {e}"));
            MarkdownError::EnhancedNetworkError {
                kind: NetworkErrorKind::ConnectionFailed,
                context,
            }
        })?;
        Ok(bytes)
    }

    /// Fetches text content from a URL with custom headers and retry logic.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch content from
    /// * `headers` - Custom headers to include in the request
    ///
    /// # Returns
    ///
    /// Returns the response body as a String on success, or a MarkdownError on failure.
    ///
    /// # Errors
    ///
    /// * `MarkdownError::InvalidUrl` - If the URL is malformed
    /// * `MarkdownError::NetworkError` - For network-related failures
    /// * `MarkdownError::AuthError` - For authentication failures (401, 403)
    pub async fn get_text_with_headers(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<String, MarkdownError> {
        let response = self.retry_request_with_headers(url, headers).await?;
        let text = response.text().await.map_err(|e| {
            let context = ErrorContext::new(url, "Read response body", "HttpClient")
                .with_info(format!("Error: {e}"));
            MarkdownError::EnhancedNetworkError {
                kind: NetworkErrorKind::ConnectionFailed,
                context,
            }
        })?;
        Ok(text)
    }

    /// Internal method to perform HTTP requests with retry logic and custom headers.
    ///
    /// Implements exponential backoff for transient failures.
    async fn retry_request_with_headers(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<Response, MarkdownError> {
        // Validate URL format
        let parsed_url = Url::parse(url).map_err(|_| {
            let context = ErrorContext::new(url, "URL validation", "HttpClient");
            MarkdownError::ValidationError {
                kind: ValidationErrorKind::InvalidUrl,
                context,
            }
        })?;

        // Ensure URL uses HTTP or HTTPS
        match parsed_url.scheme() {
            "http" | "https" => {}
            _ => {
                let context = ErrorContext::new(url, "URL scheme validation", "HttpClient")
                    .with_info(format!("Unsupported scheme: {}", parsed_url.scheme()));
                return Err(MarkdownError::ValidationError {
                    kind: ValidationErrorKind::InvalidUrl,
                    context,
                });
            }
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let mut request = self.client.get(url);

            // Add custom headers individually, which should override defaults
            for (key, value) in headers {
                request = request.header(key, value);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    // Check if this is a success or non-retryable error
                    if status.is_success() {
                        return Ok(response);
                    } else if status == 401 || status == 403 {
                        // Auth errors - don't retry
                        let auth_kind = if status == 401 {
                            AuthErrorKind::MissingToken
                        } else {
                            AuthErrorKind::PermissionDenied
                        };
                        let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                            .with_info(format!("HTTP status: {status}"));
                        return Err(MarkdownError::AuthenticationError {
                            kind: auth_kind,
                            context,
                        });
                    } else if status == 404 {
                        // Not found - don't retry
                        let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                            .with_info(format!("HTTP status: {status}"));
                        return Err(MarkdownError::EnhancedNetworkError {
                            kind: NetworkErrorKind::ServerError(status.as_u16()),
                            context,
                        });
                    } else if status.is_server_error() || status == 429 {
                        // Server errors and rate limiting - these are retryable
                        if attempt == self.max_retries {
                            let network_kind = if status == 429 {
                                NetworkErrorKind::RateLimited
                            } else {
                                NetworkErrorKind::ServerError(status.as_u16())
                            };
                            let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                                .with_info(format!(
                                    "HTTP status: {} after {} attempts",
                                    status,
                                    self.max_retries + 1
                                ));
                            return Err(MarkdownError::EnhancedNetworkError {
                                kind: network_kind,
                                context,
                            });
                        }
                        // Fall through to retry logic
                    } else {
                        // Other client errors - don't retry
                        let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                            .with_info(format!("HTTP status: {status}"));
                        return Err(MarkdownError::EnhancedNetworkError {
                            kind: NetworkErrorKind::ServerError(status.as_u16()),
                            context,
                        });
                    }
                }
                Err(e) => {
                    last_error = Some(e);

                    // Don't retry on the last attempt
                    if attempt == self.max_retries {
                        break;
                    }
                }
            }

            // Calculate delay with exponential backoff
            let delay = self.base_delay * 2_u32.pow(attempt);
            sleep(delay).await;
        }

        // If we reach here, all attempts failed with network errors
        let error = last_error.unwrap();
        Err(self.map_reqwest_error(error, url))
    }

    /// Internal method to perform HTTP requests with retry logic.
    ///
    /// Implements exponential backoff for transient failures.
    #[instrument(skip(self), fields(attempt, max_retries = self.max_retries))]
    async fn retry_request(&self, url: &str) -> Result<Response, MarkdownError> {
        debug!("Starting HTTP request with retry logic");

        // Validate URL format
        let parsed_url = Url::parse(url).map_err(|_| {
            error!("Invalid URL format: {}", url);
            let context = ErrorContext::new(url, "URL validation", "HttpClient");
            MarkdownError::ValidationError {
                kind: ValidationErrorKind::InvalidUrl,
                context,
            }
        })?;

        // Ensure URL uses HTTP or HTTPS
        match parsed_url.scheme() {
            "http" | "https" => {
                debug!("URL scheme validated: {}", parsed_url.scheme());
            }
            scheme => {
                error!("Unsupported URL scheme: {}", scheme);
                let context = ErrorContext::new(url, "URL scheme validation", "HttpClient")
                    .with_info(format!("Unsupported scheme: {scheme}"));
                return Err(MarkdownError::ValidationError {
                    kind: ValidationErrorKind::InvalidUrl,
                    context,
                });
            }
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            tracing::Span::current().record("attempt", attempt);
            debug!("Attempt {} of {}", attempt + 1, self.max_retries + 1);
            let mut request = self.client.get(url);

            // Add authentication headers based on URL domain
            if let Some(github_token) = &self.auth.github_token {
                if parsed_url.host_str().is_some_and(|host| {
                    host.contains("github") || host.starts_with("127.0.0.1") || host == "localhost"
                }) {
                    request = request.header("Authorization", format!("token {github_token}"));
                    // Add GitHub API Accept header if this looks like an API request
                    if parsed_url.path().starts_with("/repos/") {
                        request = request.header("Accept", "application/vnd.github.v3+json");
                    }
                }
            }

            if let Some(office365_token) = &self.auth.office365_token {
                if parsed_url.host_str().is_some_and(|host| {
                    host.contains("office.com")
                        || host.contains("sharepoint.com")
                        || host.contains("onedrive.com")
                }) {
                    request = request.header("Authorization", format!("Bearer {office365_token}"));
                }
            }

            if let Some(google_api_key) = &self.auth.google_api_key {
                if parsed_url
                    .host_str()
                    .is_some_and(|host| host.contains("googleapis.com"))
                {
                    request = request.header("Authorization", format!("Bearer {google_api_key}"));
                }
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    debug!("Received HTTP response: {}", status);

                    // Check if this is a success or non-retryable error
                    if status.is_success() {
                        info!("HTTP request successful: {}", status);
                        return Ok(response);
                    } else if status == 401 || status == 403 {
                        // Auth errors - don't retry
                        let auth_kind = if status == 401 {
                            AuthErrorKind::MissingToken
                        } else {
                            AuthErrorKind::PermissionDenied
                        };
                        let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                            .with_info(format!("HTTP status: {status}"));
                        return Err(MarkdownError::AuthenticationError {
                            kind: auth_kind,
                            context,
                        });
                    } else if status == 404 {
                        // Not found - don't retry
                        let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                            .with_info(format!("HTTP status: {status}"));
                        return Err(MarkdownError::EnhancedNetworkError {
                            kind: NetworkErrorKind::ServerError(status.as_u16()),
                            context,
                        });
                    } else if status.is_server_error() || status == 429 {
                        // Server errors and rate limiting - these are retryable
                        if attempt == self.max_retries {
                            let network_kind = if status == 429 {
                                NetworkErrorKind::RateLimited
                            } else {
                                NetworkErrorKind::ServerError(status.as_u16())
                            };
                            let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                                .with_info(format!(
                                    "HTTP status: {} after {} attempts",
                                    status,
                                    self.max_retries + 1
                                ));
                            return Err(MarkdownError::EnhancedNetworkError {
                                kind: network_kind,
                                context,
                            });
                        }
                        // Fall through to retry logic
                    } else {
                        // Other client errors - don't retry
                        let context = ErrorContext::new(url, "HTTP request", "HttpClient")
                            .with_info(format!("HTTP status: {status}"));
                        return Err(MarkdownError::EnhancedNetworkError {
                            kind: NetworkErrorKind::ServerError(status.as_u16()),
                            context,
                        });
                    }
                }
                Err(e) => {
                    last_error = Some(e);

                    // Don't retry on the last attempt
                    if attempt == self.max_retries {
                        break;
                    }
                }
            }

            // Calculate delay with exponential backoff
            let delay = self.base_delay * 2_u32.pow(attempt);
            sleep(delay).await;
        }

        // If we reach here, all attempts failed with network errors
        let error = last_error.unwrap();
        Err(self.map_reqwest_error(error, url))
    }

    /// Maps reqwest errors to MarkdownError variants with context.
    fn map_reqwest_error(&self, error: reqwest::Error, url: &str) -> MarkdownError {
        let url_from_error = error
            .url()
            .map(|u| u.to_string())
            .unwrap_or_else(|| url.to_string());

        if error.is_timeout() {
            let context = ErrorContext::new(&url_from_error, "HTTP request", "HttpClient")
                .with_info("Request timeout");
            MarkdownError::EnhancedNetworkError {
                kind: NetworkErrorKind::Timeout,
                context,
            }
        } else if error.is_connect() {
            let context = ErrorContext::new(&url_from_error, "HTTP request", "HttpClient")
                .with_info(format!("Connection error: {error}"));
            MarkdownError::EnhancedNetworkError {
                kind: NetworkErrorKind::ConnectionFailed,
                context,
            }
        } else if error.is_request() {
            let context =
                ErrorContext::new(&url_from_error, "HTTP request validation", "HttpClient")
                    .with_info(format!("Request error: {error}"));
            MarkdownError::ValidationError {
                kind: ValidationErrorKind::InvalidUrl,
                context,
            }
        } else {
            let context = ErrorContext::new(&url_from_error, "HTTP request", "HttpClient")
                .with_info(format!("Request failed: {error}"));
            MarkdownError::EnhancedNetworkError {
                kind: NetworkErrorKind::ConnectionFailed,
                context,
            }
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_http_client_new() {
        let client = HttpClient::new();
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.base_delay, Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_get_text_success() {
        // Setup mock server
        let mock_server = MockServer::start().await;
        let expected_body = "Hello, World!";

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string(expected_body))
            .mount(&mock_server)
            .await;

        // Test the client
        let client = HttpClient::new();
        let url = format!("{}/test", mock_server.uri());
        let result = client.get_text(&url).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_body);
    }

    #[tokio::test]
    async fn test_get_bytes_success() {
        // Setup mock server
        let mock_server = MockServer::start().await;
        let expected_body = b"Binary data";

        Mock::given(method("GET"))
            .and(path("/binary"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(expected_body))
            .mount(&mock_server)
            .await;

        // Test the client
        let client = HttpClient::new();
        let url = format!("{}/binary", mock_server.uri());
        let result = client.get_bytes(&url).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), expected_body);
    }

    #[tokio::test]
    async fn test_invalid_url_error() {
        let client = HttpClient::new();
        let result = client.get_text("not-a-valid-url").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ValidationError { kind, context } => {
                assert_eq!(kind, ValidationErrorKind::InvalidUrl);
                assert_eq!(context.url, "not-a-valid-url");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_non_http_scheme_error() {
        let client = HttpClient::new();
        let result = client.get_text("ftp://example.com/file").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ValidationError { kind, context } => {
                assert_eq!(kind, ValidationErrorKind::InvalidUrl);
                assert_eq!(context.url, "ftp://example.com/file");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_http_404_error() {
        // Setup mock server
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/notfound"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        // Test the client
        let client = HttpClient::new();
        let url = format!("{}/notfound", mock_server.uri());
        let result = client.get_text(&url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, context: _ } => match kind {
                NetworkErrorKind::ServerError(status) => {
                    assert_eq!(status, 404);
                }
                _ => panic!("Expected ServerError(404)"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_http_401_auth_error() {
        // Setup mock server
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/secure"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        // Test the client
        let client = HttpClient::new();
        let url = format!("{}/secure", mock_server.uri());
        let result = client.get_text(&url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::AuthenticationError { kind, context: _ } => {
                assert_eq!(kind, AuthErrorKind::MissingToken);
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[tokio::test]
    async fn test_http_403_auth_error() {
        // Setup mock server
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/forbidden"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        // Test the client
        let client = HttpClient::new();
        let url = format!("{}/forbidden", mock_server.uri());
        let result = client.get_text(&url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::AuthenticationError { kind, context: _ } => {
                assert_eq!(kind, AuthErrorKind::PermissionDenied);
            }
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[tokio::test]
    async fn test_retry_logic_eventual_success() {
        // Setup mock server that fails twice then succeeds
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Success!"))
            .mount(&mock_server)
            .await;

        // Test the client - should succeed after retries
        let mut client = HttpClient::new();
        client.base_delay = Duration::from_millis(10); // Speed up test
        let url = format!("{}/flaky", mock_server.uri());
        let result = client.get_text(&url).await;

        assert!(
            result.is_ok(),
            "Expected success but got error: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap(), "Success!");
    }

    #[tokio::test]
    async fn test_retry_logic_max_attempts_exceeded() {
        // Setup mock server that always fails
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/always_fails"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        // Test the client - should fail after max retries
        let mut client = HttpClient::new();
        client.base_delay = Duration::from_millis(10); // Speed up test
        let url = format!("{}/always_fails", mock_server.uri());
        let result = client.get_text(&url).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, context: _ } => match kind {
                NetworkErrorKind::ServerError(status) => {
                    assert_eq!(status, 500);
                }
                _ => panic!("Expected ServerError(500)"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[test]
    fn test_default_implementation() {
        let client = HttpClient::default();
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.base_delay, Duration::from_secs(1));
    }
}
