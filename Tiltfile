# Tiltfile for raibid-ci Development
# Orchestrates k3d cluster (k3s in Docker), Docker builds, and Tanka deployments
# Provides streamlined developer experience for DGX Spark CI development

# =============================================================================
# Configuration
# =============================================================================

# Project settings
PROJECT_NAME = 'raibid-ci'
NAMESPACE = 'raibid-system'

# Environment variables for mirroring
# These are read from your shell environment and passed to Tilt resources
GITHUB_TOKEN = os.environ.get('GITHUB_TOKEN', '')
GITEA_TOKEN = os.environ.get('GITEA_TOKEN', '')
WEBHOOK_SECRET = os.environ.get('WEBHOOK_SECRET', '')

# If GITEA_TOKEN is not set, try to read from provisioned token file
if not GITEA_TOKEN:
    # Check if token file exists and read it
    token_check = str(local('[ -f "$HOME/.config/raibid/gitea-token" ] && cat "$HOME/.config/raibid/gitea-token" || echo ""', quiet=True, echo_off=True)).strip()
    if token_check:
        GITEA_TOKEN = token_check
        print('✓ Using provisioned Gitea token from file')

# Print environment variable status (without revealing values)
if GITHUB_TOKEN:
    print('✓ GITHUB_TOKEN is set')
else:
    print('⚠ GITHUB_TOKEN not set - private repo access will be limited')

if GITEA_TOKEN:
    print('✓ GITEA_TOKEN is set')
else:
    print('⚠ GITEA_TOKEN not set - will use admin credentials')

# k3d cluster configuration
K3D_CLUSTER_NAME = 'raibid-ci'

# k3s configuration (legacy, kept for reference)
K3S_CONFIG_DIR = './infra/k3s'
K3S_INSTALL_SCRIPT = K3S_CONFIG_DIR + '/install.sh'
K3S_VALIDATE_SCRIPT = K3S_CONFIG_DIR + '/validate-installation.sh'

# Tanka configuration
TANKA_DIR = './tanka'
TANKA_ENV = 'environments/local'

# Docker build contexts
DOCKER_BUILD_CONTEXT = '.'

# =============================================================================
# Helper Functions
# =============================================================================

def check_command(cmd):
    """Check if a command exists in PATH"""
    result = local('command -v {} > /dev/null 2>&1 || echo "missing"'.format(cmd), quiet=True, echo_off=True)
    return str(result).strip() != "missing"

def is_k3s_running():
    """Check if k3s is running and healthy"""
    # Check if kubectl is available
    if not check_command('kubectl'):
        return False

    # Try to connect to cluster
    result = local('kubectl cluster-info > /dev/null 2>&1 || echo "not-running"', quiet=True, echo_off=True)
    return str(result).strip() != "not-running"

def get_k3s_context():
    """Get the current kubectl context"""
    if not check_command('kubectl'):
        return None
    result = local('kubectl config current-context 2>/dev/null || echo ""', quiet=True, echo_off=True)
    return str(result).strip() if result else None

def namespace_exists(namespace):
    """Check if a namespace exists"""
    result = local('kubectl get namespace {} > /dev/null 2>&1 || echo "missing"'.format(namespace), quiet=True, echo_off=True)
    return str(result).strip() != "missing"

def create_namespace(namespace):
    """Create a namespace if it doesn't exist"""
    if not namespace_exists(namespace):
        print('Creating namespace: {}'.format(namespace))
        local('kubectl create namespace {}'.format(namespace))

# =============================================================================
# k3d Cluster Management
# =============================================================================

def check_k3d_installed():
    """Check if k3d is installed"""
    return check_command('k3d')

def get_k3d_clusters():
    """Get list of k3d clusters"""
    if not check_k3d_installed():
        return []
    result = local('k3d cluster list --no-headers 2>/dev/null || echo ""', quiet=True, echo_off=True)
    clusters = str(result).strip().split('\n') if result else []
    return [c.split()[0] for c in clusters if c.strip()]

def k3d_cluster_exists(cluster_name):
    """Check if a specific k3d cluster exists"""
    clusters = get_k3d_clusters()
    return cluster_name in clusters

