# Tilt Setup Checklist

Quick reference for setting up the Tilt development environment for raibid-ci.

## Prerequisites Checklist

Use this checklist to ensure all prerequisites are installed before running `tilt up`.

### ✓ Required Tools

- [ ] **k3s** - Lightweight Kubernetes cluster
  ```bash
  kubectl cluster-info
  # Should show: Kubernetes control plane is running at https://127.0.0.1:6443
  ```

- [ ] **Tilt** - Development orchestrator
  ```bash
  tilt version
  # Should show: v0.33.x or higher
  ```

- [ ] **Docker** - Container runtime
  ```bash
  docker ps
  # Should show: Running containers or empty list (no errors)
  ```

- [ ] **Tanka** - Kubernetes configuration tool
  ```bash
  tk --version
  # Should show: tk version vX.Y.Z
  ```

- [ ] **kubectl** - Kubernetes CLI
  ```bash
  kubectl version
  # Should show: Client and Server versions
  ```

### ✓ Helm Charts Vendored

- [ ] **Redis chart**
  ```bash
  ls -la tanka/vendor/redis
  # Should show: Chart.yaml and other chart files
  ```

- [ ] **Gitea chart**
  ```bash
  ls -la tanka/vendor/gitea
  # Should show: Chart.yaml and other chart files
  ```

- [ ] **KEDA chart**
  ```bash
  ls -la tanka/vendor/keda
  # Should show: Chart.yaml and other chart files
  ```

- [ ] **Flux chart**
  ```bash
  ls -la tanka/vendor/flux2
  # Should show: Chart.yaml and other chart files
  ```

## Installation Steps

### 1. Install k3s

```bash
# Navigate to k3s directory
cd /home/beengud/raibid-labs/raibid-ci/infra/k3s

# Run installation script
sudo ./install.sh

# Verify installation
./validate-installation.sh
```

**Expected result**: All validation tests pass, cluster is ready.

### 2. Install Tilt

```bash
# Linux (recommended)
curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

# Verify installation
tilt version
```

**Expected result**: Tilt version displayed (v0.33.x or higher).

### 3. Verify Docker

```bash
# Check Docker is running
docker ps

# If not running, start Docker
sudo systemctl start docker
```

**Expected result**: Docker daemon is running, `docker ps` shows no errors.

### 4. Install Tanka (if not installed)

```bash
# Download latest release for Linux ARM64
curl -fsSL https://github.com/grafana/tanka/releases/download/v0.25.0/tanka-linux-arm64 -o /tmp/tk
sudo install /tmp/tk /usr/local/bin/tk
rm /tmp/tk

# Verify installation
tk --version
```

**Expected result**: Tanka version displayed.

### 5. Vendor Helm Charts

```bash
# Navigate to Tanka directory
cd /home/beengud/raibid-labs/raibid-ci/tanka

# Add Helm repositories
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add gitea-charts https://dl.gitea.io/charts/
helm repo add kedacore https://kedacore.github.io/charts
helm repo add fluxcd-community https://fluxcd-community.github.io/helm-charts

# Update Helm repos
helm repo update

# Pull charts to vendor directory
helm pull bitnami/redis --untar -d vendor/
helm pull gitea-charts/gitea --untar -d vendor/
helm pull kedacore/keda --untar -d vendor/
helm pull fluxcd-community/flux2 --untar -d vendor/

# Verify charts are vendored
ls -la vendor/
```

**Expected result**: All four chart directories exist in `vendor/`.

### 6. Validate Tanka Configuration

```bash
# From Tanka directory
cd /home/beengud/raibid-labs/raibid-ci/tanka

# Test Tanka can generate manifests
tk show environments/local
```

**Expected result**: YAML manifests are printed to stdout without errors.

### 7. Start Tilt

```bash
# From project root
cd /home/beengud/raibid-labs/raibid-ci

# Start Tilt
tilt up
```

**Expected result**:
- Tilt UI opens in browser at http://localhost:10350
- All resources start building/deploying
- No errors in Tilt startup output

## Verification Steps

After `tilt up` completes, verify all components are running:

### ✓ Docker Images Built

```bash
# Check images exist
docker images | grep raibid
```

**Expected result**:
```
raibid-server    latest    ...    30-60 seconds ago    ...
raibid-agent     latest    ...    30-60 seconds ago    ...
```

### ✓ Resources Deployed

```bash
# Check all resources in namespace
kubectl get all -n raibid-system
```

