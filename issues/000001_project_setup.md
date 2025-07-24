# Project Setup

Create the basic Rust project structure and configuration for the markdowndown library.

## Objectives

- Initialize a new Rust library project with proper structure
- Configure Cargo.toml with project metadata and basic dependencies  
- Create initial lib.rs with library module structure
- Set up basic project documentation structure

## Tasks

1. Create `Cargo.toml` with:
   - Project name: `markdowndown`
   - Version: `0.1.0`
   - Edition: `2021`
   - Description: "A Rust library for acquiring markdown from URLs with smart handling"
   - Authors and license information

2. Create `src/lib.rs` with:
   - Basic library structure
   - Module declarations for future components
   - Initial public API skeleton

3. Add `.gitignore` for Rust projects

4. Create basic `README.md` with project description

## Acceptance Criteria

- [ ] `cargo check` runs without errors
- [ ] Project structure follows Rust conventions
- [ ] Basic library compiles successfully
- [ ] Documentation builds with `cargo doc`

## Dependencies

None - this is the foundation step.

## Architecture Notes

This library will use a modular architecture:
- Core types and traits
- HTTP client wrapper  
- URL type detection
- Specific handlers for each URL type (HTML, Google Docs, Office 365, GitHub)
- Unified public API