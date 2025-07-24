# Performance Test Document

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

The goal is to measure processing time across different content types and sizes.