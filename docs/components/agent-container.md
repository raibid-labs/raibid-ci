# CI Agent Container Image

## Overview

The raibid-ci agent container is an ephemeral, ARM64-optimized build environment designed for executing Rust CI/CD workflows on NVIDIA DGX Spark hardware.

## Architecture

### Multi-Stage Build

The Dockerfile uses a multi-stage build approach to minimize final image size:

```
┌─────────────────────────────────────────────┐
│ Stage 1: Base (rust:1.82-bookworm)         │
│ - System dependencies                       │
│ - Docker CLI installation                   │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ Stage 2: Builder (extends base)            │
│ - Install cargo tools (nextest, audit,     │
│   deny)                                     │
│ - Build and cache binaries                 │
└─────────────────┬───────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────┐
│ Stage 3: Runtime (extends base)            │
│ - Copy cargo tool binaries from builder    │
│ - Configure non-root user                  │
│ - Set up workspace and permissions         │
│ - Add health check                         │
└─────────────────────────────────────────────┘
```

### Size Optimization

Target: **< 1.5 GB (1536 MB)**

Optimization strategies:
1. Multi-stage build eliminates builder artifacts
2. Cleanup of apt caches and package lists
3. Minimal base image (Debian Bookworm)
4. Copy only necessary binaries from builder stage
5. No development dependencies in final image

## Installed Components

### Base System

| Component | Version | Purpose |
|-----------|---------|---------|
| Debian | 12 (Bookworm) | Stable base OS |
| Git | 2.39+ | Source control |
| Git LFS | Latest | Large file support |
| OpenSSH | Latest | Secure communications |
| Docker CLI | Latest | Container image builds |

### Rust Toolchain

| Component | Version | Purpose |
|-----------|---------|---------|
| Rust | 1.82 (stable) | Compiler |
| Cargo | Latest | Build system |
| Rustfmt | Latest | Code formatting |
| Clippy | Latest | Linting |

### Cargo Tools

| Tool | Version | Purpose |
|------|---------|---------|
| cargo-nextest | 0.9.72 | Advanced test runner with better output |
| cargo-audit | 0.20.0 | Security vulnerability scanning |
| cargo-deny | 0.14.24 | License and dependency validation |

## Configuration

### User and Permissions

- **User**: `agent` (non-root)
- **UID/GID**: 1000/1000
- **Home**: `/home/agent`
- **Workspace**: `/workspace`

### Environment Variables

```bash
RUST_BACKTRACE=1                    # Enable detailed error traces
CARGO_HOME=/home/agent/.cargo       # Cargo cache directory
CARGO_TARGET_DIR=/workspace/target  # Build output directory
CARGO_INCREMENTAL=1                 # Enable incremental compilation
```

### Volumes (Recommended)

```yaml
volumes:
  - name: cargo-cache
    mountPath: /home/agent/.cargo
    # Purpose: Persist Cargo registry and build cache
    # Size: 10-50 GB recommended

  - name: workspace
    mountPath: /workspace
    # Purpose: Build workspace and target directory
    # Type: emptyDir or PVC

  - name: docker-socket
    mountPath: /var/run/docker.sock
    # Purpose: Docker-in-Docker for image builds
    # Optional: Required only for building container images
```

## Build Process

### Local Build

```bash
cd crates/agent

# Basic build
./build.sh

# Custom configuration
./build.sh \
  --name raibid-ci-agent \
  --tag v1.0.0 \
  --registry localhost:5000 \
  --platform linux/arm64
```

### Using Makefile

```bash
cd crates/agent

# Show available targets
make help

# Build locally
make build

# Build and push
make build-push REGISTRY=gitea.local:3000/raibid

# Run tests
make test

# Verify installation
make verify
```

### CI/CD Build

The GitHub Actions workflow automatically builds and tests the image:

- **Pull Requests**: Build and test on AMD64
- **Main Branch**: Build AMD64 and ARM64, push to GHCR
- **Cache**: Uses BuildKit registry cache for fast rebuilds

## BuildKit Caching

### Local Filesystem Cache

