# Infrastructure as Code

This directory contains all infrastructure-as-code (IaC) for the raibid-ci system. The infrastructure is separated from application code to enable independent validation, versioning, and deployment.

## Directory Structure

```
infra/
├── k3s/           # k3s cluster configuration
├── gitea/         # Gitea Git server manifests
├── redis/         # Redis job queue manifests
├── flux/          # Flux GitOps configuration
├── keda/          # KEDA autoscaling resources
├── scripts/       # Validation and deployment scripts
├── Taskfile.yml   # Task automation
└── README.md      # This file
```

## Component Overview

### k3s - Lightweight Kubernetes
- **Purpose**: Foundation Kubernetes cluster for DGX Spark
- **Location**: `/infra/k3s/`
- **Deployment**: Installed via raibid-cli
- **Dependencies**: None (base layer)

### Gitea - Git Server & OCI Registry
- **Purpose**: Self-hosted Git with container registry
- **Location**: `/infra/gitea/`
- **Deployment**: Helm chart with custom values
- **Dependencies**: k3s

### Redis - Job Queue
- **Purpose**: Redis Streams for job queue management
- **Location**: `/infra/redis/`
- **Deployment**: Helm chart with custom values
- **Dependencies**: k3s

### Flux - GitOps CD
- **Purpose**: Continuous delivery from Gitea
- **Location**: `/infra/flux/`
- **Deployment**: Flux bootstrap
- **Dependencies**: k3s, Gitea

### KEDA - Event-Driven Autoscaling
- **Purpose**: Scale CI agents based on job queue depth
- **Location**: `/infra/keda/`
- **Deployment**: Helm chart with ScaledObject
- **Dependencies**: k3s, Redis

## Deployment Sequence

Infrastructure components must be deployed in dependency order:

1. **k3s** - Base Kubernetes cluster
2. **Redis** - Job queue (can be parallel with Gitea)
3. **Gitea** - Git server (can be parallel with Redis)
4. **KEDA** - Autoscaler (requires Redis)
5. **Flux** - GitOps (requires Gitea)

### Automated Deployment

```bash
# Deploy all components in correct order
task infra:deploy-all

# Or deploy individually
task infra:deploy-k3s
task infra:deploy-redis
task infra:deploy-gitea
task infra:deploy-keda
task infra:deploy-flux
```

### Manual Deployment

```bash
# Via raibid-cli (recommended)
raibid-cli setup all

# Or individual components
raibid-cli setup k3s
raibid-cli setup redis
raibid-cli setup gitea
raibid-cli setup keda
raibid-cli setup flux
```

## Validation

All infrastructure manifests can be validated before deployment:

```bash
# Validate all manifests
task infra:validate

# Validate specific component
task infra:validate-gitea
task infra:validate-redis
task infra:validate-keda

# Lint manifests
task infra:lint
```

### Validation Scripts

Located in `/infra/scripts/`:

- `validate-manifests.sh` - YAML syntax and schema validation
- `lint-manifests.sh` - Linting with yamllint and kubeval
- `check-dependencies.sh` - Verify dependency order

## Configuration

Each component has its own configuration approach:

### Helm-based Components (Gitea, Redis, KEDA)

Configuration via Helm values files:
- `values.yaml` - Default production values
- `values-dev.yaml` - Development overrides
- `values-test.yaml` - Testing overrides

### k3s Configuration

Configuration via install scripts and config files:
- `config.yaml` - k3s cluster config
- `install-flags.txt` - Installation flags

### Flux Configuration

Configuration via GitRepository and Kustomization:
- `flux-system/` - Flux system components
- `clusters/` - Cluster-specific configs

## CI/CD Integration

Infrastructure validation is integrated into GitHub Actions:

```yaml
# .github/workflows/infra-validation.yml
- Validates all manifests on PR
- Lints YAML files
- Checks Helm chart syntax
- Verifies dependency order
```

## Manifest Standards

All Kubernetes manifests must follow these standards:

### YAML Format
- 2-space indentation
- No tabs
- UTF-8 encoding
- LF line endings

### Metadata
- All resources must have `metadata.labels`:
  - `app.kubernetes.io/name`
  - `app.kubernetes.io/component`
  - `app.kubernetes.io/part-of: raibid-ci`
  - `app.kubernetes.io/managed-by`

### Namespaces
- Each component in dedicated namespace
- Namespace naming: `raibid-{component}`
- Namespace manifests included in component directory

### Resource Limits
- All pods must have requests and limits
- CPU in millicores (e.g., `100m`)
- Memory with unit suffix (e.g., `256Mi`)

## Development Workflow

