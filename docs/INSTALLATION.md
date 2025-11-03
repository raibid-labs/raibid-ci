# Installation Guide

Complete guide for installing and setting up raibid-ci from scratch.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Installation](#detailed-installation)
  - [1. Install Rust](#1-install-rust)
  - [2. Clone and Build](#2-clone-and-build)
  - [3. Install Infrastructure Tools](#3-install-infrastructure-tools)
  - [4. Set Up Kubernetes Cluster](#4-set-up-kubernetes-cluster)
  - [5. Configure raibid-ci](#5-configure-raibid-ci)
  - [6. Deploy Infrastructure](#6-deploy-infrastructure)
- [Platform-Specific Notes](#platform-specific-notes)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)
- [Next Steps](#next-steps)

## Overview

raibid-ci is a terminal-based CI/CD management system optimized for NVIDIA DGX Spark. The installation process involves:

1. Building the raibid-cli binary from Rust source
2. Installing infrastructure tools (k3s, Tanka, Tilt, Just, Helm)
3. Setting up a local Kubernetes cluster
4. Deploying infrastructure components (Redis, Gitea, KEDA, Flux)
5. Configuring and launching the system

**Estimated Time**: 30-60 minutes (depending on platform and network speed)

## Prerequisites

### Hardware Requirements

**Minimum**:
- CPU: 4 cores
- Memory: 8GB RAM
- Disk: 50GB free space
- Network: Stable internet connection for downloads

**Recommended** (DGX Spark):
- CPU: 20 cores ARM64 (10x Cortex-X925, 10x Cortex-A725)
- Memory: 128GB LPDDR5x
- Disk: 4TB NVMe
- Network: 200 Gb/s ConnectX-7

### Operating System

**Supported Platforms**:
- Linux (Ubuntu 22.04 LTS or later) - Primary target
- Linux (Other distributions with systemd)
- macOS (Development only, not for production)

**Not Supported**:
- Windows (WSL2 may work but is untested)

### Required Permissions

- Ability to install software (either via package manager or to `~/.local/bin`)
- For k3s: `sudo` access or rootless k3s setup
- Network access to download dependencies

## Quick Start

For experienced users who want to get started quickly:

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Clone and build
git clone https://github.com/raibid-labs/raibid-ci.git
cd raibid-ci
cargo build --release

# 3. Install raibid-cli
mkdir -p ~/.local/bin
cp target/release/raibid-cli ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"

# 4. Verify installation
raibid-cli --version

# 5. Initialize and setup
raibid-cli config init
raibid-cli setup all

# 6. Launch TUI
raibid-cli tui
```

For first-time users or those encountering issues, continue with the detailed installation steps below.

## Detailed Installation

### 1. Install Rust

raibid-ci is written in Rust and must be compiled from source.

#### Linux

```bash
# Download and run rustup installer
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts (choose option 1 for default installation)
# This installs Rust to ~/.cargo

# Reload your shell environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

**Expected Output**:
```
rustc 1.82.0 (f6e511eec 2024-10-15)
cargo 1.82.0 (8f40fc59f 2024-08-21)
```

#### macOS

```bash
# Install via rustup (same as Linux)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Or via Homebrew
brew install rustup
rustup-init

# Verify installation
rustc --version
cargo --version
```

#### Add to Shell Profile

To ensure Rust is available in future sessions, add to your shell profile:

```bash
# For bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# For zsh
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc

# For fish
echo 'set -gx PATH $HOME/.cargo/bin $PATH' >> ~/.config/fish/config.fish
```

### 2. Clone and Build

#### Clone Repository

```bash
# Clone the repository
git clone https://github.com/raibid-labs/raibid-ci.git
cd raibid-ci

# Verify you're on the main branch
git branch --show-current
```

**Expected Output**:
```
main
```

#### Build Release Binary

```bash
# Build all workspace crates in release mode
cargo build --release --workspace

# This will take 5-15 minutes on first build
# Subsequent builds are much faster due to incremental compilation
```

**Expected Output**:
```
   Compiling raibid-common v0.1.0 (/home/user/raibid-ci/crates/common)
   Compiling raibid-tui v0.1.0 (/home/user/raibid-ci/crates/tui)
   Compiling raibid-cli v0.1.0 (/home/user/raibid-ci/crates/cli)
   Compiling raibid-server v0.1.0 (/home/user/raibid-ci/crates/server)
   Compiling raibid-agent v0.1.0 (/home/user/raibid-ci/crates/agent)
    Finished release [optimized] target(s) in 8m 32s
```

#### Verify Build

```bash
# Check the binary was created
ls -lh target/release/raibid-cli

# Test the binary
./target/release/raibid-cli --version
```

**Expected Output**:
```
-rwxr-xr-x 1 user user 12M Nov  3 12:00 target/release/raibid-cli
raibid-cli 0.1.0
```

**Troubleshooting**: If the binary is not at `target/release/raibid-cli`, check if `CARGO_TARGET_DIR` environment variable is set. See [Troubleshooting](#troubleshooting) section.

#### Install Binary

```bash
# Create user-local bin directory (no sudo required)
mkdir -p ~/.local/bin

# Copy binary to local bin
cp target/release/raibid-cli ~/.local/bin/

# Add to PATH (if not already in PATH)
export PATH="$HOME/.local/bin:$PATH"

# Verify installation
which raibid-cli
raibid-cli --version
```

**Expected Output**:
```
/home/user/.local/bin/raibid-cli
raibid-cli 0.1.0
```

#### Alternative: System-Wide Installation

If you need system-wide installation (not recommended for personal use):

```bash
# Requires sudo
sudo cp target/release/raibid-cli /usr/local/bin/

# Verify
which raibid-cli
```

### 3. Install Infrastructure Tools

raibid-ci requires several infrastructure tools for full functionality.

#### Install Just (Command Runner)

```bash
# Linux (using cargo)
cargo install just

# Or via package manager
# Ubuntu/Debian
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/.local/bin

# macOS
brew install just

# Verify installation
just --version
```

**Expected Output**:
```
just 1.35.0
```

#### Install Docker

Required for building container images with Tilt.

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install docker.io docker-compose

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group (optional, avoids needing sudo)
sudo usermod -aG docker $USER

# Log out and back in for group changes to take effect
# Or run: newgrp docker

# Verify installation
docker --version
docker ps
```

**Expected Output**:
```
Docker version 24.0.7, build afdd53b
CONTAINER ID   IMAGE     COMMAND   CREATED   STATUS    PORTS     NAMES
```

**macOS**:
```bash
# Install Docker Desktop
brew install --cask docker

# Start Docker Desktop from Applications
# Verify
docker --version
```

#### Install Tanka

Tanka is used for Kubernetes configuration management via Jsonnet.

```bash
# Linux (ARM64 - for DGX Spark)
curl -Lo ~/.local/bin/tk https://github.com/grafana/tanka/releases/latest/download/tk-linux-arm64
chmod +x ~/.local/bin/tk

# Linux (x86_64)
curl -Lo ~/.local/bin/tk https://github.com/grafana/tanka/releases/latest/download/tk-linux-amd64
chmod +x ~/.local/bin/tk

# macOS (Intel)
curl -Lo ~/.local/bin/tk https://github.com/grafana/tanka/releases/latest/download/tk-darwin-amd64
chmod +x ~/.local/bin/tk

# macOS (Apple Silicon)
curl -Lo ~/.local/bin/tk https://github.com/grafana/tanka/releases/latest/download/tk-darwin-arm64
chmod +x ~/.local/bin/tk

# Verify installation
tk --version
```

**Expected Output**:
```
tk version v0.26.0
```

#### Install jsonnet-bundler

Required for Tanka dependency management.

```bash
# Linux (ARM64)
curl -Lo ~/.local/bin/jb https://github.com/jsonnet-bundler/jsonnet-bundler/releases/latest/download/jb-linux-arm64
chmod +x ~/.local/bin/jb

# Linux (x86_64)
curl -Lo ~/.local/bin/jb https://github.com/jsonnet-bundler/jsonnet-bundler/releases/latest/download/jb-linux-amd64
chmod +x ~/.local/bin/jb

# macOS
brew install jsonnet-bundler

# Verify installation
jb --version
```

#### Install Helm

Required for vendoring Helm charts used by Tanka.

```bash
# Linux
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# macOS
brew install helm

# Verify installation
helm version
```

**Expected Output**:
```
version.BuildInfo{Version:"v3.16.1", GitCommit:"5a5449dc42be07001fd5771d56429132984ab3ab", GitTreeState:"clean", GoVersion:"go1.22.7"}
```

#### Install Tilt

Tilt orchestrates the development environment.

```bash
# Linux
curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

# macOS
brew install tilt

# Verify installation
tilt version
```

**Expected Output**:
```
v0.35.0, built 2024-10-15
```

### 4. Set Up Kubernetes Cluster

raibid-ci requires a Kubernetes cluster. We use k3s, a lightweight Kubernetes distribution.

#### Install k3s (Linux)

```bash
# Navigate to k3s installation directory
cd infra/k3s

# Review the install script
cat install.sh

# Run installation (requires sudo)
sudo ./install.sh

# Wait for k3s to start (takes 30-60 seconds)
# Verify k3s is running
sudo systemctl status k3s
```

**Expected Output**:
```
● k3s.service - Lightweight Kubernetes
     Loaded: loaded (/etc/systemd/system/k3s.service; enabled; vendor preset: enabled)
     Active: active (running) since Mon 2025-11-03 12:00:00 UTC; 1min ago
```

#### Configure kubectl Access

```bash
# Create kubeconfig directory
mkdir -p ~/.kube

# Copy k3s config to user directory
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $USER:$USER ~/.kube/config

# Set KUBECONFIG environment variable
export KUBECONFIG=~/.kube/config

# Add to shell profile
echo 'export KUBECONFIG=~/.kube/config' >> ~/.bashrc

# Verify kubectl works
kubectl cluster-info
kubectl get nodes
```

**Expected Output**:
```
Kubernetes control plane is running at https://127.0.0.1:6443
CoreDNS is running at https://127.0.0.1:6443/api/v1/namespaces/kube-system/services/kube-dns:dns/proxy
Metrics-server is running at https://127.0.0.1:6443/api/v1/namespaces/kube-system/services/https:metrics-server:https/proxy

NAME        STATUS   ROLES                  AGE   VERSION
localhost   Ready    control-plane,master   2m    v1.28.8+k3s1
```

#### Alternative: k3d (Docker-based k3s)

For development on macOS or if you prefer Docker-based k3s:

```bash
# Install k3d
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Or via Homebrew
brew install k3d

# Create cluster
k3d cluster create raibid-ci

# Verify
kubectl cluster-info
```

### 5. Configure raibid-ci

#### Initialize Configuration

```bash
# Return to project root
cd ~/raibid-ci

# Initialize default configuration
raibid-cli config init

# This creates ~/.config/raibid/config.yaml
```

**Expected Output**:
```
Configuration file created at: /home/user/.config/raibid/config.yaml
```

#### Review Configuration

```bash
# View generated configuration
raibid-cli config show

# Or edit manually
vim ~/.config/raibid/config.yaml
```

**Default Configuration**:
```yaml
# Cluster configuration
cluster:
  name: "raibid-ci"
  namespace: "raibid-system"
  kubeconfig: "~/.kube/config"

# API server configuration
api:
  host: "0.0.0.0"
  port: 8080
  timeout_seconds: 30

# Agent configuration
agents:
  min_count: 0
  max_count: 8
  idle_timeout_minutes: 5
  image: "raibid-agent:latest"

# Gitea configuration
gitea:
  url: "http://gitea.raibid-system.svc.cluster.local:3000"
  admin_user: "gitea"

# Redis configuration
redis:
  url: "redis://redis.raibid-system.svc.cluster.local:6379"
  stream_name: "ci:jobs"
  consumer_group: "ci-workers"

# TUI configuration
tui:
  refresh_interval_ms: 1000
  panel_proportions: [70, 15, 15]
```

#### Set Environment Variables (Optional)

For sensitive values like passwords:

```bash
# Set Gitea admin password
export RAIBID_GITEA_ADMIN_PASSWORD="your-secure-password"

# Set webhook secrets (if using GitHub/Gitea webhooks)
export RAIBID_GITHUB_WEBHOOK_SECRET="your-github-secret"
export RAIBID_GITEA_WEBHOOK_SECRET="your-gitea-secret"

# Add to shell profile to persist
echo 'export RAIBID_GITEA_ADMIN_PASSWORD="your-secure-password"' >> ~/.bashrc
```

### 6. Deploy Infrastructure

#### Vendor Helm Charts

Before deploying, vendor the Helm charts used by Tanka:

```bash
# Navigate to tanka directory
cd tanka

# Add Helm repositories
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add gitea-charts https://dl.gitea.io/charts/
helm repo add kedacore https://kedacore.github.io/charts
helm repo add fluxcd-community https://fluxcd-community.github.io/helm-charts
helm repo update

# Pull and extract charts
helm pull bitnami/redis --version 18.19.4 --untar -d vendor/
helm pull gitea-charts/gitea --version 10.1.4 --untar -d vendor/
helm pull kedacore/keda --version 2.14.0 --untar -d vendor/
helm pull fluxcd-community/flux2 --version 2.12.4 --untar -d vendor/

# Verify charts are vendored
ls -la vendor/
```

**Expected Output**:
```
drwxr-xr-x 2 user user 4096 Nov  3 12:00 redis
drwxr-xr-x 2 user user 4096 Nov  3 12:00 gitea
drwxr-xr-x 2 user user 4096 Nov  3 12:00 keda
drwxr-xr-x 2 user user 4096 Nov  3 12:00 flux2
```

**Or use Just command** (from project root):
```bash
cd ..
just tk-vendor
```

#### Deploy All Components

```bash
# Return to project root
cd ~/raibid-ci

# Deploy all infrastructure components
# This sets up: k3s, Redis, Gitea, KEDA, Flux
raibid-cli setup all
```

**Expected Output**:
```
[1/5] Setting up k3s cluster...
✓ k3s already installed at /usr/local/bin/k3s
✓ k3s cluster is running

[2/5] Deploying Redis Streams...
✓ Redis StatefulSet deployed
✓ Redis Service created
✓ Waiting for Redis to be ready... (30s)

[3/5] Deploying Gitea with OCI registry...
✓ Gitea Deployment created
✓ Gitea Service created
✓ Waiting for Gitea to be ready... (60s)

[4/5] Deploying KEDA autoscaler...
✓ KEDA CRDs installed
✓ KEDA operator deployed

[5/5] Bootstrapping Flux GitOps...
✓ Flux components installed
✓ Flux configured for Gitea

All infrastructure components deployed successfully!
```

**This process takes 5-10 minutes** as components are downloaded and started.

#### Verify Deployment

```bash
# Check all pods are running
kubectl get pods -n raibid-system

# Check services
kubectl get services -n raibid-system

# Check KEDA ScaledJobs
kubectl get scaledjob -n raibid-system
```

**Expected Output**:
```
NAME                          READY   STATUS    RESTARTS   AGE
redis-master-0                1/1     Running   0          5m
gitea-0                       1/1     Running   0          4m
keda-operator-123abc-xyz      1/1     Running   0          3m
flux-source-controller-...    1/1     Running   0          2m
```

## Platform-Specific Notes

### Ubuntu 22.04 LTS (DGX Spark Target)

- Recommended for production use
- All components tested on this platform
- Full ARM64 support
- systemd required for k3s service management

**Post-Install**:
```bash
# Ensure k3s starts on boot
sudo systemctl enable k3s

# Check resource usage
kubectl top nodes
kubectl top pods -n raibid-system
```

### Other Linux Distributions

- Ensure systemd is available for k3s
- May need to adjust package manager commands
- Docker installation varies by distribution

**Arch Linux**:
```bash
sudo pacman -S docker
sudo systemctl start docker
```

**Fedora/RHEL**:
```bash
sudo dnf install docker
sudo systemctl start docker
```

### macOS (Development Only)

- Use k3d instead of native k3s
- Docker Desktop required
- Not recommended for production workloads
- Some features may not work (e.g., GPU time-slicing)

**Limitations**:
- No systemd (k3s service management different)
- Resource limits (Docker Desktop VM)
- Networking differences

**Setup**:
```bash
# Use k3d instead of k3s
k3d cluster create raibid-ci \
  --agents 2 \
  --port 8080:80@loadbalancer \
  --port 8443:443@loadbalancer

# Configure kubectl
export KUBECONFIG="$(k3d kubeconfig write raibid-ci)"
```

### DGX Spark Specific

**CPU Affinity**: k3s is configured to reserve cores for system and Kubernetes control plane, leaving cores for workloads.

**Resource Reservations**:
- System: 2 cores, 8GB RAM
- Kubernetes: 4 cores, 16GB RAM
- Available for workloads: 14 cores, 104GB RAM

**Memory Bandwidth**: DGX Spark has 273 GB/s unified memory bandwidth - optimize agent count for best throughput.

**Recommended Configuration**:
```yaml
agents:
  min_count: 0
  max_count: 8  # 14 cores / 2 cores per agent = 7, with buffer
  idle_timeout_minutes: 5
```

## Verification

### Test CLI

```bash
# Check version
raibid-cli --version

# Show configuration
raibid-cli config show

# Check infrastructure status
raibid-cli status
```

**Expected Output**:
```
raibid-cli 0.1.0

Component     Status      Details
─────────────────────────────────
k3s           ✓ Running   1.28.8+k3s1
Redis         ✓ Running   master-0: 1/1
Gitea         ✓ Running   gitea-0: 1/1
KEDA          ✓ Running   operator: 1/1
Flux          ✓ Running   4/4 controllers
```

### Test TUI

```bash
# Launch TUI dashboard
raibid-cli tui
```

**Expected**: Interactive dashboard with 4 tabs (Jobs, Agents, Config, Logs)

**Navigation**:
- Tab / Shift+Tab: Switch tabs
- ↑/↓ or j/k: Navigate lists
- q or Ctrl+C: Quit

### Test API Server

```bash
# Run server (in separate terminal)
just server

# Test health endpoint
curl http://localhost:8080/health

# Expected output:
# {"status":"ok","uptime_seconds":5,"requests_total":1,"active_connections":1,"timestamp":"2025-11-03T12:00:00Z"}
```

### Test Development Environment (Tilt)

```bash
# Start Tilt (requires all previous steps completed)
tilt up

# Access Tilt UI at http://localhost:10350
# Press Ctrl+C to stop
```

## Troubleshooting

### Rust/Cargo Issues

#### Problem: `cargo: command not found`

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add to PATH permanently
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
```

#### Problem: Build fails with "linker not found"

**Solution** (Ubuntu/Debian):
```bash
# Install build essentials
sudo apt-get install build-essential
```

**Solution** (macOS):
```bash
# Install Xcode command line tools
xcode-select --install
```

#### Problem: Binary not found after build

**Cause**: `CARGO_TARGET_DIR` environment variable redirects build output.

**Solution**:
```bash
# Check if set
echo $CARGO_TARGET_DIR

# If set, find binary there
ls -lh $CARGO_TARGET_DIR/release/raibid-cli

# Copy to expected location
mkdir -p target/release
cp $CARGO_TARGET_DIR/release/raibid-cli target/release/

# Or unset and rebuild
unset CARGO_TARGET_DIR
cargo build --release
```

### k3s Issues

#### Problem: Permission denied installing k3s

**Solution**: k3s requires sudo for system-wide installation.
```bash
sudo ./infra/k3s/install.sh
```

#### Problem: k3s fails to start

**Check logs**:
```bash
sudo journalctl -u k3s -f
```

**Common issues**:
- Port 6443 already in use
- Insufficient permissions
- Conflicting Kubernetes installation

**Solution**: Uninstall conflicting installations
```bash
# Stop k3s
sudo systemctl stop k3s

# Uninstall k3s
sudo /usr/local/bin/k3s-uninstall.sh

# Reinstall
cd infra/k3s
sudo ./install.sh
```

#### Problem: kubectl cannot connect

**Solution**: Configure KUBECONFIG
```bash
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

# Or copy to user directory
mkdir -p ~/.kube
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $USER:$USER ~/.kube/config
export KUBECONFIG=~/.kube/config
```

### Infrastructure Deployment Issues

#### Problem: Helm charts not found

**Cause**: Helm charts not vendored before running Tanka.

**Solution**:
```bash
cd tanka
just tk-vendor
# Or manually run helm pull commands
```

#### Problem: Pods stuck in Pending state

**Check pod status**:
```bash
kubectl describe pod <pod-name> -n raibid-system
```

**Common causes**:
- Insufficient resources
- Image pull errors
- Storage issues

**Solution**: Check node resources
```bash
kubectl describe nodes
kubectl top nodes
```

#### Problem: Gitea pod crash loops

**Check logs**:
```bash
kubectl logs -f <gitea-pod-name> -n raibid-system
```

**Common issues**:
- Database initialization failed
- Persistent volume issues

**Solution**: Delete and redeploy
```bash
kubectl delete pod <gitea-pod-name> -n raibid-system
# Pod will auto-restart
```

### Configuration Issues

#### Problem: Configuration file not found

**Solution**: Initialize configuration
```bash
raibid-cli config init
```

#### Problem: Configuration validation fails

**Check validation**:
```bash
raibid-cli config validate
```

**Common issues**:
- Invalid YAML syntax
- Missing required fields
- Invalid values

**Solution**: View and fix configuration
```bash
# Show merged config
raibid-cli config show

# Edit config file
vim ~/.config/raibid/config.yaml
```

### Network/Port Issues

#### Problem: Port 8080 already in use

**Find process**:
```bash
sudo lsof -i :8080
# Or
sudo netstat -tulpn | grep :8080
```

**Solution**: Kill process or change port
```bash
# Change port in config
raibid-cli config show
# Edit port value

# Or use environment variable
export RAIBID_API_PORT=9090
```

### Docker Issues

#### Problem: Permission denied accessing Docker

**Solution**: Add user to docker group
```bash
sudo usermod -aG docker $USER

# Log out and back in, or run:
newgrp docker

# Verify
docker ps
```

#### Problem: Docker daemon not running

**Solution**:
```bash
# Linux
sudo systemctl start docker

# macOS
# Start Docker Desktop from Applications
```

## Next Steps

After successful installation:

1. **Read the User Guide**: See [docs/USER_GUIDE.md](USER_GUIDE.md) for usage instructions

2. **Configure Repository Mirroring**: Set up GitHub/Gitea repository sync
   ```bash
   raibid-cli mirror add github.com/your-org/your-repo
   ```

3. **Set Up Webhooks**: Configure webhooks for automatic CI job triggers
   - See [docs/webhook-configuration.md](webhook-configuration.md)

4. **Launch TUI Dashboard**: Monitor your CI system
   ```bash
   raibid-cli tui
   ```

5. **Explore Development Environment**: Use Tilt for development
   ```bash
   tilt up
   # Access http://localhost:10350
   ```

6. **Review Operations Guide**: Learn about maintenance and troubleshooting
   - See [docs/OPERATIONS.md](OPERATIONS.md)

7. **Review API Documentation**: Integrate with the API
   - See [docs/API.md](API.md)

## Support

For additional help:

- **Documentation**: [docs/](../docs/)
- **GitHub Issues**: https://github.com/raibid-labs/raibid-ci/issues
- **Tilt Guide**: [TILT.md](../TILT.md)
- **Tanka Guide**: [tanka/README.md](../tanka/README.md)

---

**Installation complete!** You're now ready to use raibid-ci.
