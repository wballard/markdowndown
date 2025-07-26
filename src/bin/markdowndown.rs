//! Command-line interface for the markdowndown library.
//!
//! This binary provides a user-friendly CLI for converting URLs to markdown
//! using the markdowndown library. It supports single URL conversion, batch
//! processing, and various output formats.

use clap::{Parser, Subcommand, ValueEnum};
use markdowndown::{Config, MarkdownDown};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process;
use tracing::{debug, error, info};

/// Convert URLs to markdown with smart handling for different source types
#[derive(Parser)]
#[command(name = "markdowndown")]
#[command(about = "Convert URLs to markdown")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(long_about = None)]
struct Cli {
    /// URL to convert (if no subcommand specified)
    url: Option<String>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// GitHub API token (can also use GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,

    /// Network timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Output format
    #[arg(short, long, default_value = "markdown")]
    format: OutputFormat,

    /// Exclude YAML frontmatter from output
    #[arg(long)]
    no_frontmatter: bool,

    /// Output only the frontmatter
    #[arg(long)]
    frontmatter_only: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Quiet mode (suppress all output except results)
    #[arg(short, long)]
    quiet: bool,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Custom user agent string
    #[arg(long)]
    user_agent: Option<String>,

    /// Subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available output formats
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    /// Markdown format (default)
    Markdown,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Convert URLs from a file (one per line)
    Batch {
        /// File containing URLs to convert
        file: String,
        /// Number of concurrent conversions
        #[arg(short, long, default_value = "5")]
        concurrency: usize,
        /// Output directory for converted files
        #[arg(long)]
        output_dir: Option<String>,
        /// Show conversion statistics
        #[arg(long)]
        stats: bool,
    },
    /// Detect URL type without conversion
    Detect {
        /// URL to analyze
        url: String,
    },
    /// List supported URL types
    ListTypes,
}

/// Configuration file structure for TOML files
#[derive(Debug, Deserialize, Serialize, Default)]
struct ConfigFile {
    /// HTTP configuration
    #[serde(default)]
    pub http: HttpConfig,

    /// Authentication configuration
    #[serde(default)]
    pub authentication: AuthConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,

