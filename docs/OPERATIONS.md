# Operational Runbook

Comprehensive guide for operating, maintaining, and troubleshooting raibid-ci in production.

## Table of Contents

- [Overview](#overview)
- [Daily Operations](#daily-operations)
- [Monitoring](#monitoring)
- [Maintenance Tasks](#maintenance-tasks)
- [Backup and Recovery](#backup-and-recovery)
- [Scaling](#scaling)
- [Incident Response](#incident-response)
- [Troubleshooting Guide](#troubleshooting-guide)
- [Performance Tuning](#performance-tuning)
- [Security Operations](#security-operations)
- [Quick Reference Cheatsheet](#quick-reference-cheatsheet)

## Overview

This runbook provides operational procedures for managing raibid-ci in production environments. It covers routine maintenance, incident response, troubleshooting, and optimization.

**Target Audience**: System operators, SREs, DevOps engineers

**Prerequisites**:
- raibid-ci installed and running (see [INSTALLATION.md](INSTALLATION.md))
- kubectl access to the Kubernetes cluster
- Access to system logs and monitoring tools

## Daily Operations

### Morning Health Check

Perform this check at the start of each day to ensure system health:

```bash
# 1. Check overall system status
raibid-cli status

# 2. Check cluster health
kubectl get nodes
kubectl get pods -n raibid-system

# 3. Check for pod restarts or crashes
kubectl get pods -n raibid-system -o wide | grep -E 'Restart|Error|CrashLoop'

# 4. Check disk usage on k3s node
df -h | grep -E 'Filesystem|/$|/var'

# 5. Check Redis queue depth
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XLEN ci:jobs

# 6. Check recent job failures
raibid-cli job list --status failed --limit 10

# 7. Check agent scaling
kubectl get scaledjob raibid-agent -n raibid-system
```

**Expected Results**:
- All pods in Running state
- No excessive restarts (< 5 in last 24h)
- Disk usage < 80%
- Job queue depth reasonable (< 100 pending)
- Recent failure rate < 5%

**Action if Abnormal**: See [Incident Response](#incident-response) section.

### Monitor Active Jobs

```bash
# List running jobs
raibid-cli job list --status running

# Watch job progress in TUI
raibid-cli tui

# Check job logs
raibid-cli job show <job-id>

# Stream logs in real-time (via API)
curl http://localhost:8080/jobs/<job-id>/logs
```

### Check Agent Health

```bash
# List active agents
raibid-cli agent list

# Check agent resource usage
kubectl top pods -n raibid-system -l app=raibid-agent

# Check for idle agents
raibid-cli agent list --status idle

# View agent details
raibid-cli agent show <agent-id>
```

### Review Metrics

```bash
# Server metrics (Prometheus format)
curl http://localhost:8080/metrics

# Check API health
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/health/ready
```

## Monitoring

### Key Metrics to Monitor

#### Infrastructure Metrics

**Cluster Health**:
```bash
# Node status and resources
kubectl top nodes
kubectl describe nodes

# Control plane health
kubectl get componentstatuses

# etcd health (k3s)
sudo k3s kubectl get --raw /healthz/etcd
```

**Pod Health**:
```bash
# Pod status across all namespaces
kubectl get pods --all-namespaces

# Pod resource usage
kubectl top pods -n raibid-system

# Pod events (errors, warnings)
kubectl get events -n raibid-system --sort-by='.lastTimestamp'
```

#### Application Metrics

**Job Queue Metrics**:
```bash
# Queue depth
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XLEN ci:jobs

# Consumer group info
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XINFO GROUPS ci:jobs

# Pending messages per consumer
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XPENDING ci:jobs ci-workers
```

**Agent Metrics**:
```bash
# Active agent count
kubectl get pods -n raibid-system -l app=raibid-agent --field-selector=status.phase=Running --no-headers | wc -l

# Agent resource usage
kubectl top pods -n raibid-system -l app=raibid-agent

# Agent uptime
kubectl get pods -n raibid-system -l app=raibid-agent -o wide
```

**API Server Metrics**:
```bash
# Request rate and latency
curl http://localhost:8080/metrics | grep -E 'http_requests_total|http_request_duration'

# Active connections
curl http://localhost:8080/health | jq '.active_connections'

# Error rate
curl http://localhost:8080/metrics | grep 'http_requests_total{.*status="5.."}'
```

### Monitoring Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| Disk usage | > 75% | > 85% | Clean up old images, logs |
| Memory usage (node) | > 80% | > 90% | Scale down agents, restart pods |
| CPU usage (node) | > 75% | > 90% | Scale down agents, check for runaway processes |
| Job queue depth | > 50 | > 100 | Scale up agents, check for stuck jobs |
| Job failure rate | > 10% | > 25% | Investigate failures, check agent logs |
| Pod restart count | > 5/day | > 10/day | Check logs, investigate crash loops |
| API error rate | > 1% | > 5% | Check server logs, restart if needed |

### Log Aggregation

```bash
# View server logs
kubectl logs -f -l app=raibid-server -n raibid-system

# View agent logs
kubectl logs -f -l app=raibid-agent -n raibid-system

# View Redis logs
kubectl logs -f redis-master-0 -n raibid-system

# View KEDA operator logs
kubectl logs -f -l app.kubernetes.io/name=keda-operator -n keda

# View Flux logs
kubectl logs -f -l app=source-controller -n flux-system
```

**Log Retention**: Configure log rotation to prevent disk exhaustion.

```bash
# Check log sizes
kubectl exec -it redis-master-0 -n raibid-system -- du -sh /var/log

# Configure log rotation (k3s)
sudo vim /etc/rancher/k3s/logrotate.conf
```

### Alerting Setup

Configure alerts for critical metrics using your monitoring system (e.g., Prometheus + Alertmanager).

**Recommended Alerts**:
- Node down
- Pod crash loops
- High memory/CPU usage
- Disk space low
- Job queue backing up
- High job failure rate
- API server down

## Maintenance Tasks

### Daily Maintenance

#### Clean Up Completed Jobs

```bash
# Remove old completed jobs from Redis (older than 7 days)
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli --eval /scripts/cleanup-jobs.lua

# Or manually
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XDEL ci:jobs <job-id>
```

#### Check for Updates

```bash
# Check for raibid-ci updates
cd ~/raibid-ci
git fetch origin
git log HEAD..origin/main --oneline

# Check for Helm chart updates
helm repo update
helm search repo bitnami/redis --versions | head -5
helm search repo gitea-charts/gitea --versions | head -5
```

### Weekly Maintenance

#### Update Dependencies

```bash
# Update Rust dependencies
cd ~/raibid-ci
cargo update

# Rebuild and test
cargo build --release --workspace
cargo test --workspace

# Update raibid-cli binary
sudo cp target/release/raibid-cli /usr/local/bin/
```

#### Prune Docker Images

```bash
# Remove unused images (saves disk space)
docker system prune -a --filter "until=168h"  # 7 days

# Or via k3s
sudo k3s crictl rmi --prune
```

#### Backup Configuration

```bash
# Backup raibid-ci config
cp ~/.config/raibid/config.yaml ~/backups/raibid-config-$(date +%Y%m%d).yaml

# Backup Kubernetes resources
kubectl get all -n raibid-system -o yaml > ~/backups/raibid-k8s-$(date +%Y%m%d).yaml

# Backup etcd (k3s datastore)
sudo k3s etcd-snapshot save --name raibid-backup-$(date +%Y%m%d)
```

### Monthly Maintenance

#### Rotate Certificates

k3s auto-rotates certificates, but verify:

```bash
# Check certificate expiry
sudo openssl x509 -in /var/lib/rancher/k3s/server/tls/server-ca.crt -noout -dates

# Force rotation if needed (requires restart)
sudo k3s certificate rotate
sudo systemctl restart k3s
```

#### Update Helm Charts

```bash
# Check for chart updates
cd ~/raibid-ci/tanka
helm repo update

# Pull latest versions
just tk-vendor

# Review changes
tk diff environments/local

# Apply updates
tk apply environments/local

# Verify deployment
kubectl rollout status deployment/raibid-server -n raibid-system
```

#### Performance Review

```bash
# Analyze job completion times
raibid-cli job list --status success --limit 100 | jq '.jobs[] | .duration' | awk '{sum+=$1; count++} END {print "Avg duration:", sum/count, "seconds"}'

# Check cache hit rates (if caching implemented)
curl http://localhost:8080/metrics | grep cache_hit_rate

# Review agent utilization
kubectl top pods -n raibid-system -l app=raibid-agent --no-headers | awk '{sum+=$2; count++} END {print "Avg CPU:", sum/count}'
```

#### Upgrade k3s

```bash
# Check current version
k3s --version

# Download new version
curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION=v1.28.8+k3s1 sh -

# Verify upgrade
kubectl get nodes
kubectl get pods --all-namespaces
```

### On-Demand Maintenance

#### Restart Components

```bash
# Restart API server
kubectl rollout restart deployment raibid-server -n raibid-system
kubectl rollout status deployment raibid-server -n raibid-system

# Restart agents (scales to 0 and back)
kubectl scale scaledjob raibid-agent --replicas=0 -n raibid-system
# Wait 30 seconds
kubectl scale scaledjob raibid-agent --replicas=1 -n raibid-system

# Restart Redis (caution: may lose in-flight data)
kubectl rollout restart statefulset redis-master -n raibid-system

# Restart KEDA operator
kubectl rollout restart deployment keda-operator -n keda
```

#### Clear Redis Queue

Use with caution - this removes all pending jobs:

```bash
# View queue depth first
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XLEN ci:jobs

# Clear queue
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli DEL ci:jobs

# Recreate stream
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli XGROUP CREATE ci:jobs ci-workers 0 MKSTREAM
```

#### Reset Gitea

If Gitea becomes corrupted:

```bash
# Delete Gitea pod and PVC
kubectl delete pod gitea-0 -n raibid-system
kubectl delete pvc data-gitea-0 -n raibid-system

# Redeploy
cd ~/raibid-ci
raibid-cli setup gitea

# Reconfigure repositories
raibid-cli mirror add github.com/your-org/your-repo
```

## Backup and Recovery

### Backup Strategy

**What to Backup**:
1. raibid-ci configuration files
2. Kubernetes resources and manifests
3. k3s etcd datastore
4. Redis data (optional - job queue state)
5. Gitea repositories and database
6. TLS certificates and secrets

**Backup Frequency**:
- Configuration: Daily
- Kubernetes resources: Daily
- etcd snapshots: Daily (automated by k3s)
- Gitea data: Weekly
- Secrets: Weekly

### Creating Backups

#### Configuration Backup

```bash
#!/bin/bash
# backup-config.sh

BACKUP_DIR=~/backups/raibid-ci
DATE=$(date +%Y%m%d-%H%M%S)

mkdir -p $BACKUP_DIR

# raibid-ci config
cp ~/.config/raibid/config.yaml $BACKUP_DIR/config-$DATE.yaml

# Kubernetes manifests
kubectl get all -n raibid-system -o yaml > $BACKUP_DIR/k8s-resources-$DATE.yaml
kubectl get secrets -n raibid-system -o yaml > $BACKUP_DIR/secrets-$DATE.yaml
kubectl get configmaps -n raibid-system -o yaml > $BACKUP_DIR/configmaps-$DATE.yaml

echo "Backup completed: $BACKUP_DIR"
```

#### etcd Snapshot (k3s)

```bash
# Manual snapshot
sudo k3s etcd-snapshot save --name raibid-backup-$(date +%Y%m%d)

# List snapshots
sudo k3s etcd-snapshot ls

# Snapshots stored in: /var/lib/rancher/k3s/server/db/snapshots/
```

#### Redis Backup

```bash
# Trigger Redis RDB snapshot
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli BGSAVE

# Copy snapshot from pod
kubectl cp raibid-system/redis-master-0:/data/dump.rdb ~/backups/redis-dump-$(date +%Y%m%d).rdb
```

#### Gitea Backup

```bash
# Backup Gitea data
kubectl exec -it gitea-0 -n raibid-system -- /usr/local/bin/gitea dump -c /etc/gitea/app.ini

# Copy dump from pod
kubectl cp raibid-system/gitea-0:/tmp/gitea-dump.zip ~/backups/gitea-dump-$(date +%Y%m%d).zip
```

### Automated Backup Script

```bash
#!/bin/bash
# automated-backup.sh

set -e

BACKUP_ROOT=~/backups/raibid-ci
DATE=$(date +%Y%m%d)
RETENTION_DAYS=30

mkdir -p $BACKUP_ROOT/$DATE

# Configuration
cp ~/.config/raibid/config.yaml $BACKUP_ROOT/$DATE/

# Kubernetes resources
kubectl get all,secrets,configmaps -n raibid-system -o yaml > $BACKUP_ROOT/$DATE/k8s-resources.yaml

# etcd snapshot
sudo k3s etcd-snapshot save --name raibid-$DATE

# Redis backup
kubectl exec redis-master-0 -n raibid-system -- redis-cli BGSAVE
sleep 5
kubectl cp raibid-system/redis-master-0:/data/dump.rdb $BACKUP_ROOT/$DATE/redis-dump.rdb

# Cleanup old backups
find $BACKUP_ROOT -maxdepth 1 -type d -mtime +$RETENTION_DAYS -exec rm -rf {} \;

echo "Backup completed: $BACKUP_ROOT/$DATE"
```

**Schedule with cron**:
```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * /home/user/scripts/automated-backup.sh >> /var/log/raibid-backup.log 2>&1
```

### Disaster Recovery

#### Complete System Recovery

1. **Restore k3s cluster**:
```bash
# Stop k3s
sudo systemctl stop k3s

# Restore etcd snapshot
sudo k3s server --cluster-reset --cluster-reset-restore-path=/var/lib/rancher/k3s/server/db/snapshots/raibid-backup-20251103

# Start k3s
sudo systemctl start k3s

# Verify cluster
kubectl get nodes
```

2. **Restore configuration**:
```bash
# Restore raibid-ci config
cp ~/backups/raibid-ci/20251103/config.yaml ~/.config/raibid/config.yaml
```

3. **Restore Kubernetes resources**:
```bash
# Apply backed up resources
kubectl apply -f ~/backups/raibid-ci/20251103/k8s-resources.yaml
```

4. **Restore Redis data**:
```bash
# Copy RDB file to Redis pod
kubectl cp ~/backups/raibid-ci/20251103/redis-dump.rdb raibid-system/redis-master-0:/data/dump.rdb

# Restart Redis
kubectl rollout restart statefulset redis-master -n raibid-system
```

5. **Restore Gitea**:
```bash
# Copy dump to Gitea pod
kubectl cp ~/backups/raibid-ci/20251103/gitea-dump.zip raibid-system/gitea-0:/tmp/

# Restore dump
kubectl exec -it gitea-0 -n raibid-system -- /usr/local/bin/gitea restore --from /tmp/gitea-dump.zip
```

6. **Verify system**:
```bash
raibid-cli status
kubectl get pods -n raibid-system
raibid-cli tui
```

#### Partial Recovery

**Recover single component**:
```bash
# Example: Recover only Redis
kubectl apply -f ~/backups/raibid-ci/20251103/k8s-resources.yaml --selector=app=redis
```

## Scaling

### Agent Scaling

#### Manual Scaling

```bash
# Scale agents to specific count
raibid-cli agent scale --count 5

# Scale with min/max bounds
raibid-cli agent scale --count 3 --min 1 --max 10

# Scale to zero (pause CI)
raibid-cli agent scale --count 0
```

#### Auto-Scaling Configuration

KEDA automatically scales agents based on Redis queue depth.

**View KEDA ScaledJob**:
```bash
kubectl get scaledjob raibid-agent -n raibid-system -o yaml
```

**Adjust scaling parameters**:
```bash
# Edit ScaledJob
kubectl edit scaledjob raibid-agent -n raibid-system

# Key parameters:
# - minReplicaCount: Minimum agents (default: 0)
# - maxReplicaCount: Maximum agents (default: 10)
# - pollingInterval: How often to check queue (default: 30s)
# - cooldownPeriod: Wait time before scaling down (default: 300s)
# - threshold: Queue depth per agent (default: 5)
```

**Example**: Scale more aggressively:
```yaml
spec:
  minReplicaCount: 1      # Always keep 1 agent warm
  maxReplicaCount: 15     # Allow more agents
  pollingInterval: 15     # Check queue more frequently
  cooldownPeriod: 180     # Scale down faster
  triggers:
    - type: redis-streams
      metadata:
        stream: ci:jobs
        consumerGroup: ci-workers
        pendingEntriesCount: "3"  # Lower threshold = more agents
```

#### Scaling Best Practices

**DGX Spark (20 cores, 128GB RAM)**:
- Each agent: ~2 cores, ~4GB RAM
- Max recommended agents: 8 (leaves resources for system)
- Configure: `maxReplicaCount: 8`

**Monitor scaling behavior**:
```bash
# Watch agent scaling
watch -n 5 'kubectl get pods -n raibid-system -l app=raibid-agent'

# Check KEDA metrics
kubectl logs -l app.kubernetes.io/name=keda-operator -n keda | grep raibid-agent

# View queue depth over time
watch -n 10 'kubectl exec redis-master-0 -n raibid-system -- redis-cli XLEN ci:jobs'
```

### Infrastructure Scaling

#### Add Cluster Nodes (Multi-Node k3s)

```bash
# On new node, get token from master
sudo cat /var/lib/rancher/k3s/server/node-token

# On worker node, join cluster
curl -sfL https://get.k3s.io | K3S_URL=https://master-ip:6443 K3S_TOKEN=<token> sh -

# Verify node joined
kubectl get nodes
```

#### Vertical Scaling (Resource Limits)

Adjust resource requests/limits for components:

```bash
# Edit server deployment
kubectl edit deployment raibid-server -n raibid-system

# Adjust resources:
spec:
  containers:
  - name: raibid-server
    resources:
      requests:
        memory: "512Mi"
        cpu: "500m"
      limits:
        memory: "1Gi"
        cpu: "1000m"
```

## Incident Response

### Incident Response Procedure

1. **Acknowledge**: Confirm incident and notify team
2. **Assess**: Determine severity and impact
3. **Diagnose**: Identify root cause
4. **Mitigate**: Implement immediate fix
5. **Resolve**: Verify system restored
6. **Document**: Record incident details and lessons learned

### Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| P0 - Critical | Complete system outage | Immediate | API down, k3s cluster down |
| P1 - High | Partial outage or severe degradation | < 15 min | High job failure rate, agent pods crashing |
| P2 - Medium | Degraded performance | < 1 hour | Slow job execution, queue backing up |
| P3 - Low | Minor issues, workarounds available | < 4 hours | Individual job failure, UI glitch |

### Common Incidents

#### P0: API Server Down

**Symptoms**:
- `raibid-cli` commands fail
- TUI cannot connect
- Health endpoint unreachable

**Diagnosis**:
```bash
# Check server pod status
kubectl get pods -n raibid-system -l app=raibid-server

# Check logs
kubectl logs -l app=raibid-server -n raibid-system --tail=100

# Check events
kubectl describe pod -l app=raibid-server -n raibid-system
```

**Mitigation**:
```bash
# Restart server
kubectl rollout restart deployment raibid-server -n raibid-system

# Scale up replicas temporarily
kubectl scale deployment raibid-server --replicas=2 -n raibid-system

# If image pull fails, use previous version
kubectl set image deployment/raibid-server raibid-server=raibid-server:previous -n raibid-system
```

#### P1: Redis Connection Lost

**Symptoms**:
- Jobs not queuing
- Agents not receiving work
- Connection errors in logs

**Diagnosis**:
```bash
# Check Redis pod
kubectl get pods -n raibid-system -l app=redis

# Test connection
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli PING

# Check logs
kubectl logs redis-master-0 -n raibid-system
```

**Mitigation**:
```bash
# Restart Redis
kubectl rollout restart statefulset redis-master -n raibid-system

# Verify connection
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli INFO replication
```

#### P1: Agent Pods Crashing

**Symptoms**:
- No available agents
- Jobs stuck in pending
- CrashLoopBackOff status

**Diagnosis**:
```bash
# Check pod status
kubectl get pods -n raibid-system -l app=raibid-agent

# Check logs
kubectl logs -l app=raibid-agent -n raibid-system --tail=100

# Describe failing pods
kubectl describe pod <agent-pod-name> -n raibid-system
```

**Mitigation**:
```bash
# Delete crashing pods (will restart)
kubectl delete pods -l app=raibid-agent -n raibid-system

# Scale to zero, then back up
kubectl scale scaledjob raibid-agent --replicas=0 -n raibid-system
sleep 30
kubectl scale scaledjob raibid-agent --replicas=1 -n raibid-system

# Check for resource constraints
kubectl top nodes
```

#### P2: High Job Failure Rate

**Symptoms**:
- > 10% of jobs failing
- Consistent failures across multiple repos

**Diagnosis**:
```bash
# List recent failures
raibid-cli job list --status failed --limit 20

# Check failure patterns
kubectl logs -l app=raibid-agent -n raibid-system | grep ERROR

# Check agent resources
kubectl top pods -n raibid-system -l app=raibid-agent
```

**Mitigation**:
```bash
# Retry failed jobs
raibid-cli job retry <job-id>

# If resource exhaustion, scale down temporarily
raibid-cli agent scale --count 4 --max 6

# Check for network issues
kubectl exec -it <agent-pod> -n raibid-system -- ping gitea.raibid-system.svc.cluster.local
```

### Rollback Procedures

#### Rollback Application Deployment

```bash
# View deployment history
kubectl rollout history deployment raibid-server -n raibid-system

# Rollback to previous version
kubectl rollout undo deployment raibid-server -n raibid-system

# Rollback to specific revision
kubectl rollout undo deployment raibid-server --to-revision=3 -n raibid-system

# Verify rollback
kubectl rollout status deployment raibid-server -n raibid-system
```

#### Rollback Configuration

```bash
# Restore from backup
cp ~/backups/raibid-ci/20251102/config.yaml ~/.config/raibid/config.yaml

# Restart services to pick up config
raibid-cli teardown all
raibid-cli setup all
```

## Troubleshooting Guide

### Decision Tree: Pod Not Starting

```
Pod in Pending/Error state
  |
  ├─> Check Events
  |    kubectl describe pod <pod-name> -n raibid-system
  |
  ├─> Image Pull Error?
  |    ├─> Yes: Check image exists, check registry access
  |    └─> No: Continue
  |
  ├─> Insufficient Resources?
  |    ├─> Yes: Scale down other pods, or add nodes
  |    └─> No: Continue
  |
  ├─> Volume Mount Error?
  |    ├─> Yes: Check PVC exists, check StorageClass
  |    └─> No: Continue
  |
  └─> Check Pod Logs
       kubectl logs <pod-name> -n raibid-system
```

### Decision Tree: High Latency

```
Slow job execution or API response
  |
  ├─> Check Node Resources
  |    kubectl top nodes
  |    ├─> CPU/Memory > 80%?
  |    |    └─> Scale down agents, restart pods
  |    └─> Normal: Continue
  |
  ├─> Check Network
  |    kubectl exec -it <pod> -- ping <service>
  |    ├─> High latency/packet loss?
  |    |    └─> Check network config, DNS
  |    └─> Normal: Continue
  |
  ├─> Check Redis Performance
  |    kubectl exec redis-master-0 -n raibid-system -- redis-cli INFO stats
  |    ├─> High command latency?
  |    |    └─> Check Redis resources, consider tuning
  |    └─> Normal: Continue
  |
  └─> Check Application Logs
       kubectl logs -l app=raibid-server -n raibid-system
```

### Common Issues and Solutions

#### Issue: Out of Disk Space

**Symptoms**: Pods fail to start, "no space left on device"

**Solution**:
```bash
# Check disk usage
df -h

# Clean Docker images
docker system prune -a -f

# Clean k3s images
sudo k3s crictl rmi --prune

# Clean old logs
sudo journalctl --vacuum-time=7d

# Increase disk or add volume
```

#### Issue: DNS Resolution Failing

**Symptoms**: Pods can't resolve service names

**Solution**:
```bash
# Check CoreDNS pods
kubectl get pods -n kube-system -l k8s-app=kube-dns

# Restart CoreDNS
kubectl rollout restart deployment coredns -n kube-system

# Test DNS from pod
kubectl run -it --rm debug --image=busybox --restart=Never -- nslookup redis.raibid-system.svc.cluster.local
```

#### Issue: TLS Certificate Errors

**Symptoms**: Webhook delivery fails, API requests fail

**Solution**:
```bash
# Check certificate expiry
sudo openssl x509 -in /var/lib/rancher/k3s/server/tls/server-ca.crt -noout -dates

# Rotate certificates
sudo k3s certificate rotate

# Restart k3s
sudo systemctl restart k3s
```

## Performance Tuning

### Optimize Agent Performance

**Build Cache Configuration**:
```yaml
# In agent configuration
agents:
  cache:
    enabled: true
    type: "local"  # or "s3"
    path: "/cache"
    size: "10Gi"
```

**Resource Allocation**:
```bash
# Adjust agent resource requests/limits
kubectl edit scaledjob raibid-agent -n raibid-system

# Optimize for CPU-bound workloads
resources:
  requests:
    cpu: "2000m"
    memory: "4Gi"
  limits:
    cpu: "4000m"
    memory: "8Gi"
```

### Optimize Redis Performance

```bash
# Connect to Redis
kubectl exec -it redis-master-0 -n raibid-system -- redis-cli

# Check slow log
SLOWLOG GET 10

# Tune maxmemory
CONFIG SET maxmemory 2gb
CONFIG SET maxmemory-policy allkeys-lru

# Enable AOF persistence for durability
CONFIG SET appendonly yes
```

### Optimize Kubernetes

**k3s Performance Tuning** (`/etc/rancher/k3s/config.yaml`):
```yaml
# Disable unused components
disable:
  - traefik
  - servicelb

# Adjust kubelet settings
kubelet-arg:
  - "max-pods=110"
  - "kube-api-qps=100"
  - "kube-api-burst=200"
```

**Resource Quotas** (prevent resource exhaustion):
```bash
kubectl apply -f - <<EOF
apiVersion: v1
kind: ResourceQuota
metadata:
  name: raibid-quota
  namespace: raibid-system
spec:
  hard:
    requests.cpu: "14"
    requests.memory: "104Gi"
    limits.cpu: "20"
    limits.memory: "128Gi"
    persistentvolumeclaims: "10"
EOF
```

## Security Operations

### Secret Management

**Rotate Secrets**:
```bash
# Generate new secret
NEW_SECRET=$(openssl rand -base64 32)

# Update secret in Kubernetes
kubectl create secret generic raibid-secrets \
  --from-literal=gitea-password="$NEW_SECRET" \
  --dry-run=client -o yaml | kubectl apply -f -

# Update environment variable
export RAIBID_GITEA_ADMIN_PASSWORD="$NEW_SECRET"

# Restart affected pods
kubectl rollout restart deployment raibid-server -n raibid-system
```

### Access Control

**Review RBAC**:
```bash
# List service accounts
kubectl get serviceaccounts -n raibid-system

# List roles and bindings
kubectl get roles,rolebindings -n raibid-system
kubectl describe rolebinding <binding-name> -n raibid-system
```

### Security Auditing

**Audit Kubernetes Resources**:
```bash
# Check for privileged pods
kubectl get pods --all-namespaces -o json | jq '.items[] | select(.spec.securityContext.privileged == true) | .metadata.name'

# Check for host network usage
kubectl get pods --all-namespaces -o json | jq '.items[] | select(.spec.hostNetwork == true) | .metadata.name'

# Review security policies
kubectl get podsecuritypolicies
```

**Review Logs for Suspicious Activity**:
```bash
# Check for failed authentication
kubectl logs -l app=raibid-server -n raibid-system | grep -i "unauthorized\|forbidden"

# Check for unusual API calls
kubectl logs -l app=raibid-server -n raibid-system | grep -E "POST|DELETE" | grep -v "200\|201\|204"
```

## Quick Reference Cheatsheet

### Essential Commands

```bash
# Health check
raibid-cli status
kubectl get pods -n raibid-system

# View logs
kubectl logs -f -l app=raibid-server -n raibid-system
kubectl logs -f -l app=raibid-agent -n raibid-system

# Restart components
kubectl rollout restart deployment raibid-server -n raibid-system
kubectl rollout restart statefulset redis-master -n raibid-system

# Scale agents
raibid-cli agent scale --count 5
kubectl scale scaledjob raibid-agent --replicas=5 -n raibid-system

# Check resources
kubectl top nodes
kubectl top pods -n raibid-system

# Check queue depth
kubectl exec redis-master-0 -n raibid-system -- redis-cli XLEN ci:jobs

# View job status
raibid-cli job list
raibid-cli job list --status failed
```

### Emergency Procedures

```bash
# Complete system restart
raibid-cli teardown all
sudo systemctl restart k3s
raibid-cli setup all

# Clear Redis queue (caution!)
kubectl exec redis-master-0 -n raibid-system -- redis-cli DEL ci:jobs

# Force delete stuck pod
kubectl delete pod <pod-name> -n raibid-system --force --grace-period=0

# Recover from backup
sudo k3s server --cluster-reset --cluster-reset-restore-path=/path/to/snapshot
```

### Useful kubectl Aliases

Add to `~/.bashrc` or `~/.zshrc`:

```bash
alias k='kubectl'
alias kgp='kubectl get pods -n raibid-system'
alias kgpw='kubectl get pods -n raibid-system -w'
alias kl='kubectl logs -f -n raibid-system'
alias kd='kubectl describe -n raibid-system'
alias ke='kubectl exec -it -n raibid-system'
```

---

**For additional support**, refer to:
- [Installation Guide](INSTALLATION.md)
- [API Documentation](API.md)
- [User Guide](USER_GUIDE.md)
- [GitHub Issues](https://github.com/raibid-labs/raibid-ci/issues)
