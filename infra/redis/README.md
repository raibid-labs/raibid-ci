# Redis Infrastructure

Job queue management using Redis Streams for raibid-ci.

## Files

- `helmrepository.yaml` - Bitnami Helm repository
- `helmrelease.yaml` - Flux HelmRelease
- `configmap.yaml` - Helm values as ConfigMap
- `service.yaml` - Service definition
- `init-job.yaml` - Consumer group initialization
- `validate.sh` - Deployment validation
- `test-connection.sh` - Connection testing
- `MONITORING.md` - Monitoring guide

## Deployment (Flux)

```bash
kubectl apply -f namespace.yaml
kubectl apply -f helmrepository.yaml
kubectl apply -f configmap.yaml
kubectl apply -f service.yaml
kubectl apply -f helmrelease.yaml
kubectl apply -f init-job.yaml
./validate.sh
```

## Configuration

- Stream: `raibid:jobs`
- Consumer Group: `raibid-workers`
- Memory: 450MB (prod), 200MB (dev)
- Persistence: AOF + RDB
- Metrics: Port 9121

## Operations

```bash
# Add job
XADD raibid:jobs MAXLEN ~ 10000 * job_id "abc123" repo "owner/repo"

# Read jobs
XREADGROUP GROUP raibid-workers worker-1 COUNT 1 STREAMS raibid:jobs >

# Queue depth
XLEN raibid:jobs
```

## Monitoring

See [MONITORING.md](./MONITORING.md) for details.

## References

- [Redis Streams](https://redis.io/docs/data-types/streams/)
- [Bitnami Redis Chart](https://github.com/bitnami/charts/tree/main/bitnami/redis)
- [Flux HelmRelease](https://fluxcd.io/flux/components/helm/helmreleases/)
