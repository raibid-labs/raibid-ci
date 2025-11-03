# Tilt Development Environment

This document describes how to use Tilt for developing raibid-ci with a complete local Kubernetes development environment.

## Overview

Tilt orchestrates the entire raibid-ci development stack:
- **k3s**: Lightweight Kubernetes cluster
- **Docker**: Builds server and agent images with optimized caching
- **Tanka**: Deploys all infrastructure and application components
- **Port Forwards**: Access services locally
- **Live Reload**: Automatic rebuilds on code changes

## Prerequisites

### Required Tools

1. **k3s** - Lightweight Kubernetes
   ```bash
   # Install k3s (from project root)
   cd infra/k3s
   sudo ./install.sh

   # Verify installation
   kubectl cluster-info
   ```

2. **Tilt** - Development orchestrator
   ```bash
   # Install Tilt (Linux)
   curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

   # Verify installation
   tilt version
   ```

3. **Docker** - Container runtime
   ```bash
   # Verify Docker is running
   docker ps
   ```

4. **Tanka** - Kubernetes configuration tool
   ```bash
   # Install Tanka
   # See: https://tanka.dev/install

   # Verify installation
   tk --version
   ```

5. **kubectl** - Kubernetes CLI
   ```bash
   # Usually installed with k3s
   kubectl version
   ```

### Optional Tools

- **k3d** - k3s in Docker (alternative to native k3s)
  ```bash
  # Install k3d
  curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

  # Create cluster
  k3d cluster create raibid-ci
  ```

## Quick Start

### 1. Start k3s Cluster

```bash
# Option 1: Install k3s (if not already installed)
cd infra/k3s
sudo ./install.sh

# Option 2: Start existing k3s
sudo systemctl start k3s

# Option 3: Use k3d
k3d cluster create raibid-ci
```

### 2. Verify Prerequisites

```bash
# Check cluster is running
kubectl cluster-info

# Check Docker is running
docker ps

# Check Tilt is installed
tilt version

# Check Tanka is installed
tk --version
```

### 3. Start Tilt

```bash
# From project root
tilt up

# Or, run in CI mode (headless)
tilt ci
```

### 4. Access Tilt UI

Tilt UI will automatically open in your browser at: http://localhost:10350

If it doesn't open automatically:
```bash
# Open manually
open http://localhost:10350  # macOS
xdg-open http://localhost:10350  # Linux
```

## What Tilt Does

### On Startup (`tilt up`)

1. **Validates k3s cluster**
   - Checks if kubectl can connect
   - Verifies cluster health
   - Creates required namespaces

2. **Builds Docker images**
   - Server: `raibid-server:latest`
   - Agent: `raibid-agent:latest`
   - Uses cargo-chef for optimal layer caching
   - Parallel builds (max 2 concurrent)

3. **Deploys via Tanka**
   - Infrastructure: Redis, Gitea, KEDA, Flux
   - Applications: Server, Agent
   - Respects resource dependencies

4. **Sets up port forwards**
   - Server API: http://localhost:8080
   - Server Metrics: http://localhost:8081/metrics
   - Gitea Web UI: http://localhost:3000
   - Redis: localhost:6379

5. **Configures resource groups**
   - Infrastructure: redis, gitea, keda
   - Application: server, agent
   - Tools: manual triggers

### During Development

Tilt watches for file changes and automatically:

1. **Rebuilds Docker images** when Rust source changes:
   - `crates/server/**/*.rs` → rebuilds server
   - `crates/agent/**/*.rs` → rebuilds agent
   - `crates/common/**/*.rs` → rebuilds both
   - `Cargo.toml` / `Cargo.lock` → full rebuild

2. **Re-deploys via Tanka** when configuration changes:
   - `tanka/environments/local/main.jsonnet`
   - `tanka/lib/raibid/**/*.libsonnet`
   - `tanka/lib/charts/**/*.libsonnet`

3. **Restarts pods** with new images:
   - Server pods restart after server image rebuild
   - Agent ScaledJob updates after agent image rebuild

## Tilt UI Features

### Resource Groups

Resources are organized into logical groups:

- **Infrastructure**: redis, gitea, keda
  - Core services that applications depend on

- **Application**: server, agent
  - raibid-ci server and agent components

- **Tools**: manual triggers
  - Helper commands for testing and debugging

### Resource Dependencies

Dependencies ensure correct startup order:

```
redis
  └─ server (depends on redis + raibid-server:latest)
      └─ agent (depends on server + keda + raibid-agent:latest)

keda
  └─ agent (depends on keda for autoscaling)
```

### Manual Triggers

Click these in Tilt UI to run manual actions:

