# Tests Directory

This directory contains workspace-level tests for the raibid-ci project. Tests are organized by type and purpose to support comprehensive testing across all workspace crates.

## Directory Structure

```
tests/
├── README.md           # This file - testing documentation
├── integration/        # Integration tests across multiple crates
├── e2e/               # End-to-end system tests
├── fixtures/          # Shared test data and mock configurations
└── helpers/           # Common test utilities and helpers
```

## Test Types

### Integration Tests (`integration/`)

Integration tests verify interactions between multiple crates in the workspace. These tests import and use multiple workspace crates together to test realistic workflows.

**Current tests:**
- `build_verification_test.rs` - Binary build and installation verification
- `config_test.rs` - Configuration loading and validation
- `cli_test.rs` - CLI command execution and output
- `tui_test.rs` - TUI rendering and interactions
- `additional_commands_test.rs` - Extended CLI commands
- `installation_permissions_test.rs` - Installation and permission checks
- `mock_commands_test.rs` - Mock command execution
- `redis_test.rs` - Redis integration
- `status_test.rs` - Status reporting
- `flux_test.rs` - Flux GitOps integration
- `error_handling_test.rs` - Error handling across components

**Running integration tests:**
```bash
cargo test --test '*' --no-fail-fast
cargo test --test config_test
```

### E2E Tests (`e2e/`)

End-to-end tests validate complete user workflows from start to finish. These tests:
- Simulate real user interactions
- Test the entire system stack
- Verify external integrations (k3s, Gitea, Redis, etc.)
- May require Docker or actual infrastructure

**Running E2E tests:**
```bash
cargo test --test e2e_* --no-fail-fast
# Run with external dependencies
TEST_EXTERNAL=1 cargo test --test e2e_*
```

### Fixtures (`fixtures/`)

Shared test data used across multiple test files:
- Sample configuration files
- Mock API responses
- Test Kubernetes manifests
- Example Git repositories
- Test certificates and keys

**Usage in tests:**
```rust
use std::path::PathBuf;

let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/sample_config.yaml");
```

### Helpers (`helpers/`)

Reusable test utilities and helper functions:
- Test environment setup/teardown
- Mock server builders
- Assertion helpers
- Test data generators
- Common test fixtures

**Usage in tests:**
```rust
mod helpers;
use helpers::{setup_test_env, create_mock_cluster};
```

## Running Tests

### All Tests
```bash
# Run all tests (unit + integration + E2E)
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run with specific verbosity
RUST_LOG=debug cargo test --workspace
```

### Unit Tests Only
```bash
# Unit tests live in each crate's src/ directory
cargo test --workspace --lib
```

### Integration Tests Only
```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test file
cargo test --test config_test

# Run specific test function
cargo test --test config_test test_config_validate_default
```

### E2E Tests Only
```bash
# Run all E2E tests
cargo test --test 'e2e_*'

# Run specific E2E test
cargo test --test e2e_full_workflow
```

### Filtered Tests
```bash
# Run tests matching pattern
cargo test config

# Run tests in specific package
cargo test -p raibid-cli

# Run ignored tests
cargo test -- --ignored
```

## Test Environment

### Environment Variables
```bash
# Enable external service tests
export TEST_EXTERNAL=1

# Set test log level
export RUST_LOG=debug

# Custom test configuration
export RAIBID_TEST_CONFIG=/path/to/test/config.yaml

# Skip cleanup for debugging
export RAIBID_TEST_NO_CLEANUP=1
```

### Test Isolation

Tests should be isolated and idempotent:
- Use `tempfile::TempDir` for temporary files
- Mock external dependencies by default
- Use unique identifiers for test resources
- Clean up resources in test teardown

### Test Data

Test data should be:
- Deterministic and reproducible
- Self-contained in fixtures/
- Version controlled
- Documented with clear purpose

