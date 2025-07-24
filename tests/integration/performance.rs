//! Performance and rate limiting integration tests
//!
//! Tests performance characteristics, rate limiting behavior, and error scenarios.

use markdowndown::MarkdownDown;
use std::time::{Duration, Instant};

use super::{IntegrationTestConfig, TestUtils};

/// Test rate limiting behavior across all services
#[tokio::test]
async fn test_rate_limiting_behavior() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();

    if config.skip_external_services || config.skip_slow_tests {
        println!("Skipping rate limiting test - external services disabled or slow tests skipped");
        return Ok(());
    }

    let md = MarkdownDown::new();

    // Test rapid-fire requests to the same service
    let test_urls = [
        "https://httpbin.org/html",
        "https://httpbin.org/html",
        "https://httpbin.org/html",
    ];

    let start_time = Instant::now();
    let mut request_times = Vec::new();
    let mut successes = 0;
    let mut rate_limited = 0;

    for (i, url) in test_urls.iter().enumerate() {
        let request_start = Instant::now();

        println!("Rate limiting test request {}: {url}", i + 1);

        // Apply configured rate limiting
        if i > 0 {
            TestUtils::apply_rate_limit(&config).await;
        }

        let result = md.convert_url(url).await;
        let request_duration = request_start.elapsed();
        request_times.push(request_duration);

        match result {
            Ok(markdown) => {
                successes += 1;
                println!(
                    "  ✓ Success: {} chars in {request_duration:?}",
                    markdown.as_str().len()
                );
                assert!(TestUtils::validate_markdown_quality(markdown.as_str()));
            }
            Err(e) => {
                println!("  ⚠ Failed: {e} in {request_duration:?}");
                if e.to_string().contains("rate limit")
                    || e.to_string().contains("429")
                    || e.to_string().contains("too many requests")
                {
                    rate_limited += 1;
                    println!("    Rate limited - this demonstrates proper rate limiting behavior");
                } else {
                    println!("    Other error type: {e}");
                }
            }
        }
    }

    let total_duration = start_time.elapsed();

    println!("Rate Limiting Test Summary:");
    println!("  Total requests: {}", test_urls.len());
    println!("  Successful: {successes}");
    println!("  Rate limited: {rate_limited}");
    println!("  Total time: {total_duration:?}");
    println!(
        "  Average time per request: {:?}",
        total_duration / test_urls.len() as u32
    );

    // Validate rate limiting behavior
    for (i, duration) in request_times.iter().enumerate() {
        println!("  Request {} duration: {duration:?}", i + 1);
    }

    // Should demonstrate controlled request timing
    if config.request_delay_ms > 0 && request_times.len() > 1 {
        let total_expected_delay =
            Duration::from_millis(config.request_delay_ms * (request_times.len() as u64 - 1));
        assert!(
            total_duration >= total_expected_delay,
            "Total duration should respect rate limiting delays"
        );
    }

    Ok(())
}