```bash
docker buildx build \
  --cache-from type=local,src=/tmp/buildkit-cache \
  --cache-to type=local,dest=/tmp/buildkit-cache,mode=max \
  .
```

### Registry Cache (Recommended)

```bash
./build.sh \
  --cache-from gitea.local:3000/raibid/cache:agent \
  --cache-to gitea.local:3000/raibid/cache:agent \
  --push
```

**Benefits**:
- Shared cache across CI runs
- Faster builds (reuse layers)
- Reduced network transfer
- Better cache hit rates

## Health Check

The container includes a comprehensive health check script that validates:

### System Tools
- ✓ Git, SSH, Docker availability
- ✓ Command execution

### Rust Toolchain
- ✓ rustc, cargo, rustfmt, clippy presence
- ✓ Version compatibility

### Cargo Tools
- ✓ nextest, audit, deny installation
- ✓ Executable permissions

### Environment
- ✓ Environment variables set correctly
- ✓ Filesystem permissions (writable workspace)
- ✓ Git configuration

### Health Check Execution

```bash
# Manual execution
docker run --rm raibid-ci-agent:latest /usr/local/bin/healthcheck.sh

# Kubernetes probe
livenessProbe:
  exec:
    command:
      - /usr/local/bin/healthcheck.sh
  initialDelaySeconds: 5
  periodSeconds: 30
  timeoutSeconds: 10
  failureThreshold: 3
```

## Testing

### Test Suite

The `test.sh` script provides comprehensive validation:

```bash
cd crates/agent
./test.sh
```

**Test Categories**:
1. Image metadata (architecture, size, labels)
2. System tools (git, docker, ssh)
3. Rust toolchain (rustc, cargo, rustfmt, clippy)
4. Cargo tools (nextest, audit, deny)
5. Filesystem permissions
6. Environment variables
7. Health check execution
8. Cargo functionality (build, test)

### Example Test Output

```
=== raibid-ci Agent Container Test Suite ===

Testing image metadata...
Testing: Image architecture ... PASS
Testing: Image size constraint ... PASS
  Size: 1234.56 MB (target: < 1536 MB)
Testing: Image labels ... PASS

Testing system tools...
Testing: git command ... PASS
Testing: docker command ... PASS
Testing: ssh command ... PASS

...

Test Summary
==========================================
Total:  25
Passed: 25
Failed: 0
==========================================
All tests passed!
```

## Kubernetes Deployment

### Pod Specification

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: raibid-agent-rust
  labels:
    app: raibid-agent
    type: rust
spec:
  containers:
  - name: agent
    image: gitea.local:3000/raibid/raibid-ci-agent:latest
    imagePullPolicy: Always

    resources:
      requests:
        cpu: "1000m"
        memory: "2Gi"
      limits:
        cpu: "2000m"
        memory: "4Gi"

    env:
    - name: REDIS_URL
      value: "redis://redis-master:6379"
    - name: GITEA_URL
      value: "https://gitea.local:3000"
    - name: GITEA_TOKEN
      valueFrom:
        secretKeyRef:
          name: gitea-credentials
          key: token

    volumeMounts:
    - name: cargo-cache
      mountPath: /home/agent/.cargo
    - name: workspace
      mountPath: /workspace
    - name: docker-socket
      mountPath: /var/run/docker.sock

    livenessProbe:
      exec:
        command:
          - /usr/local/bin/healthcheck.sh
      initialDelaySeconds: 5
      periodSeconds: 30

    readinessProbe:
      exec:
        command:
          - /usr/local/bin/healthcheck.sh
      initialDelaySeconds: 3
      periodSeconds: 10

  volumes:
  - name: cargo-cache
    persistentVolumeClaim:
      claimName: agent-cargo-cache
  - name: workspace
    emptyDir: {}
  - name: docker-socket
    hostPath:
      path: /var/run/docker.sock
      type: Socket
