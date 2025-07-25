# Performance Guide

This guide covers performance characteristics, optimization techniques, and best practices for using markdowndown efficiently in production environments.

## Performance Overview

markdowndown performance is primarily limited by:

1. **Network I/O** - Time to fetch content from URLs
2. **Content Size** - Larger documents take longer to process
3. **Content Complexity** - Complex HTML or formatting increases processing time
4. **URL Type** - Different converters have different performance characteristics

## Benchmark Results

Based on typical hardware (modern laptop with good internet connection):

### Conversion Times by URL Type

| URL Type | Small (< 10KB) | Medium (10-100KB) | Large (100KB-1MB) | Very Large (> 1MB) |
|----------|----------------|-------------------|-------------------|-------------------|
| **HTML** | 0.5-2s | 1-5s | 3-15s | 10-60s |
| **Google Docs** | 1-3s | 2-8s | 5-30s | 15-120s |
| **GitHub Issues** | 0.5-2s | 1-4s | 2-10s | 5-30s |
| **Office 365** | 2-8s | 5-20s | 15-90s | 30-300s |

### Memory Usage

- **Base memory**: ~5MB for library initialization
- **Per conversion**: 2-5x the final markdown size
- **Peak memory**: Document size + processing overhead
- **Concurrent conversions**: Linear scaling with number of parallel operations

### Network Performance Factors

- **Latency**: Primary factor for small documents
- **Bandwidth**: Primary factor for large documents
- **DNS resolution**: ~50-200ms first-time cost per domain
- **TLS handshake**: ~200-500ms first-time cost per domain
- **Server response time**: Varies widely by source

## Configuration for Performance

### Timeout Settings

Balance responsiveness with success rate:

```rust
use markdowndown::{MarkdownDown, Config};
use std::time::Duration;

// Fast configuration (prioritize speed)
let fast_config = Config::builder()
    .timeout_seconds(10)        // Fail fast
    .max_retries(1)            // Don't retry much
    .max_redirects(3)          // Limit redirect following
    .build();

// Reliable configuration (prioritize success)
let reliable_config = Config::builder()
    .timeout_seconds(120)      // Wait longer
    .max_retries(5)           // Retry more often
    .max_redirects(10)        // Follow more redirects
    .build();

// Balanced configuration (good for most cases)
let balanced_config = Config::builder()
    .timeout_seconds(30)       // Reasonable timeout
    .max_retries(3)           // Standard retries
    .max_redirects(5)         // Moderate redirects
    .build();
```

### Memory Optimization

```rust
use markdowndown::{MarkdownDown, Config};

let memory_optimized_config = Config::builder()
    .timeout_seconds(30)
    .max_retries(2)
    .placeholder_max_content_length(1000)  // Limit placeholder size
    .normalize_whitespace(true)             // Reduce output size
    .max_consecutive_blank_lines(1)         // Reduce blank lines
    .build();
```

### Batch Processing Optimization

```rust
use markdowndown::{MarkdownDown, Config};
use std::time::Duration;

let batch_config = Config::builder()
    .timeout_seconds(15)       // Shorter timeout for batch
    .max_retries(2)           // Fewer retries
    .include_frontmatter(false) // Skip frontmatter for speed
    .normalize_whitespace(false) // Skip normalization
    .build();
```

## Concurrent Processing

### Parallel Processing with Semaphore

Control concurrency to avoid overwhelming servers:

