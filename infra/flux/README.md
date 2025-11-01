# Flux Infrastructure

GitOps continuous delivery from Gitea.

## Overview

Flux provides GitOps-based continuous delivery for raibid-ci. It monitors the Gitea repository and automatically applies Kubernetes manifests when changes are detected.

## Manifests

- `namespace.yaml` - Flux namespace
- `gitrepository.yaml` - GitRepository source
- `kustomization.yaml` - Kustomization for deployment
- `flux-system/` - Flux system components

## Deployment

### Via raibid-cli

```bash
raibid-cli setup flux
```

### Via Flux CLI

```bash
# Install Flux CLI
curl -s https://fluxcd.io/install.sh | sudo bash

# Bootstrap Flux with Gitea
flux bootstrap generic \
  --url=http://gitea.raibid-gitea.svc.cluster.local:3000/raibid/infrastructure \
  --username=raibid-admin \
  --password=$GITEA_PASSWORD \
  --namespace=flux-system \
  --components-extra=image-reflector-controller,image-automation-controller

# Apply GitRepository and Kustomization
kubectl apply -f gitrepository.yaml
kubectl apply -f kustomization.yaml
```

## Configuration

### Default Settings

- **Namespace**: `flux-system`
- **Source**: Gitea repository
- **Branch**: `main`
- **Sync Interval**: 1 minute
- **Prune**: Enabled (delete removed resources)
- **Retry**: Exponential backoff

### GitRepository

Defines the source Git repository:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: GitRepository
metadata:
  name: raibid-infrastructure
  namespace: flux-system
spec:
  interval: 1m
  url: http://gitea.raibid-gitea.svc.cluster.local:3000/raibid/infrastructure
  ref:
    branch: main
  secretRef:
    name: gitea-credentials
```

### Kustomization

Defines how to apply manifests:

```yaml
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: raibid-ci
  namespace: flux-system
spec:
  interval: 5m
  sourceRef:
    kind: GitRepository
    name: raibid-infrastructure
  path: ./manifests
  prune: true
  wait: true
  timeout: 5m
```

## Validation

```bash
# Check Flux pods
kubectl get pods -n flux-system

# Expected pods:
# - source-controller
# - kustomize-controller
# - helm-controller
# - notification-controller

# Check GitRepository
kubectl get gitrepository -n flux-system

# Check Kustomization
kubectl get kustomization -n flux-system

# View sync status
flux get sources git
flux get kustomizations
```

## GitOps Workflow

### 1. Commit Changes

```bash
# Make changes to infrastructure
vim infra/gitea/values.yaml

# Commit and push
git add infra/gitea/values.yaml
git commit -m "Update Gitea configuration"
git push origin main
```

### 2. Flux Detects Changes

Flux polls the GitRepository every minute:
```bash
# Watch Flux reconcile
kubectl logs -n flux-system -l app=source-controller -f
```

### 3. Apply Changes

Flux applies changes automatically:
```bash
# View reconciliation
flux reconcile kustomization raibid-ci --with-source

# Check events
kubectl get events -n flux-system --sort-by='.lastTimestamp'
```

## Monitoring

### Reconciliation Status

```bash
# Check source status
flux get sources git

# Check kustomization status
flux get kustomizations

# View detailed status
kubectl describe gitrepository raibid-infrastructure -n flux-system
kubectl describe kustomization raibid-ci -n flux-system
```

### Notifications

Configure notifications for sync events:

```yaml
apiVersion: notification.toolkit.fluxcd.io/v1beta1
kind: Alert
metadata:
  name: raibid-ci-alerts
  namespace: flux-system
spec:
  providerRef:
    name: slack
  eventSeverity: info
  eventSources:
  - kind: GitRepository
    name: raibid-infrastructure
  - kind: Kustomization
    name: raibid-ci
```

## Troubleshooting

### GitRepository Not Syncing

```bash
# Check GitRepository status
kubectl describe gitrepository raibid-infrastructure -n flux-system

