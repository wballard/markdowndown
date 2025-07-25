 I tried to get a github issue, and got this bullshit:


# Converted from GitHub Issue (Preview)

Source: <https://github.com/wballard/swissarmyhammer/issues/5>

> **Note:** This is a placeholder conversion with limited formatting. For full document features, use the native GitHub Issue application.



there is no way this is good enough. When you are given a spec and a todo

I tried to get a github issue, and got this bullshit:


# Converted from GitHub Issue (Preview)

Source: <https://github.com/wballard/swissarmyhammer/issues/5>

> **Note:** This is a placeholder conversion with limited formatting. For full document features, use the native GitHub Issue application.



there is no way this is good enough. When you are given a spec and a todo

## Analysis

The issue is that GitHub URLs are being routed to the placeholder `GitHubIssueConverter` instead of the full-featured `GitHubConverter`. 

Looking at the codebase:

1. There are **two** GitHub converters:
   - `src/converters/github.rs::GitHubConverter` - Full-featured converter that uses GitHub API, fetches issue data, comments, metadata, and generates proper markdown with frontmatter
   - `src/converters/placeholder.rs::GitHubIssueConverter` - Placeholder that just scrapes HTML and produces the "bullshit" output

2. The converter registry (`src/converters/converter.rs`) is incorrectly registering the placeholder converter:
   - Line 64-66: `registry.register(UrlType::GitHubIssue, Box::new(super::placeholder::GitHubIssueConverter::new()));`
   - Line 105-112: Same issue in the configured registry

3. The full `GitHubConverter` is implemented but never used by the registry.

## Proposed Solution

1. **Fix the converter registry** to use the real `GitHubConverter` instead of the placeholder
2. **Update both registry methods** (`new()` and `with_config()`) to properly instantiate the GitHub API converter
3. **Add authentication support** from environment variables (GITHUB_TOKEN) in the registry
4. **Verify with tests** that GitHub URLs now use the API converter and produce proper markdown output with issue metadata, comments, and frontmatter

The fix should replace placeholder registrations with:
```rust
registry.register(
    UrlType::GitHubIssue,
    Box::new(super::GitHubConverter::from_env())
);
```

This will use the GitHub API converter with automatic authentication from the GITHUB_TOKEN environment variable when available.