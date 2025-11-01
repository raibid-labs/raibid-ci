//! Test Helpers Module
//!
//! Common utilities for tests across the workspace.

pub mod test_env;
pub mod generators;

// Re-export commonly used items
pub use test_env::TestEnv;
pub use generators::*;

use std::path::PathBuf;

/// Get path to a test fixture
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Load fixture file as string
pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", name))
}

/// Check if external tests are enabled
pub fn external_tests_enabled() -> bool {
    std::env::var("TEST_EXTERNAL").is_ok()
}
