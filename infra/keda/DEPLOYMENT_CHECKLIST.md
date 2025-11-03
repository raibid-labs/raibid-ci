# KEDA Deployment Checklist

Use this checklist when deploying KEDA for raibid-ci.

## Pre-Deployment

- [ ] k3s cluster is running (`kubectl cluster-info`)
- [ ] Redis is deployed and healthy (`kubectl get pods -n raibid-redis`)
- [ ] Helm 3.x is installed (`helm version`)
- [ ] kubectl has cluster access
- [ ] Sufficient cluster resources (min 500m CPU, 1Gi RAM available)

## Deployment Steps

### 1. Deploy KEDA Operator

- [ ] Add KEDA Helm repository
  ```bash
  helm repo add kedacore https://kedacore.github.io/charts
  helm repo update
  ```

- [ ] Create namespaces
  ```bash
  kubectl apply -f namespace.yaml
  ```

- [ ] Install KEDA via Helm
  ```bash
  helm upgrade --install raibid-keda kedacore/keda \
    --namespace keda \
    --version 2.12.0 \
    --values values.yaml \
    --wait
  ```

- [ ] Verify KEDA pods are running
  ```bash
  kubectl get pods -n keda
  ```
  Expected: keda-operator, keda-metrics-apiserver, keda-admission-webhooks (all Running)

- [ ] Verify CRDs are installed
  ```bash
  kubectl get crd | grep keda
  ```
  Expected: scaledobjects.keda.sh, scaledjobs.keda.sh, triggerauthentications.keda.sh

### 2. Configure Authentication

- [ ] Ensure Redis auth secret exists in raibid-ci namespace
  ```bash
  kubectl get secret raibid-redis-auth -n raibid-ci
  ```

- [ ] If missing, create Redis auth secret:
  ```bash
  kubectl create secret generic raibid-redis-auth \
    -n raibid-ci \
    --from-literal=password=<redis-password> \
    --from-literal=address=raibid-redis-master.raibid-redis.svc.cluster.local:6379
  ```

- [ ] Deploy TriggerAuthentication
  ```bash
  kubectl apply -f triggerauth.yaml
  ```

- [ ] Verify TriggerAuthentication
  ```bash
  kubectl get triggerauthentication -n raibid-ci
  kubectl describe triggerauthentication raibid-redis-trigger-auth -n raibid-ci
  ```

### 3. Deploy ScaledJob

- [ ] Review ScaledJob configuration
  - Max replicas: 10
  - Polling interval: 10s
  - Container image: ghcr.io/raibid-labs/rust-agent:latest
  - Resource limits appropriate

- [ ] Deploy ScaledJob
  ```bash
  kubectl apply -f scaledjob.yaml
  ```

- [ ] Verify ScaledJob is created
  ```bash
  kubectl get scaledjob raibid-ci-agent -n raibid-ci
  kubectl describe scaledjob raibid-ci-agent -n raibid-ci
  ```

### 4. Create Redis Consumer Group

- [ ] Create consumer group in Redis (if not exists)
  ```bash
  kubectl exec -n raibid-redis raibid-redis-master-0 -- \
    redis-cli -a <password> XGROUP CREATE raibid:jobs raibid-workers 0 MKSTREAM
  ```
  Note: Ignore "BUSYGROUP" error if group exists

## Post-Deployment Validation

### Automated Validation

- [ ] Run validation script
  ```bash
  ./validate-keda.sh
  ```
  Expected: All checks pass

### Manual Validation

- [ ] Check KEDA operator logs for errors
  ```bash
  kubectl logs -n keda -l app=keda-operator --tail=50
  ```

- [ ] Verify no error events
  ```bash
  kubectl get events -n keda --sort-by='.lastTimestamp' | tail -10
  kubectl get events -n raibid-ci --sort-by='.lastTimestamp' | tail -10
  ```

- [ ] Check ScaledJob status
  ```bash
  kubectl get scaledjob raibid-ci-agent -n raibid-ci -o yaml
  ```
  Look for: `status.conditions` showing Ready=True

