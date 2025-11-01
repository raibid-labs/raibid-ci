# k3s Configuration

Lightweight Kubernetes distribution optimized for DGX Spark.

## Overview

k3s is the foundation layer for the raibid-ci infrastructure. It provides a production-ready Kubernetes cluster with a minimal resource footprint.

## Configuration Files

- `config.yaml` - k3s cluster configuration
- `install-flags.txt` - Installation flags for k3s installer
- `rootless-config.yaml` - Rootless mode configuration

## Installation

### Via raibid-cli (Recommended)

```bash
raibid-cli setup k3s
```

### Manual Installation

```bash
# Standard installation
curl -sfL https://get.k3s.io | sh -s - --config=/path/to/config.yaml

# Rootless installation
curl -sfL https://get.k3s.io | sh -s - --rootless --config=/path/to/rootless-config.yaml
```

## Features

- **Lightweight**: <100MB memory footprint
- **Single Binary**: Easy deployment and updates
- **Integrated Components**:
  - Traefik ingress controller
  - CoreDNS
  - Local-path storage provisioner
  - Metrics server
- **Rootless Mode**: Run without root privileges (optional)

## Configuration Options

### Standard Mode

```yaml
# config.yaml
write-kubeconfig-mode: "0644"
tls-san:
  - "dgx-spark.local"
disable:
  - traefik  # Optional: disable if using custom ingress
node-label:
  - "raibid-ci=true"
```

### Rootless Mode

```yaml
# rootless-config.yaml
write-kubeconfig-mode: "0644"
rootless: true
snapshotter: "overlayfs"
```

## Resource Requirements

### Minimum
- CPU: 1 core
- Memory: 512MB
- Disk: 1GB

### Recommended (DGX Spark)
- CPU: 2 cores
- Memory: 2GB
- Disk: 10GB

## Validation

```bash
# Check cluster status
kubectl cluster-info

# Verify nodes
kubectl get nodes

# Check system pods
kubectl get pods -n kube-system

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

# Reset network
sudo systemctl stop k3s
sudo rm -rf /var/lib/rancher/k3s/agent/pod-manifests/*
sudo systemctl start k3s
```

## Uninstallation

```bash
# Via raibid-cli
raibid-cli teardown k3s

# Manual (standard)
/usr/local/bin/k3s-uninstall.sh

# Manual (rootless)
/usr/local/bin/k3s-rootless-uninstall.sh
```

## References

- [k3s Documentation](https://docs.k3s.io/)
- [k3s GitHub](https://github.com/k3s-io/k3s)
- [DGX Spark Documentation](https://developer.nvidia.com/dgx-spark)
