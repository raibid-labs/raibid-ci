# Operational Runbook

This runbook provides procedures for common operational scenarios based on failure recovery test findings.

## Common Failure Scenarios

### Redis Pod Failure

**Symptoms:**
- API returns 500 errors for job operations
- Job queue operations fail
- Logs show Redis connection errors

**Recovery Procedure:**
```bash
# 1. Check Redis pod status
kubectl get pods -n raibid-ci -l app=redis

# 2. Check Redis logs
kubectl logs -n raibid-ci -l app=redis --tail=100

# 3. If pod is CrashLooping, check persistent volume
kubectl get pvc -n raibid-ci
kubectl describe pvc redis-data -n raibid-ci

# 4. Manual restart (if needed)
kubectl delete pod -n raibid-ci -l app=redis

# 5. Verify recovery
kubectl wait --for=condition=ready pod -l app=redis -n raibid-ci --timeout=60s

# 6. Test Redis connectivity
redis-cli -h redis.raibid-ci.svc.cluster.local PING
```

**Expected Recovery Time:** < 60 seconds

**Data Loss Risk:**
- Low (if persistent volume configured)
- High (if using ephemeral storage)

---

### Gitea Service Unavailable

**Symptoms:**
- Webhook delivery fails
- Git clone operations timeout
- Registry push/pull fails

**Recovery Procedure:**
```bash
# 1. Check Gitea pod status
kubectl get pods -n raibid-ci -l app=gitea

# 2. Check Gitea logs
kubectl logs -n raibid-ci -l app=gitea --tail=100

# 3. Check database connectivity (if external DB)
kubectl exec -n raibid-ci -it deployment/gitea -- nc -zv postgres 5432

# 4. Restart Gitea if needed
kubectl rollout restart deployment/gitea -n raibid-ci

# 5. Verify recovery
kubectl wait --for=condition=ready pod -l app=gitea -n raibid-ci --timeout=120s

# 6. Test Gitea API
curl -I http://gitea.raibid-ci.svc.cluster.local:3000/api/v1/version
```

**Expected Recovery Time:** < 120 seconds

**Data Loss Risk:**
- None (repositories persist in PV)
- Configuration should be in ConfigMap

---

### API Server Not Responding

**Symptoms:**
- Webhook endpoint returns connection refused
- TUI cannot connect to API
- Health check fails

**Recovery Procedure:**
```bash
# 1. Check API server pods
kubectl get pods -n raibid-ci -l app=raibid-server

# 2. Check API server logs
kubectl logs -n raibid-ci -l app=raibid-server --tail=100

# 3. Check service endpoints
kubectl get endpoints -n raibid-ci raibid-server

# 4. Restart API server
kubectl rollout restart deployment/raibid-server -n raibid-ci

# 5. Verify recovery
kubectl wait --for=condition=ready pod -l app=raibid-server -n raibid-ci --timeout=60s

# 6. Test health endpoint
curl http://raibid-server.raibid-ci.svc.cluster.local:8080/health
```

**Expected Recovery Time:** < 60 seconds

**Data Loss Risk:**
- None (stateless service)
- In-flight requests may fail

---

### Agent Jobs Not Starting

**Symptoms:**
- Jobs stuck in "pending" status
- No agent pods spawning
- KEDA not scaling

**Recovery Procedure:**
```bash
# 1. Check job queue in Redis
redis-cli -h redis.raibid-ci.svc.cluster.local XLEN ci:jobs

# 2. Check KEDA ScaledJob
kubectl get scaledjob -n raibid-ci
kubectl describe scaledjob raibid-agent -n raibid-ci

# 3. Check KEDA operator status
kubectl get pods -n keda

# 4. Restart KEDA operator if needed
kubectl rollout restart deployment/keda-operator -n keda

# 5. Verify scaling
kubectl get pods -n raibid-ci -l app=raibid-agent

# 6. Manually trigger scaling (if needed)
kubectl patch scaledjob raibid-agent -n raibid-ci -p '{"spec":{"minReplicaCount":1}}'
```

**Expected Recovery Time:** < 90 seconds

**Data Loss Risk:**
- None (jobs remain queued)

---

### Disk Space Exhaustion

**Symptoms:**
- Build failures with "No space left on device"
- Pod evictions
- Log write failures

**Recovery Procedure:**
```bash
# 1. Check node disk usage
kubectl top nodes
kubectl describe nodes | grep -A 5 "Allocated resources"

# 2. Identify large consumers
kubectl exec -n raibid-ci -it deployment/raibid-server -- df -h

# 3. Clean up old build artifacts
# Run cache cleanup job (if configured)
kubectl create job --from=cronjob/cache-cleanup cache-cleanup-manual -n raibid-ci

# 4. Prune Docker images on nodes
# SSH to each node and run:
docker system prune -af --volumes

# 5. Expand persistent volumes if needed
kubectl edit pvc <pvc-name> -n raibid-ci
# Update storage size and apply
```

**Expected Recovery Time:** Varies (10-30 minutes)

