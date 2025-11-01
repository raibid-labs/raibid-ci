# Test Environment Configuration

This document describes the test environment setup and configuration for raibid-ci.

## Environment Variables

### Test Control Variables

- **`TEST_EXTERNAL=1`** - Enable tests that require external services (Docker, k3s, etc.)
- **`RAIBID_TEST_NO_CLEANUP=1`** - Skip cleanup after tests (useful for debugging)
- **`RAIBID_TEST_CONFIG`** - Path to custom test configuration file
- **`RUST_LOG=debug`** - Set log level for tests (trace, debug, info, warn, error)
- **`RUST_BACKTRACE=1`** - Enable backtraces on panic

### Application Variables

Tests may use these application environment variables:

- **`RAIBID_CONFIG_PATH`** - Override default config path
- **`RAIBID_API_HOST`** - Override API host
- **`RAIBID_API_PORT`** - Override API port
- **`RAIBID_REDIS_HOST`** - Override Redis host
- **`RAIBID_REDIS_PORT`** - Override Redis port

## Test Isolation

### Temporary Directories

All tests use `tempfile::TempDir` for isolated temporary directories:

```rust
use tempfile::TempDir;

let temp_dir = TempDir::new().expect("Failed to create temp directory");
let config_path = temp_dir.path().join("raibid.yaml");
// TempDir is automatically cleaned up when dropped
```

### Test Helpers

The `tests/helpers/` module provides utilities for test isolation:

```rust
use helpers::TestEnv;

let mut env = TestEnv::new();
let config_path = env.create_config("cluster:\n  name: test");
// Environment is automatically cleaned up
```

### Skipping Cleanup

For debugging, set `RAIBID_TEST_NO_CLEANUP=1` to preserve test artifacts:

```bash
RAIBID_TEST_NO_CLEANUP=1 cargo test test_name
```

The test will print the location of temporary files:
```
Skipping cleanup. Test directory: /tmp/test-abc123
```

## External Service Requirements

### Unit Tests

Unit tests (`cargo test --lib`) have no external dependencies.

### Integration Tests

Integration tests (`cargo test --test '*'`) require:

- Git binary installed
- Network access (for simulated operations)
- Filesystem permissions

Most integration tests mock external services and don't require actual infrastructure.

### E2E Tests

End-to-end tests (`cargo test --test 'e2e_*' -- --ignored`) require:

- **Docker** - For container operations
- **k3s** or **kubectl** - For Kubernetes operations
- **Git** - For repository operations
- **Network access** - For downloading dependencies
- **Root/sudo access** - For k3s installation (optional)

Enable E2E tests:
```bash
TEST_EXTERNAL=1 cargo test --test 'e2e_*' -- --ignored
```

## Local Development Setup

### Minimal Setup (Unit + Integration)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install git
sudo apt-get install git  # Ubuntu/Debian
brew install git          # macOS

# Run tests
cargo test
```

### Full Setup (Including E2E)

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
newgrp docker

# Install k3s (optional - for E2E tests)
curl -sfL https://get.k3s.io | sh -

# Run all tests including E2E
TEST_EXTERNAL=1 cargo test -- --ignored
```

## CI Environment

### GitHub Actions

The CI environment is configured in `.github/workflows/test.yml`:

- **Unit tests** run on every push/PR
- **Integration tests** run after unit tests pass
- **E2E tests** run only on main branch, schedule, or manual trigger
- **Coverage** runs in parallel with integration tests

### Required Secrets

No secrets required for tests. All credentials use mock values.

For production deployment, set:
- `GITEA_ADMIN_TOKEN`
- `REDIS_PASSWORD` (if using password auth)

## Test Data

### Fixtures

Test fixtures are stored in `tests/fixtures/`:

```
fixtures/
├── sample_config.yaml          # Complete configuration
├── minimal_config.yaml         # Minimal configuration
└── rust_agent_deployment.yaml  # Kubernetes manifest
```

