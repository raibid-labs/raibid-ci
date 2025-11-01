# Gitea Infrastructure

Self-hosted Git server with OCI container registry support.

## Overview

Gitea provides Git hosting and container registry functionality for raibid-ci. It serves as the source of truth for repositories and stores built container images.

## Manifests

- `namespace.yaml` - Gitea namespace
- `values.yaml` - Helm chart values (production)
- `values-dev.yaml` - Development overrides
- `sealed-secrets.yaml` - Sealed credentials (production only)

## Deployment

### Via raibid-cli

```bash
raibid-cli setup gitea
```

### Via Helm

```bash
# Add Helm repository
helm repo add gitea-charts https://dl.gitea.io/charts/
helm repo update

# Create namespace
kubectl apply -f namespace.yaml

# Install with production values
helm upgrade --install gitea gitea-charts/gitea \
  --namespace raibid-gitea \
  --values values.yaml \
  --wait

# Or with dev values
helm upgrade --install gitea gitea-charts/gitea \
  --namespace raibid-gitea \
  --values values.yaml \
  --values values-dev.yaml \
  --wait
```

## Configuration

### Default Settings

- **Namespace**: `raibid-gitea`
- **Service Type**: NodePort
- **HTTP Port**: 30080
- **SSH Port**: 30022
- **Storage**: 10Gi repositories, 5Gi database
- **Database**: PostgreSQL (bundled)
- **Cache**: Redis (bundled)
- **OCI Registry**: Enabled

### Admin Credentials

Credentials are randomly generated during installation and saved to:
```
~/.raibid/gitea-credentials.json
```

## Access

### Web Interface
```
http://localhost:30080
```

### Git Operations
```bash
# Clone over HTTPS
git clone http://localhost:30080/username/repo.git

# Clone over SSH
git clone ssh://git@localhost:30022/username/repo.git
```

### Container Registry
```bash
# Login
docker login localhost:30080

# Push image
docker tag myimage:latest localhost:30080/username/myimage:latest
docker push localhost:30080/username/myimage:latest
```

## Validation

```bash
# Check deployment
kubectl get all -n raibid-gitea

# Test HTTP endpoint
curl http://localhost:30080/api/v1/version

# Check logs
kubectl logs -n raibid-gitea -l app.kubernetes.io/name=gitea
```

## Backup

```bash
# Backup repositories
kubectl exec -n raibid-gitea deployment/gitea -- \
  tar czf /tmp/repos.tar.gz /data/git/repositories

kubectl cp raibid-gitea/$(kubectl get pod -n raibid-gitea -l app.kubernetes.io/name=gitea -o jsonpath='{.items[0].metadata.name}'):/tmp/repos.tar.gz \
  ./backup/gitea-repos-$(date +%Y%m%d).tar.gz

# Backup database
kubectl exec -n raibid-gitea deployment/gitea-postgresql -- \
  pg_dump -U gitea gitea > ./backup/gitea-db-$(date +%Y%m%d).sql
```

## Troubleshooting

### Pods Not Starting

```bash
# Check pod status
kubectl describe pod -n raibid-gitea -l app.kubernetes.io/name=gitea

# Check PVC
kubectl get pvc -n raibid-gitea

# Check events
kubectl get events -n raibid-gitea --sort-by='.lastTimestamp'
```

### Can't Access Web UI

```bash
# Check service
kubectl get svc -n raibid-gitea

# Port forward (alternative)
kubectl port-forward -n raibid-gitea svc/gitea-http 3000:3000
```

## Uninstallation

```bash
# Via raibid-cli
raibid-cli teardown gitea

# Via Helm
helm uninstall gitea -n raibid-gitea
kubectl delete namespace raibid-gitea
```

## References

- [Gitea Documentation](https://docs.gitea.io/)
- [Gitea Helm Chart](https://gitea.com/gitea/helm-chart)
- [Gitea Actions](https://docs.gitea.io/en-us/usage/actions/overview/)