**Prevention:**
- Configure storage quotas
- Implement automatic cache cleanup
- Monitor disk usage with alerts

---

### Network Connectivity Issues

**Symptoms:**
- Services cannot reach each other
- External webhook delivery fails
- Git clone timeouts

**Recovery Procedure:**
```bash
# 1. Test pod-to-pod connectivity
kubectl run -n raibid-ci test-pod --image=busybox --rm -it -- /bin/sh
# Inside pod:
# wget -O- http://raibid-server:8080/health
# nslookup redis

# 2. Check network policies
kubectl get networkpolicies -n raibid-ci

# 3. Check CNI plugin status
kubectl get pods -n kube-system | grep -E "calico|flannel|weave"

# 4. Check service DNS
kubectl run -n raibid-ci test-dns --image=busybox --rm -it -- nslookup redis.raibid-ci.svc.cluster.local

# 5. Restart CoreDNS if needed
kubectl rollout restart deployment/coredns -n kube-system
```

**Expected Recovery Time:** < 60 seconds

---

### Cascading Failures

**Symptoms:**
- Multiple components failing simultaneously
- System-wide degradation
- Cluster resource exhaustion

**Recovery Procedure:**
```bash
# 1. Assess overall cluster health
kubectl get nodes
kubectl top nodes
kubectl get pods -A | grep -v Running

# 2. Identify root cause
kubectl get events -A --sort-by='.lastTimestamp' | tail -50

# 3. Restart critical services in order:
# Step 1: Redis (data layer)
kubectl rollout restart deployment/redis -n raibid-ci

# Step 2: API Server (application layer)
kubectl rollout restart deployment/raibid-server -n raibid-ci

# Step 3: Gitea (source control)
kubectl rollout restart deployment/gitea -n raibid-ci

# 4. Verify all pods are running
kubectl get pods -n raibid-ci

# 5. Run health checks
curl http://raibid-server.raibid-ci.svc.cluster.local:8080/health

# 6. Monitor for stability
watch kubectl get pods -n raibid-ci
```

**Expected Recovery Time:** < 180 seconds

---

## Monitoring and Alerts

### Key Metrics to Monitor
- Pod restart count
- API response times
- Redis queue depth
- Node disk usage
- Memory usage per pod

### Recommended Alerts
- Pod restart > 3 times in 5 minutes
- API latency P95 > 2s
- Redis connection failures
- Disk usage > 85%
- Agent spawn time > 60s

### Health Check Endpoints
- API Server: `GET /health`
- Redis: `redis-cli PING`
- Gitea: `GET /api/v1/version`

---

## Backup and Restore

### Critical Data
1. **Gitea repositories** - Persistent volume at `/data/git/repositories`
2. **Redis data** - Optional persistent volume
3. **Configuration** - Kubernetes ConfigMaps and Secrets

### Backup Procedure
```bash
# Gitea repository backup
kubectl exec -n raibid-ci deployment/gitea -- tar czf /tmp/repos.tar.gz /data/git/repositories
kubectl cp raibid-ci/gitea-pod:/tmp/repos.tar.gz ./gitea-backup-$(date +%Y%m%d).tar.gz

# ConfigMaps and Secrets
kubectl get configmaps -n raibid-ci -o yaml > configmaps-backup.yaml
kubectl get secrets -n raibid-ci -o yaml > secrets-backup.yaml
```

### Restore Procedure
```bash
# Restore Gitea repositories
kubectl cp ./gitea-backup-*.tar.gz raibid-ci/gitea-pod:/tmp/repos.tar.gz
kubectl exec -n raibid-ci deployment/gitea -- tar xzf /tmp/repos.tar.gz -C /

# Restore configuration
kubectl apply -f configmaps-backup.yaml
kubectl apply -f secrets-backup.yaml
```

---

## Emergency Contacts

- **On-Call Engineer**: [Contact info]
- **Infrastructure Team**: [Contact info]
- **Escalation Path**: [Escalation procedure]

---

## Post-Incident Checklist

After resolving an incident:

1. Document timeline of events
2. Identify root cause
3. Update runbook with lessons learned
4. Add monitoring/alerting if gaps found
5. Schedule post-mortem review
6. Implement preventive measures

---

## Useful Commands

### Quick Status Check
```bash
kubectl get all -n raibid-ci
kubectl top pods -n raibid-ci
kubectl get events -n raibid-ci --sort-by='.lastTimestamp' | tail -20
```

### Log Aggregation
```bash
# All logs from last hour
kubectl logs -n raibid-ci --all-containers --since=1h -l app=raibid-server

# Follow logs in real-time
kubectl logs -n raibid-ci -f -l app=raibid-agent
```

### Force Restart Everything
```bash
# Nuclear option - restart all deployments
for deploy in $(kubectl get deployments -n raibid-ci -o name); do
    kubectl rollout restart $deploy -n raibid-ci
done
```

---

**Last Updated:** 2025-11-03
**Version:** 1.0
**Maintained By:** raibid-ci Team
