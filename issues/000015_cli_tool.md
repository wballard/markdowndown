# CLI Tool Implementation

Create a command-line interface for the markdowndown library to provide easy access to URL conversion functionality from the terminal.

## Objectives

- Provide a user-friendly CLI for converting URLs to markdown
- Support all library features through command-line options
- Enable batch processing and output formatting options
- Include comprehensive help and error reporting

## Tasks

1. Create CLI binary structure:
   - Add `[[bin]]` section to Cargo.toml for `markdowndown` binary  
   - Create `src/bin/markdowndown.rs` as main CLI entry point
   - Set up command-line argument parsing with `clap`
   - Implement proper exit codes and error handling

2. Add core CLI commands:
   - `markdowndown <URL>` - Convert single URL to markdown
   - `markdowndown batch <file>` - Convert URLs from input file
   - `markdowndown detect <URL>` - Show detected URL type without conversion
   - `markdowndown --version` - Show version information

3. Implement output options:
   - `--output <file>` - Save to file instead of stdout
   - `--format <format>` - Output format (markdown, json, yaml)
   - `--no-frontmatter` - Exclude YAML frontmatter from output
   - `--frontmatter-only` - Output only the frontmatter

4. Add configuration options:
   - `--config <file>` - Load configuration from file
   - `--github-token <token>` - GitHub API authentication
   - `--timeout <seconds>` - Network timeout configuration
   - `--user-agent <string>` - Custom user agent string

5. Create batch processing functionality:
   - Read URLs from file (one per line)
   - Progress reporting for batch operations
   - Parallel processing with configurable concurrency
   - Skip failed URLs and continue processing

6. Implement verbose and debug modes:
   - `--verbose` - Show detailed processing information
   - `--debug` - Enable debug logging
   - `--quiet` - Suppress all output except results
   - `--stats` - Show conversion statistics

7. Add validation and preview features:
   - `--dry-run` - Validate URLs without conversion
   - `--preview` - Show first N characters of output
   - `--validate-only` - Check URL accessibility without conversion
   - `--list-types` - Show supported URL types

8. Create configuration file support:
   - TOML configuration file format
   - Default config locations (~/.markdowndown.toml, ./markdowndown.toml)
   - Environment variable overrides
   - Configuration validation and error reporting

9. Add output formatting and filtering:
   - JSON output for programmatic use
   - CSV format for batch results
   - Template-based output formatting
   - Content filtering and transformation options

10. Implement comprehensive help system:
    - Detailed help for each command and option
    - Usage examples for common scenarios
    - Error message improvements with suggestions
    - Man page generation for Unix systems

## Acceptance Criteria

- [ ] CLI handles all library functionality
- [ ] Batch processing works efficiently with large URL lists
- [ ] Configuration system is flexible and well-documented
- [ ] Error messages are clear and actionable
- [ ] Help system provides comprehensive guidance
- [ ] Performance is acceptable for interactive use
- [ ] Cross-platform compatibility (Windows, macOS, Linux)
- [ ] Integration tests cover CLI functionality

## Dependencies

- Previous: [000014_documentation]
- Requires: Complete library implementation
- Add dependencies: `clap`, `tokio`, `serde`, `toml`

## CLI Structure

```rust
// src/bin/markdowndown.rs
use clap::{Parser, Subcommand};
use markdowndown::{MarkdownDown, Config};

#[derive(Parser)]
#[command(name = "markdowndown")]
#[command(about = "Convert URLs to markdown")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// URL to convert (if no subcommand specified)
    url: Option<String>,
    
    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,
    
    /// Configuration file
    #[arg(short, long)]
    config: Option<String>,
    
    /// GitHub API token
    #[arg(long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,
    
    /// Network timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert URLs from a file
    Batch {
        /// File containing URLs (one per line)
        file: String,
        /// Number of concurrent conversions
        #[arg(short, long, default_value = "5")]
        concurrency: usize,
    },
    /// Detect URL type without conversion
    Detect { url: String },
    /// List supported URL types
    ListTypes,
}
```

## Command Examples