**Expected result**:
- `deployment/raibid-server` - Running
- `deployment/redis-master` - Running
- `deployment/gitea` - Running
- `deployment/keda-operator` - Running
- `scaledjob/raibid-agent` - Created (may not have pods yet)

### ✓ Services Accessible

```bash
# Server API
curl http://localhost:8080/health
# Expected: {"status":"healthy"} or similar

# Server Metrics
curl http://localhost:8081/metrics
# Expected: Prometheus metrics output

# Gitea Web UI
curl http://localhost:3000
# Expected: HTML response (Gitea web page)

# Redis
redis-cli -h localhost -p 6379 ping
# Expected: PONG
```

### ✓ Tilt UI

Open http://localhost:10350 in browser.

**Expected result**:
- Green checkmarks for all resources
- No error logs
- Resource groups visible:
  - Infrastructure: redis, gitea, keda
  - Application: server, agent
  - Tools: manual triggers

## Troubleshooting

### Problem: k3s not running

```bash
# Check k3s service status
sudo systemctl status k3s

# If not active, start it
sudo systemctl start k3s

# Verify
kubectl cluster-info
```

### Problem: Helm charts not vendored

```bash
# Error: "chart not found in vendor/"

# Solution: Vendor charts (see step 5 above)
cd tanka
helm pull bitnami/redis --untar -d vendor/
helm pull gitea-charts/gitea --untar -d vendor/
helm pull kedacore/keda --untar -d vendor/
helm pull fluxcd-community/flux2 --untar -d vendor/
```

### Problem: Tanka fails to generate manifests

```bash
# Error: "tk show" failed

# Check Tanka syntax
cd tanka
tk show environments/local 2>&1 | less

# Common issues:
# 1. Missing chart in vendor/
# 2. Syntax error in jsonnet files
# 3. Wrong API server URL in spec.json
```

### Problem: Docker build fails

```bash
# Check Docker is running
docker ps

# Check Docker logs
sudo journalctl -u docker -f

# Try manual build
cd crates/server
docker build -f Dockerfile ../..
```

### Problem: Ports already in use

```bash
# Find what's using the port
sudo lsof -i :8080

# Kill the process or change Tiltfile port forwards
```

### Problem: Resources stuck in "Pending"

```bash
# Check pod events
kubectl describe pod <pod-name> -n raibid-system

# Common issues:
# 1. Insufficient resources (check k3s quotas)
# 2. Image pull errors (check Docker builds)
# 3. Volume mount issues (check PVCs)
```

## Quick Reference

### Start Development

```bash
cd /home/beengud/raibid-labs/raibid-ci
tilt up
```

### Stop Development

```bash
# In Tilt UI: Press Ctrl+C
# Or: tilt down
```

### Rebuild Everything

```bash
tilt down
tilt up
```

### View Logs

```bash
# In Tilt UI: Click on resource name
# Or: tilt logs <resource-name>
# Or: kubectl logs -n raibid-system -l app=raibid-server
```

### Clean Slate

```bash
# Stop Tilt
tilt down

# Delete namespace (removes all resources)
kubectl delete namespace raibid-system

# Recreate namespace
kubectl create namespace raibid-system

# Restart Tilt
tilt up
```

## Environment-Specific Notes

### DGX Spark

- **Architecture**: ARM64 (aarch64)
- **Cores**: 20 (14 available for workloads)
- **Memory**: 128GB (104GB available for workloads)
- **k3s**: Use native installation (not k3d)
- **Build speed**: Expect fast builds due to high core count

### Local Development (x86_64)

- **Docker images**: May need multi-arch builds
- **k3s alternative**: Consider using k3d
  ```bash
  k3d cluster create raibid-ci
  ```
- **Build speed**: Slower than DGX Spark

## Next Steps

After Tilt is running successfully:

1. **Explore Tilt UI**: http://localhost:10350
2. **Access services**:
   - Server API: http://localhost:8080
   - Gitea: http://localhost:3000
3. **Make code changes**: Edit files in `crates/` and watch Tilt rebuild
4. **Test agent scaling**: Click "trigger-test-job" in Tilt UI (when implemented)
5. **Review logs**: Click on resources in Tilt UI to view logs

## Documentation

- [Full Tilt Guide](../TILT.md)
- [k3s Setup](../infra/k3s/INSTALLATION.md)
- [Tanka Documentation](https://tanka.dev/)
- [Tilt Documentation](https://docs.tilt.dev/)

## Support

For issues or questions:
- Review [TILT.md](../TILT.md) documentation
- Check Tiltfile comments
- Review logs in Tilt UI
- Open issue on GitHub
