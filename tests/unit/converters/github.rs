//! Comprehensive unit tests for GitHub Issues/PRs to markdown converter.
//!
//! This module tests GitHub conversion functionality, including placeholder
//! behavior, URL handling, API integration, error scenarios, and content extraction.

use markdowndown::client::HttpClient;
use markdowndown::config::{Config, PlaceholderSettings};
use markdowndown::converters::placeholder::GitHubIssueConverter;
use markdowndown::converters::Converter;
use markdowndown::types::{MarkdownError, NetworkErrorKind, ValidationErrorKind};
use mockito::Server;

mod helpers {
    use super::*;

    /// Create a test GitHub converter with default settings
    pub fn create_test_converter() -> GitHubIssueConverter {
        GitHubIssueConverter::new()
    }

    /// Create a test GitHub converter with custom client and settings
    pub fn create_test_converter_with_config(
        client: HttpClient,
        settings: &PlaceholderSettings,
    ) -> GitHubIssueConverter {
        GitHubIssueConverter::with_client_and_settings(client, settings)
    }

    /// Sample GitHub URLs for testing
    pub fn sample_github_urls() -> Vec<&'static str> {
        vec![
            "https://github.com/owner/repo/issues/123",
            "https://github.com/microsoft/vscode/issues/42",
            "https://github.com/rust-lang/rust/issues/12345",
            "https://github.com/owner/repo/pull/456",
            "https://github.com/microsoft/vscode/pull/789",
            "https://github.com/rust-lang/rust/pull/98765",
            "https://github.com/facebook/react/issues/1",
            "https://github.com/nodejs/node/pull/999999",
        ]
    }

    /// Sample GitHub issue HTML content
    pub fn sample_github_issue_html() -> &'static str {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Feature Request: Add dark mode support · Issue #123 · owner/repo</title>
    <meta name="description" content="Add support for dark mode in the application interface">
</head>
<body>
    <div class="js-discussion">
        <div class="gh-header">
            <h1 class="gh-header-title">
                <span class="js-issue-title">Feature Request: Add dark mode support</span>
                <span class="State State--open">#123</span>
            </h1>
            
            <div class="gh-header-meta">
                <span class="author">opened this issue</span>
                <relative-time datetime="2024-01-15T10:30:00Z">on Jan 15</relative-time>
                <span class="labels">
                    <span class="IssueLabel" style="background-color: #d73a49; color: #ffffff;">enhancement</span>
                    <span class="IssueLabel" style="background-color: #0075ca; color: #ffffff;">feature-request</span>
                </span>
            </div>
        </div>
        
        <div class="comment-body">
            <h2>Problem Description</h2>
            <p>Currently, the application only supports light mode, which can be straining on the eyes during extended use, especially in low-light environments.</p>
            
            <h2>Proposed Solution</h2>
            <p>Add a dark mode toggle that:</p>
            <ul>
                <li>Switches the color scheme to dark colors</li>
                <li>Maintains readability and contrast</li>
                <li>Persists user preference across sessions</li>
                <li>Respects system dark mode preference</li>
            </ul>
            
            <h2>Additional Context</h2>
            <p>This feature has been requested by multiple users and would improve accessibility for users with light sensitivity.</p>
            
            <blockquote>
                <p><strong>Note:</strong> This would be a significant UI change that affects all components.</p>
            </blockquote>
            
            <h2>Acceptance Criteria</h2>
            <ol>
                <li>Dark mode toggle in settings/preferences</li>
                <li>All UI components properly styled for dark mode</li>
                <li>Smooth transition between light and dark modes</li>
                <li>Proper contrast ratios for accessibility compliance</li>
            </ol>
        </div>
        
        <div class="comment-reactions">
            <button class="btn-link">👍 12</button>
            <button class="btn-link">❤️ 5</button>
            <button class="btn-link">🚀 3</button>
        </div>
        
        <div class="timeline">
            <div class="timeline-comment">
                <div class="comment-body">
                    <p>Great idea! I'd love to work on this feature. Should we follow Material Design dark theme guidelines?</p>
                </div>
            </div>
            
            <div class="timeline-comment">
                <div class="comment-body">
                    <p>@contributor Yes, Material Design guidelines would be perfect. Let's also ensure we support high contrast mode.</p>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#
    }

    /// Sample GitHub pull request HTML content
    pub fn sample_github_pr_html() -> &'static str {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Fix: Resolve memory leak in data processing · Pull Request #456 · owner/repo</title>
    <meta name="description" content="This PR fixes a memory leak that occurs during large data processing operations">
