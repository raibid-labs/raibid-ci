# Test Environment Configuration
Configuration and environment setup for raibid-ci tests.

## Environment Variables
- `TEST_EXTERNAL=1` - Enable external service tests
- `RAIBID_TEST_NO_CLEANUP=1` - Skip cleanup (debugging)
- `RUST_LOG=debug` - Set log level

## Running Tests
```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --test '*'

# E2E tests (requires external services)
TEST_EXTERNAL=1 cargo test --test 'e2e_*' -- --ignored
```

See tests/README.md for more information.
