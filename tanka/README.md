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

Install required tools:
```bash
# Tanka CLI
curl -Lo ~/.local/bin/tk https://github.com/grafana/tanka/releases/latest/download/tk-linux-arm64
chmod +x ~/.local/bin/tk

# jsonnet-bundler
curl -Lo ~/.local/bin/jb https://github.com/jsonnet-bundler/jsonnet-bundler/releases/latest/download/jb-linux-arm64
chmod +x ~/.local/bin/jb

# Add to PATH
export PATH="$HOME/.local/bin:$PATH"
```

### Common Commands

```bash
# Show generated manifests
tk show environments/local

# Diff against cluster
tk diff environments/local

# Apply to cluster
tk apply environments/local

# Export manifests to YAML
tk export manifests/ environments/local

# Validate jsonnet syntax
tk fmt --test

# Install/update dependencies
jb install
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

## References

- [Tanka Documentation](https://tanka.dev)
- [jsonnet Language Guide](https://jsonnet.org/)
- [Kubernetes API Reference](https://kubernetes.io/docs/reference/)
- [Helm Support in Tanka](https://tanka.dev/helm/)
