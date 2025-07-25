# Contributing to markdowndown

Thank you for your interest in contributing to markdowndown! This guide will help you get started with development, understand our processes, and submit high-quality contributions.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Requirements](#testing-requirements)
- [Submitting Changes](#submitting-changes)
- [Issue Guidelines](#issue-guidelines)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Getting Started

### Prerequisites

- **Rust 1.70+** (2021 edition)
- **Git** for version control
- **Cargo** (comes with Rust)
- **tokio** async runtime knowledge

### Development Tools

We recommend installing these tools for the best development experience:

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional components
rustup component add rustfmt clippy

# Install cargo-watch for auto-rebuilding
cargo install cargo-watch

# Install cargo-tarpaulin for coverage (Linux only)
cargo install cargo-tarpaulin
```

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/markdowndown.git
cd markdowndown

# Add upstream remote
git remote add upstream https://github.com/wballard/markdowndown.git
```

### 2. Initial Build and Test

```bash
# Check that everything compiles
cargo check

# Run tests
cargo test

# Run integration tests (may require network access)
cargo test --test integration_tests

# Run all tests with verbose output
cargo test -- --nocapture
```

### 3. Set Up Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit with your credentials (optional, for testing authenticated features)
export GITHUB_TOKEN=ghp_your_token_here
```

### 4. Verify Setup

```bash
# Run a simple example to verify everything works
cargo run --example basic_usage

# Run benchmarks (optional)
cargo bench

# Generate documentation
cargo doc --open
```

## Code Style and Standards

### Formatting

We use `rustfmt` with default settings:

```bash
# Format all code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

**Important**: All code must be formatted before submission. Set up your editor to format on save.

### Linting

We use `clippy` for linting:

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Fail on warnings (CI configuration)
cargo clippy -- -D warnings
```

### Code Quality Standards

1. **No `unwrap()` in library code** - Use proper error handling
2. **Comprehensive documentation** - All public APIs must have rustdoc comments
3. **Examples in documentation** - Include practical examples in doc comments
4. **Error handling** - Use the structured error system, provide helpful error messages
5. **Performance awareness** - Consider memory usage and network efficiency
6. **Security** - Never log or expose sensitive information

### Documentation Standards

#### Rustdoc Comments

All public functions, structs, and modules must have documentation:

```rust
/// Converts a URL to markdown with automatic type detection.
///
/// This function automatically detects the URL type and routes to the 
/// appropriate converter for processing.
///
/// # Arguments
///
/// * `url` - The URL to convert to markdown
///
/// # Returns
///
/// Returns a `Result` containing the converted `Markdown` or a `MarkdownError`.
///
/// # Examples
///
/// ```rust
/// use markdowndown::convert_url;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let markdown = convert_url("https://example.com/article").await?;
/// println!("{}", markdown);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// * `MarkdownError::ValidationError` - If the URL format is invalid
/// * `MarkdownError::NetworkError` - For network-related failures
/// * `MarkdownError::ContentError` - If content conversion fails
pub async fn convert_url(url: &str) -> Result<Markdown, MarkdownError> {
    // Implementation
}
```

#### Code Comments

- Use `//` for single-line comments
- Use `/* */` for multi-line comments when necessary
- Explain **why**, not **what** (the code should be self-explanatory)
- Add comments for complex algorithms or business logic

### Naming Conventions

- **Functions**: `snake_case`
- **Variables**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

```rust
// Good
const MAX_RETRY_ATTEMPTS: u32 = 5;
struct MarkdownConverter;
fn convert_url_to_markdown() -> Result<Markdown, MarkdownError> { }

// Bad
const maxRetryAttempts: u32 = 5;
struct markdownConverter;
fn ConvertURLToMarkdown() -> Result<Markdown, MarkdownError> { }
```

## Testing Requirements

### Test Categories

1. **Unit Tests** - Test individual functions and modules
2. **Integration Tests** - Test complete workflows
3. **Property Tests** - Test invariants with random inputs
4. **Performance Tests** - Benchmark critical paths

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        // Test valid URLs
        let valid_url = Url::new("https://example.com".to_string());
        assert!(valid_url.is_ok());

        // Test invalid URLs
        let invalid_url = Url::new("not-a-url".to_string());
        assert!(invalid_url.is_err());
    }

    #[tokio::test]
    async fn test_conversion_basic() {
        let markdown = convert_url("https://httpbin.org/html").await;
        assert!(markdown.is_ok());
        
        let content = markdown.unwrap();
        assert!(!content.as_str().is_empty());
    }
}
```

#### Integration Tests

```rust
// tests/integration/basic_conversion.rs
use markdowndown::{convert_url, MarkdownDown, Config};

#[tokio::test]
async fn test_end_to_end_html_conversion() {
    let result = convert_url("https://httpbin.org/html").await;
    
    assert!(result.is_ok());
    let markdown = result.unwrap();
    
    // Verify content structure
    assert!(markdown.as_str().contains("<!DOCTYPE html>"));
    
    // Verify frontmatter
    assert!(markdown.frontmatter().is_some());
}
```

### Running Tests

```bash
# Run unit tests
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run specific test
cargo test test_url_validation

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode (faster)
cargo test --release
```

### Test Coverage

Aim for >90% test coverage:

```bash
# Generate coverage report (Linux/macOS)
cargo tarpaulin --out Html

# View coverage report
open tarpaulin-report.html
```

### Performance Testing

```bash
# Run benchmarks
cargo bench

# Profile a specific benchmark
cargo bench --bench conversion_benchmarks -- --profile-time=10
```

## Submitting Changes

### Branch Naming

Use descriptive branch names:

- `feature/add-pdf-support` - New features
- `fix/authentication-error` - Bug fixes
- `docs/improve-readme` - Documentation changes
- `refactor/error-handling` - Code refactoring
- `perf/optimize-parsing` - Performance improvements

### Commit Messages

Follow conventional commit format:

```
type(scope): description

More detailed explanation if needed.

Fixes #123
```

**Types:**
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(converters): add PDF document support

Add support for PDF documents by implementing a new PDF converter
that uses pdf2html for initial conversion before markdown processing.

Closes #45

fix(github): handle rate limiting in GitHub API calls

Implement exponential backoff for GitHub API rate limiting to 
improve reliability when processing multiple GitHub URLs.

Fixes #67
```

### Pre-submission Checklist

Before submitting a pull request:

- [ ] Code is formatted with `rustfmt`
- [ ] Code passes `clippy` without warnings
- [ ] All tests pass (`cargo test`)
- [ ] Integration tests pass
- [ ] New code has appropriate tests
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for notable changes)
- [ ] Examples are updated if API changes
- [ ] Performance impact is considered