def k3d_cluster_running(cluster_name):
    """Check if k3d cluster is running"""
    if not k3d_cluster_exists(cluster_name):
        return False
    # Check if cluster nodes are running
    result = local('k3d cluster list {} --no-headers 2>/dev/null | grep -q "running" && echo "running" || echo "stopped"'.format(cluster_name), quiet=True, echo_off=True)
    return str(result).strip() == "running"

def create_k3d_cluster():
    """Create k3d cluster with raibid-ci configuration"""
    print('Creating k3d cluster: {}...'.format(K3D_CLUSTER_NAME))

    # Create cluster without port mappings
    # Port forwarding is handled by Tilt resource definitions
    # This avoids port conflicts with existing services
    cmd = [
        'k3d cluster create {}'.format(K3D_CLUSTER_NAME),
        '--agents 0',  # No agent nodes (single server for dev)
        '--wait',  # Wait for cluster to be ready
        '--timeout 5m',  # Timeout for cluster creation
    ]

    local(' '.join(cmd))
    print('✓ k3d cluster created successfully')

def start_k3d_cluster():
    """Start an existing k3d cluster"""
    print('Starting k3d cluster: {}...'.format(K3D_CLUSTER_NAME))
    local('k3d cluster start {}'.format(K3D_CLUSTER_NAME))
    print('✓ k3d cluster started')

def ensure_k3d_cluster():
    """Ensure k3d cluster is running and configured"""

    print('=' * 80)
    print('k3d Cluster Setup')
    print('=' * 80)

    # Check if k3d is installed
    if not check_k3d_installed():
        print('✗ k3d is not installed')
        print('')
        print('Please install k3d first:')
        print('  macOS:   brew install k3d')
        print('  Linux:   wget -q -O - https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash')
        print('  Windows: choco install k3d')
        print('')
        print('Or visit: https://k3d.io/stable/#installation')
        fail('k3d is not installed')

    print('✓ k3d is installed')

    # Check if cluster exists
    if not k3d_cluster_exists(K3D_CLUSTER_NAME):
        print('Cluster "{}" does not exist'.format(K3D_CLUSTER_NAME))
        create_k3d_cluster()
    else:
        print('✓ Cluster "{}" exists'.format(K3D_CLUSTER_NAME))

        # Check if cluster is running
        if not k3d_cluster_running(K3D_CLUSTER_NAME):
            print('Cluster is stopped')
            start_k3d_cluster()
        else:
            print('✓ Cluster is running')

    # Verify cluster health
    print('Validating cluster health...')
    context = get_k3s_context()
    print('  Context: {}'.format(context))

    # Check node status
    local('kubectl get nodes')
    print('✓ Cluster is healthy')

    # Ensure required namespace exists
    create_namespace(NAMESPACE)
    print('✓ Namespace "{}" ready'.format(NAMESPACE))

    # Check for required CRDs (KEDA, Flux)
    print('Checking for required CRDs...')
    keda_crd_check = local('kubectl get crd scaledjobs.keda.sh > /dev/null 2>&1 || echo "missing"', quiet=True, echo_off=True)
    flux_crd_check = local('kubectl get crd gitrepositories.source.toolkit.fluxcd.io > /dev/null 2>&1 || echo "missing"', quiet=True, echo_off=True)

    if str(keda_crd_check).strip() == "missing":
        print('  ⚠ KEDA CRDs not found (will be installed by Helm chart)')
    else:
        print('  ✓ KEDA CRDs found')

    if str(flux_crd_check).strip() == "missing":
        print('  ⚠ Flux CRDs not found (will be installed by Helm chart)')
    else:
        print('  ✓ Flux CRDs found')

    print('✓ k3d cluster setup complete')
    print('')

# Run cluster setup
ensure_k3d_cluster()

# =============================================================================
# Tilt UI Configuration
# =============================================================================

# Configure Tilt settings
update_settings(
    # Limit concurrent builds for resource management
    max_parallel_updates=2,
    # Suppress unused image warnings
    # raibid-agent is used by ScaledJob which doesn't appear as a k8s_resource in Tilt
    suppress_unused_image_warnings=['raibid-server', 'raibid-agent', 'raibid-agent:latest'],
)

# Set default kubectl context to k3s
allow_k8s_contexts(['default', 'k3s', 'k3d-raibid-ci'])

# =============================================================================
# Docker Image Builds (Issue #103)
# =============================================================================

