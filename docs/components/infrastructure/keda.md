# KEDA (Kubernetes Event-Driven Autoscaling) Installation Guide

## Overview

This document describes the KEDA installation implementation for the raibid-cli project. KEDA enables event-driven autoscaling of CI agents based on Redis Streams queue depth.

## Implementation Details

### Module Location

- **Source**: `src/infrastructure/keda.rs`
- **Integration**: `src/commands/setup.rs`

### Key Components

#### 1. KedaInstaller

The main installer struct that handles KEDA deployment via Helm.

**Configuration Options:**
- `chart_version`: Optional specific KEDA Helm chart version (defaults to latest)
- `namespace`: Kubernetes namespace for KEDA (default: "keda")
- `release_name`: Helm release name (default: "raibid-keda")
- `log_level`: KEDA operator log level (default: "info")
- `metrics_server_enabled`: Enable KEDA metrics server (default: true)
- `scaled_object`: Optional ScaledObject configuration for autoscaling

#### 2. ScaledObjectConfig

Configuration for the ScaledObject CRD that defines autoscaling behavior.

**Default Configuration:**
- **Name**: `raibid-ci-agent-scaler`
- **Namespace**: `raibid-ci`
- **Stream**: `raibid:jobs`
- **Consumer Group**: `raibid-workers`
- **Min Replicas**: 0 (scale-to-zero)
- **Max Replicas**: 10
- **Polling Interval**: 10 seconds
- **Pending Entries Count**: "1" (trigger scaling when 1+ job pending)

### Installation Workflow

The `install()` method performs the following steps:

1. **Pre-flight Checks**
   - Verify kubectl is available
   - Verify Helm 3.x is installed

2. **Helm Repository Setup**
   - Add kedacore Helm repository (`https://kedacore.github.io/charts`)
   - Update repository index

3. **Namespace Creation**
   - Create `keda` namespace for operator

4. **KEDA Deployment**
   - Generate Helm values with configured settings
   - Deploy KEDA via `helm upgrade --install`
   - Wait for deployment (5-minute timeout)

5. **Readiness Wait**
   - Wait for operator pods to be ready
   - Wait for metrics server pods to be ready (if enabled)

6. **Validation**
   - Verify CRDs are installed:
     - `scaledobjects.keda.sh`
     - `scaledjobs.keda.sh`
     - `triggerauthentications.keda.sh`
   - Verify operator deployment exists

7. **ScaledObject Creation**
   - Create target namespace (`raibid-ci`)
   - Generate ScaledObject YAML manifest
   - Apply ScaledObject for Redis Streams autoscaling

### Redis Streams Integration

The ScaledObject monitors Redis Streams with the following configuration:

```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: raibid-ci-agent-scaler
  namespace: raibid-ci
spec:
  scaleTargetRef:
    name: raibid-ci-agent
    kind: Deployment
    apiVersion: apps/v1
  pollingInterval: 10
  minReplicaCount: 0
  maxReplicaCount: 10
  triggers:
  - type: redis-streams
    metadata:
      address: raibid-redis-master.raibid-redis.svc.cluster.local:6379
      stream: raibid:jobs
      consumerGroup: raibid-workers
      pendingEntriesCount: "1"
      lagCount: "5"
```

### Scaling Behavior

- **Scale Up**: When jobs are added to the `raibid:jobs` stream, KEDA detects pending entries and scales up the deployment
- **Scale Down**: When the queue is empty (no pending entries), KEDA scales down to 0 replicas after a cooldown period
- **Polling**: KEDA checks Redis every 10 seconds for queue depth changes

## Usage

### CLI Command

```bash
# Install KEDA
raibid-cli setup keda

# Install all components including KEDA
raibid-cli setup all
```

### Programmatic Usage

```rust
use raibid_cli::infrastructure::KedaInstaller;

// Default configuration
let installer = KedaInstaller::new()?;
installer.install()?;

// Custom configuration
use raibid_cli::infrastructure::{KedaConfig, ScaledObjectConfig};

let mut config = KedaConfig::default();
config.log_level = "debug".to_string();

let mut scaled_object = ScaledObjectConfig::default();
scaled_object.max_replica_count = 20;
config.scaled_object = Some(scaled_object);

let installer = KedaInstaller::with_config(config)?;
installer.install()?;
```

### Verification

After installation, verify KEDA is running:

```bash
# Check KEDA pods
kubectl get pods -n keda

# Expected output:
# NAME                                      READY   STATUS    RESTARTS   AGE
# keda-operator-xxxxxxxxxx-xxxxx            1/1     Running   0          2m
# keda-metrics-apiserver-xxxxxxxxxx-xxxxx   1/1     Running   0          2m
# keda-admission-webhooks-xxxxxxxxxx-xxxxx  1/1     Running   0          2m

# Check CRDs
kubectl get crd | grep keda

# Expected output:
# scaledobjects.keda.sh
# scaledjobs.keda.sh
# triggerauthentications.keda.sh
# clustertriggerauthentications.keda.sh

# Check ScaledObject
kubectl get scaledobject -n raibid-ci

# Expected output:
# NAME                       SCALETARGETKIND      SCALETARGETNAME      MIN   MAX   TRIGGERS   AGE
# raibid-ci-agent-scaler     apps/v1.Deployment   raibid-ci-agent      0     10    1          1m
```

### Test Autoscaling