</head>
<body>
    <div class="js-discussion">
        <div class="gh-header">
            <h1 class="gh-header-title">
                <span class="js-issue-title">Fix: Resolve memory leak in data processing</span>
                <span class="State State--merged">#456</span>
            </h1>
            
            <div class="gh-header-meta">
                <span class="author">wants to merge 3 commits into</span>
                <span class="base-ref">main</span>
                <span>from</span>
                <span class="head-ref">fix/memory-leak</span>
                <relative-time datetime="2024-01-20T14:45:00Z">5 days ago</relative-time>
            </div>
        </div>
        
        <div class="comment-body">
            <h2>Description</h2>
            <p>This pull request addresses a critical memory leak identified in the data processing pipeline. The leak was causing the application to consume excessive memory during large batch operations.</p>
            
            <h2>Changes Made</h2>
            <ul>
                <li>Fixed buffer management in <code>DataProcessor.processLargeDataset()</code></li>
                <li>Added proper cleanup for temporary objects</li>
                <li>Implemented resource pooling for frequently allocated objects</li>
                <li>Added memory usage monitoring and alerts</li>
            </ul>
            
            <h2>Testing</h2>
            <ul>
                <li>✅ All existing unit tests pass</li>
                <li>✅ Memory usage tests added and passing</li>
                <li>✅ Load testing with 1M+ records shows stable memory usage</li>
                <li>✅ Manual testing confirms leak is resolved</li>
            </ul>
            
            <h2>Performance Impact</h2>
            <table>
                <tr>
                    <th>Metric</th>
                    <th>Before</th>
                    <th>After</th>
                    <th>Improvement</th>
                </tr>
                <tr>
                    <td>Memory Usage (1M records)</td>
                    <td>2.5GB</td>
                    <td>150MB</td>
                    <td>-94%</td>
                </tr>
                <tr>
                    <td>Processing Time</td>
                    <td>45s</td>
                    <td>42s</td>
                    <td>-7%</td>
                </tr>
            </table>
            
            <h2>Breaking Changes</h2>
            <p>None. This is a backward-compatible fix.</p>
            
            <h2>Checklist</h2>
            <ul>
                <li>✅ Code follows project style guidelines</li>
                <li>✅ Self-review completed</li>
                <li>✅ Unit tests added/updated</li>
                <li>✅ Documentation updated</li>
                <li>✅ No merge conflicts</li>
            </ul>
        </div>
        
        <div class="pr-review-tools">
            <div class="diffbar">
                <span class="diffstat">
                    <span class="text-green">+127</span>
                    <span class="text-red">−89</span>
                </span>
                <span class="files-changed">5 files changed</span>
            </div>
        </div>
        
        <div class="timeline">
            <div class="timeline-comment">
                <div class="comment-body">
                    <p>LGTM! Excellent work on identifying and fixing this leak. The performance improvements are impressive.</p>
                </div>
            </div>
            
            <div class="timeline-comment">
                <div class="comment-body">
                    <p>Thanks for the thorough testing. Merging now.</p>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#
    }
}

/// Tests for GitHub converter creation
mod converter_creation_tests {
    use super::*;

    #[test]
    fn test_github_converter_new() {
        let converter = GitHubIssueConverter::new();
        assert_eq!(converter.name(), "GitHub Issue");
    }

    #[test]
    fn test_github_converter_with_client() {
        let client = HttpClient::new();
        let converter = GitHubIssueConverter::with_client(client);
        assert_eq!(converter.name(), "GitHub Issue");
    }

    #[test]
    fn test_github_converter_with_client_and_settings() {
        let client = HttpClient::new();
        let settings = PlaceholderSettings {
            max_content_length: 3000,
        };
        let converter = GitHubIssueConverter::with_client_and_settings(client, &settings);
        assert_eq!(converter.name(), "GitHub Issue");
    }
}