/// Comprehensive performance benchmark across all service types
#[tokio::test]
async fn test_comprehensive_performance_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();

    if config.skip_external_services || config.skip_slow_tests {
        println!("Skipping comprehensive performance benchmark - external services disabled or slow tests skipped");
        return Ok(());
    }

    let md = MarkdownDown::new();

    // Performance test cases across different service types and content sizes
    let benchmark_cases = [
        (
            "https://httpbin.org/html",
            "HTML",
            "Small",
            "Simple HTML page",
        ),
        (
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "HTML",
            "Large",
            "Complex Wikipedia page",
        ),
        (
            "https://doc.rust-lang.org/book/ch01-00-getting-started.html",
            "HTML",
            "Medium",
            "Documentation page",
        ),
    ];

    let mut results = Vec::new();

    for (url, service_type, size_category, description) in benchmark_cases.iter() {
        println!("Benchmarking: {description} ({service_type}, {size_category})");

        // Rate limiting
        TestUtils::apply_rate_limit(&config).await;

        let start = Instant::now();
        let result = md.convert_url(url).await;
        let duration = start.elapsed();

        match result {
            Ok(markdown) => {
                let content = markdown.as_str();
                let content_length = content.len();
                let chars_per_second = content_length as f64 / duration.as_secs_f64();

                println!("  ✓ Duration: {duration:?}");
                println!("  ✓ Content: {content_length} chars");
                println!("  ✓ Speed: {chars_per_second:.2} chars/sec");

                // Performance assertions based on content size
                let max_duration = match *size_category {
                    "Small" => config.default_timeout(),
                    "Medium" => config.default_timeout(),
                    "Large" => config.large_document_timeout(),
                    _ => config.default_timeout(),
                };

                assert!(
                    duration < max_duration,
                    "Conversion took too long for {size_category} content: {duration:?} > {max_duration:?}"
                );

                // Quality validation
                assert!(
                    TestUtils::validate_markdown_quality(content),
                    "Performance test should maintain quality"
                );

                results.push((
                    *service_type,
                    *size_category,
                    duration,
                    content_length,
                    chars_per_second,
                ));
            }
            Err(e) => {
                println!("  ⚠ Failed: {e} (may affect benchmark)");
                // Store failed result for analysis
                results.push((*service_type, *size_category, duration, 0, 0.0));
            }
        }

        println!("  ---");
    }

    // Performance analysis
    println!("Performance Benchmark Results:");
    println!("┌─────────────┬──────────┬─────────────┬─────────────┬─────────────────┐");
    println!("│ Service     │ Size     │ Duration    │ Content     │ Speed (chars/s) │");
    println!("├─────────────┼──────────┼─────────────┼─────────────┼─────────────────┤");

    for (service, size, duration, content_length, speed) in results.iter() {
        println!(
            "│ {service:11} │ {size:8} │ {duration:11?} │ {content_length:11} │ {speed:15.2} │"
        );
    }

    println!("└─────────────┴──────────┴─────────────┴─────────────┴─────────────────┘");

    // Calculate aggregate statistics
    let successful_results: Vec<_> = results
        .iter()
        .filter(|(_, _, _, content_length, _)| *content_length > 0)
        .collect();

    if !successful_results.is_empty() {
        let total_duration: Duration = successful_results
            .iter()
            .map(|(_, _, duration, _, _)| *duration)
            .sum();
        let total_content: usize = successful_results
            .iter()
            .map(|(_, _, _, content_length, _)| *content_length)
            .sum();
        let avg_speed: f64 = successful_results
            .iter()
            .map(|(_, _, _, _, speed)| *speed)
            .sum::<f64>()
            / successful_results.len() as f64;

        println!("\nAggregate Performance Statistics:");
        println!(
            "  Successful conversions: {}/{}",
            successful_results.len(),
            results.len()
        );
        println!("  Total processing time: {total_duration:?}");
        println!("  Total content processed: {total_content} chars");
        println!("  Average conversion speed: {avg_speed:.2} chars/sec");
        println!(
            "  Overall throughput: {:.2} chars/sec",
            total_content as f64 / total_duration.as_secs_f64()
        );
    }

    Ok(())
}

