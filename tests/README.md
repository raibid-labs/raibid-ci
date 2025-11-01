# Tests Directory

Comprehensive testing structure for raibid-ci workspace.

## Structure

```
tests/
├── README.md           # This file
├── TEST_ENVIRONMENT.md # Environment configuration docs
├── integration/        # Integration tests across crates
├── e2e/               # End-to-end workflow tests
├── fixtures/          # Shared test data
└── helpers/           # Common test utilities
```

## Running Tests

```bash
# All tests
cargo test --workspace

# Unit tests only
cargo test --workspace --lib

# Integration tests
cargo test --test '*'

# E2E tests (requires external services)
TEST_EXTERNAL=1 cargo test --test 'e2e_*' -- --ignored
```

## Documentation

See `TEST_ENVIRONMENT.md` for detailed setup and configuration.

---

**Issue**: #45 (WS-00: Set Up Tests Directory Structure)
