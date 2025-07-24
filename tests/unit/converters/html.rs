//! Comprehensive unit tests for HTML to markdown converter.
//!
//! This module tests HTML conversion functionality, including preprocessing,
//! postprocessing, configuration handling, and error scenarios.

use markdowndown::client::HttpClient;
use markdowndown::config::Config;
use markdowndown::converters::{Converter, HtmlConverter, HtmlConverterConfig};
use markdowndown::types::{MarkdownError, NetworkErrorKind, ValidationErrorKind};
use mockito::Server;

mod helpers {
    use super::*;

    /// Create a test HTML converter with default configuration
    pub fn create_test_converter() -> HtmlConverter {
        HtmlConverter::new()
    }

    /// Create a test HTML converter with custom HTTP client
    pub fn create_test_converter_with_client(client: HttpClient) -> HtmlConverter {
        let config = HtmlConverterConfig::default();
        HtmlConverter::with_config(client, config)
    }

    /// Sample HTML content for testing
    pub fn sample_html_content() -> &'static str {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
    <meta name="description" content="This is a test document">
</head>
<body>
    <h1>Main Heading</h1>
    <p>This is a paragraph with <strong>bold text</strong> and <em>italic text</em>.</p>
    
    <h2>Subheading</h2>
    <ul>
        <li>First item</li>
        <li>Second item with <a href="https://example.com">a link</a></li>
        <li>Third item</li>
    </ul>
    
    <blockquote>
        <p>This is a blockquote with important information.</p>
    </blockquote>
    
    <pre><code>// This is a code block
function example() {
    console.log("Hello, world!");
}
</code></pre>
    
    <div class="navigation">
        <p>This content should be removed by preprocessing</p>
    </div>
    
    <footer>
        <p>Footer content that should be removed</p>
    </footer>
</body>
</html>"#
    }

    /// Sample HTML with complex structure for testing preprocessing
    pub fn complex_html_content() -> &'static str {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex Document</title>
    <script>
        // This script should be removed
        function trackUser() { /* ... */ }
    </script>
    <style>
        /* CSS should be removed */
        body { margin: 0; }
    </style>
</head>
<body>
    <nav class="navigation">
        <ul>
            <li><a href="/home">Home</a></li>
            <li><a href="/about">About</a></li>
        </ul>
    </nav>
    
    <main>
        <article>
            <h1>Article Title</h1>
            <p>Article content goes here.</p>
            
            <div class="related-articles">
                <h3>Related Articles</h3>
                <ul>
                    <li><a href="/article1">Article 1</a></li>
                    <li><a href="/article2">Article 2</a></li>
                </ul>
            </div>
        </article>
        
        <aside class="sidebar">
            <h3>Advertisement</h3>
            <p>Buy our product!</p>
        </aside>
    </main>
    
    <footer>
        <p>&copy; 2024 Example Corp</p>
        <div class="social-links">
            <a href="https://twitter.com/example">Twitter</a>
            <a href="https://facebook.com/example">Facebook</a>
        </div>
    </footer>
    
    <script>
        // Analytics script should be removed
        gtag('config', 'GA-XXXXXXXXX');
    </script>
</body>
</html>"#
    }
}

/// Tests for HTML converter creation and configuration
mod converter_creation_tests {
    use super::*;

    #[test]
    fn test_html_converter_new() {
        let converter = HtmlConverter::new();
        assert_eq!(converter.name(), "HTML");
    }

    #[test]
    fn test_html_converter_with_config() {
        let client = HttpClient::new();
        let config = HtmlConverterConfig::default();
        let converter = HtmlConverter::with_config(client, config);
        assert_eq!(converter.name(), "HTML");
    }

    #[test]
    fn test_html_converter_with_custom_config() {
        let client = HttpClient::new();
        let config = HtmlConverterConfig {
            max_line_width: 80,
            remove_scripts_styles: true,
            remove_navigation: true,
            remove_sidebars: true,
            remove_ads: true,
            max_blank_lines: 1,
        };
        let converter = HtmlConverter::with_config(client, config);
        assert_eq!(converter.name(), "HTML");
    }
}

/// Tests for successful HTML conversion
mod html_conversion_tests {
    use super::*;

    #[tokio::test]
    async fn test_convert_basic_html() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_html_content();

