use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test basic CLI help output
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "markdowndown", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Convert URLs to markdown"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("batch"));
    assert!(stdout.contains("detect"));
    assert!(stdout.contains("list-types"));
}

/// Test version output
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "markdowndown", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

/// Test detect command
#[test]
fn test_detect_command() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "markdowndown",
            "--",
            "detect",
            "https://github.com/owner/repo/issues/123",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GitHub Issue"));
}

/// Test list-types command
#[test]
fn test_list_types_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "markdowndown", "--", "list-types"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Supported URL types:"));
    assert!(stdout.contains("HTML"));
    assert!(stdout.contains("GitHub Issue"));
    assert!(stdout.contains("Google Docs"));
    assert!(stdout.contains("Office 365"));
}

/// Test configuration file loading
#[test]
fn test_config_file_loading() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test-config.toml");

    // Create a test config file
    let config_content = r#"
[http]
timeout_seconds = 60
user_agent = "test-agent/1.0"

[authentication]
github_token = "test-token"

[output]
include_frontmatter = false
"#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    // Test that CLI accepts the config file without error
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "markdowndown",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "list-types",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

/// Test batch processing with small file
#[test]
fn test_batch_processing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let urls_file = temp_dir.path().join("test-urls.txt");

    // Create a test URLs file
    let urls_content = "# Test URLs\nhttps://example.com\nhttps://httpbin.org/html\n";
    fs::write(&urls_file, urls_content).expect("Failed to write URLs file");

    // Test batch processing
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "markdowndown",
            "--",
            "--quiet", // Suppress verbose output for test
            "batch",
            urls_file.to_str().unwrap(),
            "--concurrency",
            "1",
        ])
        .output()
        .expect("Failed to execute command");

    // Should succeed even if some URLs fail due to network issues
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found") && stdout.contains("URLs to process"));
}

/// Test invalid URL handling
#[test]
fn test_invalid_url_error() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "markdowndown",
            "--",
            "detect",
            "not-a-valid-url",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Validation error")
            || stderr.contains("InvalidUrl")
            || stderr.contains("❌")
    );
}

/// Test output formats
#[test]
fn test_output_formats() {
    // Test JSON format detection
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "markdowndown",
            "--",
            "--format",
            "json",
            "detect",
            "https://example.com",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // Test YAML format detection
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "markdowndown",
            "--",
            "--format",
            "yaml",
            "detect",
            "https://docs.google.com/document/d/abc123/edit",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

/// Test no URL provided error
#[test]
fn test_no_url_error() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "markdowndown"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No URL provided"));
}

/// Test concurrent batch processing
#[test]
fn test_batch_concurrency() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let urls_file = temp_dir.path().join("concurrent-urls.txt");

    // Create a test URLs file with multiple URLs
    let urls_content = r#"
# Test URLs for concurrency
https://example.com
https://httpbin.org/html
https://httpbin.org/json
"#;
    fs::write(&urls_file, urls_content).expect("Failed to write URLs file");

    // Test with different concurrency levels
    for concurrency in [1, 2, 3] {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "markdowndown",
                "--",
                "--quiet",
                "batch",
                urls_file.to_str().unwrap(),
                "--concurrency",
                &concurrency.to_string(),
            ])
            .output()
            .expect("Failed to execute command");

        // Should handle different concurrency levels
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Found") && stdout.contains("URLs to process"));
    }
}
