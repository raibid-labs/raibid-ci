# Tests Directory
Comprehensive testing structure for raibid-ci.

## Structure
- `integration/` - Integration tests across crates
- `e2e/` - End-to-end workflow tests
- `fixtures/` - Shared test data
- `helpers/` - Common test utilities

## Running Tests
```bash
# All tests
cargo test --workspace

# Integration tests
cargo test --test '*'
```

See TEST_ENVIRONMENT.md for details.