print('=' * 80)
print('Docker Image Builds')
print('=' * 80)

# Server image build
print('Configuring raibid-server image build...')
docker_build(
    # Image name (matches Tanka deployment)
    'raibid-server:latest',

    # Build context (repository root for workspace builds)
    context=DOCKER_BUILD_CONTEXT,

    # Dockerfile path
    dockerfile='crates/server/Dockerfile',

    # Watch paths for live updates
    # Trigger rebuild when Rust source or dependencies change
    only=[
        'crates/server/',
        'crates/common/',
        'Cargo.toml',
        'Cargo.lock',
    ],

    # Build arguments (none needed for now)
    # build_args={},

    # Use BuildKit for better caching and parallel builds
    # Note: BuildKit is default in modern Docker
)
print('✓ raibid-server build configured')

# Agent image build
print('Configuring raibid-agent image build...')
docker_build(
    # Image name (matches Tanka deployment)
    'raibid-agent:latest',

    # Build context (repository root for workspace builds)
    context=DOCKER_BUILD_CONTEXT,

    # Dockerfile path
    dockerfile='crates/agent/Dockerfile',

    # Watch paths for live updates
    # Trigger rebuild when Rust source or dependencies change
    only=[
        'crates/agent/',
        'crates/common/',
        'Cargo.toml',
        'Cargo.lock',
    ],

    # Build arguments (none needed for now)
    # build_args={},

    # Use BuildKit for better caching and parallel builds
)
print('✓ raibid-agent build configured')

print('')
print('Docker builds will run in parallel (max 2 concurrent)')
print('Builds will trigger on source file changes')
print('')

# =============================================================================
# Tanka Deployments (Issue #104)
# =============================================================================

print('=' * 80)
print('Tanka Deployments')
print('=' * 80)

# Generate Kubernetes manifests from Tanka
print('Generating manifests from Tanka...')
# Write manifests to a temporary file instead of storing in memory
# This avoids "file name too long" errors when manifests are large
manifest_file = '/tmp/raibid-tanka-manifests.yaml'
local('cd {} && tk show {} --dangerous-allow-redirect > {}'.format(TANKA_DIR, TANKA_ENV, manifest_file))

# Apply manifests to cluster from file
print('Deploying manifests to cluster...')
k8s_yaml(manifest_file)

print('✓ Tanka manifests deployed')
print('')

# =============================================================================
# Resource Definitions and Dependencies (Issue #104)
# =============================================================================

print('Configuring resource groups and dependencies...')

# Infrastructure Group: Redis, Gitea, KEDA, Flux
# These are deployed via Helm charts through Tanka

# Redis - Job queue with Streams support
k8s_resource(
    workload='redis-master',
    new_name='redis',
    labels=['infrastructure'],
    port_forwards=['6379:6379'],  # Redis default port
)

# Gitea - Git server with OCI registry
k8s_resource(
    workload='gitea',
    new_name='gitea',
    labels=['infrastructure'],
    port_forwards=['3000:3000'],  # Gitea web UI
    links=[
        link('http://localhost:3000', 'Gitea Web UI'),
    ],
)

# KEDA - Event-driven autoscaling
k8s_resource(
    workload='keda-operator',
    new_name='keda',
    labels=['infrastructure'],
    port_forwards=[],  # No direct access needed
)

# Flux - GitOps continuous delivery (optional, may not have deployment)
# k8s_resource(
#     workload='flux',
#     new_name='flux',
#     labels=['infrastructure'],
# )

# Application Group: Server, Agent

# Server - API server
k8s_resource(
    workload='raibid-server',
    new_name='server',
    labels=['application'],
    port_forwards=[
        '8080:8080',  # HTTP API
        '8081:8081',  # Metrics endpoint
    ],
    links=[
        link('http://localhost:8080', 'Server API'),
        link('http://localhost:8081/metrics', 'Server Metrics'),
    ],
    # Dependencies: wait for Redis to be ready and image to be built
    resource_deps=['redis', 'raibid-server:latest'],
)

# Agent - Auto-scaling build agents (ScaledJob, not Deployment)
# Note: KEDA ScaledJobs don't create pods until there are jobs
# ScaledJob is managed by KEDA and doesn't appear as a workload in Tilt
# The agent image build and ScaledJob deployment are tracked separately
# k8s_resource(
#     objects=['raibid-agent:scaledjob:raibid-system'],
#     new_name='agent',
#     labels=['application'],
#     # Dependencies: wait for Server, KEDA, and image to be built
#     resource_deps=['server', 'keda', 'raibid-agent:latest'],
# )