# Verify credentials
kubectl get secret gitea-credentials -n flux-system

# Force reconciliation
flux reconcile source git raibid-infrastructure

# Check source controller logs
kubectl logs -n flux-system -l app=source-controller --tail=50
```

### Kustomization Failing

```bash
# Check Kustomization status
kubectl describe kustomization raibid-ci -n flux-system

# View error messages
flux get kustomizations

# Check kustomize controller logs
kubectl logs -n flux-system -l app=kustomize-controller --tail=50

# Validate manifests locally
kustomize build ./infra/manifests/
```

### Authentication Issues

```bash
# Test Gitea connection
kubectl run test-gitea --rm -it --image=curlimages/curl -- \
  curl -u raibid-admin:$PASSWORD \
  http://gitea.raibid-gitea.svc.cluster.local:3000/api/v1/version

# Recreate credentials secret
kubectl create secret generic gitea-credentials \
  --namespace=flux-system \
  --from-literal=username=raibid-admin \
  --from-literal=password=$GITEA_PASSWORD \
  --dry-run=client -o yaml | kubectl apply -f -
```

## Advanced Configuration

### Multi-Environment Setup

Create separate Kustomizations for different environments:

```yaml
---
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: raibid-ci-dev
  namespace: flux-system
spec:
  path: ./infra/overlays/dev
  sourceRef:
    kind: GitRepository
    name: raibid-infrastructure

---
apiVersion: kustomize.toolkit.fluxcd.io/v1
kind: Kustomization
metadata:
  name: raibid-ci-prod
  namespace: flux-system
spec:
  path: ./infra/overlays/prod
  sourceRef:
    kind: GitRepository
    name: raibid-infrastructure
```

### Helm Repository

Use HelmRepository and HelmRelease for Helm charts:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1beta2
kind: HelmRepository
metadata:
  name: bitnami
  namespace: flux-system
spec:
  interval: 24h
  url: https://charts.bitnami.com/bitnami

---
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: redis
  namespace: raibid-redis
spec:
  interval: 5m
  chart:
    spec:
      chart: redis
      sourceRef:
        kind: HelmRepository
        name: bitnami
      version: "18.x"
  values:
    # Values from values.yaml
```

### Image Automation

Automatically update container images:

```yaml
apiVersion: image.toolkit.fluxcd.io/v1beta1
kind: ImageRepository
metadata:
  name: raibid-ci-agent
  namespace: flux-system
spec:
  image: gitea.raibid-gitea.svc.cluster.local:3000/raibid/ci-agent
  interval: 1m

---
apiVersion: image.toolkit.fluxcd.io/v1beta1
kind: ImagePolicy
metadata:
  name: raibid-ci-agent
  namespace: flux-system
spec:
  imageRepositoryRef:
    name: raibid-ci-agent
  policy:
    semver:
      range: '>=1.0.0'

---
apiVersion: image.toolkit.fluxcd.io/v1beta1
kind: ImageUpdateAutomation
metadata:
  name: raibid-ci
  namespace: flux-system
spec:
  interval: 1m
  sourceRef:
    kind: GitRepository
    name: raibid-infrastructure
  git:
    checkout:
      ref:
        branch: main
    commit:
      author:
        email: fluxbot@raibid.local
        name: Flux Bot
      messageTemplate: 'Update image to {{range .Updated.Images}}{{println .}}{{end}}'
  update:
    path: ./infra/manifests
    strategy: Setters
```

## Uninstallation

```bash
# Via raibid-cli
raibid-cli teardown flux

# Via Flux CLI
flux uninstall --namespace=flux-system

# Manual cleanup
kubectl delete namespace flux-system
```

## References

- [Flux Documentation](https://fluxcd.io/docs/)
- [GitOps Toolkit](https://fluxcd.io/flux/components/)
- [Flux GitHub](https://github.com/fluxcd/flux2)
- [Get Started with Flux](https://fluxcd.io/flux/get-started/)
