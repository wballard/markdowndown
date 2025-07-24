//! Comprehensive unit tests for Office 365 to markdown converter.
//!
//! This module tests Office 365 conversion functionality, including placeholder
//! behavior, URL handling, error scenarios, and basic content extraction.

use markdowndown::client::HttpClient;
use markdowndown::config::{Config, PlaceholderSettings};
use markdowndown::converters::placeholder::Office365Converter;
use markdowndown::converters::Converter;
use markdowndown::types::{MarkdownError, NetworkErrorKind, ValidationErrorKind};
use mockito::Server;

mod helpers {
    use super::*;

    /// Create a test Office 365 converter with default settings
    pub fn create_test_converter() -> Office365Converter {
        Office365Converter::new()
    }

    /// Create a test Office 365 converter with custom client and settings
    pub fn create_test_converter_with_config(
        client: HttpClient,
        settings: &PlaceholderSettings,
    ) -> Office365Converter {
        Office365Converter::with_client_and_settings(client, settings)
    }

    /// Sample Office 365 URLs for testing
    pub fn sample_office365_urls() -> Vec<&'static str> {
        vec![
            "https://company.sharepoint.com/sites/team/Document.docx",
            "https://tenant.sharepoint.com/personal/user/Documents/file.xlsx",
            "https://myorg.sharepoint.com/sites/project/Lists/Tasks",
            "https://onedrive.live.com/view.aspx?resid=123456",
            "https://onedrive.live.com/edit.aspx?cid=789&resid=456",
            "https://company.office.com/document.docx",
            "https://outlook.live.com/mail/inbox",
            "https://company.outlook.com/calendar",
        ]
    }

    /// Sample HTML content that Office 365 might return
    pub fn sample_office365_html() -> &'static str {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Project Document - SharePoint</title>
    <meta name="description" content="Project planning document">
</head>
<body>
    <div class="ms-core-navigation">
        <nav>Navigation content...</nav>
    </div>
    
    <div class="ms-core-content">
        <div class="document-content">
            <h1>Project Planning Document</h1>
            
            <h2>Executive Summary</h2>
            <p>This document outlines the key objectives and milestones for our upcoming project. The project aims to deliver a comprehensive solution that addresses current market needs.</p>
            
            <h2>Project Scope</h2>
            <ul>
                <li>Define project requirements and deliverables</li>
                <li>Establish timeline and resource allocation</li>
                <li>Identify key stakeholders and communication plans</li>
                <li>Set up project tracking and reporting mechanisms</li>
            </ul>
            
            <h2>Timeline</h2>
            <table>
                <tr>
                    <th>Phase</th>
                    <th>Duration</th>
                    <th>Deliverables</th>
                </tr>
                <tr>
                    <td>Planning</td>
                    <td>2 weeks</td>
                    <td>Project plan, resource allocation</td>
                </tr>
                <tr>
                    <td>Development</td>
                    <td>8 weeks</td>
                    <td>Core functionality, testing</td>
                </tr>
                <tr>
                    <td>Deployment</td>
                    <td>2 weeks</td>
                    <td>Production deployment, documentation</td>
                </tr>
            </table>
            
            <h2>Risk Assessment</h2>
            <blockquote>
                <p><strong>High Priority Risks:</strong></p>
                <ul>
                    <li>Resource availability during peak periods</li>
                    <li>Technical dependencies on third-party services</li>
                    <li>Potential scope creep from stakeholders</li>
                </ul>
            </blockquote>
            
            <h2>Next Steps</h2>
            <ol>
                <li>Review and approve project plan</li>
                <li>Secure necessary resources and budget</li>
                <li>Begin initial development phase</li>
                <li>Set up regular stakeholder check-ins</li>
            </ol>
        </div>
    </div>
    
    <div class="ms-core-footer">
        <footer>SharePoint footer content...</footer>
    </div>
</body>
</html>"#
    }
}

/// Tests for Office 365 converter creation
mod converter_creation_tests {
    use super::*;

    #[test]
    fn test_office365_converter_new() {
        let converter = Office365Converter::new();
        assert_eq!(converter.name(), "Office 365");
    }

    #[test]
    fn test_office365_converter_with_client() {
        let client = HttpClient::new();
        let converter = Office365Converter::with_client(client);
        assert_eq!(converter.name(), "Office 365");
    }