/// Test timeout behavior and large document handling
#[tokio::test]
async fn test_timeout_and_large_document_handling() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();

    if config.skip_external_services || config.skip_slow_tests {
        println!("Skipping timeout test - external services disabled or slow tests skipped");
        return Ok(());
    }

    // Test with custom timeout configuration
    let short_timeout_config = markdowndown::Config::builder()
        .timeout_seconds(5) // Very short timeout
        .build();
    let md_short_timeout = MarkdownDown::with_config(short_timeout_config);

    let long_timeout_config = markdowndown::Config::builder()
        .timeout_seconds(120) // Long timeout
        .build();
    let md_long_timeout = MarkdownDown::with_config(long_timeout_config);

    // Test URLs that might take varying amounts of time
    let timeout_test_urls = [
        ("https://httpbin.org/html", "Fast response"),
        (
            "https://en.wikipedia.org/wiki/Rust_(programming_language)",
            "Potentially slow response",
        ),
    ];

    for (url, description) in timeout_test_urls.iter() {
        println!("Testing timeout behavior: {description}");

        // Test with short timeout
        println!("  Testing with 5-second timeout");
        TestUtils::apply_rate_limit(&config).await;

        let short_start = Instant::now();
        let short_result = md_short_timeout.convert_url(url).await;
        let short_duration = short_start.elapsed();

        match short_result {
            Ok(markdown) => {
                println!(
                    "    ✓ Completed within timeout: {short_duration:?} ({} chars)",
                    markdown.as_str().len()
                );
                assert!(
                    short_duration < Duration::from_secs(6),
                    "Should complete within timeout period"
                );
                assert!(TestUtils::validate_markdown_quality(markdown.as_str()));
            }
            Err(e) => {
                println!("    ⚠ Timeout or error: {e} in {short_duration:?}");
                if e.to_string().contains("timeout") {
                    println!("      Timeout behavior working correctly");
                    // Timeout should occur reasonably close to the configured limit
                    assert!(
                        short_duration >= Duration::from_secs(4)
                            && short_duration <= Duration::from_secs(10),
                        "Timeout should occur near the configured limit"
                    );
                } else {
                    println!("      Other error (acceptable): {e}");
                }
            }
        }

        // Test with long timeout
        println!("  Testing with 120-second timeout");
        TestUtils::apply_rate_limit(&config).await;

        let long_start = Instant::now();
        let long_result = md_long_timeout.convert_url(url).await;
        let long_duration = long_start.elapsed();

        match long_result {
            Ok(markdown) => {
                println!(
                    "    ✓ Completed: {long_duration:?} ({} chars)",
                    markdown.as_str().len()
                );
                assert!(TestUtils::validate_markdown_quality(markdown.as_str()));

                // Long timeout should not actually take 120 seconds for simple URLs
                assert!(
                    long_duration < Duration::from_secs(60),
                    "Should complete well before timeout for simple URLs"
                );
            }
            Err(e) => {
                println!("    ⚠ Failed even with long timeout: {e} in {long_duration:?}");
                // Should not be a timeout error with long timeout
                assert!(
                    !e.to_string().contains("timeout") || long_duration >= Duration::from_secs(100),
                    "Should not timeout with 120-second limit unless truly taking that long"
                );
            }
        }

        println!("  ---");
    }

    Ok(())
}

/// Test error scenario performance and recovery
#[tokio::test]
async fn test_error_scenario_performance() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();

    if config.skip_external_services {
        println!("Skipping error scenario performance test - external services disabled");
        return Ok(());
    }

    let md = MarkdownDown::new();

    // Error scenarios that should fail quickly
    let error_scenarios = [
        (
            "https://invalid-domain-12345.com/page",
            "DNS resolution failure",
        ),
        ("https://httpbin.org/status/404", "HTTP 404 error"),
        ("https://httpbin.org/status/500", "HTTP 500 error"),
        ("https://httpbin.org/delay/10", "Slow response test"),
    ];

    let mut error_timings = Vec::new();

    for (url, description) in error_scenarios.iter() {
        println!("Testing error performance: {description}");

        TestUtils::apply_rate_limit(&config).await;

        let start = Instant::now();
        let result = md.convert_url(url).await;
        let duration = start.elapsed();

        error_timings.push((description, duration, result.is_ok()));

        match result {
            Ok(markdown) => {
                println!(
                    "  ✓ Unexpected success: {} chars in {duration:?}",
                    markdown.as_str().len()
                );

                // Some error URLs might still return content (like 404 pages)
                // This is acceptable if the content indicates an error
                let content = markdown.as_str();
                if content.contains("404")
                    || content.contains("500")
                    || content.contains("Error")
                    || content.contains("not found")
                {
                    println!("    Content indicates error - acceptable");
                } else {
                    println!("    Truly unexpected success");
                }
            }
            Err(e) => {
                println!("  ✓ Expected failure in {duration:?}: {e}");

                // Error should be reported quickly for some scenarios
                match *description {
                    "DNS resolution failure" => {
                        assert!(
                            duration < Duration::from_secs(30),
                            "DNS failure should be detected quickly: {duration:?}"
                        );
                    }
                    "HTTP 404 error" | "HTTP 500 error" => {
                        assert!(
                            duration < Duration::from_secs(60),
                            "HTTP errors should be detected reasonably quickly: {duration:?}"
                        );
                    }
                    "Slow response test" => {
                        // This one is expected to take longer, potentially until timeout
                        println!("    Slow response test duration: {duration:?}");
                    }
                    _ => {}
                }

                // Error should be descriptive
                assert!(!e.to_string().is_empty());
                assert!(e.to_string().len() > 10);
            }
        }
    }

    // Analysis of error timing performance
    println!("\nError Scenario Performance Summary:");
    println!("┌─────────────────────────────┬─────────────┬─────────┐");
    println!("│ Scenario                    │ Duration    │ Result  │");
    println!("├─────────────────────────────┼─────────────┼─────────┤");

    for (description, duration, success) in error_timings.iter() {
        let result_str = if *success { "SUCCESS" } else { "ERROR" };
        println!("│ {description:27} │ {duration:11?} │ {result_str:7} │");
    }

    println!("└─────────────────────────────┴─────────────┴─────────┘");

    // Calculate average error detection time
    let error_durations: Vec<Duration> = error_timings
        .iter()
        .filter(|(_, _, success)| !success)
        .map(|(_, duration, _)| *duration)
        .collect();

    if !error_durations.is_empty() {
        let avg_error_time: Duration =
            error_durations.iter().sum::<Duration>() / error_durations.len() as u32;
        println!("Average error detection time: {avg_error_time:?}");

        // Most errors should be detected within reasonable time
        let quick_errors = error_durations
            .iter()
            .filter(|d| **d < Duration::from_secs(30))
            .count();

        println!(
            "Quick error detection (< 30s): {quick_errors}/{}",
            error_durations.len()
        );
    }

    Ok(())
}