/// Tests for successful GitHub conversion
mod github_conversion_tests {
    use super::*;

    #[tokio::test]
    async fn test_convert_github_issue() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_github_issue_html();

        let mock = server
            .mock("GET", "/owner/repo/issues/123")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 5000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/123", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Since this is a placeholder converter, it should include basic information
        assert!(content.contains("GitHub") || content.contains("Issue"));
        assert!(content.contains(&url));

        // Should contain some extracted content
        assert!(
            content.contains("Feature Request")
                || content.contains("dark mode")
                || content.contains("Problem Description")
                || content.len() > 200
        );
    }

    #[tokio::test]
    async fn test_convert_github_pull_request() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_github_pr_html();

        let mock = server
            .mock("GET", "/owner/repo/pull/456")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 6000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/pull/456", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should contain GitHub/PR information
        assert!(content.contains("GitHub") || content.contains("Pull Request"));
        assert!(content.contains(&url));

        // Should contain extracted PR content
        assert!(
            content.contains("memory leak")
                || content.contains("Fix:")
                || content.contains("Changes Made")
                || content.contains("Testing")
        );
    }

    #[tokio::test]
    async fn test_convert_github_issue_with_authentication() {
        let mut server = Server::new_async().await;
        let simple_issue = r#"<!DOCTYPE html>
<html>
<head><title>Private Issue #789 · owner/private-repo</title></head>
<body>
    <div class="js-discussion">
        <h1>Private Repository Issue</h1>
        <div class="comment-body">
            <p>This is a private issue that requires authentication to view.</p>
            <p>The issue contains sensitive information about internal processes.</p>
        </div>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/owner/private-repo/issues/789")
            .match_header("Authorization", "token test_github_token")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(simple_issue)
            .create_async()
            .await;

        let config = Config::builder()
            .github_token("test_github_token")
            .timeout_seconds(10)
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 2000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/private-repo/issues/789", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should extract private issue content
        assert!(
            content.contains("Private Repository Issue")
                || content.contains("private issue")
                || content.contains("authentication")
        );
    }

    #[tokio::test]
    async fn test_convert_github_issue_with_markdown_content() {
        let mut server = Server::new_async().await;
        let markdown_rich_issue = r#"<!DOCTYPE html>
<html>
<head><title>Bug Report: API Error Handling · Issue #999 · owner/repo</title></head>
<body>
    <div class="js-discussion">
        <h1>Bug Report: API Error Handling</h1>
        <div class="comment-body">
            <h2>Bug Description</h2>
            <p>The API error handling is not working correctly for <code>404</code> responses.</p>
            
            <h3>Steps to Reproduce</h3>
            <ol>
                <li>Make API call to non-existent endpoint</li>
                <li>Observe error response</li>
                <li>Check error handling in application</li>
            </ol>
            
            <h3>Expected Behavior</h3>
            <p>Should display user-friendly error message.</p>
            
            <h3>Actual Behavior</h3>
            <p>Application crashes with unhandled exception.</p>
            
            <h3>Code Sample</h3>
            <pre><code class="language-javascript">
// This causes the crash
fetch('/api/nonexistent')
  .then(response =&gt; response.json())
  .then(data =&gt; console.log(data))
  .catch(error =&gt; {
    // Error handling is missing here
    throw error;
  });
            </code></pre>
            
            <h3>Environment</h3>
            <ul>
                <li><strong>OS:</strong> macOS 14.0</li>
                <li><strong>Browser:</strong> Chrome 120.0</li>
                <li><strong>Node.js:</strong> v18.17.0</li>
                <li><strong>Package Version:</strong> v2.1.0</li>
            </ul>
            
            <blockquote>
                <p><strong>Priority:</strong> High - This affects production users</p>
            </blockquote>
        </div>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/owner/repo/issues/999")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(markdown_rich_issue)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 8000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/999", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should extract structured content
        assert!(content.contains("Bug Report") || content.contains("API Error"));
        assert!(content.contains("Steps to Reproduce") || content.contains("Expected Behavior"));
        assert!(content.contains("fetch") || content.contains("javascript"));
        assert!(content.contains("macOS") || content.contains("Chrome"));
    }

    #[tokio::test]
    async fn test_convert_with_content_length_limit() {
        let mut server = Server::new_async().await;

        // Create large GitHub issue content
        let large_content = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Large Issue · Issue #1000 · owner/repo</title></head>
<body>
    <div class="js-discussion">
        <h1>Large GitHub Issue</h1>
        <div class="comment-body">
            <p>{}</p>
        </div>
    </div>
</body>
</html>"#,
            "This is a very long issue description with lots of detail. ".repeat(500)
        );

        let mock = server
            .mock("GET", "/owner/repo/issues/1000")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(&large_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 800, // Small limit
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/1000", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Content should be truncated to the limit
        assert!(content.len() <= 1200); // Should be around the limit (plus some overhead)
        assert!(content.contains("GitHub") || content.contains("Issue"));
    }
}

