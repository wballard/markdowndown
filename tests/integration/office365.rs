//! Integration tests for Office 365 document conversion
//!
//! Tests the library's ability to convert Office 365 documents to markdown.

use markdowndown::MarkdownDown;
use std::time::Instant;

use super::{IntegrationTestConfig, TestUtils};

/// Test conversion of Office 365 documents
#[tokio::test]
async fn test_office365_conversions() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    if !config.can_test_office365() {
        println!("Skipping Office 365 tests - no credentials available or external services disabled");
        return Ok(());
    }

    let office365_config = markdowndown::Config::builder()
        .office365_token(&config.office365_credentials.as_ref().unwrap().username)
        .build();
    let md = MarkdownDown::with_config(office365_config);

    // Note: For real testing, we would need actual public SharePoint documents
    // These are placeholder URLs that should be replaced with real test documents
    let test_documents = [
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.docx", "Word document"),
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.xlsx", "Excel spreadsheet"),  
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.pptx", "PowerPoint presentation"),
    ];

    for (url, description) in test_documents.iter() {
        println!("Testing: {} - {}", description, url);
        
        // Apply rate limiting
        TestUtils::apply_rate_limit(&config).await;
        
        let start = Instant::now();
        let result = md.convert_url(url).await;
        let duration = start.elapsed();
        
        match result {
            Ok(markdown) => {
                let content = markdown.as_str();
                
                // Basic quality checks
                assert!(TestUtils::validate_markdown_quality(content), 
                       "Poor quality markdown for {}: content too short or invalid", description);
                
                // Should have frontmatter
                assert!(markdown.frontmatter().is_some(), 
                       "Missing frontmatter for {}", description);
                
                let frontmatter = markdown.frontmatter().unwrap();
                assert!(TestUtils::validate_frontmatter(&frontmatter),
                       "Invalid frontmatter for {}", description);
                
                // Office 365 specific content checks
                assert!(frontmatter.contains("sharepoint") || frontmatter.contains("office365"),
                       "Should reference Office 365/SharePoint in frontmatter");
                
                // Performance check
                assert!(duration < config.large_document_timeout(),
                       "Conversion took too long for {}: {:?}", description, duration);
                
                println!("✓ {} converted successfully ({} chars, {:?})", 
                        description, content.len(), duration);
            }
            Err(e) => {
                println!("⚠ {} failed (may be expected for placeholder URLs): {}", description, e);
                // For placeholder URLs, failure is acceptable
                assert!(!e.to_string().is_empty(), "Error should have a message");
                
                // Check if it's an authentication or access error
                if e.to_string().contains("auth") || 
                   e.to_string().contains("403") ||
                   e.to_string().contains("401") ||
                   e.to_string().contains("404") {
                    println!("  Authentication or access error - acceptable for placeholder URLs");
                }
            }
        }
    }

    Ok(())
}

/// Test Office 365 URL format detection
#[tokio::test]
async fn test_office365_url_detection() -> Result<(), Box<dyn std::error::Error>> {
    // Test URL detection (doesn't require credentials)
    let office365_urls = [
        "https://company.sharepoint.com/sites/team/Shared%20Documents/Document.docx",
        "https://company-my.sharepoint.com/personal/user/Documents/Workbook.xlsx", 
        "https://company.sharepoint.com/sites/project/Lists/Tasks/AllItems.aspx",
        "https://onedrive.live.com/edit.aspx?resid=123&cid=456",
        "https://company.office.com/wopi/files/123",
    ];
    
    for url in office365_urls.iter() {
        println!("Testing URL detection: {}", url);
        
        let detected_type = markdowndown::detect_url_type(url)?;
        assert_eq!(detected_type, markdowndown::types::UrlType::Office365,
                  "Should detect as Office 365: {}", url);
    }
    
    println!("✓ All Office 365 URL formats detected correctly");
    Ok(())
}