```rust
use markdowndown::{MarkdownDown, Config};
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://httpbin.org/html",
        "https://httpbin.org/json", 
        "https://httpbin.org/xml",
        "https://httpbin.org/robots.txt",
        // Add more URLs...
    ];
    
    let config = Config::builder()
        .timeout_seconds(30)
        .build();
    
    // Limit to 5 concurrent requests
    let semaphore = Arc::new(Semaphore::new(5));
    let mut tasks = Vec::new();
    
    let start_time = Instant::now();
    
    for url in urls {
        let semaphore = semaphore.clone();
        let config = config.clone();
        
        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let md = MarkdownDown::with_config(config);
            
            let task_start = Instant::now();
            match md.convert_url(&url).await {
                Ok(markdown) => {
                    println!("✅ {}: {} chars in {:?}", 
                        url, markdown.as_str().len(), task_start.elapsed());
                    Ok((url, markdown.as_str().len(), task_start.elapsed()))
                }
                Err(e) => {
                    println!("❌ {}: {} in {:?}", 
                        url, e, task_start.elapsed());
                    Err((url, e.to_string(), task_start.elapsed()))
                }
            }
        });
        
        tasks.push(task);
    }
    
    // Collect results
    let mut successful = 0;
    let mut total_chars = 0;
    
    for task in tasks {
        match task.await? {
            Ok((_, chars, _)) => {
                successful += 1;
                total_chars += chars;
            }
            Err((_, _, _)) => {
                // Error already logged
            }
        }
    }
    
    let total_time = start_time.elapsed();
    
    println!("\n📊 Batch Results:");
    println!("   Successful: {}", successful);
    println!("   Total chars: {}", total_chars);
    println!("   Total time: {:?}", total_time);
    println!("   Throughput: {:.1} chars/sec", 
        total_chars as f64 / total_time.as_secs_f64());
    
    Ok(())
}
```

### Streaming Processing

Process URLs as they become available:

```rust
use markdowndown::{MarkdownDown, Config};
use futures::stream::{self, StreamExt};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://httpbin.org/html",
        "https://httpbin.org/json",
        "https://httpbin.org/xml",
        // More URLs...
    ];
    
    let config = Config::builder()
        .timeout_seconds(20)
        .build();
    
    let start_time = Instant::now();
    
    // Process as stream with controlled concurrency
    let results: Vec<_> = stream::iter(urls)
        .map(|url| {
            let config = config.clone();
            async move {
                let md = MarkdownDown::with_config(config);
                let task_start = Instant::now();
                
                match md.convert_url(&url).await {
                    Ok(markdown) => {
                        let duration = task_start.elapsed();
                        println!("✅ {}: {} chars in {:?}", 
                            url, markdown.as_str().len(), duration);
                        Ok((url, markdown.as_str().len(), duration))
                    }
                    Err(e) => {
                        let duration = task_start.elapsed();
                        println!("❌ {}: {} in {:?}", url, e, duration);
                        Err((url, e.to_string(), duration))
                    }
                }
            }
        })
        .buffer_unordered(3) // Process up to 3 concurrently
        .collect()
        .await;
    
    // Analyze results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let total_chars: usize = results.iter()
        .filter_map(|r| r.as_ref().ok())
        .map(|(_, chars, _)| *chars)
        .sum();
    
    let total_time = start_time.elapsed();
    
    println!("\n📈 Stream Processing Results:");
    println!("   Processed: {} URLs", results.len());
    println!("   Successful: {}", successful);
    println!("   Success rate: {:.1}%", 
        successful as f64 / results.len() as f64 * 100.0);
    println!("   Total content: {} chars", total_chars);
    println!("   Total time: {:?}", total_time);
    println!("   Average: {:?}/URL", 
        total_time / results.len() as u32);
    
    Ok(())
}
```

## Caching Strategies

### Simple In-Memory Cache