    /// Batch processing configuration
    #[serde(default)]
    pub batch: BatchConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct HttpConfig {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    pub user_agent: Option<String>,
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            user_agent: None,
            max_redirects: default_max_redirects(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct AuthConfig {
    pub github_token: Option<String>,
    pub office365_token: Option<String>,
    pub google_api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OutputConfig {
    #[serde(default = "default_true")]
    pub include_frontmatter: bool,
    #[serde(default = "default_format")]
    pub format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            include_frontmatter: default_true(),
            format: default_format(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct BatchConfig {
    #[serde(default = "default_concurrency")]
    pub default_concurrency: usize,
    #[serde(default = "default_true")]
    pub skip_failures: bool,
    #[serde(default = "default_true")]
    pub show_progress: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            default_concurrency: default_concurrency(),
            skip_failures: default_true(),
            show_progress: default_true(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

// Default value functions for serde
fn default_timeout() -> u64 {
    30
}
fn default_max_redirects() -> u32 {
    10
}
fn default_true() -> bool {
    true
}
fn default_format() -> String {
    "markdown".to_string()
}
fn default_concurrency() -> usize {
    5
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "human".to_string()
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging based on CLI arguments
    init_logging(&cli);

    // Handle the CLI command
    if let Err(e) = run_cli(cli).await {
        handle_error(e);
    }
}

/// Initialize tracing/logging based on CLI arguments
fn init_logging(cli: &Cli) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let level = if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else if cli.quiet {
        "error"
    } else {
        "warn"
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("markdowndown={level}")));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time()
                .with_writer(std::io::stderr),
        )
        .init();
}

/// Main CLI execution logic
async fn run_cli(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "Starting CLI with args: {:?}",
        std::env::args().collect::<Vec<_>>()
    );

    // Build configuration from CLI arguments
    let config = build_config(&cli)?;
    let markdowndown = MarkdownDown::with_config(config);

    match &cli.command {
        Some(Commands::Batch {
            file,
            concurrency,
            output_dir,
            stats,
        }) => {
            batch_convert(
                &markdowndown,
                file,
                *concurrency,
                output_dir.as_deref(),
                *stats,
                &cli,
            )
            .await
        }
        Some(Commands::Detect { url }) => detect_url_type(url),
        Some(Commands::ListTypes) => list_supported_types(&markdowndown),
        None => {
            // Handle single URL conversion or show help if no URL provided
            if let Some(ref url) = cli.url {
                single_convert(&markdowndown, url, &cli).await
            } else {
                eprintln!("Error: No URL provided");
                eprintln!("Use 'markdowndown --help' for usage information");
                process::exit(1);
            }
        }
    }
}

/// Build configuration from CLI arguments and config file
fn build_config(cli: &Cli) -> Result<Config, Box<dyn std::error::Error>> {
    // First, load configuration from file (if specified or from default locations)
    let file_config = load_config_file(cli.config.as_deref())?;

    // Start with config from file as base
    let mut builder = Config::builder()
        .timeout_seconds(file_config.http.timeout_seconds)
        .include_frontmatter(file_config.output.include_frontmatter);

    // Apply CLI overrides (CLI arguments take precedence over config file)
    if cli.timeout != 30 {
        // Only override if not default
        builder = builder.timeout_seconds(cli.timeout);
    }

    if cli.no_frontmatter {
        builder = builder.include_frontmatter(false);
    }

    // Authentication tokens - CLI takes precedence, then config file
    if let Some(token) = &cli.github_token {
        builder = builder.github_token(token);
    } else if let Some(token) = &file_config.authentication.github_token {
        builder = builder.github_token(token);
    }

    if let Some(token) = &file_config.authentication.office365_token {
        builder = builder.office365_token(token);
    }

    if let Some(key) = &file_config.authentication.google_api_key {
        builder = builder.google_api_key(key);
    }

    // User agent - CLI takes precedence
    if let Some(ua) = &cli.user_agent {
        builder = builder.user_agent(ua);
    } else if let Some(ua) = &file_config.http.user_agent {
        builder = builder.user_agent(ua);
    }

    Ok(builder.build())
}

/// Load configuration from file
fn load_config_file(config_path: Option<&str>) -> Result<ConfigFile, Box<dyn std::error::Error>> {
    let config_paths = if let Some(path) = config_path {
        // Use specified config file
        vec![PathBuf::from(path)]
    } else {
        // Try default locations
        find_default_config_paths()
    };

    for path in config_paths {
        if path.exists() {
            debug!("Loading configuration from: {}", path.display());
            let content = std::fs::read_to_string(&path)?;
            let config: ConfigFile = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config file {}: {}", path.display(), e))?;
            info!("Loaded configuration from: {}", path.display());
            return Ok(config);
        }
    }

    // If no config file found, return default
    debug!("No configuration file found, using defaults");
    Ok(ConfigFile::default())
}

/// Find default configuration file paths
fn find_default_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Current directory
    paths.push(PathBuf::from("./markdowndown.toml"));
    paths.push(PathBuf::from("./.markdowndown.toml"));

    // Home directory
    if let Some(home) = std::env::var_os("HOME") {
        let home_path = PathBuf::from(home);
        paths.push(home_path.join(".markdowndown.toml"));
        paths.push(home_path.join(".config/markdowndown/config.toml"));
    }

    // XDG config directory (Linux/Unix)
    if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME") {
        let xdg_path = PathBuf::from(xdg_config);
        paths.push(xdg_path.join("markdowndown/config.toml"));
    }

    debug!("Default config paths: {:?}", paths);
    paths
}

/// Convert a single URL to markdown
async fn single_convert(
    markdowndown: &MarkdownDown,
    url: &str,
    cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Converting single URL: {}", url);

    let result = markdowndown.convert_url(url).await?;

    // Format output based on CLI options
    let output = format_output(&result, cli)?;

    // Write output to file or stdout
    write_output(&output, cli.output.as_deref())?;

    Ok(())
}

/// Convert multiple URLs from a file
async fn batch_convert(
    markdowndown: &MarkdownDown,
    file: &str,
    concurrency: usize,
    output_dir: Option<&str>,
    stats: bool,
    cli: &Cli,
) -> Result<(), Box<dyn std::error::Error>> {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::fs;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::sync::Semaphore;

    info!("Starting batch conversion from file: {}", file);

    // Read URLs from file
    let file_content = fs::File::open(file).await?;
    let reader = BufReader::new(file_content);
    let mut lines = reader.lines();
    let mut urls = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let url = line.trim();
        if !url.is_empty() && !url.starts_with('#') {
            urls.push(url.to_string());
        }
    }

    if urls.is_empty() {
        eprintln!("No valid URLs found in file: {file}");
        return Ok(());
    }

    println!("Found {} URLs to process", urls.len());

    // Create output directory if specified
    if let Some(dir) = output_dir {
        fs::create_dir_all(dir).await?;
    }

    // Set up progress bar if not in quiet mode
    let pb = if !cli.quiet {
        let pb = ProgressBar::new(urls.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        Some(pb)
    } else {
        None
    };

    // Statistics tracking
    let success_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    let semaphore = Arc::new(Semaphore::new(concurrency));

    // Get the configuration to create new instances in tasks
    let config = markdowndown.config().clone();

    // Process URLs concurrently
    let mut tasks = Vec::new();

    for (index, url) in urls.into_iter().enumerate() {
        let config = config.clone();
        let output_dir = output_dir.map(String::from);
        let cli_format = cli.format;
        let include_frontmatter = !cli.no_frontmatter;
        let pb = pb.clone();
        let success_count = success_count.clone();
        let error_count = error_count.clone();
        let semaphore = semaphore.clone();

        let task = tokio::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    if let Some(ref pb) = pb {
                        pb.println(format!("❌ {url}: Failed to acquire semaphore"));
                    } else {
                        eprintln!("❌ {url}: Failed to acquire semaphore");
                    }
                    error_count.fetch_add(1, Ordering::Relaxed);
                    if let Some(ref pb) = pb {
                        pb.inc(1);
                    }
                    return;
                }
            };

            // Create a new MarkdownDown instance for this task
            let markdowndown = MarkdownDown::with_config(config);

            if let Some(ref pb) = pb {
                pb.set_message(format!("Converting: {url}"));
            }

            // Add timeout wrapper to prevent hanging
            let conversion_timeout = std::time::Duration::from_secs(60); // 60 second safety timeout
            let conversion_result = tokio::time::timeout(
                conversion_timeout,
                convert_single_url(&markdowndown, &url, cli_format, include_frontmatter),
            )
            .await;

            match conversion_result {
                Ok(Ok(content)) => {
                    // Save to file if output directory specified
                    if let Some(ref dir) = output_dir {
                        let filename = format!("{:03}.md", index + 1);
                        let filepath = Path::new(dir).join(filename);
                        if let Err(e) = fs::write(&filepath, &content).await {
                            if let Some(ref pb) = pb {
                                pb.println(format!(
                                    "❌ Failed to write {}: {}",
                                    filepath.display(),
                                    e
                                ));
                            }
                            error_count.fetch_add(1, Ordering::Relaxed);
                        } else {
                            if let Some(ref pb) = pb {
                                pb.println(format!("✅ {} -> {}", url, filepath.display()));
                            }
                            success_count.fetch_add(1, Ordering::Relaxed);
                        }
                    } else {
                        // Output to stdout with separator
                        println!("=== {url} ===");
                        println!("{content}");
                        println!();
                        success_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Ok(Err(e)) => {
                    if let Some(ref pb) = pb {
                        pb.println(format!("❌ {url}: {e}"));
                    } else {
                        eprintln!("❌ {url}: {e}");
                    }
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(_timeout) => {
                    let timeout_msg =
                        format!("Conversion timeout after {}s", conversion_timeout.as_secs());
                    if let Some(ref pb) = pb {
                        pb.println(format!("❌ {url}: {timeout_msg}"));
                    } else {
                        eprintln!("❌ {url}: {timeout_msg}");
                    }
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
            }

            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await?;
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Batch conversion complete");
    }

    // Print statistics if requested
    let successes = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);

    if stats || cli.verbose {
        println!();
        println!("Conversion Statistics:");
        println!("  Successful: {successes}");
        println!("  Failed: {errors}");
        println!("  Total: {}", successes + errors);
        println!(
            "  Success rate: {:.1}%",
            (successes as f64 / (successes + errors) as f64) * 100.0
        );
    }

    Ok(())
}

/// Helper function to convert a single URL with specified options
async fn convert_single_url(
    markdowndown: &MarkdownDown,
    url: &str,
    format: OutputFormat,
    _include_frontmatter: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let result = markdowndown
        .convert_url(url)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

    // For batch processing, we'll use a simpler format output
    match format {
        OutputFormat::Markdown => Ok(result.as_str().to_string()),
        OutputFormat::Json => {
            let json_output = serde_json::json!({
                "url": url,
                "content": result.as_str(),
                "format": "markdown"
            });
            Ok(serde_json::to_string_pretty(&json_output)?)
        }
        OutputFormat::Yaml => {
            let yaml_data = serde_yaml::Value::String(result.as_str().to_string());
            Ok(serde_yaml::to_string(&yaml_data)?)
        }
    }
}

/// Detect and display URL type
fn detect_url_type(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url_type = markdowndown::detect_url_type(url)?;
    println!("{url_type}");
    Ok(())
}

/// List all supported URL types
fn list_supported_types(markdowndown: &MarkdownDown) -> Result<(), Box<dyn std::error::Error>> {
    let types = markdowndown.supported_types();
    println!("Supported URL types:");
    for url_type in types {
        println!("  {url_type}");
    }
    Ok(())
}

/// Format output based CLI options
fn format_output(
    markdown: &markdowndown::types::Markdown,
    cli: &Cli,
) -> Result<String, Box<dyn std::error::Error>> {
    if cli.frontmatter_only {
        return match markdown.frontmatter() {
            Some(frontmatter) => Ok(frontmatter),
            None => Ok("No frontmatter found in the document".to_string()),
        };
    }

    match cli.format {
        OutputFormat::Markdown => Ok(markdown.as_str().to_string()),
        OutputFormat::Json => {
            let json_output = serde_json::json!({
                "content": markdown.as_str(),
                "format": "markdown"
            });
            Ok(serde_json::to_string_pretty(&json_output)?)
        }
        OutputFormat::Yaml => {
            let yaml_data = serde_yaml::Value::String(markdown.as_str().to_string());
            Ok(serde_yaml::to_string(&yaml_data)?)
        }
    }
}

/// Write output to file or stdout
fn write_output(
    content: &str,
    output_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    match output_file {
        Some(file_path) => {
            std::fs::write(file_path, content)?;
            debug!("Output written to: {}", file_path);
        }
        None => {
            print!("{content}");
        }
    }
    Ok(())
}

/// Handle errors and exit with appropriate code
fn handle_error(error: Box<dyn std::error::Error>) -> ! {
    error!("CLI error: {}", error);

    // Try to match specific error types for better user experience
    let error_msg = error.to_string();

    if error_msg.contains("Invalid URL") || error_msg.contains("ValidationError") {
        eprintln!("❌ Invalid URL format");
        eprintln!("💡 Make sure the URL includes http:// or https://");
        process::exit(1);
    } else if error_msg.contains("Network") || error_msg.contains("timeout") {
        eprintln!("❌ Network error occurred");
        eprintln!("💡 Check your internet connection and try again");
        process::exit(2);
    } else if error_msg.contains("Authentication") || error_msg.contains("Auth") {
        eprintln!("❌ Authentication failed");
        eprintln!("💡 Check your API tokens and permissions");
        process::exit(3);
    } else {
        eprintln!("❌ Unexpected error: {error}");
        process::exit(99);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_single_url() {
        let args = vec!["markdowndown", "https://example.com"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.url, Some("https://example.com".to_string()));
        assert!(cli.command.is_none());
        assert_eq!(cli.format, OutputFormat::Markdown);
        assert!(!cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_cli_parsing_with_options() {
        let args = vec![
            "markdowndown",
            "https://example.com",
            "--output",
            "test.md",
            "--verbose",
            "--timeout",
            "60",
            "--format",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.url, Some("https://example.com".to_string()));
        assert_eq!(cli.output, Some("test.md".to_string()));
        assert!(cli.verbose);
        assert_eq!(cli.timeout, 60);
        assert_eq!(cli.format, OutputFormat::Json);
    }

    #[test]
    fn test_cli_parsing_batch_command() {
        let args = vec!["markdowndown", "batch", "urls.txt", "--concurrency", "10"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.url.is_none());
        if let Some(Commands::Batch {
            file, concurrency, ..
        }) = cli.command
        {
            assert_eq!(file, "urls.txt");
            assert_eq!(concurrency, 10);
        } else {
            panic!("Expected batch command");
        }
    }

    #[test]
    fn test_cli_parsing_detect_command() {
        let args = vec![
            "markdowndown",
            "detect",
            "https://github.com/owner/repo/issues/123",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Some(Commands::Detect { url }) = cli.command {
            assert_eq!(url, "https://github.com/owner/repo/issues/123");
        } else {
            panic!("Expected detect command");
        }
    }

    #[test]
    fn test_cli_parsing_list_types_command() {
        let args = vec!["markdowndown", "list-types"];
        let cli = Cli::try_parse_from(args).unwrap();

        matches!(cli.command, Some(Commands::ListTypes));
    }

    #[test]
    fn test_config_file_defaults() {
        let config = ConfigFile::default();
        
        assert_eq!(config.http.timeout_seconds, 30);
        assert_eq!(config.http.max_redirects, 10);
        assert!(config.http.user_agent.is_none());
        assert!(config.authentication.github_token.is_none());
        assert!(config.authentication.office365_token.is_none());
        assert!(config.authentication.google_api_key.is_none());
        assert!(config.output.include_frontmatter);
        assert_eq!(config.output.format, "markdown");
        assert_eq!(config.batch.default_concurrency, 5);
        assert!(config.batch.skip_failures);
        assert!(config.batch.show_progress);
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.format, "human");
    }

    #[test]
    fn test_default_value_functions() {
        assert_eq!(default_timeout(), 30);
        assert_eq!(default_max_redirects(), 10);
        assert!(default_true());
        assert_eq!(default_format(), "markdown");
        assert_eq!(default_concurrency(), 5);
        assert_eq!(default_log_level(), "info");
        assert_eq!(default_log_format(), "human");
    }

    #[test]
    fn test_output_format_values() {
        assert_eq!(OutputFormat::Markdown.to_possible_value().unwrap().get_name(), "markdown");
        assert_eq!(OutputFormat::Json.to_possible_value().unwrap().get_name(), "json");
        assert_eq!(OutputFormat::Yaml.to_possible_value().unwrap().get_name(), "yaml");
    }

    #[test]
    fn test_build_config_from_defaults() {
        
        let cli = Cli {
            url: None,
            output: None,
            config: None,
            github_token: None,
            timeout: 30, // Default value
            format: OutputFormat::Markdown,
            no_frontmatter: false,
            frontmatter_only: false,
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: None,
            command: None,
        };

        let config = build_config(&cli).expect("Should build config from defaults");
        
        // Should use default values when no config file or CLI overrides
        assert_eq!(config.http.timeout.as_secs(), 30);
        assert!(config.output.include_frontmatter);
        assert!(config.auth.github_token.is_none());
    }

    #[test]
    fn test_build_config_with_cli_overrides() {
        
        let cli = Cli {
            url: None,
            output: None,
            config: None,
            github_token: Some("cli-token".to_string()),
            timeout: 60, // Override default
            format: OutputFormat::Markdown,
            no_frontmatter: true, // Override default
            frontmatter_only: false,
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: Some("cli-agent".to_string()),
            command: None,
        };

        let config = build_config(&cli).expect("Should build config with CLI overrides");
        
        // Should use CLI overrides
        assert_eq!(config.http.timeout.as_secs(), 60);
        assert!(!config.output.include_frontmatter); // Overridden by no_frontmatter
        assert_eq!(config.auth.github_token, Some("cli-token".to_string()));
        assert_eq!(config.http.user_agent, "cli-agent");
    }

    #[test]
    fn test_build_config_with_config_file() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test.toml");
        
        let config_content = r#"
[http]
timeout_seconds = 45
user_agent = "file-agent"

[authentication]
github_token = "file-token"
office365_token = "office-token"
google_api_key = "google-key"

[output]
include_frontmatter = false
"#;
        
        fs::write(&config_path, config_content).expect("Failed to write config file");
        
        let cli = Cli {
            url: None,
            output: None,
            config: Some(config_path.to_string_lossy().to_string()),
            github_token: None,
            timeout: 30, // Default, should be overridden by config file
            format: OutputFormat::Markdown,
            no_frontmatter: false,
            frontmatter_only: false,
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: None,
            command: None,
        };

        let config = build_config(&cli).expect("Should build config from file");
        
        // Should use values from config file
        assert_eq!(config.http.timeout.as_secs(), 45);
        assert!(!config.output.include_frontmatter);
        assert_eq!(config.auth.github_token, Some("file-token".to_string()));
        assert_eq!(config.auth.office365_token, Some("office-token".to_string()));
        assert_eq!(config.auth.google_api_key, Some("google-key".to_string()));
        assert_eq!(config.http.user_agent, "file-agent");
    }

    #[test]
    fn test_find_default_config_paths() {
        let paths = find_default_config_paths();
        
        // Should include current directory paths
        assert!(paths.iter().any(|p| p.file_name().unwrap() == "markdowndown.toml"));
        assert!(paths.iter().any(|p| p.file_name().unwrap() == ".markdowndown.toml"));
        
        // Should include potential HOME paths if HOME is set
        if std::env::var_os("HOME").is_some() {
            assert!(paths.iter().any(|p| p.to_string_lossy().contains(".markdowndown.toml")));
        }
        
        // Should be non-empty
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_load_config_file_default_when_not_found() {
        // Test with non-existent config file
        let result = load_config_file(Some("/nonexistent/path/config.toml"));
        
        // Should return default config when file not found
        let config = result.expect("Should return default config when file not found");
        assert_eq!(config.http.timeout_seconds, 30); // Default value
    }

    #[test]
    fn test_format_output_markdown() {
        use markdowndown::types::Markdown;
        
        let cli = Cli {
            url: None,
            output: None,
            config: None,
            github_token: None,
            timeout: 30,
            format: OutputFormat::Markdown,
            no_frontmatter: false,
            frontmatter_only: false,
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: None,
            command: None,
        };

        let markdown = Markdown::new("# Test Content\n\nThis is test content.".to_string()).expect("Valid markdown");
        let output = format_output(&markdown, &cli).expect("Should format markdown output");
        
        assert_eq!(output, "# Test Content\n\nThis is test content.");
    }

    #[test]
    fn test_format_output_json() {
        use markdowndown::types::Markdown;
        
        let cli = Cli {
            url: None,
            output: None,
            config: None,
            github_token: None,
            timeout: 30,
            format: OutputFormat::Json,
            no_frontmatter: false,
            frontmatter_only: false,
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: None,
            command: None,
        };

        let markdown = Markdown::new("# Test".to_string()).expect("Valid markdown");
        let output = format_output(&markdown, &cli).expect("Should format JSON output");
        
        // Should be valid JSON with expected structure
        assert!(output.contains("\"content\""));
        assert!(output.contains("\"format\""));
        assert!(output.contains("markdown"));
    }

    #[test]
    fn test_format_output_yaml() {
        use markdowndown::types::Markdown;
        
        let cli = Cli {
            url: None,
            output: None,
            config: None,
            github_token: None,
            timeout: 30,
            format: OutputFormat::Yaml,
            no_frontmatter: false,
            frontmatter_only: false,
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: None,
            command: None,
        };

        let markdown = Markdown::new("# Test".to_string()).expect("Valid markdown");
        let output = format_output(&markdown, &cli).expect("Should format YAML output");
        
        // Should be YAML string format
        assert!(!output.is_empty());
        // YAML string should contain the content
        assert!(output.contains("Test") || output.contains("#"));
    }

    #[test]
    fn test_format_output_frontmatter_only() {
        use markdowndown::types::Markdown;
        
        let cli = Cli {
            url: None,
            output: None,
            config: None,
            github_token: None,
            timeout: 30,
            format: OutputFormat::Markdown,
            no_frontmatter: false,
            frontmatter_only: true, // Only frontmatter
            verbose: false,
            quiet: false,
            debug: false,
            user_agent: None,
            command: None,
        };

        let markdown = Markdown::new("# Test without frontmatter".to_string()).expect("Valid markdown");
        let output = format_output(&markdown, &cli).expect("Should format frontmatter output");
        
        // Should show message when no frontmatter found
        assert!(output.contains("No frontmatter found"));
    }

    #[test]
    fn test_write_output_to_file() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_file = temp_dir.path().join("test-output.md");
        
        let content = "# Test Output\n\nThis is test content.";
        
        write_output(content, Some(output_file.to_str().unwrap())).expect("Should write to file");
        
        // File should exist and contain expected content
        assert!(output_file.exists());
        let file_content = fs::read_to_string(&output_file).expect("Should read file");
        assert_eq!(file_content, content);
    }

    #[test]
    fn test_write_output_to_stdout() {
        // Test writing to stdout (no file specified)
        let content = "Test stdout content";
        
        // This should not panic or return error
        let result = write_output(content, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_url_type_function() {
        let result = detect_url_type("https://github.com/owner/repo/issues/123");
        assert!(result.is_ok());
        
        let result = detect_url_type("https://docs.google.com/document/d/abc123/edit");
        assert!(result.is_ok());
        
        let result = detect_url_type("https://example.com");
        assert!(result.is_ok());
        
        // Invalid URL should return error
        let result = detect_url_type("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_authentication_options() {
        let args = vec![
            "markdowndown",
            "https://example.com",
            "--github-token",
            "test-token",
            "--user-agent",
            "test-agent/1.0",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.github_token, Some("test-token".to_string()));
        assert_eq!(cli.user_agent, Some("test-agent/1.0".to_string()));
    }

    #[test]
    fn test_cli_frontmatter_options() {
        let args = vec![
            "markdowndown",
            "https://example.com",
            "--no-frontmatter",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.no_frontmatter);
        assert!(!cli.frontmatter_only);

        let args = vec![
            "markdowndown",
            "https://example.com",
            "--frontmatter-only",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(!cli.no_frontmatter);
        assert!(cli.frontmatter_only);
    }

    #[test]
    fn test_cli_logging_options() {
        let args = vec![
            "markdowndown",
            "https://example.com",
            "--verbose",
            "--debug",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.verbose);
        assert!(cli.debug);

        let args = vec![
            "markdowndown",
            "https://example.com",
            "--quiet",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.quiet);
        assert!(!cli.verbose);
    }

    #[test]
    fn test_batch_command_full_options() {
        let args = vec![
            "markdowndown",
            "batch",
            "urls.txt",
            "--concurrency",
            "10",
            "--output-dir",
            "output",
            "--stats",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Some(Commands::Batch {
            file,
            concurrency,
            output_dir,
            stats,
        }) = cli.command
        {
            assert_eq!(file, "urls.txt");
            assert_eq!(concurrency, 10);
            assert_eq!(output_dir, Some("output".to_string()));
            assert!(stats);
        } else {
            panic!("Expected batch command");
        }
    }
}
