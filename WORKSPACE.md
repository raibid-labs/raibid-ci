# Raibid-CI Cargo Workspace Structure

This document describes the Cargo workspace organization for the raibid-ci project.

## Overview

The project is organized as a Cargo workspace with multiple crates to enable modular development and parallel work across different components.

## Workspace Members

### 1. **raibid-common** (`crates/common/`)
Common types, utilities, and infrastructure components shared across the workspace.

**Provides:**
- Configuration management (`config` module)
- Infrastructure deployment and management (`infrastructure` module)
  - k3s cluster setup
  - Gitea installation
  - Redis deployment
  - KEDA autoscaler
  - Flux GitOps
- Shared error types
- Utility functions

**Key Exports:**
```rust
use raibid_common::Config;
use raibid_common::infrastructure::{K3sInstaller, GiteaInstaller, RedisInstaller, KedaInstaller, FluxInstaller};
```

### 2. **raibid-cli** (`crates/cli/`)
Command-line interface and command implementations.

**Provides:**
- CLI argument parsing
- Command implementations (setup, teardown, status, config)
- Binary entry point (`raibid` executable)

**Dependencies:**
- `raibid-common` (for config and infrastructure)
- `raibid-tui` (for TUI dashboard)

**Binary:** `raibid`

### 3. **raibid-tui** (`crates/tui/`)
Terminal User Interface using ratatui.

**Provides:**
- Interactive dashboard for monitoring CI/CD jobs
- Agent status visualization
- Queue metrics display
- Real-time updates

**Dependencies:**
- `raibid-common` (for shared types)

### 4. **raibid-server** (`crates/server/`)
API server for job dispatching and TUI communication.

**Status:** Placeholder for future implementation

**Will Provide:**
- Job queue management
- Agent registration and health checks
- Real-time status updates
- WebSocket connections for monitoring

### 5. **raibid-agent** (`crates/agent/`)
CI agent runner for executing builds.

**Status:** Placeholder for future implementation

**Will Provide:**
- Job polling from Redis Streams
- Build execution in isolated environments
- Cache management
- Result reporting

## Workspace Configuration

### Dependency Management

Dependencies are defined at the workspace level in the root `Cargo.toml` and inherited by member crates using `workspace = true`:

```toml
[workspace.dependencies]
anyhow = "1"
tokio = { version = "1", features = ["full"] }
# ... more dependencies

[dependencies]
anyhow = { workspace = true }
tokio = { workspace = true }
```

### Build Profiles

Shared build profiles are defined at the workspace level:

- **dev**: Unoptimized with debug info
- **release**: Fully optimized with LTO and stripping
- **test**: Unoptimized with debug info
- **bench**: Optimized without debug info

## Building

### Build All Crates
```bash
cargo build --workspace
```

### Build Specific Crate
```bash
cargo build -p raibid-cli
cargo build -p raibid-common
```

### Build Release
```bash
cargo build --workspace --release
```

## Testing

### Run All Tests
```bash
cargo test --workspace
```

### Run Tests for Specific Crate
```bash
cargo test -p raibid-common
cargo test -p raibid-tui
```

## Cargo Aliases

Convenient aliases are defined in `.cargo/config.toml`:

```bash
cargo check-all      # Check all crates
cargo test-all       # Test all crates
cargo build-all      # Build all crates
cargo clippy-all     # Run clippy on all crates
cargo fmt-all        # Format check all crates
```

Individual crate aliases:
```bash
cargo check-common
cargo check-cli
cargo check-tui
cargo check-server
cargo check-agent
```

## Directory Structure

```
raibid-ci/
├── Cargo.toml                 # Workspace root configuration
├── Cargo.lock                 # Locked dependencies (for reproducible builds)
├── rust-toolchain.toml        # Rust toolchain specification
├── .cargo/
│   └── config.toml           # Cargo configuration and aliases
├── crates/
│   ├── common/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config/
│   │       └── infrastructure/
│   ├── cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli/
│   │       └── commands/
│   ├── tui/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── server/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── agent/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
└── target/                   # Build artifacts (workspace-wide)
```

## Migration Notes

The workspace was created from a monolithic structure. Old code has been moved to:
- `src.old/` - Original source code
- `tests.old/` - Original tests
- `examples.old/` - Original examples

These will be migrated to appropriate workspace members in future issues.

## Benefits of Workspace Structure

1. **Modular Development**: Each component can be developed independently
2. **Parallel Work**: Multiple teams can work on different crates simultaneously
3. **Clear Dependencies**: Explicit inter-crate dependencies
4. **Faster Incremental Builds**: Only changed crates are rebuilt
5. **Better Code Organization**: Separation of concerns
6. **Independent Versioning**: Crates can version independently in the future
7. **Reusability**: Common functionality easily shared

## Future Work

- Migrate existing tests to appropriate workspace members
- Implement server crate (API server)
- Implement agent crate (CI runner)
- Add integration tests at workspace level
- Consider workspace-wide examples directory
