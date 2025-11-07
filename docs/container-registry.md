# Container Registry

The raibid-ci system includes a self-hosted OCI-compliant container registry powered by Gitea. This registry stores Docker images locally, enabling offline development and eliminating dependencies on external registries.

## Overview

- **Provider:** Gitea OCI Container Registry
- **Registry URL:** `localhost:3000` (same HTTP port as Gitea)
- **Namespace:** `raibid-admin`
- **Protocol:** HTTP (local development only)
- **Authentication:** Gitea user credentials
- **Note:** The OCI registry runs on the same HTTP port (3000) as the Gitea web interface, exposed via NodePort 30080

## Quick Start

### 1. Login to Registry

```bash
docker login localhost:3000 -u raibid-admin -p adminadmin
```

**Note:** For production deployments, use secure credentials and HTTPS.

### 2. Tag and Push Images

```bash
# Tag your image
docker tag my-app:latest localhost:3000/raibid-admin/my-app:latest

# Push to registry
docker push localhost:3000/raibid-admin/my-app:latest
```

### 3. Pull Images

```bash
# Pull from registry
docker pull localhost:3000/raibid-admin/my-app:latest
```

## Integration with Tilt

The Tiltfile is configured to automatically build and push images to the local Gitea registry:

```python
# In Tiltfile
REGISTRY_HOST = 'localhost:3000'
REGISTRY_NAMESPACE = 'raibid-admin'

docker_build(
    '{}/{}/raibid-server'.format(REGISTRY_HOST, REGISTRY_NAMESPACE),
    context='.',
    dockerfile='crates/server/Dockerfile',
)
```

When you run `tilt up`, images are automatically:
1. Built using Docker BuildKit
2. Tagged with the registry prefix
3. Pushed to the Gitea registry
4. Deployed to the Kubernetes cluster

## Kubernetes Integration

The cluster is configured to pull images from the local registry without authentication (insecure registry for local development):

```yaml
# Images in Kubernetes manifests
spec:
  containers:
  - name: raibid-server
    image: localhost:3000/raibid-admin/raibid-server:latest
    imagePullPolicy: IfNotPresent
```

## Registry Configuration

### Gitea Configuration

The registry is enabled in the Gitea Helm chart configuration:

```jsonnet
// In tanka/lib/charts/gitea.libsonnet
gitea: {
  config: {
    // Enable package registry
    packages: {
      ENABLED: true,
    },

    // Enable OCI Container Registry
    'packages.container': {
      ENABLED: true,
    },
  },
}

// Note: Registry uses the same HTTP port as Gitea (3000)
// No additional ports needed - the registry API is accessible at /v2/ on the main HTTP port
```

### Docker Daemon Configuration

For local development with insecure registries, configure Docker:

```json
// /etc/docker/daemon.json (Linux)
// ~/.docker/daemon.json (Docker Desktop)
{
  "insecure-registries": ["localhost:3000"]
}
```

Restart Docker after configuration changes:

```bash
# Linux
sudo systemctl restart docker

# macOS/Windows (Docker Desktop)
# Restart Docker Desktop from the system tray
```

## Web UI Access

Browse and manage container images via Gitea's web interface:

1. **Navigate to Gitea:**
   ```bash
   # Open browser to
   http://localhost:3000
   ```

2. **Login:**
   - Username: `raibid-admin`
   - Password: `adminadmin`

3. **View Packages:**
   - Click "Packages" in the top navigation
   - Select "Container" filter
   - Browse your OCI images

4. **Image Details:**
   - Click on an image to view:
     - Available tags
     - Size information
     - Pull commands
     - Manifest details

## Storage

Container images are stored in Gitea's persistent volume:

```yaml
# Gitea persistence
persistence:
  enabled: true
  size: 10Gi
  storageClass: local-path
  accessModes: ['ReadWriteOnce']
```

Images are stored in the `/data/packages` directory within the Gitea pod.

## Cleanup

To remove old or unused images:

1. **Via Web UI:**
   - Navigate to the package page
   - Click the image
   - Select tags to delete
   - Click "Delete"

2. **Via API:**
   ```bash
   # List all packages
   curl -H "Authorization: token $GITEA_TOKEN" \
     http://localhost:3000/api/v1/packages/raibid-admin?type=container

   # Delete a specific version
   curl -X DELETE \
     -H "Authorization: token $GITEA_TOKEN" \
     http://localhost:3000/api/v1/packages/raibid-admin/container/my-app/v1.0.0
   ```

## Troubleshooting

### Login Fails

**Error:** `Error response from daemon: Get "https://localhost:3000/v2/": http: server gave HTTP response to HTTPS client`

**Solution:** Add `localhost:3000` to insecure registries in Docker daemon configuration.

### Push Fails with 401 Unauthorized

**Error:** `unauthorized: authentication required`

**Solution:** Login to the registry:
```bash
docker login localhost:3000 -u raibid-admin -p adminadmin
```

### Image Pull Fails in Kubernetes

**Error:** `Failed to pull image "localhost:3000/raibid-admin/my-app:latest": rpc error`

**Solution:** Ensure the registry port (30080) is accessible from within the cluster. Check that the Gitea service is running:
```bash
kubectl get svc -n raibid-system gitea-http
```

### Registry Not Accessible

**Error:** `dial tcp 127.0.0.1:30080: connect: connection refused`

**Solution:** Verify Gitea is running and the HTTP port is exposed:
```bash
# Check Gitea pod
kubectl get pod -n raibid-system -l app.kubernetes.io/name=gitea

# Check service ports
kubectl get svc -n raibid-system gitea-http -o yaml | grep -A 10 ports

# Test registry endpoint
curl http://localhost:3000/v2/
```

## Production Considerations

For production deployments:

1. **HTTPS/TLS:** Configure TLS certificates for secure communication
2. **Authentication:** Use strong passwords and rotate credentials regularly
3. **Storage:** Use persistent storage with backup/restore capabilities
4. **Monitoring:** Track registry usage and storage capacity
5. **Rate Limiting:** Configure rate limits to prevent abuse
6. **Content Trust:** Enable Docker Content Trust for image signing
7. **Vulnerability Scanning:** Integrate image scanning tools

## References

- [Gitea Packages Documentation](https://docs.gitea.io/en-us/usage/packages/overview/)
- [OCI Distribution Spec](https://github.com/opencontainers/distribution-spec)
- [Docker Registry HTTP API V2](https://docs.docker.com/registry/spec/api/)
