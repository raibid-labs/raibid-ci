# Redis Deployment for Job Queue

## Overview

This document describes the Redis deployment implementation for the raibid-cli job queue system. Redis is deployed using Helm to a k3s cluster and configured with Redis Streams for job queue management.

## Architecture

### Components

- **Redis Master**: Single Redis instance (standalone architecture for MVP)
- **Persistence**: AOF (Append-Only File) with `everysec` fsync for durability
- **Authentication**: Password-based authentication enabled by default
- **Metrics**: Prometheus metrics enabled (without ServiceMonitor for MVP)

### Redis Streams Structure

- **Stream Name**: `raibid:jobs`
- **Consumer Group**: `raibid-workers`
- **Max Stream Length**: 10,000 entries (configurable)

## Deployment

### Prerequisites

1. k3s cluster running and accessible
2. kubectl configured with cluster access
3. Helm 3.x installed

### Installation

```bash
# Install Redis component
raibid-cli setup redis
```

### Configuration

Default configuration in `RedisConfig`:

```rust
RedisConfig {
    namespace: "raibid-redis",
    release_name: "raibid-redis",
    persistence_enabled: true,
    persistence_size: "8Gi",
    auth_enabled: true,
    password: None,  // Auto-generated
    sentinel_enabled: false,
    replica_count: 0,
    streams_config: RedisStreamsConfig {
        queue_stream: "raibid:jobs",
        consumer_group: "raibid-workers",
        max_length: 10000,
    },
}
```

### Helm Chart

- **Repository**: Bitnami (https://charts.bitnami.com/bitnami)
- **Chart**: `bitnami/redis`
- **Architecture**: Standalone (no replicas for MVP)

### Resource Limits

```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "100m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

## Persistence

### AOF Configuration

```
appendonly yes
appendfsync everysec
```

- **appendonly**: Enables AOF persistence
- **appendfsync everysec**: Syncs to disk every second (balance between performance and durability)

### Storage

- **Size**: 8Gi (default)
- **Storage Class**: Uses k3s default storage class
- **Persistence Enabled**: Yes

### Backup Strategy

For MVP:
- AOF provides point-in-time recovery
- Persistent volumes ensure data survives pod restarts

Future considerations:
- Scheduled snapshots to object storage
- Redis RDB snapshots for faster recovery
- Multi-region replication

## Authentication

### Password Generation

- **Length**: 32 characters
- **Character Set**: Alphanumeric (A-Z, a-z, 0-9)
- **Storage**: Saved to `~/.raibid/redis-credentials.json`

### Connection Details

Stored in credentials file:
```json
{
  "host": "raibid-redis-master.raibid-redis.svc.cluster.local",
  "port": 6379,
  "password": "<generated-password>",
  "namespace": "raibid-redis",
  "stream": "raibid:jobs",
  "consumer_group": "raibid-workers"
}
```

### Connection URL Format

```
redis://:<password>@<host>:6379
```

## Health Checks

### Readiness Check

The installer validates Redis is ready by:

1. Waiting for pod to reach Ready state
2. Executing PING command via `redis-cli`
3. Verifying PONG response

### Monitoring

Metrics are exposed for monitoring:
- **Port**: 9121
- **Format**: Prometheus
- **ServiceMonitor**: Disabled (can be enabled for production)

## Job Queue Operations

### Stream Initialization

Consumer group is created automatically:

```redis
XGROUP CREATE raibid:jobs raibid-workers $ MKSTREAM
```

### Publishing Jobs

```redis
XADD raibid:jobs * job_id <id> job_type <type> payload <json>
```

### Consuming Jobs

```redis
XREADGROUP GROUP raibid-workers <consumer-id> COUNT 1 STREAMS raibid:jobs >
```

### Stream Management

- **Max Length**: 10,000 entries
- **Trimming**: Automatic when max length is reached
- **Persistence**: All stream data is persisted via AOF

## Troubleshooting

### Check Redis Status

```bash
kubectl get pods -n raibid-redis
kubectl logs -n raibid-redis <pod-name>
```

### Test Connection

```bash
kubectl exec -n raibid-redis <pod-name> -- redis-cli -a <password> PING
```

### Verify Stream

```bash
kubectl exec -n raibid-redis <pod-name> -- redis-cli -a <password> XINFO GROUPS raibid:jobs
```

### Check Persistence

```bash
kubectl exec -n raibid-redis <pod-name> -- redis-cli -a <password> CONFIG GET appendonly
```

## Rollback

If installation fails, automatic rollback will:

1. Uninstall Helm release
2. Delete namespace and all resources
3. Clean up temporary files

Manual rollback:

```bash
helm uninstall raibid-redis -n raibid-redis
kubectl delete namespace raibid-redis
```

## Scaling Considerations

### Current MVP Setup

- Single Redis instance
- No replication
- No Sentinel
- Suitable for development and small deployments

### Future Enhancements

1. **High Availability**
   - Enable Redis Sentinel
   - Add replica nodes
   - Automatic failover

2. **Performance**
   - Redis Cluster for horizontal scaling
   - Read replicas for load distribution
   - Connection pooling

3. **Monitoring**
   - Enable ServiceMonitor for Prometheus
   - Set up alerting rules
   - Dashboard for queue metrics

4. **Backup**
   - Automated RDB snapshots
   - S3/object storage integration
   - Point-in-time recovery

## Security

### Network Isolation

- Deployed in dedicated namespace
- ClusterIP service (internal only)
- No external exposure

### Authentication

- Password authentication required
- Credentials stored with 600 permissions
- Auto-generated strong passwords

### Future Security Enhancements

- TLS encryption for connections
- mTLS for client authentication
- Network policies for namespace isolation
- Secret management via external secret store (Vault, etc.)

## Testing

### Unit Tests

Located in `tests/redis_test.rs`:
- Configuration validation
- Password generation
- Helm values generation
- Connection URL formatting
- Credential saving

### Integration Tests

Requires k3s cluster:
```bash
cargo test --test redis_test
```

Skip integration tests:
```bash
cargo test --test redis_test --lib
```

## References

- [Redis Streams Documentation](https://redis.io/docs/data-types/streams/)
- [Bitnami Redis Helm Chart](https://github.com/bitnami/charts/tree/main/bitnami/redis)
- [Redis Persistence Documentation](https://redis.io/docs/management/persistence/)
