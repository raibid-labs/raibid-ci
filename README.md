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

# Verify the build succeeded
ls -lh target/release/raibid-cli
./target/release/raibid-cli --version

# Install to user-local directory (recommended, no sudo required)
mkdir -p ~/.local/bin
cp target/release/raibid-cli ~/.local/bin/

# Add to PATH if not already (add to your ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.local/bin:$PATH"
```

**Verify Build Succeeded:**

After building, verify the binary was created:

```bash
# For native build
ls -lh target/release/raibid-cli
./target/release/raibid-cli --version

# Expected output: raibid-cli 0.1.0
```

If the binary is not found at `target/release/raibid-cli`, you may have `CARGO_TARGET_DIR` environment variable set. See the [Troubleshooting](#troubleshooting) section below.

**Alternative: System-wide Installation**

If you prefer system-wide installation:

```bash
# Install to /usr/local/bin (requires sudo)
sudo cp target/release/raibid-cli /usr/local/bin/
```

#### For DGX Spark (ARM64)

```bash
# Add ARM64 target (if not already installed)
rustup target add aarch64-unknown-linux-gnu

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Verify the build succeeded
ls -lh target/aarch64-unknown-linux-gnu/release/raibid-cli

# Verify architecture (optional)
file target/aarch64-unknown-linux-gnu/release/raibid-cli

# Install to user-local directory
mkdir -p ~/.local/bin
cp target/aarch64-unknown-linux-gnu/release/raibid-cli ~/.local/bin/
```

**Verify ARM64 Build:**

```bash
# Check binary exists
ls -lh target/aarch64-unknown-linux-gnu/release/raibid-cli

# Verify it's an ARM64 binary
file target/aarch64-unknown-linux-gnu/release/raibid-cli
# Expected output: ELF 64-bit LSB executable, ARM aarch64...

# Test version (only works on ARM64 systems)
./target/aarch64-unknown-linux-gnu/release/raibid-cli --version
```

#### Installation Directory

By default, `raibid-cli setup all` installs infrastructure binaries (k3s, flux) to `~/.local/bin`. This directory:

- Requires no sudo/elevated permissions
- Follows the XDG Base Directory specification
- Works for single-user setups

If `~/.local/bin` is not in your PATH, raibid-cli will display a warning with instructions to add it.

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

### Container Registry

Gitea includes an OCI-compliant container registry for storing Docker images. The registry is automatically configured when Gitea is deployed.

**Registry Access:**
- **Registry URL:** `localhost:30500`
- **Authentication:** Uses Gitea credentials (`raibid-admin` / `adminadmin` for local dev)
- **Images:** Stored under namespace `raibid-admin`

**Push Images:**

```bash
# Log in to the registry
docker login localhost:30500 -u raibid-admin -p adminadmin

# Tag an image for the registry
docker tag my-image:latest localhost:30500/raibid-admin/my-image:latest

# Push to the registry
docker push localhost:30500/raibid-admin/my-image:latest
```

**Pull Images:**

```bash
# Pull from the registry
docker pull localhost:30500/raibid-admin/my-image:latest
```

**Tilt Integration:**

When using Tilt for development, Docker images are automatically built and pushed to the Gitea registry:
- `localhost:30500/raibid-admin/raibid-server:latest`
- `localhost:30500/raibid-admin/raibid-agent:latest`

The Kubernetes cluster is configured to pull images from this local registry without requiring imagePullSecrets for development.

**Registry Web UI:**

Browse the registry via Gitea's web interface:
1. Navigate to http://localhost:3000
2. Login as `raibid-admin`
3. Click on "Packages" in the top navigation
4. Select "Container" to view OCI images

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

### Permission Denied During Setup

**Problem**: `raibid-cli setup all` fails with "Permission denied (os error 13)"

**Cause**: Trying to install to a directory that requires elevated permissions (like `/usr/local/bin`)

**Solution**:

By default, raibid-cli now installs to `~/.local/bin` which requires no sudo. If you're seeing this error:

1. **Check your configuration** - You may have a custom `install_dir` set:
   ```bash
   raibid-cli config show
   ```

2. **Use the default user-local installation** (recommended):
   - Remove any custom `install_dir` from your config
   - Ensure `~/.local/bin` is in your PATH:
     ```bash
     # Add to ~/.bashrc or ~/.zshrc
     export PATH="$HOME/.local/bin:$PATH"

     # Reload shell
     source ~/.bashrc  # or source ~/.zshrc
     ```

3. **Or use a custom writable directory**:
   ```bash
   # Create a custom bin directory
   mkdir -p ~/bin

   # Set it in your config (create ~/.config/raibid/config.yaml)
   echo "install_dir: $HOME/bin" > ~/.config/raibid/config.yaml

   # Add to PATH
   export PATH="$HOME/bin:$PATH"
   ```

4. **System-wide installation** (not recommended):
   If you need system-wide installation, you'll need to manually install binaries:
   ```bash
   # Download binaries first
   raibid-cli setup all --download-only

   # Then manually copy with sudo
   sudo cp ~/.cache/raibid/k3s /usr/local/bin/
   sudo cp ~/.cache/raibid/flux /usr/local/bin/
   ```

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

## Documentation

Comprehensive documentation for installation, operation, and development:

### Getting Started
- **[Installation Guide](docs/INSTALLATION.md)** - Complete setup from bare metal to operational system
- **[User Guide](docs/USER_GUIDE.md)** - Usage instructions and examples

### Operations
- **[Operations Runbook](docs/OPERATIONS.md)** - Maintenance, troubleshooting, and incident response
- **[API Documentation](docs/API.md)** - REST API reference and examples

### Development
- **[Tilt Development Guide](TILT.md)** - Local development with Tilt and Tanka
- **[Tanka Configuration](tanka/README.md)** - Kubernetes deployment with Jsonnet
- **[Orchestrator Guide](ORCHESTRATOR.md)** - Multi-agent development system

## Links

- **Documentation**: [docs/](docs/)
- **GitHub Repository**: https://github.com/raibid-labs/raibid-cli
- **Issue Tracker**: https://github.com/raibid-labs/raibid-cli/issues

---

**Built for developers, by developers. Optimized for NVIDIA DGX Spark.**

*Last Updated: 2025-11-03*
*Status: WS-01 & WS-04 Complete - Production Infrastructure Ready* 🚀