## Functional Testing

### Test Scale From Zero

- [ ] Ensure no jobs are running
  ```bash
  kubectl get jobs -n raibid-ci
  ```

- [ ] Add test job to Redis
  ```bash
  kubectl port-forward -n raibid-redis svc/raibid-redis-master 6379:6379 &
  redis-cli -a <password> XADD raibid:jobs '*' \
    job_id test-001 \
    repo raibid-labs/test \
    branch main \
    commit abc123
  ```

- [ ] Watch KEDA create job (within 15 seconds)
  ```bash
  kubectl get jobs -n raibid-ci -w
  ```
  Expected: New job appears

- [ ] Watch pod spawn
  ```bash
  kubectl get pods -n raibid-ci -w
  ```
  Expected: New pod in Running state

- [ ] Clean up test
  ```bash
  kubectl delete jobs -n raibid-ci -l app=raibid-ci-agent
  ```

### Test Autoscaling

- [ ] Run autoscaling test script
  ```bash
  ./test-autoscaling.sh 5
  ```
  Expected: 5 jobs created, pods spawn, scaling works

## Performance Verification

- [ ] Measure scale-up latency
  - [ ] Queue detection: < 15 seconds
  - [ ] Job creation: < 5 seconds
  - [ ] Pod start (cached image): < 30 seconds
  - [ ] Total latency: < 50 seconds

- [ ] Verify resource usage
  ```bash
  kubectl top pods -n keda
  kubectl top pods -n raibid-ci
  ```

- [ ] Check KEDA operator resource consumption
  Expected: < 100m CPU, < 200Mi RAM

## Monitoring Setup

- [ ] Configure log aggregation for KEDA operator
- [ ] Set up alerts for KEDA failures
- [ ] Create dashboard for queue depth and scaling metrics
- [ ] Configure job success/failure tracking

## Documentation

- [ ] Update deployment runbook with any environment-specific notes
- [ ] Document any custom configuration changes
- [ ] Record KEDA operator version deployed
- [ ] Note any known issues or workarounds

## Troubleshooting Checklist

If issues occur, check:

- [ ] KEDA operator logs: `kubectl logs -n keda -l app=keda-operator`
- [ ] ScaledJob events: `kubectl describe scaledjob raibid-ci-agent -n raibid-ci`
- [ ] Redis connectivity: `kubectl exec -n raibid-redis raibid-redis-master-0 -- redis-cli PING`
- [ ] TriggerAuth secret: `kubectl get secret raibid-redis-auth -n raibid-ci`
- [ ] Consumer group exists: `kubectl exec -n raibid-redis raibid-redis-master-0 -- redis-cli XINFO GROUPS raibid:jobs`
- [ ] Resource quotas: `kubectl describe resourcequota -n raibid-ci`
- [ ] Image pull secrets: `kubectl get pods -n raibid-ci` (check for ImagePullBackOff)

## Rollback Procedure

If deployment fails and rollback is needed:

1. [ ] Delete ScaledJob
   ```bash
   kubectl delete scaledjob raibid-ci-agent -n raibid-ci
   ```

2. [ ] Delete TriggerAuthentication
   ```bash
   kubectl delete triggerauthentication raibid-redis-trigger-auth -n raibid-ci
   ```

3. [ ] Uninstall KEDA
   ```bash
   helm uninstall raibid-keda -n keda
   ```

4. [ ] Delete namespace (if needed)
   ```bash
   kubectl delete namespace keda
   ```

5. [ ] Review logs and errors before redeployment

## Sign-Off

- [ ] Deployment completed by: _________________ Date: _________
- [ ] Validation completed by: _________________ Date: _________
- [ ] Approved for production: ________________ Date: _________

## Notes

Record any deployment-specific notes, issues, or deviations from standard procedure:

```
_________________________________________________________________________

_________________________________________________________________________

_________________________________________________________________________
```