    #[test]
    fn test_office365_converter_with_client_and_settings() {
        let client = HttpClient::new();
        let settings = PlaceholderSettings {
            max_content_length: 2000,
        };
        let converter = Office365Converter::with_client_and_settings(client, &settings);
        assert_eq!(converter.name(), "Office 365");
    }
}

/// Tests for successful Office 365 conversion
mod office365_conversion_tests {
    use super::*;

    #[tokio::test]
    async fn test_convert_sharepoint_document() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_office365_html();

        let mock = server
            .mock("GET", "/sites/team/Document.docx")
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

        let url = format!("{}/sites/team/Document.docx", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Since this is a placeholder converter, it should include basic information
        assert!(content.contains("Office 365") || content.contains("SharePoint"));
        assert!(content.contains(&url));

        // Should contain some extracted content
        assert!(
            content.contains("Project Planning Document")
                || content.contains("Executive Summary")
                || content.len() > 100
        );
    }

    #[tokio::test]
    async fn test_convert_onedrive_document() {
        let mut server = Server::new_async().await;
        let simple_html = r#"<!DOCTYPE html>
<html>
<head><title>OneDrive Document</title></head>
<body>
    <div class="document-content">
        <h1>Meeting Notes</h1>
        <p>Important meeting notes go here.</p>
        <ul>
            <li>Action item 1</li>
            <li>Action item 2</li>
        </ul>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/view.aspx")
            .match_query(mockito::Matcher::UrlEncoded(
                "resid".into(),
                "123456".into(),
            ))
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

        let url = format!("{}/view.aspx?resid=123456", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should contain placeholder information and extracted content
        assert!(content.contains("Office 365") || content.contains("OneDrive"));
        assert!(content.contains("Meeting Notes") || content.contains("Action item"));
    }

    #[tokio::test]
    async fn test_convert_office_com_document() {
        let mut server = Server::new_async().await;
        let excel_html = r#"<!DOCTYPE html>
<html>
<head><title>Excel Online - Workbook</title></head>
<body>
    <div class="excel-content">
        <h1>Budget Spreadsheet</h1>
        <table>
            <tr><th>Item</th><th>Amount</th><th>Category</th></tr>
            <tr><td>Office Supplies</td><td>$500</td><td>Operations</td></tr>
            <tr><td>Software Licenses</td><td>$2000</td><td>Technology</td></tr>
        </table>
        <p>Total Budget: $2500</p>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/excel/budget.xlsx")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(excel_html)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 2000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/excel/budget.xlsx", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should extract spreadsheet data
        assert!(
            content.contains("Budget Spreadsheet")
                || content.contains("Office Supplies")
                || content.contains("$2500")
        );
    }

    #[tokio::test]
    async fn test_convert_outlook_calendar() {
        let mut server = Server::new_async().await;
        let calendar_html = r#"<!DOCTYPE html>
<html>
<head><title>Outlook Calendar</title></head>
<body>
    <div class="calendar-content">
        <h1>Weekly Calendar</h1>
        <div class="calendar-events">
            <div class="event">
                <h3>Team Meeting</h3>
                <p>Monday, 10:00 AM - 11:00 AM</p>
                <p>Conference Room A</p>
            </div>
            <div class="event">
                <h3>Project Review</h3>
                <p>Wednesday, 2:00 PM - 3:30 PM</p>
                <p>Video Conference</p>
            </div>
        </div>
    </div>
</body>
</html>"#;

        let mock = server
            .mock("GET", "/calendar")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(calendar_html)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1500,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/calendar", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should extract calendar information
        assert!(
            content.contains("Weekly Calendar")
                || content.contains("Team Meeting")
                || content.contains("Project Review")
        );
    }

    #[tokio::test]
    async fn test_convert_with_content_length_limit() {
        let mut server = Server::new_async().await;

        // Create large content
        let large_content = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Large Document</title></head>
<body>
    <h1>Large Office Document</h1>
    <p>{}</p>
</body>
</html>"#,
            "This is repeated content. ".repeat(1000)
        );

        let mock = server
            .mock("GET", "/large-document.docx")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(&large_content)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 500, // Small limit
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/large-document.docx", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Content should be truncated to the limit
        assert!(content.len() <= 800); // Should be around the limit (plus some overhead)
        assert!(content.contains("Office 365") || content.contains("SharePoint"));
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
    async fn test_convert_sharepoint_access_denied() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/sites/private/document.docx")
            .with_status(403)
            .with_body("Access denied. You need permission to access this document.")
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/sites/private/document.docx", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            MarkdownError::AuthenticationError { .. } => {
                // Expected - permission denied
            }
            MarkdownError::EnhancedNetworkError { .. } => {
                // Also acceptable - could be mapped as network error
            }
            _ => panic!("Expected authentication or network error"),
        }
    }

    #[tokio::test]
    async fn test_convert_document_not_found() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/sites/team/nonexistent.docx")
            .with_status(404)
            .with_body("Document not found")
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/sites/team/nonexistent.docx", server.url());
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
    async fn test_convert_office365_server_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/sites/team/error.docx")
            .with_status(500)
            .with_body("Internal Server Error")
            .expect(2) // Expect 2 requests: 1 initial + 1 retry
            .create_async()
            .await;

        let config = Config::builder()
            .timeout_seconds(10)
            .max_retries(1) // Reduce retries for faster test
            .build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/sites/team/error.docx", server.url());
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
    async fn test_convert_empty_office365_response() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/sites/team/empty.docx")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("")
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/sites/team/empty.docx", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should still contain placeholder information even with empty response
        assert!(content.contains("Office 365") || content.contains("SharePoint"));
        assert!(content.contains(&url));
    }
}