1. **trigger-test-job**
   - Sends a test job to Redis queue
   - Useful for testing agent scaling
   - Status: TODO (needs implementation)

2. **scale-agent**
   - Manually scales agent ScaledJob
   - Bypasses KEDA autoscaling temporarily
   - Command: `kubectl scale scaledjob raibid-agent --replicas=1`

3. **view-server-logs**
   - Streams server logs in Tilt UI
   - Shows last 100 lines
   - Command: `kubectl logs -l app=raibid-server --tail=100 -f`

## Service Access

### Server API

```bash
# HTTP API (main endpoint)
curl http://localhost:8080/health

# Metrics endpoint (Prometheus format)
curl http://localhost:8081/metrics
```

### Gitea Web UI

Open in browser: http://localhost:3000

Default credentials:
- Username: `gitea` (check Tanka secrets config)
- Password: `gitea` (check Tanka secrets config)

### Redis

```bash
# Connect with redis-cli
redis-cli -h localhost -p 6379

# List streams
XINFO STREAMS

# Check job queue
XLEN raibid:jobs
```

## Development Workflows

### Workflow 1: Full-Stack Development

```bash
# Start Tilt
tilt up

# Edit Rust code in crates/server/src/main.rs
# Tilt automatically:
#   1. Rebuilds raibid-server image (~30-60s with caching)
#   2. Pushes to cluster
#   3. Restarts server pods
#   4. Logs appear in Tilt UI

# View results in Tilt UI or via curl
curl http://localhost:8080/health
```

### Workflow 2: Configuration Changes

```bash
# Edit Tanka configuration
vim tanka/environments/local/main.jsonnet

# Tilt automatically:
#   1. Detects jsonnet file change
#   2. Runs `tk show` to generate new manifests
#   3. Applies changes to cluster
#   4. Resources update (may restart pods)
```

### Workflow 3: Testing Agent Scaling

```bash
# Start Tilt
tilt up

# In Tilt UI, click "trigger-test-job"
# (Currently TODO - implement test job script)

# Watch agent pods scale up:
kubectl get pods -n raibid-system -w

# Or use Tilt UI to watch "agent" resource
```

### Workflow 4: Local Development (No Containers)

For faster iteration without Docker overhead:

```bash
# Terminal 1: Run server locally
cd crates/server
cargo watch -x run

# Terminal 2: Run agent locally
cd crates/agent
cargo watch -x run

# Terminal 3: Run infrastructure in Tilt
# Comment out server/agent in Tiltfile
tilt up
```

## Live Reload (Issue #106)

**Status**: Not implemented

**Rationale**:
- Rust is compiled - requires full recompilation on changes
- cargo-chef provides optimal Docker layer caching
- Live reload would be SLOWER than full rebuild:
  - No Docker layer cache benefits
  - Container filesystem overhead
  - Need full build toolchain in runtime image

**Current approach**:
- Full Docker rebuild with cargo-chef caching
- Dependency layer cached (rebuilds only on Cargo.toml changes)
- Source changes trigger fast incremental builds
- Typical rebuild: 30-60 seconds

**Alternative**:
- Use `cargo watch -x run` for instant local development
- Use Tilt for full-stack integration testing

## Troubleshooting

### Tilt won't start - k3s not running

```bash
# Error: "k3s cluster is not running"

# Solution 1: Start k3s
sudo systemctl start k3s

# Solution 2: Install k3s
cd infra/k3s && sudo ./install.sh

# Solution 3: Use k3d
k3d cluster create raibid-ci
```

### Docker build failures

```bash
# Error: Docker build failed

# Check Docker is running
docker ps

# Check Docker daemon logs
sudo journalctl -u docker -f

# Try manual build
cd crates/server
docker build -f Dockerfile ../..
```

### Tanka deployment failures

```bash
# Error: "tk show" failed

# Check Tanka config
cd tanka
tk show environments/local

# Common issues:
# 1. Helm charts not vendored (see below)
# 2. Invalid jsonnet syntax
# 3. Missing CRDs
```

### Helm charts not vendored

```bash
# Error: Helm chart not found in vendor/

# Solution: Vendor Helm charts
cd tanka

# Add Helm repos
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add gitea-charts https://dl.gitea.io/charts/
helm repo add kedacore https://kedacore.github.io/charts
helm repo add fluxcd-community https://fluxcd-community.github.io/helm-charts

# Pull charts
helm pull bitnami/redis --untar -d vendor/
helm pull gitea-charts/gitea --untar -d vendor/
helm pull kedacore/keda --untar -d vendor/
helm pull fluxcd-community/flux2 --untar -d vendor/
```

### Port already in use