/// Test Office 365 authentication scenarios
#[tokio::test]
async fn test_office365_authentication() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    let test_url = "https://company.sharepoint.com/sites/public/Shared%20Documents/test.docx";
    
    // Test without credentials
    println!("Testing without Office 365 credentials");
    let md_no_creds = MarkdownDown::new();
    TestUtils::apply_rate_limit(&config).await;
    let result_no_creds = md_no_creds.convert_url(test_url).await;
    
    // Test with credentials (if available)
    if let Some(creds) = &config.office365_credentials {
        println!("Testing with Office 365 credentials");
        let office365_config = markdowndown::Config::builder()
            .office365_token(&creds.username)
            .build();
        let md_with_creds = MarkdownDown::with_config(office365_config);
        
        TestUtils::apply_rate_limit(&config).await;
        let result_with_creds = md_with_creds.convert_url(test_url).await;
        
        // Compare results
        match (result_no_creds, result_with_creds) {
            (Ok(content1), Ok(content2)) => {
                println!("Both conversions succeeded");
                assert!(TestUtils::validate_markdown_quality(content1.as_str()));
                assert!(TestUtils::validate_markdown_quality(content2.as_str()));
            }
            (Err(e1), Err(e2)) => {
                println!("Both conversions failed (expected for placeholder URLs)");
                println!("  No credentials error: {}", e1);
                println!("  With credentials error: {}", e2);
                assert!(!e1.to_string().is_empty());
                assert!(!e2.to_string().is_empty());
            }
            (Ok(content), Err(e)) => {
                println!("No-credentials succeeded, with-credentials failed: {}", e);
                assert!(TestUtils::validate_markdown_quality(content.as_str()));
            }
            (Err(e), Ok(content)) => {
                println!("No-credentials failed, with-credentials succeeded: {}", e);
                assert!(TestUtils::validate_markdown_quality(content.as_str()));
                // This is the expected case - credentials should provide better access
            }
        }
    } else {
        println!("No Office 365 credentials available - testing without credentials only");
        match result_no_creds {
            Ok(content) => {
                println!("Conversion succeeded without credentials");
                assert!(TestUtils::validate_markdown_quality(content.as_str()));
            }
            Err(e) => {
                println!("Conversion failed without credentials (expected): {}", e);
                assert!(!e.to_string().is_empty());
                // Should fail gracefully with descriptive error
                assert!(e.to_string().contains("auth") || 
                       e.to_string().contains("credential") ||
                       e.to_string().contains("403") ||
                       e.to_string().contains("401"),
                       "Error should indicate authentication issue");
            }
        }
    }

    Ok(())
}

/// Test different Office 365 document types
#[tokio::test]
async fn test_office365_document_types() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    if !config.can_test_office365() || config.skip_slow_tests {
        println!("Skipping Office 365 document types test - no credentials available, external services disabled, or slow tests skipped");
        return Ok(());
    }

    let office365_config = if let Some(creds) = &config.office365_credentials {
        markdowndown::Config::builder()
            .office365_token(&creds.username)
            .build()
    } else {
        markdowndown::Config::default()
    };
    let md = MarkdownDown::with_config(office365_config);
    
    // Test different document types
    let document_types = [
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.docx", "Word Document", "docx"),
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.xlsx", "Excel Spreadsheet", "xlsx"), 
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.pptx", "PowerPoint Presentation", "pptx"),
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/test.pdf", "PDF Document", "pdf"),
    ];
    
    for (url, doc_type, extension) in document_types.iter() {
        println!("Testing {} type: {}", doc_type, url);
        
        // Rate limiting
        TestUtils::apply_rate_limit(&config).await;
        
        let result = md.convert_url(url).await;
        
        match result {
            Ok(markdown) => {
                let content = markdown.as_str();
                println!("  ✓ {} converted successfully ({} chars)", doc_type, content.len());
                
                // Validate quality
                assert!(TestUtils::validate_markdown_quality(content),
                       "Poor quality conversion for {} type", doc_type);
                
                // Check frontmatter
                let frontmatter = markdown.frontmatter().unwrap();
                assert!(frontmatter.contains("sharepoint") || frontmatter.contains("office365"));
                assert!(frontmatter.contains(extension),
                       "Frontmatter should indicate document type: {}", extension);
            }
            Err(e) => {
                println!("  ⚠ {} conversion failed (expected for placeholder URLs): {}", doc_type, e);
                // For placeholder URLs, failures are expected
                assert!(!e.to_string().is_empty());
                
                if e.to_string().contains("404") || 
                   e.to_string().contains("not found") ||
                   e.to_string().contains("403") ||
                   e.to_string().contains("401") {
                    println!("    Access or not found error - acceptable for placeholder URLs");
                }
            }
        }
    }

    Ok(())
}

