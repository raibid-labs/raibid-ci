# Redis Monitoring Guide

## Key Metrics

- `redis_up` - Instance availability
- `redis_stream_length{stream="raibid:jobs"}` - Queue depth
- `redis_memory_used_bytes` - Memory usage

## Monitoring Commands

```bash
# Get password
export REDIS_PASSWORD=$(kubectl get secret raibid-redis -n raibid-redis -o jsonpath='{.data.redis-password}' | base64 -d)

# Queue depth
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a "$REDIS_PASSWORD" --no-auth-warning XLEN raibid:jobs

# Consumer group
kubectl exec -n raibid-redis raibid-redis-master-0 -- \
  redis-cli -a "$REDIS_PASSWORD" --no-auth-warning XINFO GROUPS raibid:jobs
```

## Metrics Endpoint

```bash
kubectl port-forward -n raibid-redis svc/raibid-redis-metrics 9121:9121
curl http://localhost:9121/metrics
```

## References

- [Redis Monitoring](https://redis.io/topics/admin)
- [Prometheus Redis Exporter](https://github.com/oliver006/redis_exporter)
