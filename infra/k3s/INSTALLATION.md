# k3s Installation Runbook

Complete guide for installing and configuring k3s on DGX Spark.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Pre-Installation Checklist](#pre-installation-checklist)
- [Installation Methods](#installation-methods)
- [Post-Installation Verification](#post-installation-verification)
- [Troubleshooting](#troubleshooting)
- [Rollback Procedure](#rollback-procedure)

## Prerequisites

### Hardware Requirements

- **Platform**: NVIDIA DGX Spark
- **Architecture**: ARM64 (aarch64)
- **CPU**: 20 cores (10x Cortex-X925, 10x Cortex-A725)
- **Memory**: 128GB LPDDR5x
- **Storage**: 20GB+ available in `/var/lib`

### Software Requirements

- **OS**: Ubuntu 22.04 LTS
- **Kernel**: 5.15+
- **User**: Non-root user with sudo privileges
- **Network**: Internet connectivity for downloading k3s

### Optional (for Rootless Mode)

- `slirp4netns`
- `fuse-overlayfs`
- `uidmap`
- User subordinate UID/GID mappings

## Pre-Installation Checklist

Before running the installation script, verify:

- [ ] System architecture is ARM64: `uname -m` shows `aarch64`
- [ ] At least 4GB RAM available: `free -h`
- [ ] At least 20GB disk space in `/var/lib`: `df -h /var/lib`
- [ ] User has sudo privileges: `sudo -v`
- [ ] No existing k3s installation (or planned upgrade)
- [ ] Firewall allows required ports (see below)

### Required Ports

| Port | Protocol | Purpose | Direction |
|------|----------|---------|-----------|
| 6443 | TCP | Kubernetes API | Inbound |
| 10250 | TCP | Kubelet metrics | Inbound |
| 8472 | UDP | Flannel VXLAN | Bidirectional |

## Installation Methods

### Method 1: Automated Installation (Recommended)

Use the provided installation script for a fully automated setup.

#### Standard Mode (Requires Root)

```bash
# Navigate to k3s directory
cd /home/beengud/raibid-labs/raibid-ci/infra/k3s

# Run installation script
sudo ./install.sh
```

**What it does:**
1. Checks system architecture and requirements
2. Downloads k3s v1.28.4+k3s1 for ARM64
3. Verifies checksum for security
4. Installs k3s binary to `/usr/local/bin`
5. Configures k3s with DGX Spark optimizations
6. Creates namespaces, storage, and resource quotas
7. Configures CoreDNS customizations
8. Validates installation

**Duration**: ~5 minutes

#### Rootless Mode (No Root Required)

```bash
# Navigate to k3s directory
cd /home/beengud/raibid-labs/raibid-ci/infra/k3s

# Run installation script in rootless mode
./install.sh --rootless
```

**What it does:**
1. Checks rootless prerequisites
2. Installs rootless dependencies if needed
3. Configures subordinate UID/GID mappings
4. Installs k3s in rootless mode for user `raibid-agent`
5. Configures user-level kubeconfig
6. Applies manifests

**Duration**: ~7 minutes (includes dependency installation)

**Note**: Rootless mode has some limitations:
- No LoadBalancer service type
- No HostPort access
- No privileged containers
- Slower networking (uses slirp4netns)

### Method 2: Manual Installation

For advanced users who need fine-grained control.

#### Step 1: Download and Verify k3s

```bash
# Set version
K3S_VERSION=v1.28.4+k3s1

# Download k3s binary
curl -sfL "https://github.com/k3s-io/k3s/releases/download/${K3S_VERSION}/k3s-arm64" \
  -o /tmp/k3s

# Download checksum
curl -sfL "https://github.com/k3s-io/k3s/releases/download/${K3S_VERSION}/sha256sum-arm64.txt" \
  -o /tmp/k3s-checksum.txt

# Verify checksum
expected=$(grep "k3s-arm64" /tmp/k3s-checksum.txt | awk '{print $1}')
actual=$(sha256sum /tmp/k3s | awk '{print $1}')

if [ "$expected" = "$actual" ]; then
  echo "Checksum verified"
else
  echo "Checksum mismatch!"
  exit 1
fi

# Install binary
sudo install -o root -g root -m 0755 /tmp/k3s /usr/local/bin/k3s
sudo ln -sf /usr/local/bin/k3s /usr/local/bin/kubectl
```

#### Step 2: Configure k3s

```bash
# Create config directory
sudo mkdir -p /etc/rancher/k3s

# Copy configuration files
sudo cp config.yaml /etc/rancher/k3s/config.yaml
sudo cp registries.yaml /etc/rancher/k3s/registries.yaml
```

#### Step 3: Install k3s Service

```bash
# Run k3s installer
curl -sfL https://get.k3s.io | sh -s - --config=/etc/rancher/k3s/config.yaml

# Wait for k3s to be ready
sudo k3s kubectl get nodes
```

#### Step 4: Setup Kubeconfig

```bash
# Create .kube directory
mkdir -p ~/.kube

# Copy kubeconfig
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $(id -u):$(id -g) ~/.kube/config
chmod 600 ~/.kube/config

# Test kubectl
kubectl cluster-info
```

#### Step 5: Apply Manifests

```bash
# Create namespaces
kubectl apply -f namespaces.yaml

# Configure storage
kubectl apply -f storageclass.yaml

# Apply resource quotas
kubectl apply -f resource-quotas.yaml

# Customize CoreDNS
kubectl apply -f coredns-custom.yaml
kubectl rollout restart deployment/coredns -n kube-system
```

### Method 3: Via raibid-cli (Future)

Once the raibid-cli tool is fully implemented:

```bash
# One-command installation
raibid-cli setup k3s

# Or with options
raibid-cli setup k3s --rootless --version=v1.28.4+k3s1
```

## Post-Installation Verification

After installation, run the validation script:

```bash
# Navigate to k3s directory
cd /home/beengud/raibid-labs/raibid-ci/infra/k3s

# Run validation tests
./validate-installation.sh
```

### Expected Output

```
==================================
k3s Installation Validation
==================================

Testing: k3s binary exists... PASS
Testing: k3s service is active... PASS
Testing: kubectl command available... PASS
Testing: kubectl cluster communication... PASS
Testing: Node is Ready... PASS
Testing: Node has raibid-ci label... PASS

Checking namespaces...
Testing: Namespace kube-system exists... PASS
Testing: Namespace raibid-ci exists... PASS
Testing: Namespace raibid-infrastructure exists... PASS
Testing: Namespace raibid-monitoring exists... PASS

Checking system pods...
Testing: CoreDNS is running... PASS
Testing: Metrics server is running... PASS
Testing: Local storage class exists... PASS

Testing storage provisioning...
Testing: PVC creation and binding... PASS

Checking networking...
Testing: CNI plugins exist... PASS
Testing: DNS resolution... PASS
Testing: kubeconfig is readable... PASS

Checking resource configuration...
Testing: Max pods configuration... PASS

Checking platform...
Testing: k3s is ARM64 binary... PASS

==================================
Validation Summary
==================================
Total tests:   16
Passed tests:  16
Failed tests:  0

All validation tests passed!
k3s cluster is ready for use.
```

### Manual Verification Commands

```bash
# Check cluster info
kubectl cluster-info

# Check node status
kubectl get nodes -o wide

# Check all namespaces
kubectl get namespaces

# Check system pods
kubectl get pods -A

# Check storage classes
kubectl get storageclass

# Check resource quotas
kubectl get resourcequota -A

# Check limit ranges
kubectl get limitrange -A

# Test PVC creation
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: test-pvc
  namespace: default
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: local-path
  resources:
    requests:
      storage: 1Gi
EOF

# Verify PVC is bound
kubectl get pvc test-pvc

# Cleanup
kubectl delete pvc test-pvc
```

## Troubleshooting

### Installation Fails

**Problem**: Installation script exits with error.

**Solution**:
1. Check system logs:
   ```bash
   sudo journalctl -u k3s -n 50
   ```

2. Verify architecture:
   ```bash
   uname -m  # Should show aarch64
   ```

3. Check available resources:
   ```bash
   free -h
   df -h /var/lib
   ```

4. Ensure no firewall blocking:
   ```bash
   sudo ufw status
   ```

### k3s Service Won't Start

**Problem**: k3s service fails to start.

**Solution**:
1. Check service status:
   ```bash
   sudo systemctl status k3s
   ```

2. View detailed logs:
   ```bash
   sudo journalctl -u k3s -f
   ```

3. Check configuration:
   ```bash
   sudo cat /etc/rancher/k3s/config.yaml
   ```

4. Restart service:
   ```bash
   sudo systemctl restart k3s
   ```

### kubectl Commands Fail

**Problem**: kubectl cannot communicate with cluster.

**Solution**:
1. Check kubeconfig:
   ```bash
   ls -la ~/.kube/config
   cat ~/.kube/config
   ```

2. Verify k3s is running:
   ```bash
   sudo systemctl status k3s
   ```

3. Check API server:
   ```bash
   curl -k https://localhost:6443/livez?verbose
   ```

4. Recreate kubeconfig:
   ```bash
   sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
   sudo chown $(id -u):$(id -g) ~/.kube/config
   ```

### Pods Not Starting

**Problem**: Pods stuck in Pending or CrashLoopBackOff.

**Solution**:
1. Describe the pod:
   ```bash
   kubectl describe pod <pod-name> -n <namespace>
   ```

2. Check events:
   ```bash
   kubectl get events -n <namespace> --sort-by='.lastTimestamp'
   ```

3. Check resource availability:
   ```bash
   kubectl top nodes
   kubectl describe node
   ```

4. Check logs:
   ```bash
   kubectl logs <pod-name> -n <namespace>
   ```

### Storage Issues

**Problem**: PVCs not binding.

**Solution**:
1. Check storage class:
   ```bash
   kubectl get storageclass
   ```

2. Check local-path provisioner:
   ```bash
   kubectl get pods -n kube-system -l app=local-path-provisioner
   kubectl logs -n kube-system -l app=local-path-provisioner
   ```

3. Verify storage directory:
   ```bash
   ls -la /var/lib/rancher/k3s/storage
   ```

4. Check PVC status:
   ```bash
   kubectl describe pvc <pvc-name>
   ```

### DNS Not Working

**Problem**: Pods cannot resolve DNS names.

**Solution**:
1. Check CoreDNS:
   ```bash
   kubectl get pods -n kube-system -l k8s-app=kube-dns
   kubectl logs -n kube-system -l k8s-app=kube-dns
   ```

2. Test DNS from a pod:
   ```bash
   kubectl run test --rm -it --image=busybox -- nslookup kubernetes.default
   ```

3. Check CoreDNS config:
   ```bash
   kubectl get configmap coredns -n kube-system -o yaml
   ```

4. Restart CoreDNS:
   ```bash
   kubectl rollout restart deployment/coredns -n kube-system
   ```

## Rollback Procedure

If installation fails and you need to start over:

### Standard Mode

```bash
# Stop k3s service
sudo systemctl stop k3s

# Uninstall k3s
sudo /usr/local/bin/k3s-uninstall.sh

# Remove configuration
sudo rm -rf /etc/rancher/k3s

# Remove data
sudo rm -rf /var/lib/rancher/k3s

# Remove kubeconfig
rm -rf ~/.kube

# Verify cleanup
ps aux | grep k3s  # Should show nothing
```

### Rootless Mode

```bash
# Stop k3s service
systemctl --user stop k3s-rootless

# Uninstall k3s
k3s-rootless-uninstall.sh

# Remove configuration
rm -rf ~/.config/k3s

# Remove data
rm -rf ~/.local/share/k3s

# Remove kubeconfig
rm -rf ~/.kube

# Verify cleanup
ps aux | grep k3s  # Should show nothing
```

### After Rollback

1. Review errors from previous installation
2. Fix any issues (resources, configuration, etc.)
3. Run installation script again

## Resource Allocation Summary

After successful installation, the DGX Spark resources are allocated as follows:

| Component | CPU Reservation | Memory Reservation |
|-----------|-----------------|-------------------|
| System | 4 cores | 16GB |
| k3s (Kubernetes) | 2 cores | 8GB |
| Infrastructure | 6 cores (quota) | 32GB (quota) |
| CI Agents | 10 cores (quota) | 80GB (quota) |
| Monitoring | 2 cores (quota) | 8GB (quota) |

**Total Reserved**: 6 cores + quotas / 24GB + quotas

**Available for Workloads**: ~14 cores / ~104GB (after system reservations)

## Next Steps

After successful k3s installation:

1. **Deploy Redis** (Job Queue)
   ```bash
   cd ../redis
   raibid-cli setup redis
   ```

2. **Deploy Gitea** (Git Server + OCI Registry)
   ```bash
   cd ../gitea
   raibid-cli setup gitea
   ```

3. **Deploy KEDA** (Autoscaling)
   ```bash
   cd ../keda
   raibid-cli setup keda
   ```

4. **Deploy Flux** (GitOps)
   ```bash
   cd ../flux
   raibid-cli setup flux
   ```

## References

- [k3s Official Documentation](https://docs.k3s.io/)
- [k3s GitHub Releases](https://github.com/k3s-io/k3s/releases)
- [k3s Rootless Mode](https://docs.k3s.io/advanced#running-k3s-in-rootless-mode)
- [DGX Spark Documentation](https://developer.nvidia.com/dgx-spark)
- [Issue #56: WS-04 k3s Installation Manifests](https://github.com/raibid-labs/raibid-ci/issues/56)