/// Tests for error handling
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_convert_invalid_url() {
        let converter = helpers::create_test_converter();
        let result = converter.convert("not-a-valid-url").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::ValidationError { kind, .. } => {
                assert_eq!(kind, ValidationErrorKind::InvalidUrl);
            }
            _ => panic!("Expected ValidationError for invalid URL"),
        }
    }

    #[tokio::test]
    async fn test_convert_private_repo_access_denied() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/owner/private-repo/issues/123")
            .with_status(404) // GitHub returns 404 for private repos without access
            .with_body("Not Found")
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/private-repo/issues/123", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => match kind {
                NetworkErrorKind::ServerError(status) => {
                    assert_eq!(status, 404);
                }
                _ => panic!("Expected ServerError(404)"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_convert_issue_not_found() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/owner/repo/issues/999999")
            .with_status(404)
            .with_body("Issue not found")
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/999999", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => match kind {
                NetworkErrorKind::ServerError(status) => {
                    assert_eq!(status, 404);
                }
                _ => panic!("Expected ServerError(404)"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_convert_github_rate_limit() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/owner/repo/issues/123")
            .with_status(429)
            .with_header("X-RateLimit-Remaining", "0")
            .with_header("X-RateLimit-Reset", "1640995200")
            .with_body("API rate limit exceeded")
            .create_async()
            .await;

        let config = Config::builder()
            .timeout_seconds(10)
            .max_retries(0) // No retries for error handling test
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/123", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => {
                match kind {
                    NetworkErrorKind::RateLimited => {
                        // Expected rate limit error
                    }
                    _ => panic!("Expected RateLimited error"),
                }
            }
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_convert_github_server_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/owner/repo/issues/123")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let config = Config::builder()
            .timeout_seconds(10)
            .max_retries(0) // No retries for error handling test
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/123", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::EnhancedNetworkError { kind, .. } => match kind {
                NetworkErrorKind::ServerError(status) => {
                    assert_eq!(status, 500);
                }
                _ => panic!("Expected ServerError(500)"),
            },
            _ => panic!("Expected EnhancedNetworkError"),
        }
    }

    #[tokio::test]
    async fn test_convert_malformed_github_url() {
        let converter = helpers::create_test_converter();

        let malformed_urls = [
            "https://github.com/owner/repo/issues/",
            "https://github.com/owner/repo/pull/",
            "https://github.com/owner/repo/issues/abc",
            "https://github.com/owner/repo/pull/xyz",
            "https://github.com/owner/issues/123",
            "https://github.com/repo/issues/123",
        ];

        for url in malformed_urls {
            let result = converter.convert(url).await;
            // Some URLs might be rejected as invalid, others might result in 404s
            assert!(result.is_err(), "Should fail for malformed URL: {url}");
        }
    }
}

/// Tests for placeholder-specific functionality
mod placeholder_functionality_tests {
    use super::*;

    #[tokio::test]
    async fn test_placeholder_content_includes_github_info() {
        let mut server = Server::new_async().await;
        let simple_html = r#"<!DOCTYPE html>
<html>
<head><title>Test Issue #123 · owner/repo</title></head>
<body>
    <div class="js-discussion">
        <h1>Test Issue</h1>
        <div class="comment-body">
            <p>This is a test issue for placeholder testing.</p>
        </div>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/owner/repo/issues/123")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(simple_html)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/123", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should contain service identification
        assert!(content.contains("GitHub") || content.contains("Issue"));

        // Should contain the original URL
        assert!(content.contains(&url));

        // Should contain extracted content
        assert!(content.contains("Test Issue") || content.contains("placeholder testing"));

        // Should indicate this is a placeholder conversion
        assert!(
            content.contains("placeholder")
                || content.contains("preview")
                || content.contains("limited")
        );
    }

