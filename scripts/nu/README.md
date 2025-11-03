# Raibid-CI Nushell Scripts

Nushell scripts and modules for raibid-ci development automation.

## Directory Structure

```
scripts/nu/
├── README.md           # This file
├── config.nu           # Project configuration and helper functions
├── env.nu             # Environment variables and PATH setup
├── modules/           # Reusable Nushell modules
│   ├── kubectl.nu     # Kubernetes/k3s utilities
│   ├── redis.nu       # Redis and Redis Streams utilities
│   └── gitea.nu       # Gitea API utilities
└── examples/          # Example scripts demonstrating module usage
    ├── check-cluster.nu   # k3s cluster health check
    ├── check-redis.nu     # Redis health check
    ├── check-gitea.nu     # Gitea health check
    └── dev-workflow.nu    # Development workflow automation
```

## Quick Start

### 1. Install Nushell

```bash
# Via Homebrew (recommended)
brew install nushell

# Or via cargo
cargo install nu

# Verify installation
nu --version  # Should be 0.96 or later
```

### 2. Load Project Environment

```bash
# Start Nushell
nu

# Load environment and config
source scripts/nu/env.nu
source scripts/nu/config.nu

# Check project status
project-status
```

### 3. Run Examples

```bash
# Check k3s cluster
nu scripts/nu/examples/check-cluster.nu

# Check Redis
nu scripts/nu/examples/check-redis.nu

# Check Gitea
nu scripts/nu/examples/check-gitea.nu

# Development workflow
nu scripts/nu/examples/dev-workflow.nu --check-all
nu scripts/nu/examples/dev-workflow.nu --build
nu scripts/nu/examples/dev-workflow.nu --test
```

## Core Files

### env.nu

Sets up environment variables for the project:

- `RAIBID_ROOT` - Project root directory
- `RAIBID_SCRIPTS` - Scripts directory path
- `RAIBID_MODULES` - Nushell modules path
- `RAIBID_K3S_CONFIG` - Kubernetes config path
- `RAIBID_GITEA_URL` - Gitea instance URL
- `RAIBID_REDIS_URL` - Redis connection URL

**Usage:**

```nu
source scripts/nu/env.nu
echo $env.RAIBID_ROOT
```

### config.nu

Provides configuration and helper functions:

**Functions:**

- `project-info` - Get project metadata
- `project-status` - Show current project status
- `log-success`, `log-error`, `log-warning`, `log-info` - Colored logging
- `command-exists` - Check if command is available
- `check-dev-prerequisites` - Validate development tools
- `setup-paths` - Initialize project paths
- `setup-module-path` - Configure module loading

**Usage:**

```nu
source scripts/nu/config.nu
log-success "Operation completed"
check-dev-prerequisites
```

## Modules

### kubectl.nu

Kubernetes/k3s cluster management utilities.

**Key Functions:**

```nu
use modules/kubectl.nu *

kubectl-check-cluster          # Check cluster connectivity
kubectl-get-nodes              # List cluster nodes
kubectl-get-pods "namespace"   # Get pods in namespace
kubectl-get-services "ns"      # Get services in namespace
kubectl-ensure-namespace "ns"  # Create namespace if missing
kubectl-logs "pod" "ns"        # View pod logs
kubectl-apply "manifest.yaml"  # Apply Kubernetes manifest
```