## Issue Guidelines

### Reporting Bugs

Use the bug report template and include:

1. **Environment Information:**
   - OS and version
   - Rust version (`rustc --version`)
   - markdowndown version

2. **Reproduction Steps:**
   ```rust
   // Minimal reproduction case
   use markdowndown::convert_url;
   
   #[tokio::main]
   async fn main() {
       let result = convert_url("https://problematic-url.com").await;
       println!("Result: {:?}", result);
   }
   ```

3. **Expected vs. Actual Behavior**

4. **Error Messages** (full text)

5. **Additional Context** (configuration, network setup, etc.)

### Feature Requests

Before requesting a feature:

1. **Search existing issues** for similar requests
2. **Consider the scope** - does it fit the project goals?
3. **Think about implementation** - provide suggestions if possible
4. **Consider alternatives** - are there workarounds?

Use the feature request template and include:
- **Use case description**
- **Proposed solution**
- **Alternative solutions considered**
- **Additional context**

### Security Issues

**Do not open public issues for security vulnerabilities.**

Instead:
- Email security issues privately
- Include detailed reproduction steps
- Allow reasonable time for fixes before disclosure

## Pull Request Process

### 1. Preparation

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name

# Make your changes
# ... develop and test ...

# Commit changes
git add .
git commit -m "feat: add your feature description"
```

### 2. Pre-submission

```bash
# Run full test suite
cargo test --all-features