```rust
use markdowndown::{MarkdownDown, Config};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct CacheEntry {
    content: String,
    timestamp: Instant,
    access_count: u64,
}

impl CacheEntry {
    fn new(content: String) -> Self {
        Self {
            content,
            timestamp: Instant::now(),
            access_count: 1,
        }
    }
    
    fn is_expired(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
    
    fn access(&mut self) -> &str {
        self.access_count += 1;
        &self.content
    }
}

struct MarkdownCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    md: MarkdownDown,
    ttl: Duration,
}

impl MarkdownCache {
    fn new(config: Config, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            md: MarkdownDown::with_config(config),
            ttl,
        }
    }
    
    async fn convert_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get_mut(url) {
                if !entry.is_expired(self.ttl) {
                    println!("🎯 Cache hit for: {}", url);
                    return Ok(entry.access().to_string());
                } else {
                    // Remove expired entry
                    cache.remove(url);
                    println!("⏰ Cache expired for: {}", url);
                }
            }
        }
        
        // Not in cache or expired, fetch new
        println!("🌐 Fetching: {}", url);
        let start = Instant::now();
        let markdown = self.md.convert_url(url).await?;
        let fetch_time = start.elapsed();
        
        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(url.to_string(), CacheEntry::new(markdown.as_str().to_string()));
        }
        
        println!("✅ Cached: {} in {:?}", url, fetch_time);
        Ok(markdown.as_str().to_string())
    }
    
    fn cache_stats(&self) -> (usize, u64) {
        let cache = self.cache.lock().unwrap();
        let size = cache.len();
        let total_accesses = cache.values().map(|e| e.access_count).sum();
        (size, total_accesses)
    }
    
    fn clear_expired(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.retain(|_, entry| !entry.is_expired(self.ttl));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .timeout_seconds(30)
        .build();
    
    let cache = MarkdownCache::new(config, Duration::from_secs(300)); // 5 minute TTL
    
    let urls = vec![
        "https://httpbin.org/html",
        "https://httpbin.org/json",
        "https://httpbin.org/html", // Duplicate - should hit cache
        "https://httpbin.org/xml",
        "https://httpbin.org/json", // Duplicate - should hit cache
    ];
    
    for url in urls {
        match cache.convert_url(url).await {
            Ok(content) => {
                println!("   Content: {} chars\n", content.len());
            }
            Err(e) => {
                println!("   Error: {}\n", e);
            }
        }
    }
    
    let (cache_size, total_accesses) = cache.cache_stats();
    println!("📊 Cache stats: {} entries, {} total accesses", cache_size, total_accesses);
    
    Ok(())
}
```

### Persistent Cache with File Storage

```rust
use markdowndown::{MarkdownDown, Config};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct CachedResult {
    url: String,
    content: String,
    timestamp: u64,
    size: usize,
}

impl CachedResult {
    fn new(url: String, content: String) -> Self {
        let size = content.len();
        Self {
            url,
            content,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            size,
        }
    }
    
    fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now - self.timestamp > ttl_seconds
    }
}

struct PersistentCache {
    cache_dir: std::path::PathBuf,
    md: MarkdownDown,
    ttl_seconds: u64,
}

impl PersistentCache {
    fn new<P: AsRef<Path>>(cache_dir: P, config: Config, ttl_seconds: u64) -> Result<Self, std::io::Error> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            md: MarkdownDown::with_config(config),
            ttl_seconds,
        })
    }
    
    fn cache_path(&self, url: &str) -> std::path::PathBuf {
        let hash = format!("{:x}", md5::compute(url.as_bytes()));
        self.cache_dir.join(format!("{}.json", hash))
    }
    
    async fn convert_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let cache_path = self.cache_path(url);
        
        // Try to load from cache
        if cache_path.exists() {
            if let Ok(cached_data) = std::fs::read_to_string(&cache_path) {
                if let Ok(cached_result) = serde_json::from_str::<CachedResult>(&cached_data) {
                    if !cached_result.is_expired(self.ttl_seconds) {
                        println!("🎯 Cache hit: {} ({} chars)", url, cached_result.size);
                        return Ok(cached_result.content);
                    } else {
                        println!("⏰ Cache expired: {}", url);
                        std::fs::remove_file(&cache_path).ok(); // Clean up expired cache
                    }
                }
            }
        }
        
        // Fetch from network
        println!("🌐 Fetching: {}", url);
        let start = std::time::Instant::now();
        let markdown = self.md.convert_url(url).await?;
        let fetch_time = start.elapsed();
        
        // Save to cache
        let cached_result = CachedResult::new(url.to_string(), markdown.as_str().to_string());
        if let Ok(json) = serde_json::to_string(&cached_result) {
            if let Err(e) = std::fs::write(&cache_path, json) {
                eprintln!("⚠️ Failed to write cache: {}", e);
            } else {
                println!("💾 Cached: {} in {:?}", url, fetch_time);
            }
        }
        
        Ok(markdown.as_str().to_string())
    }
    
    fn cache_size(&self) -> Result<(usize, u64), std::io::Error> {
        let mut file_count = 0;
        let mut total_bytes = 0;
        
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                file_count += 1;
                total_bytes += entry.metadata()?.len();
            }
        }
        
        Ok((file_count, total_bytes))
    }
    
    fn cleanup_expired(&self) -> Result<usize, std::io::Error> {
        let mut cleaned = 0;
        
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(data) = std::fs::read_to_string(&path) {
                    if let Ok(cached_result) = serde_json::from_str::<CachedResult>(&data) {
                        if cached_result.is_expired(self.ttl_seconds) {
                            std::fs::remove_file(&path)?;
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        
        Ok(cleaned)
    }
}
```