[Full documentation](../../docs/guides/nushell.md#kubectl-module)

### redis.nu

Redis and Redis Streams utilities.

**Key Functions:**

```nu
use modules/redis.nu *

redis-check-connection         # Check Redis connectivity
redis-ping                     # Ping Redis server
redis-get "key"               # Get key value
redis-set "key" "value"       # Set key-value
redis-stream-add "stream" {}  # Add to stream
redis-stream-read "stream"    # Read from stream
redis-stream-create-group     # Create consumer group
```

[Full documentation](../../docs/guides/nushell.md#redis-module)

### gitea.nu

Gitea API utilities.

**Key Functions:**

```nu
use modules/gitea.nu *

gitea-check-connection         # Check Gitea connectivity
gitea-version                  # Get Gitea version
gitea-list-repos              # List repositories
gitea-create-repo "name"      # Create repository
gitea-mirror-repo "url" "name" # Mirror repository
gitea-list-branches "owner" "repo"
gitea-create-webhook "owner" "repo" "url"
```

[Full documentation](../../docs/guides/nushell.md#gitea-module)

## Examples

### check-cluster.nu

Comprehensive k3s cluster health check.

**Features:**

- Cluster connectivity validation
- Node status listing
- Pod inventory across namespaces
- Service discovery

**Usage:**

```bash
nu scripts/nu/examples/check-cluster.nu
```

### check-redis.nu

Redis instance health check.

**Features:**

- Connection validation
- Memory usage reporting
- Server information
- Stream inspection

**Usage:**

```bash
nu scripts/nu/examples/check-redis.nu
```

### check-gitea.nu

Gitea instance health check.

**Features:**

- Connection validation
- Version information
- User authentication check
- Repository listing

**Usage:**

```bash
# Set token for authenticated checks
export GITEA_TOKEN="your-token"
nu scripts/nu/examples/check-gitea.nu
```

### dev-workflow.nu

Development workflow automation.

**Features:**

- Prerequisite checking
- Infrastructure validation
- Project building
- Test execution
- Status reporting

**Usage:**

```bash
nu scripts/nu/examples/dev-workflow.nu --status
nu scripts/nu/examples/dev-workflow.nu --check-all
nu scripts/nu/examples/dev-workflow.nu --build
nu scripts/nu/examples/dev-workflow.nu --test
```

## Writing Scripts

### Basic Template

```nu
#!/usr/bin/env nu
# Script description

# Source environment and config
source scripts/nu/env.nu
source scripts/nu/config.nu

# Import modules
use modules/kubectl.nu *

def main [] {
    log-info "Starting script..."

    # Your logic here

    log-success "Script complete!"
}

main
```

### Using Modules

```nu
#!/usr/bin/env nu
source scripts/nu/env.nu

# Import specific functions
use modules/kubectl.nu [kubectl-get-pods kubectl-get-nodes]

# Or import all
use modules/redis.nu *

def main [] {
    # Use imported functions
    let pods = (kubectl-get-pods "default")
    let ping = (redis-ping)

    print $pods
    print $ping
}

main
```

### Error Handling

```nu
def safe-operation [] {
    try {
        kubectl-get-pods "raibid-ci"
    } catch {
        log-warning "Namespace doesn't exist, creating..."
        kubectl-ensure-namespace "raibid-ci"
        kubectl-get-pods "raibid-ci"
    }
}
```

## Best Practices

1. **Always source environment files**

   ```nu
   source scripts/nu/env.nu
   source scripts/nu/config.nu
   ```

2. **Use modules for reusable code**

   ```nu
   use modules/kubectl.nu *
   ```

3. **Use logging functions**

   ```nu
   log-info "Message"
   log-success "Success"
   log-error "Error"
   log-warning "Warning"
   ```

4. **Handle errors gracefully**

   ```nu
   try { ... } catch { ... }
   ```

5. **Make scripts executable**

   ```bash
   chmod +x scripts/nu/examples/my-script.nu
   ```

6. **Use shebang for direct execution**

   ```nu
   #!/usr/bin/env nu
   ```

## Environment Variables

Set these for full functionality:

```bash
# Gitea authentication
export GITEA_TOKEN="your-gitea-token"

# Custom URLs (optional, defaults provided)
export RAIBID_GITEA_URL="http://localhost:3000"
export RAIBID_REDIS_URL="redis://localhost:6379"
export RAIBID_K3S_CONFIG="$HOME/.kube/config"
```

## Testing Scripts

Test scripts without running:

```bash
# Check syntax
nu --commands "source scripts/nu/examples/check-cluster.nu"

# Dry run with debugging
nu --log-level debug scripts/nu/examples/check-cluster.nu
```

## CI Integration

Scripts are validated in CI. See [CI Configuration](../../.github/workflows/scripts.yml).

## Troubleshooting

### Module Not Found

Ensure modules directory is in `NU_LIB_DIRS`:

```nu
source scripts/nu/config.nu  # This sets up module path
$env.NU_LIB_DIRS  # Verify path is included
```

### Command Not Found

Check if required tools are installed:

```nu
command-exists "kubectl"
command-exists "redis-cli"
command-exists "curl"
```

### Version Issues

Ensure Nushell 0.96 or later:

```bash
nu --version
```

## Additional Resources

- [Nushell Guide](../../docs/guides/nushell.md) - Comprehensive guide
- [Nushell Official Docs](https://www.nushell.sh/)
- [Examples Directory](./examples/) - More examples
- [Module Source](./modules/) - Module implementations

## Contributing

When adding new scripts or modules:

1. Follow existing patterns
2. Add documentation comments
3. Include usage examples
4. Update this README
5. Test with CI validation
6. Add to examples if appropriate
