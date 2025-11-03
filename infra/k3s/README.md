# k3s Configuration

Lightweight Kubernetes distribution optimized for DGX Spark ARM64 platform.

## Overview

k3s is the foundation layer for the raibid-ci infrastructure. It provides a production-ready Kubernetes cluster with a minimal resource footprint, specifically configured for the DGX Spark's ARM64 architecture.

## Quick Start

```bash
# Automated installation (recommended)
./install.sh

# Rootless mode (no root required)
./install.sh --rootless

# Validate installation
./validate-installation.sh
```

## Configuration Files

| File | Purpose |
|------|---------|
| `config.yaml` | Standard mode k3s cluster configuration |
| `rootless-config.yaml` | Rootless mode configuration |
| `install-flags.txt` | Installation flags reference |
| `namespaces.yaml` | Namespace definitions for CI, infrastructure, monitoring |
| `registries.yaml` | OCI registry configuration for Gitea |
| `storageclass.yaml` | Local storage provisioner configuration |
| `resource-quotas.yaml` | Resource limits and quotas for namespaces |
| `coredns-custom.yaml` | Custom DNS entries and CoreDNS configuration |
| `install.sh` | Automated installation script with checksum verification |
| `validate-installation.sh` | Post-installation validation script |
| `INSTALLATION.md` | Detailed installation runbook |

## Installation

### Automated Installation (Recommended)

The automated installation script handles everything:

```bash
cd /home/beengud/raibid-labs/raibid-ci/infra/k3s
sudo ./install.sh
```

**What it does:**
- Verifies ARM64 architecture
- Downloads k3s with checksum verification
- Configures DGX Spark optimizations
- Creates namespaces and applies manifests
- Sets up storage and resource quotas
- Validates installation

See [INSTALLATION.md](./INSTALLATION.md) for detailed installation guide.

### Via raibid-cli (Future)

```bash
raibid-cli setup k3s
```

### Manual Installation

