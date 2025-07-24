//! Performance benchmarks for markdowndown library.
//!
//! This module contains benchmarks for testing the performance of
//! various components of the markdowndown library.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use markdowndown::client::HttpClient;
use markdowndown::config::Config;
use markdowndown::converters::{HtmlConverter, HtmlConverterConfig};
use markdowndown::detection::UrlDetector;
use markdowndown::types::{Markdown, Url};
use markdowndown::{detect_url_type, MarkdownDown};
use std::time::Duration;

/// Sample HTML content for benchmarking
#[allow(dead_code)]
fn sample_html_content() -> &'static str {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Benchmark Document</title>
    <meta name="description" content="A document for performance testing">
</head>
<body>
    <article>
        <h1>Performance Test Document</h1>
        <p>This document is used for benchmarking the HTML to markdown conversion process.</p>
        
        <h2>Section 1: Introduction</h2>
        <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
        <ul>
            <li>First bullet point with <strong>bold text</strong></li>
            <li>Second bullet point with <em>italic text</em></li>
            <li>Third bullet point with <a href="https://example.com">a hyperlink</a></li>
        </ul>
        
        <h2>Section 2: Content</h2>
        <blockquote>
            <p>This is a blockquote that should be preserved in the markdown conversion.</p>
        </blockquote>
        
        <pre><code>function example() {
    console.log("This is a code block");
    return "benchmark";
}</code></pre>
        
        <h2>Section 3: Tables</h2>
        <table>
            <thead>
                <tr>
                    <th>Column 1</th>
                    <th>Column 2</th>
                    <th>Column 3</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>Row 1, Col 1</td>
                    <td>Row 1, Col 2</td>
                    <td>Row 1, Col 3</td>
                </tr>
                <tr>
                    <td>Row 2, Col 1</td>
                    <td>Row 2, Col 2</td>
                    <td>Row 2, Col 3</td>
                </tr>
            </tbody>
        </table>
    </article>
</body>
</html>"#
}

/// Sample markdown content for benchmarking
fn sample_markdown_content() -> &'static str {
    r#"# Performance Test Document

This document is used for benchmarking markdown processing operations.

## Section 1: Introduction

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

* First bullet point with **bold text**
* Second bullet point with *italic text*
* Third bullet point with [a hyperlink](https://example.com)

## Section 2: Content

> This is a blockquote that should be preserved during processing.

```javascript
function example() {
    console.log("This is a code block");
    return "benchmark";
}
```

## Section 3: Performance Notes

This content is designed to test various markdown processing scenarios:

1. **Headers**: Multiple levels of headers
2. **Lists**: Both ordered and unordered lists
3. **Text formatting**: Bold, italic, and links
4. **Code blocks**: Both inline and block code
5. **Blockquotes**: Quote formatting
6. **Complex structures**: Nested elements

The goal is to measure processing time across different content types and sizes."#
}

/// Benchmark URL type detection
fn bench_url_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_detection");

    let test_urls = vec![
        ("html", "https://example.com/article.html"),
        (
            "google_docs",
            "https://docs.google.com/document/d/abc123/edit",
        ),
        ("office365", "https://company.sharepoint.com/doc.docx"),
        ("github_issue", "https://github.com/owner/repo/issues/123"),
    ];

    for (name, url) in test_urls {
        group.bench_with_input(BenchmarkId::new("detect_type", name), url, |b, url| {
            b.iter(|| detect_url_type(black_box(url)))
        });
    }

    group.finish();
}

/// Benchmark MarkdownDown creation and configuration
fn bench_markdowndown_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdowndown_creation");

    // Benchmark default creation
    group.bench_function("new_default", |b| b.iter(MarkdownDown::new));

    // Benchmark creation with custom config
    group.bench_function("new_with_config", |b| {
        let config = Config::builder()
            .timeout_seconds(30)
            .max_retries(3)
            .user_agent("benchmark/1.0")
            .build();

        b.iter(|| MarkdownDown::with_config(black_box(config.clone())))
    });

    group.finish();
}