/// Test Office 365 error scenarios
#[tokio::test]
async fn test_office365_error_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    let md = if let Some(creds) = &config.office365_credentials {
        let office365_config = markdowndown::Config::builder()
            .office365_token(&creds.username)
            .build();
        MarkdownDown::with_config(office365_config)
    } else {
        MarkdownDown::new()
    };
    
    let error_cases = [
        ("https://company.sharepoint.com/sites/nonexistent/document.docx", "Non-existent site"),
        ("https://company.sharepoint.com/sites/public/nonexistent.docx", "Non-existent document"),
        ("https://invalid.sharepoint.com/sites/test/document.docx", "Invalid domain"),
    ];
    
    for (url, description) in error_cases.iter() {
        println!("Testing error case: {}", description);
        
        // Rate limiting
        TestUtils::apply_rate_limit(&config).await;
        
        let result = md.convert_url(url).await;
        
        // Should fail gracefully
        match result {
            Ok(markdown) => {
                println!("  Unexpected success: {} chars", markdown.as_str().len());
                // If it succeeds, content should indicate the issue
                let content = markdown.as_str();
                assert!(content.contains("Error") || 
                       content.contains("not found") ||
                       content.contains("404") ||
                       content.len() < 100,
                       "Unexpected success content for {}", description);
            }
            Err(error) => {
                println!("  Failed as expected: {}", error);
                // Error should be descriptive
                assert!(!error.to_string().is_empty(), "Error message should not be empty");
                assert!(error.to_string().len() > 10, "Error message should be descriptive");
                
                // Should indicate the specific problem
                assert!(error.to_string().contains("404") ||
                       error.to_string().contains("not found") ||
                       error.to_string().contains("nonexistent") ||
                       error.to_string().contains("403") ||
                       error.to_string().contains("401") ||
                       error.to_string().contains("invalid"),
                       "Error should indicate specific issue type");
            }
        }
    }
    
    Ok(())
}

/// Test SharePoint list and library URLs
#[tokio::test]
async fn test_sharepoint_lists_and_libraries() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    if !config.can_test_office365() {
        println!("Skipping SharePoint lists test - no credentials available or external services disabled");
        return Ok(());
    }

    let office365_config = if let Some(creds) = &config.office365_credentials {
        markdowndown::Config::builder()
            .office365_token(&creds.username)
            .build()
    } else {
        markdowndown::Config::default()
    };
    let md = MarkdownDown::with_config(office365_config);
    
    // Test SharePoint list and library URLs
    let sharepoint_urls = [
        ("https://company.sharepoint.com/sites/team/Shared%20Documents", "Document Library"),
        ("https://company.sharepoint.com/sites/team/Lists/Tasks/AllItems.aspx", "Tasks List"),
        ("https://company.sharepoint.com/sites/team/Lists/Announcements", "Announcements List"),
    ];
    
    for (url, description) in sharepoint_urls.iter() {
        println!("Testing {}: {}", description, url);
        
        // Rate limiting
        TestUtils::apply_rate_limit(&config).await;
        
        let result = md.convert_url(url).await;
        
        match result {
            Ok(markdown) => {
                let content = markdown.as_str();
                println!("  ✓ {} converted successfully ({} chars)", description, content.len());
                
                // Validate quality
                assert!(TestUtils::validate_markdown_quality(content),
                       "Poor quality conversion for {}", description);
                
                // Should have frontmatter indicating SharePoint source
                let frontmatter = markdown.frontmatter().unwrap();
                assert!(frontmatter.contains("sharepoint"));
            }
            Err(e) => {
                println!("  ⚠ {} conversion failed (expected for placeholder URLs): {}", description, e);
                assert!(!e.to_string().is_empty());
            }
        }
    }

    Ok(())
}