    #[tokio::test]
    async fn test_multiple_github_url_types_handled() {
        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        for url in helpers::sample_github_urls() {
            // Test that all URL types are accepted (even if they fail with network errors)
            let result = converter.convert(url).await;

            // Should either succeed or fail with a network error (not validation error)
            match result {
                Ok(_) => {
                    // Success is fine if we can connect
                }
                Err(MarkdownError::EnhancedNetworkError { .. }) => {
                    // Network error is expected since we're not mocking these
                }
                Err(MarkdownError::ValidationError { .. }) => {
                    panic!("Should not reject valid GitHub URL: {url}");
                }
                Err(e) => {
                    panic!("Unexpected error for URL {url}: {e:?}");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_github_converter_extracts_issue_metadata() {
        let mut server = Server::new_async().await;
        let issue_with_metadata = r#"<!DOCTYPE html>
<html>
<head><title>Enhancement: Improve Performance · Issue #555 · owner/repo</title></head>
<body>
    <div class="js-discussion">
        <div class="gh-header">
            <h1>Enhancement: Improve Performance</h1>
            <div class="gh-header-meta">
                <span class="State State--open">Open</span>
                <span class="labels">
                    <span class="IssueLabel">performance</span>
                    <span class="IssueLabel">enhancement</span>
                </span>
                <span class="assignees">Assigned to: @developer</span>
                <span class="milestone">Milestone: v2.0</span>
            </div>
        </div>
        
        <div class="comment-body">
            <p>We need to improve the performance of the search functionality.</p>
        </div>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/owner/repo/issues/555")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(issue_with_metadata)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 2000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/555", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should extract issue metadata
        assert!(content.contains("Enhancement") || content.contains("Improve Performance"));
        assert!(content.contains("performance") || content.contains("enhancement"));
        assert!(
            content.contains("Open") || content.contains("@developer") || content.contains("v2.0")
        );
        assert!(content.contains("search functionality"));
    }
}

/// Integration tests
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_github_conversion() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_github_issue_html();

        let mock = server
            .mock("GET", "/owner/repo/issues/123")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_header("content-length", &html_content.len().to_string())
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder()
            .user_agent("integration-test/1.0")
            .timeout_seconds(10)
            .max_retries(3)
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 10000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/123", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should contain extracted content
        assert!(content.contains("Feature Request") || content.contains("dark mode"));
        assert!(content.contains("Problem Description") || content.contains("Proposed Solution"));
        assert!(content.contains("Acceptance Criteria") || content.contains("enhancement"));

        // Should identify as GitHub content
        assert!(content.contains("GitHub") || content.contains("Issue"));

        // Verify frontmatter if present
        if let Some(frontmatter) = markdown.frontmatter() {
            assert!(frontmatter.contains("title:"));
            assert!(frontmatter.contains("url:"));
            assert!(frontmatter.contains("converter: GitHubIssueConverter"));
        }
    }

    #[tokio::test]
    async fn test_github_conversion_with_retry_logic() {
        let mut server = Server::new_async().await;

        // First request fails, second succeeds
        let failing_mock = server
            .mock("GET", "/owner/repo/issues/retry-test")
            .with_status(503)
            .expect(2) // Should be called twice (initial + 1 retry)
            .create_async()
            .await;

        let success_mock = server
            .mock("GET", "/owner/repo/issues/retry-test")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(r#"<html><body><div class="js-discussion"><h1>Issue after retry</h1></div></body></html>"#)
            .expect(1)
            .create_async().await;

        let config = Config::builder()
            .timeout_seconds(10)
            .max_retries(3)
            .retry_delay(std::time::Duration::from_millis(10))
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/owner/repo/issues/retry-test", server.url());
        let result = converter.convert(&url).await;

        failing_mock.assert_async().await;
        success_mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.content_only().contains("Issue after retry"));
    }
}
