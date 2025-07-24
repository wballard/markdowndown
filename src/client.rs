//! HTTP client wrapper for network operations.
//!
//! This module provides a robust HTTP client with retry logic, timeout handling,
//! and proper error mapping for the markdowndown library.

use crate::types::MarkdownError;
use bytes::Bytes;
use reqwest::{Client, Response};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

/// HTTP client configuration with retry logic and error handling.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    max_retries: u32,
    base_delay: Duration,
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
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("markdowndown/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        HttpClient {
            client,
            max_retries: 3,
            base_delay: Duration::from_secs(1),
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
    pub async fn get_text(&self, url: &str) -> Result<String, MarkdownError> {
        let response = self.retry_request(url).await?;
        let text = response
            .text()
            .await
            .map_err(|e| MarkdownError::NetworkError {
                message: format!("Failed to read response body: {e}"),
            })?;
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
        let bytes = response
            .bytes()
            .await
            .map_err(|e| MarkdownError::NetworkError {
                message: format!("Failed to read response body: {e}"),
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
        let text = response
            .text()
            .await
            .map_err(|e| MarkdownError::NetworkError {
                message: format!("Failed to read response body: {e}"),
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
        let parsed_url = Url::parse(url).map_err(|_| MarkdownError::InvalidUrl {
            url: url.to_string(),
        })?;

        // Ensure URL uses HTTP or HTTPS
        match parsed_url.scheme() {
            "http" | "https" => {}
            _ => {
                return Err(MarkdownError::InvalidUrl {
                    url: url.to_string(),
                })
            }
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let mut request = self.client.get(url);

            // Add custom headers
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
                        return Err(MarkdownError::AuthError {
                            message: format!(
                                "Authentication failed: {} {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("Unknown")
                            ),
                        });
                    } else if status == 404 {
                        // Not found - don't retry
                        return Err(MarkdownError::NetworkError {
                            message: format!(
                                "HTTP error: {} {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("Unknown")
                            ),
                        });
                    } else if status.is_server_error() || status == 429 {
                        // Server errors and rate limiting - these are retryable
                        if attempt == self.max_retries {
                            return Err(MarkdownError::NetworkError {
                                message: format!(
                                    "HTTP error: {} {}",
                                    status.as_u16(),
                                    status.canonical_reason().unwrap_or("Unknown")
                                ),
                            });
                        }
                        // Fall through to retry logic
                    } else {
                        // Other client errors - don't retry
                        return Err(MarkdownError::NetworkError {
                            message: format!(
                                "HTTP error: {} {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("Unknown")
                            ),
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
        Err(self.map_reqwest_error(error))
    }

    /// Internal method to perform HTTP requests with retry logic.
    ///
    /// Implements exponential backoff for transient failures.
    async fn retry_request(&self, url: &str) -> Result<Response, MarkdownError> {
        // Validate URL format
        let parsed_url = Url::parse(url).map_err(|_| MarkdownError::InvalidUrl {
            url: url.to_string(),
        })?;

        // Ensure URL uses HTTP or HTTPS
        match parsed_url.scheme() {
            "http" | "https" => {}
            _ => {
                return Err(MarkdownError::InvalidUrl {
                    url: url.to_string(),
                })
            }
        }

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status = response.status();

                    // Check if this is a success or non-retryable error
                    if status.is_success() {
                        return Ok(response);
                    } else if status == 401 || status == 403 {
                        // Auth errors - don't retry
                        return Err(MarkdownError::AuthError {
                            message: format!(
                                "Authentication failed: {} {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("Unknown")
                            ),
                        });
                    } else if status == 404 {
                        // Not found - don't retry
                        return Err(MarkdownError::NetworkError {
                            message: format!(
                                "HTTP error: {} {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("Unknown")
                            ),
                        });
                    } else if status.is_server_error() || status == 429 {
                        // Server errors and rate limiting - these are retryable
                        if attempt == self.max_retries {
                            return Err(MarkdownError::NetworkError {
                                message: format!(
                                    "HTTP error: {} {}",
                                    status.as_u16(),
                                    status.canonical_reason().unwrap_or("Unknown")
                                ),
                            });
                        }
                        // Fall through to retry logic
                    } else {
                        // Other client errors - don't retry
                        return Err(MarkdownError::NetworkError {
                            message: format!(
                                "HTTP error: {} {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("Unknown")
                            ),
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
        Err(self.map_reqwest_error(error))
    }

    /// Maps reqwest errors to MarkdownError variants.
    fn map_reqwest_error(&self, error: reqwest::Error) -> MarkdownError {
        if error.is_timeout() {
            MarkdownError::NetworkError {
                message: "Request timeout".to_string(),
            }
        } else if error.is_connect() {
            MarkdownError::NetworkError {
                message: format!("Connection failed: {error}"),
            }
        } else if error.is_request() {
            MarkdownError::InvalidUrl {
                url: error
                    .url()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
            }
        } else {
            MarkdownError::NetworkError {
                message: format!("HTTP request failed: {error}"),
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
            MarkdownError::InvalidUrl { url } => {
                assert_eq!(url, "not-a-valid-url");
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[tokio::test]
    async fn test_non_http_scheme_error() {
        let client = HttpClient::new();
        let result = client.get_text("ftp://example.com/file").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::InvalidUrl { url } => {
                assert_eq!(url, "ftp://example.com/file");
            }
            _ => panic!("Expected InvalidUrl error"),
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
            MarkdownError::NetworkError { message } => {
                assert!(message.contains("404"));
            }
            _ => panic!("Expected NetworkError"),
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
            MarkdownError::AuthError { message } => {
                assert!(message.contains("401"));
            }
            _ => panic!("Expected AuthError"),
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
            MarkdownError::AuthError { message } => {
                assert!(message.contains("403"));
            }
            _ => panic!("Expected AuthError"),
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
            MarkdownError::NetworkError { message } => {
                assert!(message.contains("500"));
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_default_implementation() {
        let client = HttpClient::default();
        assert_eq!(client.max_retries, 3);
        assert_eq!(client.base_delay, Duration::from_secs(1));
    }
}