Add a test job to Redis to trigger scaling:

```bash
# Port-forward to Redis
kubectl port-forward -n raibid-redis svc/raibid-redis-master 6379:6379

# In another terminal, add a test job
redis-cli XADD raibid:jobs * job_id test-001 repo raibid/test branch main

# Watch deployments scale
kubectl get deployment -n raibid-ci -w

# You should see the deployment scale from 0 to 1
```

## Error Handling

The installer implements comprehensive error handling:

- **Rollback**: On failure, `uninstall()` is called to clean up partial installations
- **Idempotency**: Repeated installations are safe (checks for existing resources)
- **Validation**: Post-installation validation ensures all components are healthy
- **Timeouts**: Helm operations have 5-minute timeouts to prevent hanging

## Dependencies

### Prerequisites

- **k3s**: Must be installed and running
- **kubectl**: Required for Kubernetes API access
- **Helm 3.x**: Required for chart deployment
- **Redis**: Should be installed first for ScaledObject to function

### Component Dependencies

As defined in `Component::dependencies()`:
```rust
Component::Keda => vec![Component::K3s]
```

KEDA requires k3s to be installed first. The setup command will enforce this dependency.

## Architecture Notes

### Resource Requirements

KEDA components have the following default resource limits:

**Operator:**
- Requests: 100m CPU, 128Mi memory
- Limits: 500m CPU, 512Mi memory

**Metrics Server:**
- Requests: 100m CPU, 128Mi memory
- Limits: 500m CPU, 512Mi memory

**Admission Webhooks:**
- Requests: 50m CPU, 64Mi memory
- Limits: 200m CPU, 256Mi memory

**Total:** ~250m CPU, ~320Mi memory (requests)

### High Availability

The default configuration runs single replicas for all components (sufficient for MVP). For production:

- Increase `operator.replicaCount` to 2+
- Enable pod anti-affinity
- Use external metrics storage (Prometheus)

### Security

- RBAC is enabled by default
- Admission webhooks validate ScaledObject/ScaledJob manifests
- Service accounts are created with minimal required permissions

## Testing

### Unit Tests

Located in `src/infrastructure/keda.rs`:

```bash
cargo test infrastructure::keda::tests
```

**Test Coverage:**
- Default configuration values
- Helm values generation
- ScaledObject YAML generation
- Configuration customization
- Target kind variants (Deployment vs Job)

### Integration Tests

To test on a live cluster:

1. Ensure k3s is running
2. Run setup command:
   ```bash
   cargo run -- setup keda
   ```
3. Verify installation:
   ```bash
   kubectl get all -n keda
   kubectl get scaledobject -n raibid-ci
   ```

## Troubleshooting

### Common Issues

**1. Helm Not Found**
```
Error: helm not found. Please install Helm 3.x
```
Solution: Install Helm from https://helm.sh/docs/intro/install/

**2. kubectl Not Available**
```
Error: kubectl not found. Please ensure k3s is installed and kubectl is in PATH.
```
Solution: Install k3s first: `raibid-cli setup k3s`

**3. Namespace Already Exists**
```
Error: namespaces "keda" already exists
```
This is normal and handled gracefully. Installation will continue.

**4. Pods Not Ready**
```
Error: KEDA operator pods not ready
```
Check pod logs:
```bash
kubectl logs -n keda -l app=keda-operator
```

**5. CRDs Missing**
```
Error: KEDA CRD not found: scaledobjects.keda.sh
```
This indicates Helm installation failed. Check Helm output and try reinstalling.

### Debug Mode

Enable debug logging:

```rust
let mut config = KedaConfig::default();
config.log_level = "debug".to_string();
```

Or set environment variable:
```bash
RUST_LOG=debug raibid-cli setup keda
```

## Uninstallation

To remove KEDA:

```bash
# Using the installer
let installer = KedaInstaller::new()?;
installer.uninstall()?;

# Or manually
helm uninstall raibid-keda -n keda
kubectl delete namespace keda
kubectl delete scaledobject raibid-ci-agent-scaler -n raibid-ci
```

**Warning**: Uninstalling KEDA will stop autoscaling. Ensure no critical workloads depend on it.

## Future Enhancements

Potential improvements for post-MVP:

1. **ScaledJob Support**: Use ScaledJob instead of ScaledObject for ephemeral job-based agents
2. **Multiple Triggers**: Combine Redis Streams with cron/time-based scaling
3. **TriggerAuthentication**: Secure Redis credentials with TriggerAuthentication CRD
4. **External Scalers**: Custom scaler for advanced logic
5. **Prometheus Integration**: Scale based on custom metrics from Prometheus
6. **Fallback Triggers**: HTTP-based trigger as fallback for Redis

## References

- **KEDA Documentation**: https://keda.sh/docs/
- **Redis Streams Scaler**: https://keda.sh/docs/scalers/redis-streams/
- **Helm Chart**: https://github.com/kedacore/charts
- **GitHub**: https://github.com/kedacore/keda
- **CNCF Project**: https://www.cncf.io/projects/keda/

## Related Documentation

- [Redis Installation](./redis-deployment.md) - Redis setup required for ScaledObject
- [k3s Setup](../../technology-research.md#1-k3s---lightweight-kubernetes-distribution) - Prerequisite
- [Technology Research](../../technology-research.md#4-keda---kubernetes-event-driven-autoscaling) - KEDA overview