## Memory Management

### Controlling Memory Usage

```rust
use markdowndown::{MarkdownDown, Config};

// Configuration for memory-constrained environments
let memory_config = Config::builder()
    .timeout_seconds(15)                    // Shorter timeout
    .max_retries(1)                        // Fewer retries
    .placeholder_max_content_length(500)   // Limit placeholder size
    .include_frontmatter(false)            // Skip frontmatter
    .normalize_whitespace(true)            // Reduce content size
    .max_consecutive_blank_lines(1)        // Minimize blank lines
    .build();

async fn memory_efficient_convert(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let md = MarkdownDown::with_config(memory_config);
    let result = md.convert_url(url).await?;
    
    // Return only content, drop the Markdown wrapper immediately
    Ok(result.content_only())
}
```

### Large Document Handling

```rust
use markdowndown::{MarkdownDown, Config};
use std::time::Duration;

async fn handle_large_document(url: &str, max_size_mb: usize) -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .timeout_seconds(300)  // 5 minutes for large docs
        .max_retries(1)       // Don't retry large docs
        .build();
    
    let md = MarkdownDown::with_config(config);
    
    println!("📥 Processing large document: {}", url);
    let start = std::time::Instant::now();
    
    let result = md.convert_url(url).await?;
    let duration = start.elapsed();
    let size_mb = result.as_str().len() as f64 / 1024.0 / 1024.0;
    
    if size_mb > max_size_mb as f64 {
        return Err(format!("Document too large: {:.1}MB > {}MB", size_mb, max_size_mb).into());
    }
    
    println!("✅ Processed {:.1}MB in {:?}", size_mb, duration);
    Ok(result.as_str().to_string())
}
```

## Performance Monitoring

### Conversion Metrics

