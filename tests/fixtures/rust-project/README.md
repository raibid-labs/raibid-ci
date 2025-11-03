# Test Fixture - Rust Project

This is a minimal Rust library project used as a test fixture for E2E CI pipeline testing.

## Purpose

This project is used to test the complete raibid-ci pipeline:
- Building Rust projects
- Running tests
- Caching dependencies
- Docker image creation
- Registry publishing

## Structure

- `src/lib.rs` - Simple library with basic functionality
- `Cargo.toml` - Minimal dependencies (serde, serde_json)
- Tests included to verify CI runs tests successfully

## Usage

This fixture should be:
1. Pushed to a test repository in Gitea
2. Used in E2E tests to trigger builds
3. Modified to test cache invalidation scenarios

## Build Commands

```bash
cargo build
cargo test
cargo build --release
```

## Expected Behavior

- All tests should pass
- Build time should be ~10-30 seconds (first build)
- Cached builds should be <5 seconds
