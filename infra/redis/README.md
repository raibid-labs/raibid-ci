# Redis Infrastructure

Job queue management using Redis Streams.

## Overview

Redis provides the job queue for raibid-ci using Redis Streams. It serves as the message broker between job submission and agent execution.

## Manifests

- `namespace.yaml` - Redis namespace
- `values.yaml` - Helm chart values (production)
- `values-dev.yaml` - Development overrides

## Deployment

### Via raibid-cli

```bash
raibid-cli setup redis
```

### Via Helm

```bash
# Add Helm repository
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo update

# Create namespace
kubectl apply -f namespace.yaml

# Install with production values
helm upgrade --install raibid-redis bitnami/redis \
  --namespace raibid-redis \
  --values values.yaml \
  --wait

# Or with dev values
helm upgrade --install raibid-redis bitnami/redis \
  --namespace raibid-redis \
  --values values.yaml \
  --values values-dev.yaml \
  --wait
```

## Configuration

### Default Settings

- **Namespace**: `raibid-redis`
- **Architecture**: Standalone (single master)
- **Persistence**: Enabled (8Gi)
- **Authentication**: Password-protected
- **Stream**: `raibid:jobs`
- **Consumer Group**: `raibid-workers`

### Connection Details

Credentials are saved to:
```
~/.raibid/redis-credentials.json
```

Connection string:
```
redis://:<password>@raibid-redis-master.raibid-redis.svc.cluster.local:6379
```

## Redis Streams

### Stream Configuration

- **Stream Name**: `raibid:jobs`
- **Consumer Group**: `raibid-workers`
- **Max Length**: 10,000 entries
- **Trimming**: Automatic

### Job Operations

```bash
# Add job to stream
XADD raibid:jobs * job_id abc123 repo owner/repo branch main

# Read jobs (consumer)
XREADGROUP GROUP raibid-workers worker-1 COUNT 1 STREAMS raibid:jobs >

# Acknowledge job
XACK raibid:jobs raibid-workers <entry-id>

# Check pending jobs
XPENDING raibid:jobs raibid-workers
```

## Validation

```bash
# Check deployment
kubectl get all -n raibid-redis

# Test connection
kubectl exec -n raibid-redis raibid-redis-master-0 -- redis-cli -a $PASSWORD PING

# Check stream
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a $PASSWORD XINFO GROUPS raibid:jobs

# Check logs
kubectl logs -n raibid-redis raibid-redis-master-0
```

## Monitoring

### Metrics

Redis metrics are exposed on port 9121:
```bash
# Port forward metrics
kubectl port-forward -n raibid-redis svc/raibid-redis-metrics 9121:9121

# Scrape metrics
curl http://localhost:9121/metrics
```

### Key Metrics

- `redis_connected_clients` - Active client connections
- `redis_memory_used_bytes` - Memory usage
- `redis_stream_length{stream="raibid:jobs"}` - Queue depth
- `redis_commands_processed_total` - Command throughput

## Persistence

### AOF Configuration

```
appendonly yes
appendfsync everysec
```

Data is persisted to disk every second with AOF (Append-Only File).

### Backup

```bash
# Trigger save
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a $PASSWORD SAVE

# Copy dump file
kubectl cp raibid-redis/raibid-redis-master-0:/data/dump.rdb \
  ./backup/redis-dump-$(date +%Y%m%d).rdb
```

### Restore

```bash
# Copy backup to pod
kubectl cp ./backup/redis-dump.rdb \
  raibid-redis/raibid-redis-master-0:/data/dump.rdb

# Restart Redis to load dump
kubectl delete pod -n raibid-redis raibid-redis-master-0
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod -n raibid-redis raibid-redis-master-0

# Check PVC
kubectl get pvc -n raibid-redis

# Check events
kubectl get events -n raibid-redis --sort-by='.lastTimestamp'
```

### Connection Issues

```bash
# Test from within cluster
kubectl run redis-test --rm -it --image=redis -- redis-cli \
  -h raibid-redis-master.raibid-redis.svc.cluster.local \
  -a $PASSWORD PING

# Port forward for local testing
kubectl port-forward -n raibid-redis svc/raibid-redis-master 6379:6379
redis-cli -a $PASSWORD PING
```

### Stream Issues

```bash
# Check stream exists
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a $PASSWORD EXISTS raibid:jobs

# Check consumer group
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a $PASSWORD XINFO GROUPS raibid:jobs

# Recreate consumer group if needed
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a $PASSWORD XGROUP CREATE raibid:jobs raibid-workers $ MKSTREAM
```

## Uninstallation

```bash
# Via raibid-cli
raibid-cli teardown redis

# Via Helm
helm uninstall raibid-redis -n raibid-redis
kubectl delete namespace raibid-redis
```

## References

- [Redis Documentation](https://redis.io/docs/)
- [Redis Streams](https://redis.io/docs/data-types/streams/)
- [Bitnami Redis Chart](https://github.com/bitnami/charts/tree/main/bitnami/redis)
