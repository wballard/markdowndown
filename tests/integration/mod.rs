//! Integration test modules
//!  
//! This module contains all integration tests that interact with real external services.

pub mod config;
pub mod html_sites;
pub mod google_docs;
pub mod github_issues;
pub mod office365;
pub mod end_to_end;
pub mod performance;

// Re-export common testing utilities
pub use config::{IntegrationTestConfig, TestUrls, TestUtils};