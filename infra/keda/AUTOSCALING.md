# KEDA Autoscaling Behavior Documentation

## Overview

This document describes how KEDA autoscales raibid-ci agents based on Redis Streams queue depth using ScaledJob.

## Scaling Lifecycle

### 1. Queue Empty (Scale-to-Zero)

**State**: No jobs in Redis Streams
**KEDA Action**: No Kubernetes Jobs exist
**Resource Usage**: Zero (only KEDA operator running)

```
Redis Queue: []
Kubernetes Jobs: 0
Agent Pods: 0
```

### 2. Job Added to Queue

**Event**: Developer pushes code, triggering CI job
**Action**: Job dispatcher adds entry to Redis Streams

```bash
XADD raibid:jobs * \
  job_id abc123 \
  repo raibid-labs/app \
  branch feature/new \
  commit def456
```

**Redis State**:
```
Stream: raibid:jobs
Length: 1
Pending Entries: 1 (in consumer group raibid-workers)
```

### 3. KEDA Detects Job (Polling)

**Timing**: Within 10 seconds (polling interval)
**Action**: KEDA queries Redis Streams trigger

KEDA runs this logic:
```
pending_count = XPENDING raibid:jobs raibid-workers
if pending_count >= pendingEntriesCount (1):
    create_kubernetes_job()
```

**KEDA Logs**:
```
[INFO] scaledjob/raibid-ci-agent: Scaling from 0 to 1 jobs
[INFO] redis-streams: pending entries = 1, threshold = 1
```

### 4. Kubernetes Job Created

**Timing**: Within 15 seconds of queue detection
**Action**: KEDA creates Job from ScaledJob template

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: raibid-ci-agent-abc123
  namespace: raibid-ci
  ownerReferences:
  - apiVersion: keda.sh/v1alpha1
    kind: ScaledJob
    name: raibid-ci-agent
spec:
  template:
    spec:
      containers:
      - name: rust-agent
        image: ghcr.io/raibid-labs/rust-agent:latest
        # ... environment, resources, etc
```

### 5. Pod Scheduled and Running

**Timing**: Within 30 seconds (image pull + pod start)
**Actions**:
1. Kubernetes scheduler assigns pod to node
2. Kubelet pulls container image (if not cached)
3. Container starts, agent begins execution

**Agent Actions**:
```
1. Connect to Redis
2. Read from consumer group: XREADGROUP raibid-workers consumer1 raibid:jobs
3. Process job (build, test, etc)
4. Acknowledge job: XACK raibid:jobs raibid-workers <job-id>
5. Exit with code 0 (success) or 1 (failure)
```

### 6. Job Completion

**Timing**: Variable (depends on job duration)
**Actions**:
- Pod exits
- Job status updated to Complete or Failed
- Pod enters Completed state

**Kubernetes State**:
```
Job: raibid-ci-agent-abc123
  Status: Complete
  Succeeded: 1
  Failed: 0
  Start Time: 2025-11-01T10:00:00Z
  Completion Time: 2025-11-01T10:05:30Z
  Duration: 5m30s
```

### 7. Job History Management

**KEDA Action**: Keep completed jobs based on history limits

```yaml
successfulJobsHistoryLimit: 3  # Keep last 3 successful
failedJobsHistoryLimit: 5      # Keep last 5 failed
```

Old jobs are automatically deleted to prevent resource accumulation.

### 8. Return to Scale-to-Zero

**Condition**: No pending entries in Redis Streams
**Timing**: Immediate (no cooldown for ScaledJob)
**Action**: No new jobs created

```
Redis Queue: [] (all processed)
Kubernetes Jobs: 3 (completed, kept for history)
Active Pods: 0
```

## Scaling Scenarios

### Scenario A: Single Job

```
Time  Queue  Jobs  Pods  Action
0s    0      0     0     Idle
10s   1      0     0     KEDA detects job
15s   1      1     0     Job created
20s   1      1     1     Pod running
5m    0      1     0     Job complete, pod terminated
```

### Scenario B: Burst of Jobs

```
Time  Queue  Jobs  Pods  Action
0s    0      0     0     Idle
5s    10     0     0     10 jobs added
15s   10     10    5     KEDA creates 10 jobs, 5 pods running
20s   10     10    10    All 10 pods running (max replicas)
25s   8      10    10    2 jobs complete
30s   5      10    10    5 jobs complete
35s   2      10    8     8 jobs complete
40s   0      10    2     All jobs complete, last 2 pods finishing
45s   0      10    0     All pods terminated
```

### Scenario C: Continuous Flow

```
Time  Queue  Jobs  Pods  Action
0s    5      5     5     5 jobs processing
10s   5      5     5     2 complete, 2 new jobs added
20s   5      5     5     Steady state (jobs in = jobs out)
30s   8      8     8     Burst: 3 new jobs added
40s   10     10    10    Max replicas reached, 2 jobs queued
50s   7      10    10    3 jobs complete
60s   5      7     7     Back to steady state
```

### Scenario D: Overload (Queue Backup)

```
Time  Queue  Jobs  Pods  Action
0s    50     0     0     Massive job backlog
10s   50     10    5     KEDA creates max jobs (10), 5 running
20s   50     10    10    All 10 pods running
5m    45     10    10    5 jobs complete, 5 new jobs started
10m   40     10    10    Still at max capacity
15m   30     10    10    Processing continues
...
Queue processes at: 10 jobs per average_job_duration
```

**Key Point**: Queue will process at maximum throughput (10 concurrent jobs). Excess jobs wait in Redis.

## Scaling Triggers

### Redis Streams Trigger

KEDA queries Redis for pending entries:

```bash
# What KEDA runs
XPENDING raibid:jobs raibid-workers