/// Benchmark markdown content processing
fn bench_markdown_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("markdown_processing");

    let medium_content = sample_markdown_content().repeat(5);
    let large_content = sample_markdown_content().repeat(20);
    let content_sizes = vec![
        ("small", sample_markdown_content()),
        ("medium", &medium_content),
        ("large", &large_content),
    ];

    for (size_name, content) in content_sizes {
        // Benchmark markdown creation
        group.bench_with_input(
            BenchmarkId::new("create_markdown", size_name),
            &content,
            |b, content| b.iter(|| Markdown::new(black_box(content.to_string()))),
        );

        // Benchmark content extraction
        if let Ok(markdown) = Markdown::new(content.to_string()) {
            group.bench_with_input(
                BenchmarkId::new("content_only", size_name),
                &markdown,
                |b, markdown| b.iter(|| black_box(markdown.content_only())),
            );
        }
    }

    group.finish();
}

/// Benchmark URL creation and validation
fn bench_url_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_creation");

    let test_urls = vec![
        ("short", "https://example.com"),
        ("medium", "https://example.com/path/to/document.html"),
        ("long", "https://example.com/very/long/path/with/many/segments/and/parameters?param1=value1&param2=value2&param3=value3"),
    ];

    for (name, url) in test_urls {
        group.bench_with_input(BenchmarkId::new("create_url", name), url, |b, url| {
            b.iter(|| Url::new(black_box(url.to_string())))
        });
    }

    group.finish();
}

/// Benchmark HTML converter configuration
fn bench_html_converter_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("html_converter_config");

    // Benchmark default configuration
    group.bench_function("default_config", |b| b.iter(HtmlConverterConfig::default));

    // Benchmark custom configuration
    group.bench_function("custom_config", |b| {
        b.iter(|| HtmlConverterConfig {
            max_line_width: black_box(120),
            remove_scripts_styles: black_box(true),
            remove_navigation: black_box(true),
            remove_sidebars: black_box(true),
            remove_ads: black_box(true),
            max_blank_lines: black_box(2),
        })
    });

    // Benchmark converter creation with config
    group.bench_function("create_converter", |b| {
        let client = HttpClient::new();
        let config = HtmlConverterConfig::default();

        b.iter(|| HtmlConverter::with_config(black_box(client.clone()), black_box(config.clone())))
    });

    group.finish();
}

/// Benchmark configuration building
fn bench_config_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_building");

    // Benchmark simple config
    group.bench_function("simple_config", |b| {
        b.iter(|| {
            Config::builder()
                .timeout_seconds(black_box(30))
                .max_retries(black_box(3))
                .build()
        })
    });

    // Benchmark complex config
    group.bench_function("complex_config", |b| {
        b.iter(|| {
            Config::builder()
                .timeout_seconds(black_box(45))
                .max_retries(black_box(5))
                .user_agent(black_box("test-agent/1.0"))
                .github_token(black_box("token123"))
                .office365_token(black_box("office_token"))
                .include_frontmatter(black_box(true))
                .custom_frontmatter_field(black_box("field1"), black_box("value1"))
                .custom_frontmatter_field(black_box("field2"), black_box("value2"))
                .normalize_whitespace(black_box(true))
                .max_consecutive_blank_lines(black_box(2))
                .build()
        })
    });

    group.finish();
}

/// Benchmark UrlDetector operations
fn bench_url_detector(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_detector");

    let detector = UrlDetector::new();
    let test_urls = [
        "https://example.com/page.html",
        "https://docs.google.com/document/d/abc123/edit",
        "https://company.sharepoint.com/doc.docx",
        "https://github.com/owner/repo/issues/123",
    ];

    // Benchmark URL normalization
    for (i, url) in test_urls.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("normalize_url", i), url, |b, url| {
            b.iter(|| detector.normalize_url(black_box(url)))
        });
    }

    // Benchmark type detection
    for (i, url) in test_urls.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("detect_type", i), url, |b, url| {
            b.iter(|| detector.detect_type(black_box(url)))
        });
    }

    group.finish();
}

/// Benchmark memory usage and allocation patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark repeated markdown creation (tests memory allocation)
    group.bench_function("repeated_markdown_creation", |b| {
        let content = sample_markdown_content();
        b.iter(|| {
            for _ in 0..100 {
                let _ = Markdown::new(black_box(content.to_string()));
            }
        })
    });

    // Benchmark large content processing
    group.bench_function("large_content_processing", |b| {
        let large_content = sample_markdown_content().repeat(50);
        b.iter(|| {
            let markdown = Markdown::new(black_box(large_content.clone())).unwrap();
            let _ = black_box(markdown.content_only());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_url_detection,
    bench_markdowndown_creation,
    bench_markdown_processing,
    bench_url_creation,
    bench_html_converter_config,
    bench_config_building,
    bench_url_detector,
    bench_memory_usage
);

criterion_main!(benches);