```

### KEDA ScaledJob

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledJob
metadata:
  name: raibid-agent-rust
spec:
  jobTargetRef:
    template:
      spec:
        containers:
        - name: agent
          image: gitea.local:3000/raibid/raibid-ci-agent:latest
          command: ["/usr/local/bin/raibid-agent"]
          # ... same configuration as above
        restartPolicy: OnFailure

  pollingInterval: 10
  maxReplicaCount: 10
  minReplicaCount: 0

  triggers:
  - type: redis-streams
    metadata:
      addressFromEnv: REDIS_URL
      stream: ci-jobs
      consumerGroup: ci-workers
      pendingEntriesCount: "1"
```

## Performance Considerations

### Build Cache Strategy

1. **Persistent Volume for Cargo Cache**
   - Size: 10-50 GB per agent type
   - Retention: 7-30 days
   - Shared: Across pods of same type

2. **Target Directory**
   - Use emptyDir or PVC
   - Clean between jobs for isolation
   - Size: 5-10 GB

3. **Docker BuildKit Cache**
   - Use registry cache type
   - Mode: `max` for full layer caching
   - Prune: Weekly or size-based

### Resource Allocation

**Recommended**:
- CPU: 2 cores (request: 1, limit: 2)
- Memory: 4 GB (request: 2 GB, limit: 4 GB)
- Ephemeral Storage: 20 GB

**Justification**:
- Rust compilation is CPU-intensive
- Linking requires memory
- Dependency downloads need storage

### Startup Time

**Cold Start** (no cache): ~60 seconds
- Image pull: 20-30s
- Container start: 5-10s
- Cache population: 30-40s

**Warm Start** (with cache): ~15 seconds
- Image pull (cached): 2-5s
- Container start: 5-10s
- Cache hit: <1s

## Troubleshooting

### Image Too Large

```bash
# Check layer sizes
docker history raibid-ci-agent:latest

# Common causes:
# 1. Apt cache not cleaned
# 2. Cargo registry in final image
# 3. Debug symbols not stripped
```

**Solution**: Verify multi-stage build and cleanup commands

### Permission Errors

```bash
# Container runs as UID 1000
# Ensure volumes have correct permissions

# Fix PVC permissions
kubectl exec -it pod-name -- chown -R agent:agent /home/agent/.cargo
```

### Build Failures

```bash
# Enable debug output
docker buildx build --progress=plain .

# Check BuildKit logs
docker buildx ls
docker buildx inspect --bootstrap
```

### Health Check Failing

```bash
# Run manually for detailed output
docker run --rm raibid-ci-agent:latest /usr/local/bin/healthcheck.sh

# Common issues:
# 1. Missing tools (check Dockerfile)
# 2. Permission problems (check user/group)
# 3. Environment variables (check ENV directives)
```

## Security

### Non-Root Execution

Container runs as user `agent` (UID 1000) for security:
- No privilege escalation
- Limited filesystem access
- Reduced attack surface

### Docker Socket Access

Mounting Docker socket requires careful consideration:

**Risks**:
- Equivalent to root access on host
- Container escape possible
- Shared daemon state

**Alternatives**:
- Docker-in-Docker (DinD) sidecar
- Kaniko for Dockerfile builds
- Buildah/Podman (rootless)

**Recommendation**: Use DinD sidecar for isolation

### Secrets Management

Never include secrets in image:
- Use Kubernetes Secrets for credentials
- Mount as environment variables or files
- Rotate regularly

## Future Enhancements

### Planned Features

1. **sccache Integration**
   - Distributed compilation cache
   - Faster rebuilds across agents
   - Shared cache in Redis

2. **Additional Languages**
   - Go agent variant
   - Node.js agent variant
   - Python agent variant

3. **GPU Support**
   - CUDA toolkit for ML builds
   - GPU time-slicing
   - Accelerated test execution

4. **Advanced Caching**
   - Incremental test execution
   - Dependency graph analysis
   - Predictive cache warming

## References

- [Dockerfile](../../crates/agent/Dockerfile)
- [Build Script](../../crates/agent/build.sh)
- [Test Suite](../../crates/agent/test.sh)
- [Health Check](../../crates/agent/healthcheck.sh)
- [GitHub Actions Workflow](../../.github/workflows/agent-container.yml)