# Returns:
# [
#   lowest_pending_id,
#   highest_pending_id,
#   pending_count,
#   consumers
# ]
```

**Scaling Logic**:
```python
def should_scale():
    pending = get_pending_count()
    running = get_running_jobs()

    desired = min(pending, max_replicas)

    if desired > running:
        create_jobs(desired - running)

    # ScaledJob doesn't scale down - jobs complete naturally
```

### Trigger Metadata

```yaml
metadata:
  address: raibid-redis-master.raibid-redis.svc.cluster.local:6379
  stream: raibid:jobs
  consumerGroup: raibid-workers
  pendingEntriesCount: "1"  # Minimum to trigger
  streamLength: "5"         # Total stream length threshold
  lagCount: "5"             # Consumer lag threshold
  activationLagCount: "0"   # Start scaling immediately
```

### Multi-Metric Scaling

KEDA can combine multiple metrics:

```yaml
triggers:
- type: redis-streams
  metadata:
    pendingEntriesCount: "1"
- type: cron
  metadata:
    timezone: UTC
    start: 0 8 * * 1-5    # 8 AM weekdays
    end: 0 18 * * 1-5     # 6 PM weekdays
    desiredReplicas: "5"  # Keep 5 warm during business hours
```

## Scaling Strategies

### Default Strategy

**Algorithm**: 1 job per pending entry
**Behavior**: Conservative, predictable

```yaml
scalingStrategy:
  strategy: "default"
```

**Example**:
- 5 pending entries → 5 Jobs created
- 20 pending entries, max 10 → 10 Jobs created

### Accurate Strategy

**Algorithm**: Precise calculation, minimal overprovision
**Behavior**: Slower to scale, most efficient

```yaml
scalingStrategy:
  strategy: "accurate"
```

**Use Case**: Cost-sensitive environments, predictable workloads

### Eager Strategy

**Algorithm**: Aggressive scaling
**Behavior**: Fast response, may overprovision

```yaml
scalingStrategy:
  strategy: "eager"
```

**Use Case**: Time-sensitive CI, fast feedback required

### Custom Strategy

**Algorithm**: User-defined logic

```yaml
scalingStrategy:
  strategy: "custom"
  customScalingQueueLengthDeduction: 1
  customScalingRunningJobPercentage: "0.5"
  pendingPodConditions:
  - "Ready"
  - "PodScheduled"
```

**Parameters**:
- `customScalingQueueLengthDeduction`: Subtract from queue length (accounts for already-running jobs)
- `customScalingRunningJobPercentage`: Consider percentage of running jobs
- `pendingPodConditions`: Wait for pod conditions before counting as "running"

## Performance Characteristics

### Latency Metrics

| Metric | Target | Actual (Typical) |
|--------|--------|------------------|
| Queue detection | 10s | 5-15s (polling interval) |
| Job creation | 5s | 2-5s |
| Pod start (cached image) | 10s | 5-15s |
| Pod start (pull image) | 60s | 30-120s |
| **Total (cached)** | **25s** | **15-35s** |
| **Total (uncached)** | **75s** | **45-150s** |

### Throughput

**Maximum Throughput**: `max_replicas / average_job_duration`

Examples:
- 10 max replicas, 5-minute jobs: **2 jobs/minute = 120 jobs/hour**
- 10 max replicas, 30-second jobs: **20 jobs/minute = 1,200 jobs/hour**

### Resource Efficiency

**Idle Cost**: $0 (scale-to-zero)
**Active Cost**: Only running jobs
**Overhead**: KEDA operator (~250m CPU, ~320Mi RAM)

## Scaling Policies

### Job History Retention

```yaml
successfulJobsHistoryLimit: 3
failedJobsHistoryLimit: 5
```

**Why**:
- Keep recent jobs for debugging
- Prevent resource accumulation
- Failed jobs retained longer for troubleshooting

### Polling Interval

```yaml
pollingInterval: 10  # seconds
```

**Trade-offs**:
- **Lower (5s)**: Faster response, higher Redis load
- **Higher (30s)**: Lower overhead, slower response

**Recommendation**: 10s for most workloads

### Maximum Replicas

```yaml
maxReplicaCount: 10
```

**Calculation**:
```python
max_replicas = min(
    available_cluster_resources / job_resource_request,
    desired_parallelism,
    cost_budget_limit
)
```

**DGX Spark Example** (20 cores, 128GB RAM):
```python
# Each job: 1 CPU, 2GB RAM
max_cpu_replicas = 20 / 1 = 20
max_mem_replicas = 128 / 2 = 64
max_replicas = min(20, 64) = 20

