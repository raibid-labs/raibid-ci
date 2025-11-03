# Tanka Configuration for raibid-ci

This directory contains Tanka (jsonnet + Kubernetes) configurations for deploying raibid-ci components.

## Directory Structure

```
tanka/
├── README.md                    # This file
├── jsonnetfile.json             # Dependency management
├── jsonnetfile.lock.json        # Locked dependency versions
│
├── environments/                # Environment-specific configs
│   └── local/                   # Local k3s development environment
│       ├── spec.json            # Environment specification
│       └── main.jsonnet         # Main entry point for local env
│
├── lib/                         # Reusable libraries
│   └── raibid/                  # raibid-specific libraries
│       ├── config.libsonnet     # Configuration helpers
│       ├── util.libsonnet       # Utility functions
│       └── helm.libsonnet       # Helm chart wrappers
│
├── components/                  # Component definitions
│   ├── infra/                   # Infrastructure components
│   │   ├── redis.jsonnet        # Redis with Streams
│   │   ├── gitea.jsonnet        # Gitea with OCI registry
│   │   ├── keda.jsonnet         # KEDA autoscaling
│   │   └── flux.jsonnet         # Flux GitOps
│   └── apps/                    # Application components
│       ├── server.jsonnet       # raibid-server deployment
│       ├── agent.jsonnet        # raibid-agent ScaledJob
│       └── config.jsonnet       # Secrets and ConfigMaps
│
└── vendor/                      # Vendored dependencies (auto-generated)
    ├── k8s-libsonnet/           # Kubernetes library
    ├── ksonnet-util/            # Ksonnet utilities
    ├── tanka-util/              # Tanka utilities (Helm support)
    └── doc-util/                # Documentation utilities
```

## Environment Configuration

### Local Environment (`environments/local`)

**Target**: Local k3s cluster at `https://127.0.0.1:6443`
**Namespace**: `raibid-system`
**Purpose**: Development and testing

Configured in `environments/local/spec.json`:
```json
{
  "apiServer": "https://127.0.0.1:6443",
  "namespace": "raibid-system"
}
```

## Usage

### Prerequisites

**Required Tools:**
- Tanka CLI (`tk`)
- jsonnet-bundler (`jb`)
- Helm (for chart vendoring)
- kubectl (for cluster access)

**Installation:**

```bash
# Tanka CLI (adjust for your architecture)
curl -Lo ~/.local/bin/tk https://github.com/grafana/tanka/releases/latest/download/tk-linux-arm64
chmod +x ~/.local/bin/tk

# jsonnet-bundler (adjust for your architecture)
curl -Lo ~/.local/bin/jb https://github.com/jsonnet-bundler/jsonnet-bundler/releases/latest/download/jb-linux-arm64
chmod +x ~/.local/bin/jb

# Helm (via Homebrew or package manager)
brew install helm
# Or: curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
```

### Setup: Vendor Helm Charts

**IMPORTANT**: Before using Tanka, you must vendor the Helm charts:

```bash
cd tanka

# Add Helm repositories
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add gitea-charts https://dl.gitea.io/charts/
helm repo add kedacore https://kedacore.github.io/charts
helm repo add fluxcd-community https://fluxcd-community.github.io/helm-charts
helm repo update

# Pull and extract charts to vendor/
helm pull bitnami/redis --version 18.19.4 --untar -d vendor/
helm pull gitea-charts/gitea --version 10.1.4 --untar -d vendor/
helm pull kedacore/keda --version 2.14.0 --untar -d vendor/
helm pull fluxcd-community/flux2 --version 2.12.4 --untar -d vendor/

# Verify vendored charts
ls -la vendor/
```

**Or use the justfile command:**
```bash
just tk-vendor  # From project root
```

### Common Commands

```bash
# Show generated manifests (interactive)
tk show environments/local

# Show with output redirect (for piping)
tk show environments/local --dangerous-allow-redirect

# Count generated resources
tk show environments/local --dangerous-allow-redirect | grep -c "^kind:"

# Diff against cluster (requires k3s)
tk diff environments/local

# Apply to cluster (requires k3s)
tk apply environments/local

# Export manifests to YAML files
tk export manifests/ environments/local

# Validate jsonnet syntax
tk fmt --test

# Install/update jsonnet dependencies
jb install
```