print('✓ Resource groups configured:')
print('  - Infrastructure: redis, gitea, keda')
print('  - Application: server')
print('  - Images: raibid-server:latest, raibid-agent:latest')
print('✓ Dependencies configured:')
print('  - server depends on: redis, raibid-server:latest')
print('  - raibid-agent ScaledJob managed by KEDA (scales on job queue)')
print('')

# Watch Tanka files for changes and re-deploy
print('Watching Tanka files for changes...')
watch_file(TANKA_DIR + '/environments/local/main.jsonnet')
watch_file(TANKA_DIR + '/lib/raibid/')
watch_file(TANKA_DIR + '/lib/charts/')

print('✓ Auto-reload enabled for Tanka files')
print('')

# =============================================================================
# Repository Mirroring (Automatic on Startup)
# =============================================================================

print('=' * 80)
print('Repository Mirroring')
print('=' * 80)

# Provision Credentials Secret - Create k8s secret with GitHub and Gitea credentials
# This secret is used by mirroring operations
local('kubectl create secret generic raibid-credentials -n {} \
    --from-literal=github-token={} \
    --from-literal=gitea-admin-user=raibid-admin \
    --from-literal=gitea-admin-password=r8sA8CPHD9!bt6d \
    --dry-run=client -o yaml | kubectl apply -f -'.format(
        NAMESPACE,
        GITHUB_TOKEN if GITHUB_TOKEN else 'unset'
    ), quiet=True, echo_off=True)

print('✓ Credentials secret provisioned (raibid-credentials)')

# Repository mirroring is now handled by Kubernetes Job
# See: tanka/lib/raibid/mirror-setup-job.libsonnet
print('✓ Repository mirroring configured via Kubernetes Job')
print('  - Job: setup-gitea-mirrors')
print('  - Runs after Gitea is deployed and ready')
print('  - Uses raibid-credentials secret for GitHub token')
print('  - Fetches and mirrors all raibid-labs private repos')
print('  - Auto-sync every 8 hours via Gitea native mirroring')
print('')

# =============================================================================
# Manual Triggers and Shortcuts (Issue #105)
# =============================================================================

print('=' * 80)
print('Manual Triggers and Shortcuts')
print('=' * 80)

# Run Mirror Setup - Manually re-run the mirror setup job
local_resource(
    name='run-mirror-setup',
    cmd='''
#!/bin/bash
set -e

echo "Re-running mirror setup job..."
echo ""

# Delete existing job if it exists
kubectl delete job setup-gitea-mirrors -n {} 2>/dev/null || echo "No existing job to delete"

# Wait a moment for cleanup
sleep 2

# Recreate the job (Tanka will regenerate it)
cd ./tanka && tk apply environments/local --dangerous-auto-approve

echo ""
echo "✓ Mirror setup job recreated"
echo "Check logs: kubectl logs -n {} -l app.kubernetes.io/component=mirror-setup -f"
'''.format(NAMESPACE, NAMESPACE),
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
    resource_deps=['gitea'],
)

# Trigger Test Job - Send a test job to Redis queue (after mirroring)
local_resource(
    name='trigger-test-job',
    cmd='''
#!/bin/bash
set -e

echo "Triggering test build job..."
echo ""

# Get first mirrored repository from Gitea
REPO=$(curl -s http://localhost:3000/api/v1/user/repos | jq -r '.[0].full_name' 2>/dev/null)

if [ -z "$REPO" ] || [ "$REPO" = "null" ]; then
    echo "✗ No repositories found in Gitea"
    echo "Hint: Run 'run-mirror-setup' first to sync repositories"
    exit 1
fi

echo "Using repository: $REPO"

# Trigger job via Redis
redis-cli -h localhost -p 6379 XADD raibid:jobs '*' \
    job "{\"id\":\"test-$(date +%s)\",\"repo\":\"$REPO\",\"branch\":\"main\",\"commit\":\"HEAD\",\"status\":\"Pending\"}"

echo ""
echo "✓ Test job triggered for: $REPO"
echo "Check agent logs to see job execution"
''',
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
    resource_deps=['redis'],
)