# Check formatting and linting
cargo fmt -- --check
cargo clippy -- -D warnings

# Update documentation if needed
cargo doc

# Run integration tests
cargo test --test integration_tests
```

### 3. Submit Pull Request

1. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request** on GitHub

3. **Fill out the PR template** completely

4. **Assign reviewers** (optional, maintainers will assign)

### 4. Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Address feedback** by pushing new commits
4. **Squash commits** if requested
5. **Merge** when approved

### PR Template Example

```markdown
## Description
Brief description of changes.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] New tests added for new functionality
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or breaking changes documented)
```

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

### Release Checklist

1. **Update CHANGELOG.md** with new version
2. **Update Cargo.toml** version
3. **Update documentation** if needed
4. **Run full test suite**
5. **Create git tag** (`git tag v1.2.3`)
6. **Push tag** (`git push origin v1.2.3`)
7. **Publish to crates.io** (`cargo publish`)
8. **Create GitHub release** with changelog

### Changelog Format

```markdown
# Changelog

## [1.2.3] - 2024-01-15

### Added
- New PDF document support
- Configuration validation helpers

### Changed
- Improved error messages for network failures
- Updated dependencies

### Fixed
- Authentication bug in GitHub converter
- Memory leak in batch processing

### Deprecated
- Old configuration format (use Config::builder())

### Removed
- Legacy error types (replaced with enhanced errors)

### Security
- Fixed potential token exposure in logs
```

## Development Workflow

### Daily Development

```bash
# Start development session
cargo watch -x check -x test

# In another terminal, run specific examples
cargo run --example basic_usage

# Make changes, tests run automatically via cargo-watch

# Before committing
cargo fmt
cargo clippy
cargo test
```

### Working with Dependencies

```bash
# Add new dependency
cargo add serde --features derive

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit
```

### Debugging

```bash
# Run with debug info
RUST_LOG=debug cargo run --example basic_usage

# Run tests with output
cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test specific_test_name
```

## Code Review Guidelines

### For Contributors

- **Keep PRs focused** - one feature/fix per PR
- **Write clear descriptions** - explain what and why
- **Respond to feedback** promptly and constructively
- **Test thoroughly** - include edge cases
- **Document changes** - update docs and examples

### For Reviewers

- **Be constructive** - suggest improvements, don't just criticize
- **Consider the bigger picture** - how does this fit the project?
- **Check for edge cases** - what could go wrong?
- **Verify tests** - are they comprehensive and correct?
- **Look for performance implications** - memory usage, network calls

## Communication

- **GitHub Issues** - Bug reports and feature requests
- **Pull Requests** - Code discussions
- **Discussions** - General questions and ideas

### Guidelines

- **Be respectful** and professional
- **Search before posting** - avoid duplicates
- **Provide context** - help others understand your situation
- **Be patient** - maintainers are volunteers

## Getting Help

### Before Asking

1. **Read the documentation** thoroughly
2. **Search existing issues** and discussions
3. **Try the troubleshooting guide**
4. **Test with minimal examples**

### When Asking for Help

Include:
- **Clear problem description**
- **Steps to reproduce**
- **Expected vs actual behavior**
- **Environment details**
- **Code examples** (minimal, complete)

## Recognition

Contributors are recognized in:
- **CHANGELOG.md** for significant contributions
- **GitHub contributors** page
- **Release notes** for major features

## Questions?

If you have questions about contributing:

1. **Check existing documentation** first
2. **Search GitHub issues** for similar questions
3. **Open a discussion** for general questions
4. **Open an issue** for specific problems

Thank you for contributing to markdowndown! 🚀