```rust
use markdowndown::{MarkdownDown, Config};
use std::time::{Instant, Duration};
use std::collections::HashMap;

#[derive(Debug)]
struct ConversionMetrics {
    url: String,
    success: bool,
    duration: Duration,
    content_size: usize,
    error_type: Option<String>,
}

struct PerformanceMonitor {
    metrics: Vec<ConversionMetrics>,
    md: MarkdownDown,
}

impl PerformanceMonitor {
    fn new(config: Config) -> Self {
        Self {
            metrics: Vec::new(),
            md: MarkdownDown::with_config(config),
        }
    }
    
    async fn convert_url(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        match self.md.convert_url(url).await {
            Ok(markdown) => {
                let metrics = ConversionMetrics {
                    url: url.to_string(),
                    success: true,
                    duration: start.elapsed(),
                    content_size: markdown.as_str().len(),
                    error_type: None,
                };
                
                self.metrics.push(metrics);
                Ok(markdown.as_str().to_string())
            }
            Err(e) => {
                let metrics = ConversionMetrics {
                    url: url.to_string(),
                    success: false,
                    duration: start.elapsed(),
                    content_size: 0,
                    error_type: Some(e.to_string()),
                };
                
                self.metrics.push(metrics);
                Err(e.into())
            }
        }
    }
    
    fn report(&self) {
        let total = self.metrics.len();
        let successful = self.metrics.iter().filter(|m| m.success).count();
        let failed = total - successful;
        
        let total_duration: Duration = self.metrics.iter().map(|m| m.duration).sum();
        let avg_duration = total_duration / total as u32;
        
        let total_content: usize = self.metrics.iter().map(|m| m.content_size).sum();
        
        let fastest = self.metrics.iter()
            .filter(|m| m.success)
            .min_by_key(|m| m.duration);
        
        let slowest = self.metrics.iter()
            .filter(|m| m.success)
            .max_by_key(|m| m.duration);
        
        println!("📊 Performance Report:");
        println!("   Total conversions: {}", total);
        println!("   Successful: {} ({:.1}%)", successful, 
            successful as f64 / total as f64 * 100.0);
        println!("   Failed: {} ({:.1}%)", failed, 
            failed as f64 / total as f64 * 100.0);
        println!("   Average duration: {:?}", avg_duration);
        println!("   Total content: {} chars", total_content);
        println!("   Throughput: {:.1} chars/sec", 
            total_content as f64 / total_duration.as_secs_f64());
        
        if let Some(fastest) = fastest {
            println!("   Fastest: {:?} ({})", fastest.duration, fastest.url);
        }
        
        if let Some(slowest) = slowest {
            println!("   Slowest: {:?} ({})", slowest.duration, slowest.url);
        }
        
        // Error analysis
        let mut error_counts: HashMap<String, usize> = HashMap::new();
        for metric in &self.metrics {
            if let Some(error_type) = &metric.error_type {
                let simplified = if error_type.contains("timeout") {
                    "timeout"
                } else if error_type.contains("network") {
                    "network"
                } else if error_type.contains("auth") {
                    "authentication"
                } else {
                    "other"
                };
                *error_counts.entry(simplified.to_string()).or_insert(0) += 1;
            }
        }
        
        if !error_counts.is_empty() {
            println!("   Error breakdown:");
            for (error_type, count) in error_counts {
                println!("     {}: {}", error_type, count);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .timeout_seconds(30)
        .build();
    
    let mut monitor = PerformanceMonitor::new(config);
    
    let urls = vec![
        "https://httpbin.org/html",
        "https://httpbin.org/json",
        "https://httpbin.org/xml",
        "https://httpbin.org/delay/2",
        "https://invalid-domain.nonexistent",
    ];
    
    for url in urls {
        match monitor.convert_url(url).await {
            Ok(content) => {
                println!("✅ {}: {} chars", url, content.len());
            }
            Err(e) => {
                println!("❌ {}: {}", url, e);
            }
        }
    }
    
    monitor.report();
    Ok(())
}
```

## Optimization Tips

### URL-Specific Optimizations

```rust
use markdowndown::{MarkdownDown, Config, detect_url_type, types::UrlType};

async fn optimized_convert(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url_type = detect_url_type(url)?;
    
    let config = match url_type {
        UrlType::Html => {
            // HTML pages - balance speed and reliability
            Config::builder()
                .timeout_seconds(30)
                .max_retries(2)
                .normalize_whitespace(true)
                .build()
        }
        UrlType::GoogleDocs => {
            // Google Docs - may be slow but usually reliable
            Config::builder()
                .timeout_seconds(90)
                .max_retries(1)  // Don't retry slow docs
                .build()
        }
        UrlType::GitHubIssue => {
            // GitHub - fast API but rate limited
            Config::builder()
                .timeout_seconds(45)
                .max_retries(3)  // Retry rate limit errors
                .github_token(std::env::var("GITHUB_TOKEN").ok())
                .build()
        }
        UrlType::Office365 => {
            // Office 365 - often slow and unreliable
            Config::builder()
                .timeout_seconds(180)
                .max_retries(1)  // Don't retry slow docs
                .build()
        }
    };
    
    let md = MarkdownDown::with_config(config);
    let result = md.convert_url(url).await?;
    Ok(result.as_str().to_string())
}
```

### Batch Processing Optimization