/// Test memory usage and resource efficiency
#[tokio::test]
async fn test_memory_and_resource_efficiency() -> Result<(), Box<dyn std::error::Error>> {
    let config = IntegrationTestConfig::from_env();

    if config.skip_external_services || config.skip_slow_tests {
        println!(
            "Skipping memory efficiency test - external services disabled or slow tests skipped"
        );
        return Ok(());
    }

    // Test processing multiple documents with resource monitoring
    let md = MarkdownDown::new();
    let test_url = "https://httpbin.org/html";

    let num_iterations = 10;
    let mut content_sizes = Vec::new();
    let mut processing_times = Vec::new();

    println!("Testing resource efficiency over {num_iterations} iterations");

    let start_time = Instant::now();

    for i in 0..num_iterations {
        TestUtils::apply_rate_limit(&config).await;

        let iteration_start = Instant::now();
        let result = md.convert_url(test_url).await;
        let iteration_duration = iteration_start.elapsed();

        match result {
            Ok(markdown) => {
                let content_size = markdown.as_str().len();
                content_sizes.push(content_size);
                processing_times.push(iteration_duration);

                println!(
                    "  Iteration {}: {content_size} chars in {iteration_duration:?}",
                    i + 1
                );

                // Validate consistent quality
                assert!(
                    TestUtils::validate_markdown_quality(markdown.as_str()),
                    "Quality should remain consistent across iterations"
                );

                // Memory should not accumulate significantly
                // (This is a basic check - more sophisticated memory monitoring would require additional tools)
                drop(markdown); // Explicit drop to help with memory cleanup
            }
            Err(e) => {
                println!("  Iteration {} failed: {e}", i + 1);
            }
        }
    }

    let total_duration = start_time.elapsed();

    // Resource efficiency analysis
    if !content_sizes.is_empty() && !processing_times.is_empty() {
        let avg_content_size = content_sizes.iter().sum::<usize>() / content_sizes.len();
        let avg_processing_time =
            processing_times.iter().sum::<Duration>() / processing_times.len() as u32;
        let total_content = content_sizes.iter().sum::<usize>();

        println!("\nResource Efficiency Summary:");
        println!(
            "  Successful iterations: {}/{num_iterations}",
            content_sizes.len()
        );
        println!("  Average content size: {avg_content_size} chars");
        println!("  Average processing time: {avg_processing_time:?}");
        println!("  Total content processed: {total_content} chars");
        println!("  Total processing time: {total_duration:?}");
        println!(
            "  Overall throughput: {:.2} chars/sec",
            total_content as f64 / total_duration.as_secs_f64()
        );

        // Performance consistency checks
        let min_time = processing_times.iter().min().unwrap();
        let max_time = processing_times.iter().max().unwrap();
        let time_variance = max_time.as_millis() as f64 / min_time.as_millis() as f64;

        println!("  Processing time range: {min_time:?} - {max_time:?}");
        println!("  Time variance ratio: {time_variance:.2}x");

        // Processing times should be reasonably consistent (within 3x variance)
        assert!(
            time_variance < 3.0,
            "Processing times should be reasonably consistent across iterations"
        );

        // Content sizes should be identical for the same URL
        let min_size = *content_sizes.iter().min().unwrap();
        let max_size = *content_sizes.iter().max().unwrap();
        assert_eq!(
            min_size, max_size,
            "Content size should be consistent for the same URL"
        );
    }

    Ok(())
}