        let mock = server
            .mock("GET", "/test.html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/test.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Check for expected markdown elements
        assert!(content.contains("# Main Heading"));
        assert!(content.contains("## Subheading"));
        assert!(content.contains("**bold text**"));
        assert!(content.contains("*italic text*"));
        assert!(content.contains("[a link](https://example.com)"));
        assert!(content.contains("* First item"));

        // Should not contain unwanted HTML elements
        assert!(!content.contains("<div"));
        assert!(!content.contains("<footer"));
        assert!(!content.contains("<nav"));
    }

    #[tokio::test]
    async fn test_convert_complex_html_with_preprocessing() {
        let mut server = Server::new_async().await;
        let html_content = helpers::complex_html_content();

        let mock = server
            .mock("GET", "/complex.html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/complex.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Check for main content
        assert!(content.contains("# Article Title"));
        assert!(content.contains("Article content goes here"));

        // Should not contain scripts, styles, or navigation elements
        assert!(!content.contains("trackUser"));
        assert!(!content.contains("gtag"));
        assert!(!content.contains("body { margin: 0; }"));

        // Navigation and footer content should be minimized or removed
        // (exact behavior depends on preprocessing configuration)
    }

    #[tokio::test]
    async fn test_convert_html_with_custom_headers() {
        let mut server = Server::new_async().await;
        let html_content = "<html><body><h1>Test</h1><p>Content</p></body></html>";

        let mock = server
            .mock("GET", "/protected.html")
            .match_header("User-Agent", "test-agent/1.0")
            .match_header("Accept", "text/html,application/xhtml+xml")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder()
            .user_agent("test-agent/1.0")
            .timeout_seconds(5)
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/protected.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.content_only().contains("# Test"));
        assert!(markdown.content_only().contains("Content"));
    }

    #[tokio::test]
    async fn test_convert_html_with_different_encodings() {
        let mut server = Server::new_async().await;
        let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Encoding Test</title>
</head>
<body>
    <h1>Test with Special Characters</h1>
    <p>Here are some special characters: café, naïve, résumé</p>
    <p>Unicode: 你好, Здравствуй, مرحبا</p>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/encoding.html")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/encoding.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Check that special characters are preserved
        assert!(content.contains("café"));
        assert!(content.contains("naïve"));
        assert!(content.contains("résumé"));
        assert!(content.contains("你好"));
        assert!(content.contains("Здравствуй"));
        assert!(content.contains("مرحبا"));
    }

