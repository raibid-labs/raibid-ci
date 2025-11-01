# Test Fixtures

This directory contains test data and sample files used across integration and E2E tests.

## Files

### Configuration Files

- **`sample_config.yaml`** - Complete, valid configuration with all options
- **`minimal_config.yaml`** - Minimal configuration with only required fields

### Kubernetes Manifests

- **`rust_agent_deployment.yaml`** - Sample Rust agent deployment with PVC and service

## Usage

Load fixtures in tests using the helper function:

```rust
use std::path::PathBuf;

let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/sample_config.yaml");

let content = std::fs::read_to_string(fixture_path).unwrap();
```

Or use the helper module:

```rust
mod helpers;
use helpers::load_fixture;

let config = load_fixture("sample_config.yaml");
```

## Adding New Fixtures

When adding new fixtures:

1. Use descriptive names that indicate the purpose
2. Add documentation comments explaining what the fixture represents
3. Keep fixtures minimal but valid
4. Update this README with the new fixture
5. Version control all fixtures

## Guidelines

- Fixtures should be self-contained and not depend on external resources
- Use placeholder values for secrets (e.g., `${SECRET}`)
- Keep fixtures up to date with schema changes
- Validate fixtures are well-formed before committing