### Basic Usage
```bash
# Convert single URL
markdowndown https://example.com/article

# Save to file
markdowndown https://docs.google.com/document/d/abc123/edit -o output.md

# With GitHub token
markdowndown https://github.com/owner/repo/issues/123 --github-token ghp_xxx
```

### Batch Processing
```bash
# Convert URLs from file
markdowndown batch urls.txt

# With custom concurrency and output directory
markdowndown batch urls.txt --concurrency 10 --output-dir ./markdown/

# Show progress and statistics
markdowndown batch urls.txt --verbose --stats
```

### Configuration and Options
```bash
# Use configuration file
markdowndown --config ~/.markdowndown.toml https://example.com

# Debug mode with timeout
markdowndown --debug --timeout 60 https://slow-site.com

# Quiet mode (only output results)
markdowndown --quiet https://example.com
```

## Configuration File Format

```toml
# ~/.markdowndown.toml
[http]
timeout_seconds = 60
user_agent = "my-scraper/1.0"
max_redirects = 10

[authentication]
github_token = "ghp_your_token_here"

[output]
include_frontmatter = true
format = "markdown"

[batch]
default_concurrency = 5
skip_failures = true
show_progress = true

[logging]
level = "info"
format = "human"
```

## Output Formats

### Markdown (Default)
```markdown
---
source_url: "https://example.com/article"
exporter: "html2markdown"
date_downloaded: "2024-01-15T10:30:00Z"
---

# Article Title

Article content here...
```

### JSON Format
```json
{
  "url": "https://example.com/article",
  "success": true,
  "content": "# Article Title\n\nArticle content...",
  "frontmatter": {
    "source_url": "https://example.com/article",
    "exporter": "html2markdown", 
    "date_downloaded": "2024-01-15T10:30:00Z"
  },
  "processing_time_ms": 1250
}
```

### CSV Format (for batch results)
```csv
url,success,content_length,processing_time_ms,error
https://example.com/article1,true,2500,1200,
https://example.com/article2,false,0,500,"Network timeout"
```

## Error Handling

```rust
fn handle_cli_error(error: markdowndown::MarkdownError) -> ! {
    match error {
        MarkdownError::ValidationError(_, _) => {
            eprintln!("❌ Invalid URL format");
            eprintln!("💡 Make sure the URL includes http:// or https://");
            std::process::exit(1);
        }
        MarkdownError::NetworkError(_, _) => {
            eprintln!("❌ Network error occurred");
            eprintln!("💡 Check your internet connection and try again");
            std::process::exit(2);
        }
        MarkdownError::AuthenticationError(_, _) => {
            eprintln!("❌ Authentication failed");
            eprintln!("💡 Check your API tokens and permissions");
            std::process::exit(3);
        }
        _ => {
            eprintln!("❌ Unexpected error: {}", error);
            std::process::exit(99);
        }
    }
}
```

## Progress Reporting

```rust
use indicatif::{ProgressBar, ProgressStyle};

async fn process_urls_with_progress(urls: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let pb = ProgressBar::new(urls.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));

    for (i, url) in urls.iter().enumerate() {
        pb.set_message(format!("Converting: {}", url));
        
        match convert_url(url).await {
            Ok(_) => pb.println(format!("✅ {}", url)),
            Err(e) => pb.println(format!("❌ {}: {}", url, e)),
        }
        
        pb.inc(1);
    }
    
    pb.finish_with_message("Batch conversion complete");
    Ok(())
}
```

## Installation and Distribution

```bash
# Install from crates.io
cargo install markdowndown

# Install from source
git clone https://github.com/username/markdowndown
cd markdowndown
cargo install --path .

# Cross-compilation for different platforms
cargo build --target x86_64-pc-windows-gnu
cargo build --target x86_64-apple-darwin
cargo build --target x86_64-unknown-linux-gnu
```

## Testing Strategy

- Unit tests for CLI argument parsing
- Integration tests with temporary files
- End-to-end tests with real URLs
- Performance tests for batch processing
- Cross-platform compatibility testing

## Future Enhancements

- Shell completion scripts (bash, zsh, fish)
- Watch mode for monitoring URL changes
- Plugin system for custom converters
- GUI wrapper for desktop use
- Docker image for containerized usage