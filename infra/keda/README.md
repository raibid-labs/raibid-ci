# KEDA Infrastructure

Kubernetes Event-Driven Autoscaling for CI agents.

## Overview

KEDA enables event-driven autoscaling of raibid-ci agents based on Redis Streams queue depth. When jobs are queued, KEDA automatically scales up agent pods. When the queue is empty, agents scale down to zero.

## Manifests

- `namespace.yaml` - KEDA namespace
- `values.yaml` - Helm chart values
- `scaledobject.yaml` - ScaledObject for Redis Streams
- `triggerauth.yaml` - TriggerAuthentication for Redis credentials

## Deployment

### Via raibid-cli

```bash
raibid-cli setup keda
```

### Via Helm

```bash
# Add Helm repository
helm repo add kedacore https://kedacore.github.io/charts
helm repo update

# Create namespace
kubectl apply -f namespace.yaml

# Install KEDA
helm upgrade --install raibid-keda kedacore/keda \
  --namespace keda \
  --values values.yaml \
  --wait

# Apply ScaledObject and TriggerAuthentication
kubectl apply -f triggerauth.yaml
kubectl apply -f scaledobject.yaml
```

## Configuration

### Default Settings

- **Namespace**: `keda`
- **Log Level**: `info`
- **Metrics Server**: Enabled
- **Min Replicas**: 0 (scale-to-zero)
- **Max Replicas**: 10
- **Polling Interval**: 10 seconds

### Scaling Behavior

- **Scale Up**: When `pendingEntriesCount > 0` in Redis Stream
- **Scale Down**: When queue is empty (cooldown after 5 minutes)
- **Scale Target**: `raibid-ci-agent` Deployment

## ScaledObject

The ScaledObject defines how KEDA scales the agent deployment:

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
  minReplicaCount: 0
  maxReplicaCount: 10
  pollingInterval: 10
  triggers:
  - type: redis-streams
    metadata:
      address: raibid-redis-master.raibid-redis.svc.cluster.local:6379
      stream: raibid:jobs
      consumerGroup: raibid-workers
      pendingEntriesCount: "1"
```

## Validation

```bash
# Check KEDA pods
kubectl get pods -n keda

# Expected pods:
# - keda-operator
# - keda-metrics-apiserver
# - keda-admission-webhooks

# Check CRDs
kubectl get crd | grep keda

# Check ScaledObject
kubectl get scaledobject -n raibid-ci

# View ScaledObject status
kubectl describe scaledobject raibid-ci-agent-scaler -n raibid-ci
```

## Testing Autoscaling

### Add Test Job

```bash
# Port forward to Redis
kubectl port-forward -n raibid-redis svc/raibid-redis-master 6379:6379

# Add job to stream
redis-cli -a $PASSWORD XADD raibid:jobs * \
  job_id test-001 \
  repo raibid/test \
  branch main

# Watch deployment scale
kubectl get deployment -n raibid-ci -w

# You should see replicas increase from 0 to 1
```

### Monitor Scaling

```bash
# View KEDA operator logs
kubectl logs -n keda -l app=keda-operator -f

# View HPA created by KEDA
kubectl get hpa -n raibid-ci

# Check scaling events
kubectl get events -n raibid-ci --sort-by='.lastTimestamp' | grep ScaledObject
```

## Troubleshooting

### ScaledObject Not Scaling

```bash
# Check ScaledObject status
kubectl describe scaledobject raibid-ci-agent-scaler -n raibid-ci

# Check KEDA operator logs
kubectl logs -n keda -l app=keda-operator --tail=100

# Verify Redis connection
kubectl run redis-test --rm -it --image=redis -- redis-cli \
  -h raibid-redis-master.raibid-redis.svc.cluster.local \
  -a $PASSWORD PING

# Check stream exists
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a $PASSWORD EXISTS raibid:jobs
```

### Authentication Issues

```bash
# Check TriggerAuthentication
kubectl get triggerauthentication -n raibid-ci

# Verify secret exists
kubectl get secret raibid-redis-auth -n raibid-ci

# Check secret contains password
kubectl get secret raibid-redis-auth -n raibid-ci -o jsonpath='{.data.password}' | base64 -d
```

### KEDA Pods Not Ready

```bash
# Check pod status
kubectl get pods -n keda

# Describe failing pod
kubectl describe pod -n keda <pod-name>

# Check logs
kubectl logs -n keda <pod-name>

# Check webhook service
kubectl get svc -n keda
```

## Advanced Configuration

### Custom Scaling Parameters

Edit `scaledobject.yaml` to customize:

```yaml
spec:
  minReplicaCount: 1  # Keep at least 1 replica
  maxReplicaCount: 20  # Allow up to 20 replicas
  pollingInterval: 5  # Check every 5 seconds
  cooldownPeriod: 300  # Wait 5 minutes before scale down
  triggers:
  - type: redis-streams
    metadata:
      pendingEntriesCount: "5"  # Scale when 5+ jobs pending
      lagCount: "10"  # Additional lag-based scaling
```

### Multiple Triggers

Add additional scaling triggers:

```yaml
triggers:
- type: redis-streams
  metadata:
    stream: raibid:jobs
- type: cron
  metadata:
    timezone: America/New_York
    start: 0 8 * * 1-5  # Scale up at 8 AM weekdays
    end: 0 18 * * 1-5  # Scale down at 6 PM
    desiredReplicas: "5"
```

## Uninstallation

```bash
# Via raibid-cli
raibid-cli teardown keda

# Manual
kubectl delete scaledobject raibid-ci-agent-scaler -n raibid-ci
kubectl delete triggerauthentication raibid-redis-trigger-auth -n raibid-ci
helm uninstall raibid-keda -n keda
kubectl delete namespace keda
```

## References

- [KEDA Documentation](https://keda.sh/docs/)
- [Redis Streams Scaler](https://keda.sh/docs/scalers/redis-streams/)
- [KEDA Helm Chart](https://github.com/kedacore/charts)
- [ScaledObject Spec](https://keda.sh/docs/concepts/scaling-deployments/)