## Coverage Reporting

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Generate XML for CI
cargo tarpaulin --workspace --out Xml
```

### View Coverage
```bash
# Open HTML report
xdg-open coverage/index.html  # Linux
open coverage/index.html      # macOS
```

### Coverage Targets
- **Unit tests**: 80%+ coverage per crate
- **Integration tests**: Cover critical workflows
- **E2E tests**: Cover happy paths and major error cases

## CI Integration

Tests run automatically in CI on:
- Every push to feature branches
- All pull requests
- Scheduled nightly builds

### CI Test Matrix
```yaml
# .github/workflows/test.yml
matrix:
  rust: [stable, beta, nightly]
  os: [ubuntu-latest, macos-latest]
  arch: [x86_64, aarch64]
```

### Required Checks
- All unit tests pass
- All integration tests pass
- E2E tests pass (main branch only)
- Code coverage meets thresholds
- No clippy warnings
- No rustfmt issues

## Writing New Tests

### Integration Test Template
```rust
//! Integration test for [feature name]
//!
//! This test verifies [what is being tested].

use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_feature_name() {
    // Setup
    let temp_dir = TempDir::new().unwrap();

    // Execute
    let output = Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("command")
        .output()
        .unwrap();

    // Assert
    assert!(output.status.success());

    // Cleanup (if needed)
}
```

### E2E Test Template
```rust
//! E2E test for [workflow name]
//!
//! Tests complete user workflow: [description]

#[test]
#[ignore = "requires external services"]
fn test_e2e_workflow() {
    // Setup environment
    // Execute workflow
    // Verify results
    // Cleanup
}
```

### Using Fixtures
```rust
use std::fs;
use std::path::PathBuf;

fn load_fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[test]
fn test_with_fixture() {
    let config = load_fixture("sample_config.yaml");
    // Use config in test
}
```

### Using Helpers
```rust
mod helpers;

#[test]
fn test_with_helpers() {
    let env = helpers::setup_test_env();
    let cluster = helpers::create_mock_cluster(&env);

    // Run test with helper utilities

    helpers::teardown_test_env(env);
}
```

## Best Practices

### Test Naming
- Use descriptive names: `test_config_validate_with_invalid_yaml`
- Group related tests with prefixes: `test_config_*`, `test_cli_*`
- Use `#[ignore]` for slow tests: `#[ignore = "slow"]`

### Test Organization
- One test file per major feature
- Group related tests in modules
- Use doc comments to explain test purpose
- Keep tests focused and small

### Assertions
```rust
// Use specific assertions
assert_eq!(actual, expected);
assert!(condition, "failure message");

// Use predicates for complex checks
use predicates::prelude::*;
assert!(predicate::str::contains("expected").eval(&output));

// Use assert_cmd for CLI tests
cmd.assert()
    .success()
    .stdout(predicate::str::contains("Success"));
```

### Mocking
- Mock external services by default
- Use feature flags for optional external tests
- Document required external services clearly
- Provide docker-compose for local testing

### Performance
- Mark slow tests with `#[ignore]`
- Use `--no-fail-fast` to run all tests
- Run expensive tests in CI only
- Profile test execution: `cargo test -- --show-output`

## Troubleshooting

### Tests Fail Locally But Pass in CI
- Check environment variables
- Verify file paths (relative vs absolute)
- Check for race conditions
- Ensure cleanup runs properly

### Tests Are Slow
- Run in parallel: `cargo test -- --test-threads=4`
- Use mocks instead of real services
- Cache expensive setup operations
- Profile with `cargo test -- --nocapture`

### Flaky Tests
- Check for timing issues (add waits/retries)
- Verify resource cleanup
- Check for shared state between tests
- Use unique identifiers per test

### Coverage Too Low
- Add unit tests for uncovered code
- Test error paths explicitly
- Add integration tests for workflows
- Review coverage report for gaps

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [assert_cmd Documentation](https://docs.rs/assert_cmd/)
- [predicates Documentation](https://docs.rs/predicates/)
- [tempfile Documentation](https://docs.rs/tempfile/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

## Contributing

When adding new tests:
1. Follow the test templates above
2. Add documentation explaining test purpose
3. Use fixtures for test data
4. Keep tests isolated and idempotent
5. Update this README if adding new patterns

---

**Last Updated**: 2025-11-01
**Issue Reference**: #45 (WS-00: Set Up Tests Directory Structure)