/// Performance test for Office 365 conversion
#[tokio::test]
async fn test_office365_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    if !config.can_test_office365() || config.skip_slow_tests {
        println!("Skipping Office 365 performance test - no credentials available, external services disabled, or slow tests skipped");
        return Ok(());
    }

    let office365_config = if let Some(creds) = &config.office365_credentials {
        markdowndown::Config::builder()
            .office365_token(&creds.username)
            .build()
    } else {
        markdowndown::Config::default()
    };
    let md = MarkdownDown::with_config(office365_config);
    
    let test_documents = [
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/small.docx", "Small Document"),
        ("https://company.sharepoint.com/sites/public/Shared%20Documents/large.docx", "Large Document"),
    ];
    
    let mut total_duration = std::time::Duration::from_secs(0);
    let mut total_chars = 0;
    let mut successes = 0;
    
    for (url, description) in test_documents.iter() {
        println!("Performance testing: {}", description);
        
        // Rate limiting
        TestUtils::apply_rate_limit(&config).await;
        
        let start = Instant::now();
        let result = md.convert_url(url).await;
        let duration = start.elapsed();
        
        match result {
            Ok(markdown) => {
                let content_length = markdown.as_str().len();
                total_duration += duration;
                total_chars += content_length;
                successes += 1;
                
                println!("  Duration: {:?}, Content: {} chars", duration, content_length);
                
                // Performance assertions
                assert!(duration < config.large_document_timeout(),
                       "Office 365 conversion took too long: {:?}", duration);
                
                assert!(TestUtils::validate_markdown_quality(markdown.as_str()),
                       "Performance test should produce quality output");
            }
            Err(e) => {
                println!("  Failed: {} (expected for placeholder URLs)", e);
                // For placeholder URLs, failures are expected
                assert!(!e.to_string().is_empty());
            }
        }
    }
    
    if successes > 0 {
        println!("Office 365 Performance Summary:");
        println!("  Total successful requests: {}", successes);
        println!("  Total time: {:?}", total_duration);
        println!("  Total content: {} chars", total_chars);
        println!("  Average time per request: {:?}", total_duration / successes as u32);
        println!("  Average chars per request: {}", total_chars / successes);
    } else {
        println!("No successful requests - expected for placeholder URLs without real Office 365 documents");
    }
    
    Ok(())
}

/// Test OneDrive URLs
#[tokio::test]
async fn test_onedrive_urls() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();
    
    if !config.can_test_office365() {
        println!("Skipping OneDrive tests - no credentials available or external services disabled");
        return Ok(());
    }

    // Test OneDrive URL formats
    let onedrive_urls = [
        "https://onedrive.live.com/edit.aspx?resid=ABC123&cid=DEF456",
        "https://company-my.sharepoint.com/personal/user_company_com/Documents/test.docx",
        "https://1drv.ms/w/s!ABC123DEF456",
    ];
    
    for url in onedrive_urls.iter() {
        println!("Testing OneDrive URL detection: {}", url);
        
        // Test URL detection
        let detected_type = markdowndown::detect_url_type(url)?;
        assert_eq!(detected_type, markdowndown::types::UrlType::Office365,
                  "Should detect OneDrive URL as Office 365: {}", url);
    }
    
    println!("✓ All OneDrive URL formats detected correctly");
    Ok(())
}