### Example Workflows

**1. Preview Generated YAML:**
```bash
cd tanka
tk show environments/local | less
```

**2. Validate Without Cluster:**
```bash
cd tanka
tk show environments/local --dangerous-allow-redirect > /tmp/output.yaml
echo "Generated $(grep -c '^kind:' /tmp/output.yaml) resources"
```

**3. Deploy to k3s:**
```bash
cd tanka
# Check what will change
tk diff environments/local

# Apply changes
tk apply environments/local
```

## Dependencies

Managed via `jsonnetfile.json`:
- **k8s-libsonnet** (1.29): Kubernetes API library
- **ksonnet-util**: Ksonnet utilities
- **tanka-util**: Tanka utilities with Helm support
- **doc-util**: Documentation utilities

Install/update with:
```bash
jb install
```

## Development Workflow

1. **Edit**: Modify jsonnet files in `lib/` or `components/`
2. **Show**: Preview with `tk show environments/local`
3. **Diff**: Check changes with `tk diff environments/local`
4. **Apply**: Deploy with `tk apply environments/local`

## Integration with Tilt

The Tanka configurations are orchestrated by Tilt for local development.
See `../Tiltfile` for the complete dev environment setup.

## Troubleshooting

### Error: "mapping key already defined" during helm.template

**Problem**: Duplicate labels when using `commonLabels` in Helm values.

**Cause**: `config.labels.forComponent()` and Helm charts both add the same labels.

**Solution**: Don't set `commonLabels` in chart wrappers. Let Helm use its own labels.

### Error: "Expected a comma before next field" in chart libsonnet files

**Problem**: Jsonnet syntax error in `new()` functions.

**Cause**: Trying to return a local variable from within an object literal.

**Solution**: Use direct return pattern:
```jsonnet
new(...)::
  local values = {...};
  helm.template(...)  // Direct return, no wrapping
```

### Error: "Helm chart not found" or "no such file or directory"

**Problem**: Helm charts not vendored to `vendor/` directory.

**Solution**: Run the Helm vendoring commands:
```bash
cd tanka
helm repo add bitnami https://charts.bitnami.com/bitnami
# ... (see "Setup: Vendor Helm Charts" above)
```

### Error: "Redirected output is discouraged"

**Problem**: Tanka blocks output redirection by default for safety.

**Solution**: Use `--dangerous-allow-redirect` flag:
```bash
tk show environments/local --dangerous-allow-redirect > output.yaml
```

Or set environment variable:
```bash
export TANKA_DANGEROUS_ALLOW_REDIRECT=true
tk show environments/local > output.yaml
```

### Traces export connection refused

**Symptom**: Warning message about "Post https://localhost:4318/v1/traces"

**Impact**: None - this is a harmless warning about telemetry.

**Solution**: Ignore this warning, or disable traces if desired.

## Testing and Validation

The Tanka configuration has been tested and generates:
- **90 Kubernetes resources**
- **23,485 lines of YAML**
- Includes: StatefulSets, Deployments, Services, CRDs, RBAC, ConfigMaps, Secrets

**Resource Types:**
- Infrastructure: Redis, Gitea, KEDA, Flux
- Applications: raibid-server, raibid-agent (ScaledJob)
- CRDs: CustomResourceDefinitions for KEDA and Flux

**Validation Commands:**
```bash
# Generate and count resources
cd tanka
tk show environments/local --dangerous-allow-redirect | grep -c "^kind:"
# Output: 90

# See resource breakdown
tk show environments/local --dangerous-allow-redirect | grep "^kind:" | sort | uniq -c
```

## References

- [Tanka Documentation](https://tanka.dev)
- [jsonnet Language Guide](https://jsonnet.org/)
- [Kubernetes API Reference](https://kubernetes.io/docs/reference/)
- [Helm Support in Tanka](https://tanka.dev/helm/)
- [Testing Report](../TESTING_REPORT.md) - Detailed testing results