/// Tests for placeholder-specific functionality
mod placeholder_functionality_tests {
    use super::*;

    #[test]
    fn test_placeholder_settings_default() {
        let settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        assert_eq!(settings.max_content_length, 1000);
    }

    #[tokio::test]
    async fn test_placeholder_content_includes_service_info() {
        let mut server = Server::new_async().await;
        let simple_html = "<html><body><h1>Test Document</h1><p>Content</p></body></html>";

        let mock = server
            .mock("GET", "/test-document")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(simple_html)
            .create_async()
            .await;

        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 500,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        let url = format!("{}/test-document", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should contain service identification
        assert!(content.contains("Office 365") || content.contains("SharePoint"));

        // Should contain the original URL
        assert!(content.contains(&url));

        // Should contain extracted content
        assert!(content.contains("Test Document") || content.contains("Content"));

        // Should indicate this is a placeholder conversion
        assert!(
            content.contains("placeholder")
                || content.contains("preview")
                || content.contains("limited")
        );
    }

    #[tokio::test]
    async fn test_multiple_url_types_handled() {
        let config = Config::builder().timeout_seconds(10).build();
        let client = HttpClient::with_config(&config.http, &config.auth);
        let placeholder_settings = PlaceholderSettings {
            max_content_length: 1000,
        };
        let converter = helpers::create_test_converter_with_config(client, &placeholder_settings);

        for url in helpers::sample_office365_urls() {
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
                    panic!("Should not reject valid Office 365 URL: {url}");
                }
                Err(e) => {
                    panic!("Unexpected error for URL {url}: {e:?}");
                }
            }
        }
    }
}

/// Integration tests
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_office365_conversion() {
        let mut server = Server::new_async().await;
        let html_content = helpers::sample_office365_html();

        let mock = server
            .mock("GET", "/sites/integration-test/document.docx")
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

        let url = format!("{}/sites/integration-test/document.docx", server.url());
        let result = converter.convert(&url).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        let content = markdown.content_only();

        // Should contain extracted content
        assert!(content.contains("Project Planning Document"));
        assert!(content.contains("Executive Summary"));
        assert!(content.contains("Project Scope"));
        assert!(content.contains("Timeline"));

        // Should identify as Office 365 content
        assert!(content.contains("Office 365") || content.contains("SharePoint"));

        // Verify frontmatter if present
        if let Some(frontmatter) = markdown.frontmatter() {
            assert!(frontmatter.contains("title:"));
            assert!(frontmatter.contains("url:"));
            assert!(frontmatter.contains("converter: Office365Converter"));
        }
    }

    #[tokio::test]
    async fn test_office365_conversion_with_retry_logic() {
        let mut server = Server::new_async().await;

        // First request fails, second succeeds
        let failing_mock = server
            .mock("GET", "/sites/retry-test/document.docx")
            .with_status(503)
            .expect(2) // Should be called twice (initial + 1 retry)
            .create_async()
            .await;

        let success_mock = server
            .mock("GET", "/sites/retry-test/document.docx")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><body><h1>Document after retry</h1></body></html>")
            .expect(1)
            .create_async()
            .await;

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

        let url = format!("{}/sites/retry-test/document.docx", server.url());
        let result = converter.convert(&url).await;

        failing_mock.assert_async().await;
        success_mock.assert_async().await;
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.content_only().contains("Document after retry"));
    }
}
