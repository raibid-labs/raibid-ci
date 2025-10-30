# raibid-cli

> **DGX Spark Personal CI Agent Pool - TUI-first developer tool for managing self-hosted CI agents**

A terminal-based management interface for running ephemeral, auto-scaling CI/CD infrastructure on NVIDIA DGX Spark. Built with Rust and Ratatui for a responsive, SSH-friendly developer experience.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Features

### Production Infrastructure 🚀

- **k3s Cluster** - Automated Kubernetes cluster setup on ARM64 and x86_64
- **Gitea + OCI Registry** - Self-hosted Git server with container registry
- **Redis Streams** - Job queue management with consumer groups
- **KEDA Autoscaling** - Event-driven autoscaling (scale 0→10 based on queue depth)
- **Flux GitOps** - Continuous delivery from Gitea repository
- **Real-time Status** - Live cluster monitoring via Kubernetes API

### Developer Experience 💻

- **Interactive TUI Dashboard** - Real-time monitoring with Ratatui-based terminal interface
- **Job Management** - List, view, cancel, and retry CI jobs
- **Agent Management** - Monitor and scale build agents dynamically
- **Repository Mirroring** - Sync GitHub repositories to local Gitea instance
- **Flexible Configuration** - YAML/TOML configuration with environment variable overrides
- **Comprehensive Testing** - 100+ unit tests with TDD workflow

## Quick Start

### Prerequisites