See [INSTALLATION.md](./INSTALLATION.md#method-2-manual-installation) for step-by-step manual installation.

## Features

### Lightweight & Fast
- Single binary: ~100MB
- Fast startup: <2 minutes to cluster ready
- Low memory footprint: ~512MB base
- Optimized for ARM64

### Integrated Components
- **Traefik**: Disabled (using custom ingress)
- **CoreDNS**: Customized for raibid-ci
- **Local-path storage**: Configured for DGX Spark
- **Metrics server**: Enabled for autoscaling
- **Flannel CNI**: VXLAN backend for networking

### Security Features
- Secrets encryption at rest
- TLS for all components
- RBAC enabled by default
- Network policies support

### DGX Spark Optimizations
- ARM64 native binary
- Resource reservations (4 cores, 16GB for system)
- Kubernetes reservations (2 cores, 8GB for k3s)
- Max 110 pods per node
- Overlayfs snapshotter for performance

## Configuration Options

### Standard Mode

```yaml
# /etc/rancher/k3s/config.yaml
write-kubeconfig-mode: "0644"
node-label:
  - "raibid-ci=true"
  - "arch=arm64"
disable:
  - traefik
secrets-encryption: true
snapshotter: "overlayfs"
```

### Rootless Mode

```yaml
# ~/.config/k3s/config.yaml
rootless: true
write-kubeconfig-mode: "0644"
snapshotter: "overlayfs"
disable:
  - traefik
```

### Resource Reservations

| Component | CPU | Memory |
|-----------|-----|--------|
| System Reserved | 4000m | 16Gi |
| Kubernetes Reserved | 2000m | 8Gi |
| **Available for Workloads** | **14 cores** | **104Gi** |

### Namespace Quotas

| Namespace | CPU Quota | Memory Quota | Storage Quota |
|-----------|-----------|--------------|---------------|
| raibid-ci | 10 cores | 80Gi | 100Gi |
| raibid-infrastructure | 6-8 cores | 32-40Gi | 500Gi |
| raibid-monitoring | 2-4 cores | 8-16Gi | 100Gi |

## Resource Requirements

### Minimum
- CPU: 2 cores
- Memory: 4GB
- Disk: 20GB

### Recommended (DGX Spark)
- CPU: 20 cores (10x Cortex-X925, 10x Cortex-A725)
- Memory: 128GB LPDDR5x
- Disk: 100GB+ NVMe

## Validation

### Automated Validation

```bash
./validate-installation.sh
```

Tests performed:
- k3s binary and service status
- kubectl connectivity
- Node ready state and labels
- Namespace creation
- System pods (CoreDNS, metrics-server)
- Storage provisioning
- DNS resolution
- Networking (CNI plugins)
- Resource quotas and limits
- Platform verification (ARM64)

### Manual Validation

```bash
# Check cluster status
kubectl cluster-info
kubectl get nodes -o wide

# Verify namespaces
kubectl get namespaces

# Check system pods
kubectl get pods -n kube-system

# Test storage
kubectl get storageclass
kubectl get pvc -A

# Verify resource quotas
kubectl get resourcequota -A
kubectl get limitrange -A

# Test cluster
kubectl run test --image=nginx --rm -it -- /bin/sh
```

## Troubleshooting

### Service Not Starting

```bash
# Check service status
sudo systemctl status k3s

# View logs
sudo journalctl -u k3s -f

# Restart service
sudo systemctl restart k3s
```

### Network Issues

```bash
# Check CNI plugins
ls /var/lib/rancher/k3s/data/current/bin

# Test DNS
kubectl run test --rm -it --image=busybox -- nslookup kubernetes.default

# Restart CoreDNS
kubectl rollout restart deployment/coredns -n kube-system
```

### Storage Issues

```bash
# Check storage provisioner
kubectl get pods -n kube-system -l app=local-path-provisioner
kubectl logs -n kube-system -l app=local-path-provisioner

# Verify storage directory
ls -la /var/lib/rancher/k3s/storage
```

See [INSTALLATION.md](./INSTALLATION.md#troubleshooting) for comprehensive troubleshooting guide.

## Uninstallation

### Standard Mode

```bash
# Via raibid-cli (future)
raibid-cli teardown k3s

# Manual
sudo /usr/local/bin/k3s-uninstall.sh
```

### Rootless Mode

```bash
# Via raibid-cli (future)
raibid-cli teardown k3s

# Manual
k3s-rootless-uninstall.sh
```

## Upgrading

### Via Automated Script

```bash
# Set desired version
export K3S_VERSION=v1.29.0+k3s1

# Run install script (handles upgrade)
sudo ./install.sh
```

### Manual Upgrade

```bash
# Stop k3s
sudo systemctl stop k3s

# Download new version
curl -sfL https://get.k3s.io | K3S_VERSION=v1.29.0+k3s1 sh -

# Restart k3s
sudo systemctl start k3s
```

## Monitoring

### Cluster Metrics

```bash
# Node metrics
kubectl top nodes

# Pod metrics
kubectl top pods -A

# Describe node for detailed info
kubectl describe node
```

### Health Checks

```bash
# API server health
kubectl get --raw='/livez?verbose'

# Component status
kubectl get componentstatus

# Events
kubectl get events -A --sort-by='.lastTimestamp'
```

## Architecture

### Deployment Model

```
DGX Spark (ARM64)
├─ System Layer (4 cores, 16GB)
│  └─ Ubuntu 22.04 LTS
├─ k3s Layer (2 cores, 8GB)
│  ├─ API Server
│  ├─ Controller Manager
│  ├─ Scheduler
│  ├─ CoreDNS
│  ├─ Flannel CNI
│  └─ Local Path Provisioner
├─ Infrastructure Layer (6-8 cores, 32-40GB)
│  ├─ Gitea (namespace: raibid-infrastructure)
│  ├─ Redis (namespace: raibid-infrastructure)
│  ├─ KEDA (namespace: keda)
│  └─ Flux (namespace: flux-system)
├─ CI Layer (10 cores, 80GB)
│  └─ CI Agents (namespace: raibid-ci)
└─ Monitoring Layer (2-4 cores, 8-16GB)
   └─ Observability Stack (namespace: raibid-monitoring)
```

### Network Architecture

- **Cluster CIDR**: 10.42.0.0/16
- **Service CIDR**: 10.43.0.0/16
- **Flannel Backend**: VXLAN
- **DNS**: CoreDNS (10.43.0.10)

### Storage Architecture

- **Provisioner**: local-path (Rancher)
- **Storage Path**: /var/lib/rancher/k3s/storage
- **Reclaim Policy**: Delete
- **Volume Binding**: WaitForFirstConsumer

## Best Practices

### Resource Management
- Always set resource requests and limits
- Use resource quotas to prevent resource exhaustion
- Monitor resource usage regularly

### Security
- Enable secrets encryption (already configured)
- Use RBAC for access control
- Apply network policies for isolation
- Rotate credentials regularly

### High Availability
- For production, consider multi-node setup
- Regular backups of etcd data
- Monitor cluster health proactively

### Performance
- Use overlayfs snapshotter for better I/O
- Configure appropriate resource reservations
- Enable metrics server for autoscaling
- Tune garbage collection thresholds

## References

### Official Documentation
- [k3s Documentation](https://docs.k3s.io/)
- [k3s GitHub](https://github.com/k3s-io/k3s)
- [k3s Releases](https://github.com/k3s-io/k3s/releases)

### DGX Spark
- [DGX Spark Documentation](https://developer.nvidia.com/dgx-spark)
- [ARM64 Optimizations](https://docs.k3s.io/installation/requirements#operating-systems)

### Related Documentation
- [Installation Runbook](./INSTALLATION.md)
- [Main Infrastructure README](../README.md)
- [Project README](../../README.md)
- [Issue #56](https://github.com/raibid-labs/raibid-ci/issues/56)

## Support

For issues or questions:
- Review [INSTALLATION.md](./INSTALLATION.md) troubleshooting section
- Check [k3s documentation](https://docs.k3s.io/)
- Open issue on [GitHub](https://github.com/raibid-labs/raibid-ci/issues)