```rust
use markdowndown::{MarkdownDown, Config};
use futures::stream::{self, StreamExt};
use std::collections::HashMap;

async fn optimized_batch_convert(urls: Vec<String>) -> HashMap<String, Result<String, String>> {
    // Group URLs by type for type-specific optimization
    let mut url_groups: HashMap<String, Vec<String>> = HashMap::new();
    
    for url in urls {
        let url_type = markdowndown::detect_url_type(&url)
            .map(|t| t.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        url_groups.entry(url_type).or_default().push(url);
    }
    
    let mut all_results = HashMap::new();
    
    for (url_type, urls_of_type) in url_groups {
        println!("📦 Processing {} URLs of type: {}", urls_of_type.len(), url_type);
        
        // Configure based on URL type
        let (config, concurrency) = match url_type.as_str() {
            "HTML" => (
                Config::builder().timeout_seconds(20).build(),
                5  // More concurrent for HTML
            ),
            "Google Docs" => (
                Config::builder().timeout_seconds(60).build(),
                2  // Less concurrent for Google Docs
            ),
            "GitHub Issue" => (
                Config::builder()
                    .timeout_seconds(30)
                    .github_token(std::env::var("GITHUB_TOKEN").ok())
                    .build(),
                3  // Moderate for GitHub (rate limits)
            ),
            _ => (
                Config::builder().timeout_seconds(30).build(),
                3  // Default
            ),
        };
        
        // Process this group
        let group_results: Vec<_> = stream::iter(urls_of_type)
            .map(|url| {
                let config = config.clone();
                async move {
                    let md = MarkdownDown::with_config(config);
                    match md.convert_url(&url).await {
                        Ok(markdown) => (url, Ok(markdown.as_str().to_string())),
                        Err(e) => (url, Err(e.to_string())),
                    }
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;
        
        // Add to overall results
        for (url, result) in group_results {
            all_results.insert(url, result);
        }
    }
    
    all_results
}
```

## Performance Best Practices

### 1. Choose Appropriate Timeouts
- **HTML pages**: 20-30 seconds
- **Google Docs**: 60-120 seconds  
- **GitHub Issues**: 30-45 seconds
- **Office 365**: 120-300 seconds

### 2. Control Concurrency
- **Local processing**: 5-10 concurrent requests
- **Production servers**: 10-20 concurrent requests
- **Rate-limited APIs**: 2-5 concurrent requests

### 3. Implement Caching
- Cache successful conversions for repeated URLs
- Use appropriate TTLs (5-60 minutes)
- Consider persistent caching for long-running applications

### 4. Handle Errors Efficiently
- Don't retry non-retryable errors
- Use exponential backoff for retries
- Fail fast for clearly invalid inputs

### 5. Monitor Performance
- Track conversion times by URL type
- Monitor error rates and types
- Alert on performance degradation

### 6. Optimize Memory Usage
- Process large batches in chunks
- Clear results immediately after processing
- Use streaming for very large datasets

### 7. Network Optimization
- Reuse HTTP connections when possible
- Consider geographic proximity to targets
- Implement request deduplication

## Troubleshooting Performance Issues

### Slow Conversions
1. **Check network latency** to target servers
2. **Increase timeout** for large documents
3. **Reduce concurrent requests** to avoid overwhelming servers
4. **Implement caching** for repeated URLs

### High Memory Usage
1. **Process in smaller batches**
2. **Disable frontmatter** if not needed
3. **Limit placeholder content length**
4. **Clear results immediately** after processing

### High Error Rates
1. **Check timeout settings** - may be too aggressive
2. **Implement proper retry logic** with backoff
3. **Monitor rate limits** on APIs
4. **Add authentication tokens** where required

### Poor Throughput
1. **Increase concurrency** within reason
2. **Optimize configuration** per URL type
3. **Implement caching** for repeated requests
4. **Use streaming processing** for large batches

## Next Steps

- Review the [Configuration Guide](configuration.md) for performance-related settings
- Check the [Error Handling Guide](error-handling.md) for efficient error handling
- See [examples/batch_processing.rs](../examples/batch_processing.rs) for practical performance examples
- Explore the [Troubleshooting Guide](troubleshooting.md) for performance issue resolution