Load fixtures in tests:
```rust
use helpers::load_fixture;

let config = load_fixture("sample_config.yaml");
```

### Generated Data

Use generators for dynamic test data:

```rust
use helpers::generators::*;

let config = generate_test_config();
let manifest = generate_test_manifest("my-app", "image:latest", 3);
let commit_hash = generate_commit_hash();
```

## Debugging Tests

### Running Specific Tests

```bash
# Run single test
cargo test test_name

# Run test with output
cargo test test_name -- --nocapture

# Run test with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Keep test artifacts
RAIBID_TEST_NO_CLEANUP=1 cargo test test_name
```

### Common Issues

#### Test Timeout

Increase timeout for slow tests:
```rust
#[test]
#[timeout(std::time::Duration::from_secs(60))]
fn slow_test() {
    // ...
}
```

#### Permission Denied

Run with appropriate permissions or mock the operation:
```bash
sudo -E cargo test  # Run as root (not recommended)
# OR
# Mock the operation in test
```

#### Port Already in Use

Tests use ephemeral ports by default. If conflicts occur:
```rust
// Use port 0 for auto-assignment
let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
let port = listener.local_addr().unwrap().port();
```

#### Flaky Tests

For timing-dependent tests:
```rust
use std::time::Duration;
use std::thread::sleep;

// Add reasonable wait times
sleep(Duration::from_millis(100));

// Use retry logic for network operations
for attempt in 0..3 {
    match operation() {
        Ok(result) => return result,
        Err(e) if attempt < 2 => {
            sleep(Duration::from_secs(1));
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

## Performance Testing

### Benchmarks

Benchmarks are in `benches/` directory (not yet implemented):

```bash
cargo bench
```

### Load Testing

For load testing the API:

```bash
# Install drill (HTTP load testing)
cargo install drill

# Run load test
drill --benchmark load_test.yml
```

## Coverage Targets

### Minimum Coverage Requirements

- **Crate-level**: 80% line coverage
- **Critical paths**: 90% coverage
- **Error handling**: 100% coverage of error types

### Generating Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# Open report
xdg-open coverage/index.html  # Linux
open coverage/index.html      # macOS
```

### Coverage Configuration

Coverage settings are in `tarpaulin.toml`. Adjust as needed:

```toml
[coverage]
line-coverage-threshold = 80
branch-coverage-threshold = 70
```

## Continuous Integration

### Test Matrix

Tests run on:
- **Rust versions**: stable, beta, nightly (experimental)
- **Platforms**: ubuntu-latest
- **Test types**: unit, integration, doc, E2E (main only)

### Required Checks

Pull requests must pass:
- [ ] All unit tests
- [ ] All integration tests
- [ ] Code formatting (`cargo fmt`)
- [ ] Linting (`cargo clippy`)
- [ ] Documentation builds
- [ ] No compiler warnings

### Optional Checks

- Code coverage report (informational)
- E2E tests (main branch only)
- Nightly Rust (allowed to fail)

## Best Practices

### Test Naming

- Use descriptive names: `test_config_loads_from_file`
- Prefix by feature: `test_redis_*`, `test_keda_*`
- Suffix by scenario: `*_success`, `*_failure`, `*_timeout`

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod unit {
        use super::*;

        #[test]
        fn test_function_name() { }
    }

    mod integration {
        use super::*;

        #[test]
        fn test_integration_scenario() { }
    }
}
```

### Error Messages

Provide clear error messages:
```rust
assert_eq!(
    actual, expected,
    "Config should have cluster name 'test' but got '{}'",
    actual
);
```

### Cleanup

Always clean up resources:
```rust
let _guard = cleanup_guard(|| {
    // Cleanup code runs even if test panics
    remove_test_files();
});
```

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [tarpaulin](https://github.com/xd009642/tarpaulin)
- [assert_cmd](https://docs.rs/assert_cmd/)
- [predicates](https://docs.rs/predicates/)