### Adding New Infrastructure

1. Create component directory under `/infra/`
2. Add manifests with proper labels and namespaces
3. Create Helm values file if using Helm
4. Add validation tests
5. Update dependency chain if needed
6. Document in component README

### Testing Changes

```bash
# Validate changes
task infra:validate

# Dry-run deployment
kubectl apply --dry-run=client -f infra/component/

# Deploy to test cluster
task infra:deploy-test
```

### Submitting Changes

1. Create feature branch from `main`
2. Make infrastructure changes
3. Validate locally: `task infra:validate`
4. Commit with descriptive message
5. Push and create PR
6. CI will validate automatically
7. Merge after approval

## Troubleshooting

### Validation Failures

```bash
# Check YAML syntax
yamllint infra/

# Check Kubernetes schemas
kubeval infra/**/*.yaml

# Verify Helm charts
helm lint infra/gitea/chart
```

### Deployment Issues

```bash
# Check component status
kubectl get all -n raibid-{component}

# View component logs
kubectl logs -n raibid-{component} -l app.kubernetes.io/name={component}

# Describe failing pods
kubectl describe pod -n raibid-{component} {pod-name}
```

### Rollback

```bash
# Rollback via Helm (for Helm-based components)
helm rollback {release} -n {namespace}

# Or tear down and redeploy
raibid-cli teardown {component}
raibid-cli setup {component}
```

## Security Considerations

### Secrets Management
- Never commit secrets to Git
- Use k8s Secrets or external secret managers
- Rotate credentials regularly

### Network Policies
- Isolate namespaces with NetworkPolicies
- Restrict ingress/egress traffic
- Allow only required pod-to-pod communication

### RBAC
- Apply least-privilege principle
- Use dedicated ServiceAccounts
- Limit cluster-admin usage

### Image Security
- Use specific image tags (not `latest`)
- Scan images for vulnerabilities
- Use private registry for production

## Monitoring

### Health Checks
- All pods have liveness and readiness probes
- Services have health check endpoints
- Ingress has health checks configured

### Metrics
- Prometheus metrics enabled for all components
- ServiceMonitors for metric scraping
- Custom metrics for autoscaling

### Logging
- Structured JSON logs
- Centralized log aggregation (future)
- Log retention policies

## Backup and Recovery

### GitOps State
- Flux ensures declarative state
- Git is source of truth
- Restore by re-applying from Git

### Persistent Data

#### Gitea Repositories
```bash
# Backup
kubectl exec -n raibid-gitea {pod} -- tar czf /tmp/repos.tar.gz /data/git
kubectl cp raibid-gitea/{pod}:/tmp/repos.tar.gz ./backup/repos.tar.gz

# Restore
kubectl cp ./backup/repos.tar.gz raibid-gitea/{pod}:/tmp/repos.tar.gz
kubectl exec -n raibid-gitea {pod} -- tar xzf /tmp/repos.tar.gz -C /
```

#### Redis Data
```bash
# Backup
kubectl exec -n raibid-redis {pod} -- redis-cli SAVE
kubectl cp raibid-redis/{pod}:/data/dump.rdb ./backup/redis.rdb

# Restore
kubectl cp ./backup/redis.rdb raibid-redis/{pod}:/data/dump.rdb
kubectl delete pod -n raibid-redis {pod}  # Restart to load dump
```

## Production Readiness Checklist

Before deploying to production:

- [ ] All manifests validated
- [ ] Resource limits configured
- [ ] Secrets externalized
- [ ] Network policies applied
- [ ] RBAC configured
- [ ] Monitoring enabled
- [ ] Logging configured
- [ ] Backup strategy implemented
- [ ] Disaster recovery tested
- [ ] Documentation updated

## References

### Official Documentation
- [k3s Documentation](https://docs.k3s.io/)
- [Gitea Documentation](https://docs.gitea.io/)
- [Redis Documentation](https://redis.io/docs/)
- [Flux Documentation](https://fluxcd.io/docs/)
- [KEDA Documentation](https://keda.sh/docs/)

### Helm Charts
- [Gitea Helm Chart](https://gitea.com/gitea/helm-chart)
- [Bitnami Redis Chart](https://github.com/bitnami/charts/tree/main/bitnami/redis)
- [KEDA Helm Chart](https://github.com/kedacore/charts)

### Related Documentation
- [Project README](/README.md)
- [CLAUDE.md](/CLAUDE.md) - Project overview
- [Workstreams](/docs/workstreams/) - Development workstreams

## Support

For issues or questions:
- Open issue on GitHub
- Check [docs/](../docs/) for detailed guides
- Review component-specific READMEs