**Rust is required to build raibid-cli.** Install it using [rustup](https://rustup.rs):

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

For Windows, download and run [rustup-init.exe](https://rustup.rs).

### Installation

#### From Source

```bash
# Clone the repository
git clone https://github.com/raibid-labs/raibid-cli.git
cd raibid-cli

# Build release binary
cargo build --release

# Install to /usr/local/bin (optional)
sudo cp target/release/raibid-cli /usr/local/bin/
```

#### For DGX Spark (ARM64)

```bash
# Add ARM64 target (if not already installed)
rustup target add aarch64-unknown-linux-gnu

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Binary will be at: target/aarch64-unknown-linux-gnu/release/raibid-cli
```

### First Run

1. **Initialize configuration:**
   ```bash
   raibid-cli config init
   ```

2. **Set up infrastructure:**
   ```bash
   raibid-cli setup all
   ```

3. **Launch TUI dashboard:**
   ```bash
   raibid-cli tui
   ```

## Commands

### TUI Dashboard

Launch the interactive terminal UI for real-time monitoring and management:

```bash
raibid-cli tui
```

**TUI Features:**
- **Jobs Tab** - View running, pending, successful, and failed jobs
- **Agents Tab** - Monitor agent status, CPU/memory usage, and uptime
- **Config Tab** - View current configuration
- **Logs Tab** - Real-time log streaming

**Keyboard Shortcuts:**
- `Tab` / `Shift+Tab` - Navigate between tabs
- `1-4` - Jump directly to tab (1=Jobs, 2=Agents, 3=Config, 4=Logs)
- `↑/↓` or `j/k` - Navigate list items
- `Enter` - View details of selected item
- `f` - Open filter menu
- `/` - Search mode
- `r` - Refresh data
- `?` - Show help screen
- `q` or `Ctrl+C` - Quit

### Infrastructure Commands

Manage infrastructure components:

```bash
# Setup commands
raibid-cli setup k3s       # Bootstrap k3s cluster
raibid-cli setup gitea     # Deploy Gitea with OCI registry
raibid-cli setup redis     # Deploy Redis Streams
raibid-cli setup keda      # Deploy KEDA autoscaler
raibid-cli setup flux      # Bootstrap Flux GitOps
raibid-cli setup all       # Setup all components in order

# Teardown commands
raibid-cli teardown <component>  # Remove a specific component
raibid-cli teardown all          # Remove all components

# Status commands
raibid-cli status          # Show all component status
raibid-cli status k3s      # Show k3s cluster status
```

### Job Management

Manage CI/CD jobs:

```bash
# List jobs
raibid-cli job list                      # List all jobs
raibid-cli job list --status running     # Filter by status
raibid-cli job list --repo raibid/core   # Filter by repository
raibid-cli job list --limit 10           # Limit results
raibid-cli job list --output json        # JSON output

# View job details
raibid-cli job show <job-id>             # Show job details
raibid-cli job show <job-id> --output json

# Manage jobs
raibid-cli job cancel <job-id>           # Cancel a job (with confirmation)
raibid-cli job cancel <job-id> --force   # Cancel without confirmation
raibid-cli job retry <job-id>            # Retry a failed job
```

### Agent Management

Manage build agents:

```bash
# List agents
raibid-cli agent list                    # List all agents
raibid-cli agent list --status idle      # Filter by status
raibid-cli agent list --output json      # JSON output

# View agent details
raibid-cli agent show <agent-id>         # Show agent details
raibid-cli agent show <agent-id> --output json

# Manage agents
raibid-cli agent restart <agent-id>      # Restart an agent (with confirmation)
raibid-cli agent restart <agent-id> --force
raibid-cli agent scale --count 5         # Scale to 5 agents
raibid-cli agent scale --count 3 --min 2 --max 8
```

### Repository Mirroring

Sync GitHub repositories to local Gitea:

```bash
# Add mirrors
raibid-cli mirror add github.com/user/repo              # Add mirror
raibid-cli mirror add github.com/user/repo --name my-repo
raibid-cli mirror add github.com/user/repo --sync-interval 30

# List mirrors
raibid-cli mirror list                   # List all mirrors
raibid-cli mirror list --output json     # JSON output

# Sync mirrors
raibid-cli mirror sync github.com/user/repo         # Sync repository
raibid-cli mirror sync github.com/user/repo --force # Force sync

# Remove mirrors
raibid-cli mirror remove github.com/user/repo       # Remove (with confirmation)
raibid-cli mirror remove github.com/user/repo --force
```

### Configuration Management

Manage configuration files:

```bash
# Initialize configuration
raibid-cli config init                   # Create config file
raibid-cli config init --output custom.yaml
raibid-cli config init --minimal         # Minimal config
raibid-cli config init --force           # Overwrite existing

# View configuration
raibid-cli config show                   # Show merged config (YAML)
raibid-cli config show --format json     # JSON format
raibid-cli config show --format toml     # TOML format
raibid-cli config show --file path/to/config.yaml

# Validate configuration
raibid-cli config validate               # Validate merged config
raibid-cli config validate path/to/config.yaml

# Show config path
raibid-cli config path                   # Show config file location
```

### Global Options

```bash
raibid-cli --verbose <command>    # Enable verbose logging
raibid-cli --version              # Show version
raibid-cli --help                 # Show help
```

## Configuration

Configuration files are loaded in priority order (highest to lowest):

1. **Environment variables** - `RAIBID_*` prefixed variables
2. **Local file** - `./raibid.yaml` in current directory
3. **User file** - `~/.config/raibid/config.yaml`
4. **System file** - `/etc/raibid/config.yaml`
5. **Built-in defaults**

### Example Configuration

```yaml
# Cluster configuration
cluster:
  name: "dgx-spark-ci"
  namespace: "raibid-ci"
  kubeconfig: "~/.kube/config"

# API server configuration
api:
  host: "localhost"
  port: 8080
  timeout_seconds: 30

# Agent configuration
agents:
  min_count: 0           # Scale to zero when idle
  max_count: 8           # Maximum concurrent agents
  idle_timeout_minutes: 5
  image: "raibid/rust-builder:latest"

# Gitea configuration
gitea:
  url: "http://gitea.raibid-ci.svc.cluster.local:3000"
  admin_user: "admin"
  # admin_password loaded from RAIBID_GITEA_ADMIN_PASSWORD

# Redis configuration
redis:
  url: "redis://redis.raibid-ci.svc.cluster.local:6379"
  stream_name: "ci-jobs"
  consumer_group: "ci-workers"

# TUI configuration
tui:
  refresh_interval_ms: 1000
  panel_proportions: [70, 15, 15]  # [main, header, footer]
```

### Environment Variables

Override configuration with environment variables:

```bash
export RAIBID_API_HOST="api.example.com"
export RAIBID_API_PORT="9090"
export RAIBID_AGENTS_MAX_COUNT="16"
export RAIBID_GITEA_ADMIN_PASSWORD="secret"
```

## Development

### Prerequisites

- **Rust** - 1.70+ (latest stable recommended)
- **Cargo** - Rust package manager

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check binary size
ls -lh target/release/raibid-cli
```

### Testing

```bash
# Run all tests
cargo test --all-features

# Run specific test file
cargo test --test cli_test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_version_flag
```

### Code Quality

```bash
# Run clippy linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Testing TUI Locally

The TUI uses mock data for development and testing:

```bash
# Run TUI with debug logging
RUST_LOG=debug cargo run -- tui

# Build and run release version
cargo build --release
./target/release/raibid-cli tui
```

## Architecture

### Project Structure

```
raibid-cli/
├── src/
│   ├── cli/              # CLI argument parsing (clap)
│   ├── commands/         # Command implementations
│   │   ├── config.rs     # Configuration management
│   │   ├── setup.rs      # Infrastructure setup
│   │   ├── teardown.rs   # Infrastructure teardown
│   │   ├── status.rs     # Status checking
│   │   ├── job.rs        # Job management
│   │   ├── agent.rs      # Agent management
│   │   └── mirror.rs     # Repository mirroring
│   ├── config/           # Configuration loading and validation
│   ├── infrastructure/   # Infrastructure provisioning
│   │   ├── k3s.rs        # k3s cluster installer
│   │   ├── gitea.rs      # Gitea + OCI registry installer
│   │   ├── redis.rs      # Redis Streams installer
│   │   ├── keda.rs       # KEDA autoscaler installer
│   │   ├── flux.rs       # Flux GitOps installer
│   │   ├── status.rs     # Real-time status checking
│   │   ├── error.rs      # Error handling types
│   │   ├── retry.rs      # Retry logic with backoff
│   │   ├── rollback.rs   # Transaction-based rollback
│   │   ├── preflight.rs  # Pre-flight validation
│   │   └── healthcheck.rs # Health check utilities
│   ├── tui/              # Terminal UI (Ratatui)
│   │   ├── app.rs        # Application state
│   │   ├── ui.rs         # UI rendering
│   │   ├── events.rs     # Event handling
│   │   └── mock_data.rs  # Mock data generator
│   ├── lib.rs            # Library entry point
│   └── main.rs           # Binary entry point
├── tests/                # Integration tests
│   ├── cli_test.rs
│   ├── config_test.rs
│   ├── tui_test.rs
│   ├── redis_test.rs
│   ├── flux_test.rs
│   ├── status_test.rs
│   └── error_handling_test.rs
├── docs/                 # Documentation
│   ├── USER_GUIDE.md
│   ├── gitea-installation.md
│   ├── redis-deployment.md
│   ├── keda-installation.md
│   └── error-recovery.md
└── Cargo.toml
```

### Dependencies

**Core:**
- `clap` - CLI argument parsing
- `anyhow` - Error handling
- `tracing` - Structured logging

**Infrastructure:**
- `kube` - Kubernetes API client
- `k8s-openapi` - Kubernetes resource types
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `sha256` - Checksum verification

**TUI:**
- `ratatui` - Terminal UI framework
- `crossterm` - Terminal manipulation

**Config:**
- `serde` - Serialization framework
- `serde_yaml` - YAML support
- `toml` - TOML support
- `serde_json` - JSON support

**Display:**
- `comfy-table` - ASCII table rendering
- `colored` - Terminal colors
- `dialoguer` - Interactive prompts

See `Cargo.toml` for full dependency list.

## System Requirements

### Minimum Requirements

- **OS**: Linux (Ubuntu 22.04+), macOS, Windows (WSL2)
- **Memory**: 100MB RAM
- **Disk**: 10MB for binary

### Target Platform: NVIDIA DGX Spark

- **CPU**: 20 cores ARM64 (10x Cortex-X925, 10x Cortex-A725)
- **Memory**: 128GB LPDDR5x unified memory
- **Storage**: Up to 4TB NVMe
- **Network**: 200 Gb/s ConnectX-7

### Resource Footprint

- **Base infrastructure**: ~4 cores, ~4GB RAM
- **Per agent**: ~2 cores, ~4GB RAM
- **TUI client**: <10MB RAM

## Roadmap

### ✅ Completed: WS-01 - CLI/TUI Application

- ✅ CLI scaffolding with clap
- ✅ Ratatui TUI with 4-tab dashboard (Jobs, Agents, Config, Logs)
- ✅ Enhanced TUI widgets and navigation
- ✅ Interactive controls and popups
- ✅ Job, agent, and mirror commands
- ✅ Configuration management with multi-source loading
- ✅ Comprehensive testing (100+ tests) and documentation

### ✅ Completed: WS-04 - Infrastructure Provisioning

- ✅ k3s cluster installation with binary verification
- ✅ Gitea deployment with OCI registry via Helm
- ✅ Redis Streams for job queue management
- ✅ KEDA autoscaler with Redis Streams trigger
- ✅ Flux GitOps bootstrap with Gitea integration
- ✅ Real-time status monitoring via Kubernetes API
- ✅ Enhanced error handling with retry logic and rollback

### 🔜 Next: WS-02 - API Server & Job Execution

- API server implementation in Rust
- Job dispatcher with Redis Streams
- Agent lifecycle management
- Build execution and caching

### 📋 Future Workstreams

- **WS-03**: CI agent implementations (Rust, multi-language)
- **WS-05**: Repository mirroring automation
- **WS-06**: Integration testing and production deployment

## Troubleshooting

### Binary Not Found After Building

**Problem**: `cargo build --release` completes but no binary in `target/release/`

**Cause**: You have `CARGO_TARGET_DIR` environment variable set, which redirects build output.

**Solution**:

```bash
# Check if CARGO_TARGET_DIR is set
echo $CARGO_TARGET_DIR

# If set, your binary is at:
ls -lh $CARGO_TARGET_DIR/release/raibid-cli

# Option 1: Copy to expected location
mkdir -p target/release
cp $CARGO_TARGET_DIR/release/raibid-cli target/release/

# Option 2: Unset and rebuild
unset CARGO_TARGET_DIR
cargo build --release

# Option 3: Build with explicit target dir
cargo build --release --target-dir ./target
```

### Cargo Not Found

**Problem**: `cargo: command not found`

**Solution**: Install Rust and add to PATH

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source $HOME/.cargo/env

# Verify
cargo --version
```

### TUI Not Rendering Properly

```bash
# Check terminal compatibility
echo $TERM

# Try different terminal emulator
# Recommended: Alacritty, WezTerm, iTerm2, Windows Terminal
```

### Configuration Not Loading

```bash
# Check config file location
raibid-cli config path

# Validate config syntax
raibid-cli config validate

# Show merged config
raibid-cli config show
```

### Build Errors

```bash
# Update Rust toolchain
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

## Contributing

This project is currently in active development. Contributions are welcome!

### Development Guidelines

1. Follow Rust best practices and idioms
2. Write tests for new features
3. Run clippy and rustfmt before committing
4. Update documentation for user-facing changes

### Testing Changes

```bash
# Run full test suite
cargo test --all-features

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license for your use.

## Acknowledgments

- Built with [Ratatui](https://ratatui.rs) for terminal UI
- CLI parsing with [clap](https://github.com/clap-rs/clap)
- Optimized for [NVIDIA DGX Spark](https://www.nvidia.com/en-us/data-center/dgx-spark/)

## Links

- **Documentation**: [docs/](docs/)
- **User Guide**: [docs/USER_GUIDE.md](docs/USER_GUIDE.md)
- **Orchestrator Guide**: [ORCHESTRATOR.md](ORCHESTRATOR.md) - Multi-agent development system
- **GitHub Repository**: https://github.com/raibid-labs/raibid-cli
- **Issue Tracker**: https://github.com/raibid-labs/raibid-cli/issues

---

**Built for developers, by developers. Optimized for NVIDIA DGX Spark.**

*Last Updated: 2025-10-30*
*Status: WS-01 & WS-04 Complete - Production Infrastructure Ready* 🚀
