# Failure Recovery and Chaos Engineering Tests

Test suite for validating system resilience and recovery under failure conditions.

## Test Scenarios

### Pod Deletion Tests
- **Redis Pod Deletion** - Test data persistence and recovery
- **Gitea Pod Restart** - Test repository data integrity
- **Agent Termination** - Test job recovery after agent failure
- **API Server Crash** - Test service availability and recovery

### Network Failure Tests
- **Network Partition** - Simulate network isolation
- **Connection Timeout** - Test timeout handling

### Resource Exhaustion Tests
- **Disk Full** - Test behavior when disk is full
- **OOM Kill** - Test recovery from out-of-memory kills
- **CPU Throttling** - Test performance under CPU constraints

### Component Failures
- **KEDA Scaler Failure** - Test autoscaling recovery
- **Cascading Failures** - Test multiple simultaneous failures

## Expected Recovery Times

### Individual Component Failures
- **Redis**: < 60s recovery
- **Gitea**: < 120s recovery
- **API Server**: < 60s recovery
- **KEDA**: < 30s recovery

### Data Persistence
- **Redis**: Depends on volume configuration
- **Gitea**: Should preserve all repository data
- **Job Queue**: Should not lose queued jobs

### Cascading Failures
- **Multiple Components**: < 180s total recovery

## Running Tests

```bash
# Run all failure recovery tests
just test-failure

# Run specific test
TEST_EXTERNAL=1 cargo test --test failure_test test_redis_pod_deletion -- --ignored --nocapture

# Run with custom namespace
export RAIBID_NAMESPACE=custom-namespace
just test-failure
```

## Requirements

- Kubernetes cluster (k3s or full)
- kubectl configured and authenticated
- raibid-ci deployed in cluster
- Sufficient cluster resources

## Safety Considerations

These tests are **destructive** and should only be run in:
- Development environments
- Staging environments
- Dedicated test clusters

**NEVER** run these tests in production.

## Test Behavior

### Non-Destructive Checks
- Health endpoint verification
- Pod readiness checks
- Service availability monitoring

### Destructive Operations
- Pod deletions (kubectl delete pod)
- Network policy application
- Resource limit enforcement

## Expected Behavior

### Successful Recovery
- Pods restart automatically
- Services become available within timeout
- Data is preserved (if persistent volumes configured)
- Jobs are requeued or marked as failed
- System continues normal operation

### Data Loss Scenarios
- Ephemeral storage: Data loss expected
- Persistent volumes: Data should persist
- In-flight jobs: May need retry

## Troubleshooting

### Pod Not Recovering
```bash
# Check pod events
kubectl describe pod -n raibid-ci <pod-name>

# Check controller logs
kubectl logs -n raibid-ci -l app=<component>

# Check resource constraints
kubectl top pods -n raibid-ci
```

### Data Loss
```bash
# Check persistent volume claims
kubectl get pvc -n raibid-ci

# Check volume mounts
kubectl describe pod -n raibid-ci <pod-name> | grep -A 10 Volumes

# Verify backup configuration
```

### Slow Recovery
```bash
# Check image pull time
kubectl get events -n raibid-ci --sort-by='.lastTimestamp'

# Check pod scheduling
kubectl get pods -n raibid-ci -o wide

# Check node resources
kubectl top nodes
```

## Operational Runbook

See `RUNBOOK.md` for detailed operational procedures derived from these tests.

## CI Integration

Failure tests run in CI:
- Weekly chaos engineering schedule
- On infrastructure changes
- Manual workflow dispatch

Tests help build confidence in:
- Kubernetes configuration
- Persistent volume setup
- Restart policies
- Resource limits
- Health checks