```bash
# Error: Port 8080 already in use

# Find process using port
sudo lsof -i :8080

# Kill process or change Tiltfile port forwards
```

### Resources not starting

```bash
# Check resource status in Tilt UI
# Look for error messages in logs

# Check Kubernetes resources
kubectl get all -n raibid-system

# Describe failing pods
kubectl describe pod <pod-name> -n raibid-system

# View pod logs
kubectl logs <pod-name> -n raibid-system
```

## Performance Optimization

### Docker Build Caching

The Dockerfiles use cargo-chef for optimal caching:

1. **Dependencies cached** in separate layer
2. **Source changes** only rebuild application layer
3. **Cargo.toml changes** trigger full rebuild

Expected build times:
- First build: 5-10 minutes (downloads all dependencies)
- Dependency change: 2-5 minutes (rebuilds dependency layer)
- Source change: 30-60 seconds (cached dependencies)

### Tilt Build Optimization

```python
# In Tiltfile:
update_settings(
    max_parallel_updates=2,  # Limit concurrent builds
)
```

Adjust based on your machine:
- DGX Spark (20 cores): `max_parallel_updates=4`
- Desktop (8 cores): `max_parallel_updates=2`
- Laptop (4 cores): `max_parallel_updates=1`

### Resource Limits

k3s configuration reserves resources for system and Kubernetes.

**Available for workloads** (on DGX Spark):
- CPU: 14 cores
- Memory: 104Gi
- Storage: ~3.8TB

See `infra/k3s/resource-quotas.yaml` for namespace quotas.

## Advanced Usage

### Custom Tanka Environment

```bash
# Create new environment
cd tanka
tk env add environments/dev --namespace=raibid-dev

# Edit Tiltfile to use new environment
TANKA_ENV = 'environments/dev'
```

### Selective Resource Deployment

```bash
# Edit Tiltfile to disable resources

# Comment out resources you don't need:
# k8s_resource(
#     workload='gitea',
#     ...
# )

# Restart Tilt
tilt down
tilt up
```

### Custom Docker Build Args

```python
# In Tiltfile, add build args:
docker_build(
    'raibid-server:latest',
    context=DOCKER_BUILD_CONTEXT,
    dockerfile='crates/server/Dockerfile',
    build_args={
        'RUST_VERSION': '1.82',
        'BUILD_MODE': 'debug',
    },
)
```

### Debugging

```bash
# View all Tilt output
tilt up --stream

# Disable auto-updates
tilt up --trigger-mode=manual

# View Tilt logs
tilt logs <resource-name>

# Get Tilt diagnostics
tilt dump
```

## CI Mode

Run Tilt in CI/headless mode:

```bash
# Run Tilt without UI
tilt ci

# Tilt will:
# 1. Build all images
# 2. Deploy all resources
# 3. Wait for resources to be ready
# 4. Exit with status code

# Use in CI pipelines:
tilt ci && tilt down
```

## Cleanup

### Stop Tilt

```bash
# In Tilt UI: Press Ctrl+C
# Or from CLI:
tilt down
```

This stops Tilt but **leaves resources running** in cluster.

### Delete Resources

```bash
# Delete all resources deployed by Tanka
kubectl delete namespace raibid-system

# Or delete specific resources
kubectl delete deployment raibid-server -n raibid-system
```

### Stop k3s

```bash
# Stop k3s service
sudo systemctl stop k3s

# Or delete k3d cluster
k3d cluster delete raibid-ci
```

### Uninstall k3s

```bash
# Complete k3s removal
sudo /usr/local/bin/k3s-uninstall.sh
```

## References

### Documentation

- [Tilt Documentation](https://docs.tilt.dev/)
- [Tanka Documentation](https://tanka.dev/)
- [k3s Documentation](https://docs.k3s.io/)
- [Docker Documentation](https://docs.docker.com/)

### Project Files

- `Tiltfile` - Main Tilt configuration
- `tanka/environments/local/main.jsonnet` - Tanka environment
- `crates/server/Dockerfile` - Server image build
- `crates/agent/Dockerfile` - Agent image build
- `infra/k3s/` - k3s installation and configuration

### Related Issues

- Issue #102: Create base Tiltfile with k3s management ✓
- Issue #103: Configure Docker image builds in Tiltfile ✓
- Issue #104: Integrate Tanka deployments in Tiltfile ✓
- Issue #105: Configure port forwards and shortcuts in Tiltfile ✓
- Issue #106: Configure live reload for Rust development ✓ (skipped)

## Support

For issues or questions:
- Review this documentation
- Check [Tilt documentation](https://docs.tilt.dev/)
- Review Tiltfile comments
- Open issue on GitHub