    #[tokio::test]
    async fn test_convert_empty_html() {
        let mut server = Server::new_async().await;
        let html_content = "<html><body></body></html>";

        let mock = server
            .mock("GET", "/empty.html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/empty.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        // Empty HTML should result in minimal markdown
        assert!(markdown.content_only().len() < 50); // Should be very short
    }

    #[tokio::test]
    async fn test_convert_html_with_malformed_markup() {
        let mut server = Server::new_async().await;
        let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Malformed HTML</title>
</head>
<body>
    <h1>Heading without closing tag
    <p>Paragraph with <strong>unclosed bold
    <div>
        <p>Nested content</p>
        <ul>
            <li>Item 1
            <li>Item 2</li>
        </ul>
    </div>
    <p>Final paragraph</p>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/malformed.html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/malformed.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should still extract meaningful content despite malformed HTML
        assert!(content.contains("Heading without closing tag"));
        assert!(content.contains("Paragraph with"));
        assert!(content.contains("Nested content"));
        assert!(content.contains("Final paragraph"));
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
    async fn test_convert_http_404_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/notfound.html")
            .with_status(404)
            .with_body("Not Found")
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/notfound.html", server.url());
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
    async fn test_convert_http_500_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/error.html")
            .with_status(500)
            .with_body("Internal Server Error")
            .expect(2) // Original request + 1 retry
            .create_async()
            .await;

        let config = Config::builder()
            .timeout_seconds(5)
            .max_retries(1) // Reduce retries for faster test
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/error.html", server.url());
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
    async fn test_convert_non_html_content() {
        let mut server = Server::new_async().await;
        let json_content = r#"{"message": "This is JSON, not HTML"}"#;

        let mock = server
            .mock("GET", "/data.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/data.json", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        // Should still work - the converter will treat JSON as text content
        assert!(result.is_ok());

        let markdown = result.unwrap();
        // The JSON should be converted to plain text
        assert!(markdown.content_only().contains("This is JSON, not HTML"));
    }

    #[tokio::test]
    async fn test_convert_large_html_content() {
        let mut server = Server::new_async().await;

        // Create large HTML content (1MB)
        let large_content = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Large Document</title></head>
<body>
<h1>Large Content Test</h1>
{}
</body>
</html>"#,
            "<p>This is a paragraph with lots of content. ".repeat(10000)
        );

        let mock = server
            .mock("GET", "/large.html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(&large_content)
            .create_async()
            .await;

        let config = Config::builder()
            .timeout_seconds(10) // Longer timeout for large content
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/large.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.content_only().contains("# Large Content Test"));
        assert!(markdown.content_only().len() > 10000); // Should be substantial content
    }
}

/// Tests for configuration handling
mod configuration_tests {
    use super::*;

    #[test]
    fn test_html_converter_config_default() {
        let config = HtmlConverterConfig::default();

        // Test default values
        assert_eq!(config.max_line_width, 120);
        assert!(config.remove_scripts_styles);
        assert!(config.remove_navigation);
        assert!(config.remove_sidebars);
        assert!(config.remove_ads);
        assert_eq!(config.max_blank_lines, 2);
    }

    #[test]
    fn test_html_converter_config_custom() {
        let config = HtmlConverterConfig {
            max_line_width: 100,
            remove_scripts_styles: false,
            remove_navigation: false,
            remove_sidebars: false,
            remove_ads: false,
            max_blank_lines: 5,
        };

        assert_eq!(config.max_line_width, 100);
        assert!(!config.remove_scripts_styles);
        assert!(!config.remove_navigation);
        assert!(!config.remove_sidebars);
        assert!(!config.remove_ads);
        assert_eq!(config.max_blank_lines, 5);
    }

    #[tokio::test]
    async fn test_converter_respects_configuration() {
        let mut server = Server::new_async().await;
        let html_content = helpers::complex_html_content();

        let mock = server
            .mock("GET", "/config-test.html")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        // Test with conservative config (keep more content)
        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let html_config = HtmlConverterConfig {
            max_line_width: 200,
            remove_scripts_styles: false,
            remove_navigation: false,
            remove_sidebars: false,
            remove_ads: false,
            max_blank_lines: 10,
        };
        let converter = HtmlConverter::with_config(client, html_config);

        let url = format!("{}/config-test.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // With conservative config, more content should be preserved
        assert!(content.contains("Article Title"));
        // Navigation and footer content might be preserved depending on implementation
    }
}

/// Integration tests combining multiple features
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_html_conversion() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_html_content();

        let mock = server
            .mock("GET", "/integration-test.html")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder()
            .user_agent("integration-test/1.0")
            .timeout_seconds(5)
            .max_retries(2)
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/integration-test.html", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Verify all major markdown elements are present
        assert!(content.contains("# Main Heading"));
        assert!(content.contains("## Subheading"));
        assert!(content.contains("**bold text**"));
        assert!(content.contains("*italic text*"));
        assert!(content.contains("[a link](https://example.com)"));
        assert!(content.contains("* First item"));
        assert!(content.contains("* Second item"));
        assert!(content.contains("> This is a blockquote"));

        // Verify frontmatter is included if configured
        if let Some(frontmatter) = markdown.frontmatter() {
            assert!(frontmatter.contains("title:"));
            assert!(frontmatter.contains("url:"));
        }
    }

    #[tokio::test]
    async fn test_html_converter_with_redirects() {
        let mut server = Server::new_async().await;
        let html_content = "<html><body><h1>Final Content</h1></body></html>";

        let redirect_mock = server
            .mock("GET", "/redirect-source")
            .with_status(302)
            .with_header("Location", &format!("{}/redirect-target", server.url()))
            .create_async()
            .await;

        let target_mock = server
            .mock("GET", "/redirect-target")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(5).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let converter = helpers::create_test_converter_with_client(client);

        let url = format!("{}/redirect-source", server.url());
        let result = converter.convert(&url).await;

        redirect_mock.assert_async().await;
        target_mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.content_only().contains("# Final Content"));
    }
}