# Scale Agent - Manually trigger agent scaling
local_resource(
    name='scale-agent',
    cmd='kubectl scale scaledjob raibid-agent --replicas=1 -n {}'.format(NAMESPACE),
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
    # No dependencies - can be triggered any time after Tanka deploys the ScaledJob
)

# View Server Logs - Quick access to server logs
local_resource(
    name='view-server-logs',
    cmd='kubectl logs -n {} -l app=raibid-server --tail=100 -f'.format(NAMESPACE),
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL,
    labels=['tools'],
    resource_deps=['server'],
)

print('✓ Manual triggers configured:')
print('  - run-mirror-setup: Re-run mirror setup job')
print('  - trigger-test-job: Send test job to Redis queue')
print('  - scale-agent: Manually trigger agent scaling')
print('  - view-server-logs: Quick log access')
print('')

# =============================================================================
# Port Forwards Summary (Issue #105)
# =============================================================================

print('Port forwards configured:')
print('  - Server API:     http://localhost:8080')
print('  - Server Metrics: http://localhost:8081/metrics')
print('  - Gitea Web UI:   http://localhost:3000')
print('  - Redis:          localhost:6379')
print('')

# =============================================================================
# Live Reload Configuration (Issue #106)
# =============================================================================

# Issue #106: Live reload for Rust development
#
# Decision: Live reload is NOT implemented for Rust builds.
#
# Rationale:
# 1. Rust is a compiled language - changes require full recompilation
# 2. cargo-chef already provides optimal layer caching in Dockerfiles
# 3. Live reload would require:
#    - Syncing source files into running container
#    - Running cargo build inside container
#    - Restarting the binary
# 4. This approach is SLOWER than full rebuild due to:
#    - No Docker layer caching benefits
#    - Container filesystem overhead
#    - Need to install full build toolchain in runtime image
# 5. cargo-chef approach is faster because:
#    - Dependency layer is cached (only rebuilds on Cargo.toml changes)
#    - Source changes trigger fast incremental builds
#    - Docker BuildKit provides parallel builds
#    - Runtime image stays minimal (no build toolchain)
#
# Recommendation: Use full Docker rebuilds with cargo-chef caching.
# Typical rebuild time after source change: 30-60 seconds (dependencies cached)
# This is acceptable for development workflow.
#
# Alternative for local development without containers:
# - Use `cargo watch -x run` directly on host for instant rebuilds
# - Use Tilt only for integration testing with full stack
#

print('=' * 80)
print('Live Reload (Issue #106)')
print('=' * 80)
print('Live reload: Skipped (not implemented)')
print('Reason: Full Docker rebuild with cargo-chef is faster than live reload')
print('Typical rebuild time: 30-60 seconds with cached dependencies')
print('')
print('For instant local development:')
print('  - Use "cargo watch -x run" directly on host')
print('  - Use Tilt for full-stack integration testing')
print('')

# =============================================================================
# Tilt UI Configuration
# =============================================================================

# Configure Tilt settings
update_settings(
    # Limit concurrent builds for resource management
    max_parallel_updates=2,
    # Suppress unused image warnings
    # raibid-agent is used by ScaledJob which doesn't appear as a k8s_resource in Tilt
    suppress_unused_image_warnings=['raibid-server', 'raibid-agent', 'raibid-agent:latest'],
)

# Set default kubectl context to k3s
allow_k8s_contexts(['default', 'k3s', 'k3d-raibid-ci'])

print('=' * 80)
print('Tilt Configuration Complete')
print('=' * 80)
print('Project: {}'.format(PROJECT_NAME))
print('Namespace: {}'.format(NAMESPACE))
print('Cluster: k3d-{}'.format(K3D_CLUSTER_NAME))
print('Context: {}'.format(get_k3s_context()))
print('')
print('Status:')
print('  ✓ k3d cluster management configured')
print('  ✓ Docker builds configured (server + agent)')
print('  ✓ Tanka deployments configured')
print('  ✓ Port forwards and shortcuts configured')
print('  ⚠ Live reload: Skipped (Issue #106 - full rebuild recommended)')
print('')
print('Run "tilt up" to start the development environment')
print('The k3d cluster will be automatically created if it doesn\'t exist')
print('=' * 80)