# With 50% reserved for system:
max_replicas = 10
```

## Autoscaling Best Practices

### 1. Right-Size Resource Requests

```yaml
resources:
  requests:
    cpu: 1000m      # Based on actual usage
    memory: 2Gi     # 80% of typical usage
  limits:
    cpu: 4000m      # 150-200% of requests
    memory: 8Gi     # 150-200% of requests
```

### 2. Use Consumer Groups Correctly

```bash
# Create consumer group before deploying ScaledJob
redis-cli XGROUP CREATE raibid:jobs raibid-workers 0 MKSTREAM
```

### 3. Monitor Queue Depth

```bash
# Watch queue depth
watch -n 5 'redis-cli XLEN raibid:jobs'

# Check pending entries
redis-cli XPENDING raibid:jobs raibid-workers SUMMARY
```

### 4. Set Appropriate Job Timeouts

```yaml
jobTargetRef:
  template:
    spec:
      activeDeadlineSeconds: 3600  # Kill after 1 hour
      backoffLimit: 2               # Retry failed jobs twice
```

### 5. Implement Health Checks

```yaml
containers:
- name: rust-agent
  livenessProbe:
    exec:
      command: ["/bin/sh", "-c", "pgrep -f rust-agent"]
    initialDelaySeconds: 30
    periodSeconds: 10
  readinessProbe:
    exec:
      command: ["/bin/sh", "-c", "test -f /tmp/healthy"]
    initialDelaySeconds: 5
    periodSeconds: 5
```

### 6. Use Pod Anti-Affinity

Spread jobs across nodes:

```yaml
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchLabels:
            app: raibid-ci-agent
        topologyKey: kubernetes.io/hostname
```

### 7. Enable Metrics Collection

```yaml
# Agent should expose metrics
- name: METRICS_ENABLED
  value: "true"
- name: METRICS_PORT
  value: "9090"
```

## Troubleshooting Scaling Issues

### Jobs Not Scaling

**Symptom**: Queue has jobs, but no Kubernetes Jobs created

**Debug Steps**:
```bash
# 1. Check KEDA operator logs
kubectl logs -n keda -l app=keda-operator --tail=100

# 2. Check ScaledJob status
kubectl describe scaledjob raibid-ci-agent -n raibid-ci

# 3. Verify trigger authentication
kubectl get secret raibid-redis-auth -n raibid-ci

# 4. Test Redis connection
kubectl run redis-test --rm -it --image=redis -- \
  redis-cli -h raibid-redis-master.raibid-redis.svc.cluster.local PING

# 5. Check pending entries
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli XPENDING raibid:jobs raibid-workers
```

### Slow Scaling

**Symptom**: Jobs created but pods take too long to start

**Debug Steps**:
```bash
# Check pod events
kubectl describe pod -n raibid-ci <pod-name>

# Common issues:
# - Image pull (pull image to all nodes beforehand)
# - Resource constraints (check node resources)
# - Scheduling delays (check node availability)
```

### Stuck Jobs

**Symptom**: Jobs running but never complete

**Debug Steps**:
```bash
# Check job logs
kubectl logs -n raibid-ci job/<job-name>

# Check Redis ACK
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli XPENDING raibid:jobs raibid-workers

# Common issues:
# - Agent not calling XACK
# - Agent crashed before completion
# - Redis connection lost
```

## Metrics and Monitoring

### Key Metrics to Track

1. **Queue Depth**: `XLEN raibid:jobs`
2. **Pending Entries**: `XPENDING raibid:jobs raibid-workers`
3. **Active Jobs**: `kubectl get jobs -n raibid-ci`
4. **Job Success Rate**: `successful_jobs / total_jobs`
5. **Average Job Duration**: Time from start to completion
6. **Time to Scale**: Time from queue add to pod running
7. **Resource Utilization**: CPU/memory usage per job

### Prometheus Metrics

```yaml
# KEDA exposes metrics
- keda_scaler_errors_total
- keda_scaled_job_paused
- keda_scaledjob_max_replicas
```

## References

- [KEDA ScaledJob Spec](https://keda.sh/docs/concepts/scaling-jobs/)
- [Redis Streams Scaler](https://keda.sh/docs/scalers/redis-streams/)
- [Scaling Strategies](https://keda.sh/docs/concepts/scaling-jobs/#scaling-strategy)
- [Performance Tuning](https://keda.sh/docs/operate/performance-tuning/)

