# Tanka + Tilt Deployment Project Issues

This document contains all issues for migrating raibid-ci deployment to Tanka + Tilt.

## Workstream Organization

**Workstream 1: Foundation** - Tanka project structure and base configuration
**Workstream 2: Infrastructure** - Tanka configs for external dependencies
**Workstream 3: Applications** - Tanka configs for raibid components
**Workstream 4: Docker** - Container images for server and agent
**Workstream 5: Tilt Integration** - Development orchestration with Tilt
**Workstream 6: Documentation** - Developer experience and guides

---

## Workstream 1: Foundation

### Issue 1.1: Initialize Tanka Project Structure

**Title**: feat: initialize Tanka project with base structure

**Description**:
Set up the foundational Tanka project structure following best practices from tanka.dev and the mop-core reference.

**Tasks**:
- [ ] Install Tanka CLI (`tk`)
- [ ] Initialize Tanka project: `tk init`
- [ ] Create directory structure:
  ```
  tanka/
  ├── environments/
  │   └── local/
  │       ├── main.jsonnet
  │       └── spec.json
  ├── lib/
  │   ├── raibid/
  │   │   ├── config.libsonnet
  │   │   └── util.libsonnet
  │   └── k8s.libsonnet
  ├── vendor/
  └── jsonnetfile.json
  ```
- [ ] Configure `jsonnetfile.json` with dependencies:
  - grafana/jsonnet-libs (tanka-util for Helm)
  - jsonnet-libs/k8s-libsonnet
- [ ] Run `jb install` to vendor dependencies
- [ ] Create `spec.json` for local k3s cluster (https://127.0.0.1:6443)
- [ ] Set default namespace: `raibid-system`

**Acceptance Criteria**:
- `tk show environments/local` runs without errors
- Dependencies are vendored correctly
- Base structure follows Tanka conventions

**Labels**: tanka, infrastructure, foundation

**Dependencies**: None

---

### Issue 1.2: Create Base Jsonnet Libraries

**Title**: feat: create reusable jsonnet libraries for common patterns

**Description**:
Build reusable jsonnet libraries that will be used across all component configurations. These provide abstractions for common Kubernetes patterns and raibid-specific configuration.

**Tasks**:
- [ ] Create `lib/raibid/config.libsonnet`:
  - Namespace management
  - Common labels and annotations
  - Resource naming conventions
- [ ] Create `lib/raibid/util.libsonnet`:
  - Helper functions for merging configs
  - Environment variable builders
  - Secret reference helpers
- [ ] Create `lib/raibid/helm.libsonnet`:
  - Wrapper for Helm chart integration
  - Chart vendoring helpers
  - Values file merging utilities
- [ ] Create `lib/k8s.libsonnet`:
  - Kubernetes API shortcuts
  - Resource template generators
- [ ] Add comprehensive inline documentation
- [ ] Create example usage in `examples/` directory

**Acceptance Criteria**:
- Libraries can be imported and used in jsonnet files
- Helper functions work correctly with test cases
- Documentation explains usage patterns

**Labels**: tanka, jsonnet, libraries

**Dependencies**: Issue 1.1

---

## Workstream 2: Infrastructure Components

### Issue 2.1: Wrap Redis Helm Chart in Tanka

**Title**: feat: create Tanka configuration for Redis with Streams

**Description**:
Wrap the Bitnami Redis Helm chart in jsonnet to manage Redis deployment via Tanka. Configure for Redis Streams support as the job queue.

**Tasks**:
- [ ] Create `lib/charts/redis.libsonnet`:
  - Import tanka-util helm wrapper
  - Define chart location: `vendor/redis`
- [ ] Vendor Redis Helm chart: `helm pull bitnami/redis --untar -d tanka/vendor/`
- [ ] Create values configuration in jsonnet:
  - Architecture: standalone (MVP)
  - Redis Streams enabled
  - Persistence: enabled
  - Resource limits
  - Service configuration
- [ ] Add Redis to `environments/local/main.jsonnet`
- [ ] Configure initialization job for streams setup
- [ ] Add healthcheck configuration

**Reference Configuration** (from `infra/redis/values.yaml`):
```yaml
architecture: standalone
auth:
  enabled: false  # MVP - add auth later
master:
  persistence:
    enabled: true
    size: 8Gi
```

**Acceptance Criteria**:
- `tk show environments/local` includes Redis resources
- `tk diff environments/local` shows expected changes
- Configuration matches existing `infra/redis/` functionality

**Labels**: tanka, redis, infrastructure, helm

**Dependencies**: Issue 1.1, Issue 1.2

---

### Issue 2.2: Wrap Gitea Helm Chart in Tanka

**Title**: feat: create Tanka configuration for Gitea with OCI registry

**Description**:
Wrap the Gitea Helm chart in jsonnet to manage Gitea deployment with OCI registry support via Tanka.

**Tasks**:
- [ ] Create `lib/charts/gitea.libsonnet`
- [ ] Vendor Gitea Helm chart: `helm pull gitea-charts/gitea --untar -d tanka/vendor/`
- [ ] Create values configuration in jsonnet:
  - OCI registry enabled
  - PostgreSQL database
  - Persistent storage
  - Service configuration (NodePort for dev)
- [ ] Add Gitea to `environments/local/main.jsonnet`
- [ ] Configure init container for database migrations
- [ ] Add post-install configuration hook

**Reference Configuration** (from `infra/gitea/values.yaml`):
```yaml
gitea:
  config:
    packages:
      ENABLED: true
    server:
      PROTOCOL: http
      DOMAIN: gitea.local
```

**Acceptance Criteria**:
- Gitea deployment includes OCI registry
- PostgreSQL dependency correctly configured
- Configuration matches existing `infra/gitea/` functionality

**Labels**: tanka, gitea, infrastructure, helm

**Dependencies**: Issue 1.1, Issue 1.2

---

### Issue 2.3: Wrap KEDA Helm Chart in Tanka

**Title**: feat: create Tanka configuration for KEDA autoscaling

**Description**:
Wrap the KEDA Helm chart in jsonnet to manage KEDA deployment for event-driven autoscaling via Tanka.

**Tasks**:
- [ ] Create `lib/charts/keda.libsonnet`
- [ ] Vendor KEDA Helm chart: `helm pull kedacore/keda --untar -d tanka/vendor/`
- [ ] Create values configuration in jsonnet:
  - Operator configuration
  - Metrics server
  - Admission webhooks
  - Service account setup
- [ ] Add KEDA to `environments/local/main.jsonnet`
- [ ] Create ScaledJob CRD wrapper in `lib/raibid/scaledjob.libsonnet`
- [ ] Add TriggerAuthentication helpers

**Reference Configuration** (from `infra/keda/values.yaml`):
```yaml
operator:
  replicaCount: 1
metricsServer:
  replicaCount: 1
```

**Acceptance Criteria**:
- KEDA operator deploys successfully
- CRDs are included in output
- Configuration matches existing `infra/keda/` functionality

**Labels**: tanka, keda, infrastructure, helm, autoscaling

**Dependencies**: Issue 1.1, Issue 1.2

---

### Issue 2.4: Wrap Flux Helm Chart in Tanka

**Title**: feat: create Tanka configuration for Flux GitOps

**Description**:
Wrap the Flux Helm chart in jsonnet to manage Flux deployment for GitOps workflows via Tanka.

**Tasks**:
- [ ] Create `lib/charts/flux.libsonnet`
- [ ] Vendor Flux Helm chart: `helm pull fluxcd-community/flux2 --untar -d tanka/vendor/`
- [ ] Create values configuration in jsonnet:
  - Source controller
  - Kustomize controller
  - Helm controller
  - Notification controller
- [ ] Add Flux to `environments/local/main.jsonnet`
- [ ] Configure GitRepository CRD for Gitea
- [ ] Configure Kustomization CRD for auto-deployment

**Reference Configuration** (from `infra/flux/`):
```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: GitRepository
metadata:
  name: raibid-ci
  namespace: raibid-system
spec:
  interval: 1m0s
  url: http://gitea.raibid-gitea.svc.cluster.local:3000/raibid/raibid-ci.git
  ref:
    branch: main
```

**Acceptance Criteria**:
- Flux controllers deploy successfully
- GitRepository can connect to Gitea
- Configuration matches existing `infra/flux/` functionality

**Labels**: tanka, flux, infrastructure, helm, gitops

**Dependencies**: Issue 1.1, Issue 1.2, Issue 2.2

---

## Workstream 3: Application Deployments

### Issue 3.1: Create Tanka Configuration for Server

**Title**: feat: create Tanka configuration for raibid-server deployment

**Description**:
Create jsonnet configuration for deploying the raibid-server application with proper Kubernetes resources (Deployment, Service, ConfigMap, etc.).

**Tasks**:
- [ ] Create `lib/raibid/server.libsonnet`:
  - Deployment with health probes
  - Service (ClusterIP)
  - ConfigMap for configuration
  - Environment variable management
- [ ] Add server to `environments/local/main.jsonnet`
- [ ] Configure resource limits:
  - CPU: 500m request, 1000m limit
  - Memory: 512Mi request, 1Gi limit
- [ ] Add liveness and readiness probes (HTTP on `/health`)
- [ ] Configure logging (JSON format for production)
- [ ] Add pod anti-affinity preferences
- [ ] Create HorizontalPodAutoscaler (optional)

**Acceptance Criteria**:
- Server deployment is properly templated
- Health checks are configured
- Configuration follows Kubernetes best practices
- Resources are appropriately limited

**Labels**: tanka, server, application, deployment

**Dependencies**: Issue 1.1, Issue 1.2, Issue 4.1

---

### Issue 3.2: Create Tanka Configuration for Agent ScaledJob

**Title**: feat: create Tanka configuration for raibid-agent ScaledJob

**Description**:
Create jsonnet configuration for deploying the raibid-agent as a KEDA ScaledJob that scales based on Redis Streams queue depth.

**Tasks**:
- [ ] Create `lib/raibid/agent.libsonnet`:
  - Job template with agent container
  - KEDA ScaledJob wrapper
  - TriggerAuthentication for Redis
  - Redis Streams scaler configuration
- [ ] Add agent ScaledJob to `environments/local/main.jsonnet`
- [ ] Configure scaling parameters:
  - pollingInterval: 10s
  - maxReplicaCount: 10
  - minReplicaCount: 0 (scale to zero)
  - lagThreshold: 5 (messages)
- [ ] Configure agent resources:
  - CPU: 1000m request, 2000m limit
  - Memory: 2Gi request, 4Gi limit
- [ ] Add workspace volume (emptyDir)
- [ ] Configure graceful shutdown

**Reference Configuration** (from `infra/keda/scaledjob.yaml`):
```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledJob
metadata:
  name: raibid-agent-rust
spec:
  pollingInterval: 10
  maxReplicaCount: 10
  scalingStrategy:
    strategy: "default"
  jobTargetRef:
    template:
      spec:
        containers:
        - name: agent
          image: raibid-agent:latest
```

**Acceptance Criteria**:
- ScaledJob properly configured for Redis Streams
- Agent scales from 0 to 10 based on queue depth
- Configuration matches existing `infra/keda/scaledjob.yaml`

**Labels**: tanka, agent, application, keda, autoscaling

**Dependencies**: Issue 1.1, Issue 1.2, Issue 2.3, Issue 4.2

---

### Issue 3.3: Create Tanka Configuration for Secrets and ConfigMaps

**Title**: feat: create Tanka configuration for application secrets and config

**Description**:
Create jsonnet configuration for managing application secrets and configuration data needed by server and agents.

**Tasks**:
- [ ] Create `lib/raibid/secrets.libsonnet`:
  - Secret templates for sensitive data
  - ConfigMap templates for non-sensitive config
  - External secrets integration (future)
- [ ] Add common ConfigMap with:
  - Redis connection parameters
  - Gitea URLs
  - Queue stream names
  - Log levels
- [ ] Create Secret placeholders:
  - Redis auth (future)
  - Gitea credentials
  - Docker registry credentials
- [ ] Add to `environments/local/main.jsonnet`
- [ ] Document secret management workflow

**Note**: For MVP, secrets can be empty/disabled. Production deployment will need proper secret management.

**Acceptance Criteria**:
- ConfigMaps contain all necessary configuration
- Secrets are properly templated (even if empty)
- Configuration is referenced correctly by server and agent

**Labels**: tanka, configuration, secrets

**Dependencies**: Issue 1.1, Issue 1.2

---

## Workstream 4: Docker Images

### Issue 4.1: Create Dockerfile for Server

**Title**: feat: create optimized Dockerfile for raibid-server

**Description**:
Create a multi-stage Dockerfile for building and running the raibid-server application optimized for ARM64 (DGX Spark).

**Tasks**:
- [ ] Create `crates/server/Dockerfile`:
  - Stage 1: Rust builder with all dependencies
  - Stage 2: Build the server binary
  - Stage 3: Minimal runtime image (Debian slim)
- [ ] Configure build optimizations:
  - Use cargo chef for dependency caching
  - Multi-arch support (ARM64 + x86_64)
  - Link-time optimization (LTO)
- [ ] Add healthcheck script
- [ ] Configure non-root user execution
- [ ] Add OCI image labels
- [ ] Optimize layer caching for fast rebuilds

**Target Image Size**: < 100MB

**Example Multi-stage Structure**:
```dockerfile
FROM rust:1.82-bookworm AS chef
# Install cargo-chef
FROM chef AS planner
# Generate recipe
FROM chef AS builder
# Build application
FROM debian:bookworm-slim AS runtime
# Copy binary and run
```

**Acceptance Criteria**:
- Image builds successfully on ARM64 and x86_64
- Binary runs and serves HTTP requests
- Healthcheck passes
- Image follows Docker best practices

**Labels**: docker, server, build

**Dependencies**: None (can start in parallel)

---

### Issue 4.2: Optimize Agent Dockerfile

**Title**: feat: optimize agent Dockerfile with build stage

**Description**:
Enhance the existing agent Dockerfile (`crates/agent/Dockerfile`) to include build stage for the agent binary and optimize for production.

**Tasks**:
- [ ] Add builder stage to existing Dockerfile:
  - Build raibid-agent binary from source
  - Use cargo-chef for dependency caching
  - Copy binary to runtime stage
- [ ] Update runtime stage:
  - Copy agent binary from builder
  - Configure startup command
  - Add environment variables
- [ ] Test with actual agent code
- [ ] Optimize build cache layers
- [ ] Add docker-compose for local testing

**Current State**: Dockerfile exists but doesn't build agent binary

**Acceptance Criteria**:
- Agent binary is built and included in image
- Image runs agent successfully
- Build uses layer caching effectively
- Image size is optimized

**Labels**: docker, agent, build

**Dependencies**: None (can start in parallel)

---

### Issue 4.3: Create Docker Compose for Local Testing

**Title**: feat: create docker-compose.yml for local service testing

**Description**:
Create a Docker Compose configuration for testing server and agent containers locally without Kubernetes.

**Tasks**:
- [ ] Create `docker-compose.yml` in root:
  - Redis service
  - Server service
  - Agent service (manual trigger)
- [ ] Configure networking between services
- [ ] Add volume mounts for development
- [ ] Create `.env.example` with configuration
- [ ] Add health checks for all services
- [ ] Document usage in README

**Acceptance Criteria**:
- `docker-compose up` starts all services
- Services can communicate with each other
- Health checks pass
- Useful for integration testing

**Labels**: docker, testing, development

**Dependencies**: Issue 4.1, Issue 4.2

---

## Workstream 5: Tilt Integration

### Issue 5.1: Create Base Tiltfile with K3s Management

**Title**: feat: create Tiltfile for k3s cluster management

**Description**:
Create the base Tiltfile that manages the local k3s cluster and provides foundation for resource orchestration.

**Tasks**:
- [ ] Create `Tiltfile` in root directory
- [ ] Add k3s cluster lifecycle management:
  - Check if k3s is running
  - Start k3s if needed (via script)
  - Configure kubectl context
- [ ] Add cluster validation:
  - Verify cluster is ready
  - Check required namespaces
  - Validate CRDs are installed
- [ ] Add helper functions:
  - Namespace creation
  - Context switching
  - Resource cleanup
- [ ] Configure Tilt UI settings:
  - Resource groups
  - Log highlighting
  - Port forwards

**Acceptance Criteria**:
- `tilt up` starts k3s cluster if needed
- Cluster is properly configured and ready
- Tilt UI displays cluster status

**Labels**: tilt, k3s, orchestration

**Dependencies**: None (can start in parallel)

---

### Issue 5.2: Add Docker Build to Tiltfile

**Title**: feat: configure Docker image builds in Tiltfile

**Description**:
Add Docker build configuration to Tiltfile for server and agent images with live reload support.

**Tasks**:
- [ ] Add `docker_build()` for server:
  - Dockerfile path: `crates/server/Dockerfile`
  - Context: repository root
  - Build args for optimization
  - Live update configuration
- [ ] Add `docker_build()` for agent:
  - Dockerfile path: `crates/agent/Dockerfile`
  - Context: repository root
  - Build args for optimization
- [ ] Configure build triggers:
  - Watch Rust source files
  - Watch Cargo.toml files
  - Rebuild on changes
- [ ] Add build optimization:
  - Use BuildKit
  - Layer caching
  - Parallel builds
- [ ] Add image push (for remote dev)

**Acceptance Criteria**:
- Images build on `tilt up`
- Changes trigger rebuilds
- Builds are fast with caching
- Live updates work for development

**Labels**: tilt, docker, build

**Dependencies**: Issue 4.1, Issue 4.2, Issue 5.1

---

### Issue 5.3: Add Tanka Deployment to Tiltfile

**Title**: feat: integrate Tanka deployments in Tiltfile

**Description**:
Integrate Tanka deployment commands into Tiltfile so `tilt up` deploys all Kubernetes resources via Tanka.

**Tasks**:
- [ ] Add Tanka integration to Tiltfile:
  - Use `local()` to run `tk show` and parse YAML
  - Or use `k8s_yaml(local('tk show environments/local'))`
- [ ] Create resource definitions for each component:
  - Redis (from Tanka)
  - Gitea (from Tanka)
  - KEDA (from Tanka)
  - Flux (from Tanka)
  - Server (from Tanka)
  - Agent ScaledJob (from Tanka)
- [ ] Configure resource dependencies:
  - Server depends on Redis
  - Agent depends on Server and KEDA
  - Flux depends on Gitea
- [ ] Add resource grouping in Tilt UI:
  - Infrastructure (Redis, Gitea, KEDA, Flux)
  - Applications (Server, Agent)
- [ ] Configure auto-reload on Tanka changes

**Acceptance Criteria**:
- `tilt up` deploys all resources via Tanka
- Resources appear in Tilt UI
- Dependencies are respected
- Changes to jsonnet trigger re-deployment

**Labels**: tilt, tanka, deployment, orchestration

**Dependencies**: Issue 5.1, Issue 5.2, All Workstream 2 issues, Issue 3.1, Issue 3.2

---

### Issue 5.4: Add Port Forwarding and Shortcuts to Tiltfile

**Title**: feat: configure port forwards and shortcuts in Tiltfile

**Description**:
Add convenient port forwards and Tilt UI shortcuts for accessing services during development.

**Tasks**:
- [ ] Add port forwards:
  - Server: 8080 -> server-service:8080
  - Gitea: 3000 -> gitea-service:3000
  - Redis: 6379 -> redis-service:6379
- [ ] Add Tilt UI links:
  - Server API: http://localhost:8080
  - Gitea Web: http://localhost:3000
  - Redis CLI: redis-cli -h localhost
- [ ] Add Tilt buttons/triggers:
  - "Trigger Job" - Send test job to Redis
  - "Scale Agent" - Manually trigger agent scale
  - "Reset Data" - Clear Redis and restart
- [ ] Add log streaming configuration:
  - Server logs highlighted
  - Agent logs with job context
- [ ] Document shortcuts in Tiltfile comments

**Acceptance Criteria**:
- All services accessible via localhost
- Tilt UI has clickable links
- Buttons trigger expected actions
- Developer experience is smooth

**Labels**: tilt, development, dx

**Dependencies**: Issue 5.3

---

### Issue 5.5: Add Live Reload for Development

**Title**: feat: configure live reload for Rust development in Tilt

**Description**:
Add live reload capabilities to Tiltfile so Rust code changes are quickly reflected in running containers without full rebuilds.

**Tasks**:
- [ ] Configure `live_update` for server:
  - Sync Rust source files
  - Run `cargo build` inside container
  - Restart server binary
  - Fall back to full rebuild on Cargo.toml changes
- [ ] Configure `live_update` for agent (optional):
  - Agent runs as jobs, so full rebuild is fine
  - Or implement similar sync for testing
- [ ] Add file watching optimization:
  - Ignore `target/` directory
  - Watch `src/`, `Cargo.toml`, `Cargo.lock`
- [ ] Add restart triggers:
  - Restart on binary change
  - Restart on config change
- [ ] Test and optimize for performance

**Note**: Live reload for compiled languages like Rust is tricky. May need cargo-watch in container.

**Acceptance Criteria**:
- Source changes trigger fast recompile
- Server restarts with new code
- Faster than full Docker rebuild
- Falls back correctly when needed

**Labels**: tilt, development, live-reload, dx

**Dependencies**: Issue 5.2, Issue 5.3

---

## Workstream 6: Documentation & Polish

### Issue 6.1: Document Tanka Project Structure

**Title**: docs: create comprehensive Tanka project documentation

**Description**:
Create documentation explaining the Tanka project structure, how to use it, and how to extend it.

**Tasks**:
- [ ] Create `tanka/README.md`:
  - Project structure overview
  - How Tanka works with Helm
  - How to add new components
  - How to modify configurations
- [ ] Document jsonnet libraries:
  - Each library in `lib/` gets doc comments
  - Usage examples
  - Common patterns
- [ ] Create troubleshooting guide:
  - Common errors
  - How to debug jsonnet
  - How to inspect generated YAML
- [ ] Add Tanka workflow guide:
  - Development workflow
  - Testing changes
  - Deploying to production

**Acceptance Criteria**:
- Documentation is clear and comprehensive
- Examples are working and tested
- New contributors can understand the structure
- Troubleshooting covers common issues

**Labels**: documentation, tanka

**Dependencies**: All Workstream 1 and 2 issues

---

### Issue 6.2: Document Tilt Development Workflow

**Title**: docs: create Tilt development workflow documentation

**Description**:
Create documentation explaining how to use Tilt for development, debugging, and testing.

**Tasks**:
- [ ] Create `docs/TILT_WORKFLOW.md`:
  - How to start development environment
  - How to use Tilt UI
  - How to view logs
  - How to debug issues
  - How to trigger actions
- [ ] Update root README.md:
  - Quick start with Tilt
  - Prerequisites (just, tilt, tanka, k3s)
  - First-time setup steps
- [ ] Create video/GIF walkthrough:
  - `tilt up` demo
  - Showing live reload
  - Showing Tilt UI features
- [ ] Document common workflows:
  - Making changes to server
  - Testing agent jobs
  - Updating infrastructure

**Acceptance Criteria**:
- New developers can get started quickly
- Common workflows are documented
- Screenshots/videos show Tilt in action
- README is updated with Tilt instructions

**Labels**: documentation, tilt, dx

**Dependencies**: All Workstream 5 issues

---

### Issue 6.3: Update Justfile with Tanka and Tilt Commands

**Title**: feat: add Tanka and Tilt commands to justfile

**Description**:
Update the justfile with convenient commands for working with Tanka and Tilt.

**Tasks**:
- [ ] Add Tanka commands:
  - `just tk-show` - Show generated YAML
  - `just tk-diff` - Diff against cluster
  - `just tk-apply` - Apply to cluster
  - `just tk-fmt` - Format jsonnet files
  - `just tk-validate` - Validate jsonnet
- [ ] Add Tilt commands:
  - `just dev` - Start Tilt (alias for `tilt up`)
  - `just dev-down` - Stop Tilt (alias for `tilt down`)
  - `just dev-ci` - Run Tilt CI mode
- [ ] Add combined commands:
  - `just deploy-local` - Deploy via Tanka directly
  - `just reset-local` - Reset local cluster
- [ ] Update justfile documentation

**Acceptance Criteria**:
- Commands work correctly
- Commands are well documented
- Shortcuts improve developer experience

**Labels**: dx, justfile, tooling

**Dependencies**: All Workstream 1-5 issues

---

### Issue 6.4: Create CI/CD Workflow for Tanka Validation

**Title**: ci: add GitHub Actions workflow for Tanka validation

**Description**:
Create GitHub Actions workflow that validates Tanka configurations on every PR.

**Tasks**:
- [ ] Create `.github/workflows/tanka-validate.yml`:
  - Install Tanka and jsonnet tools
  - Install jsonnet-bundler
  - Vendor dependencies
  - Run `tk fmt --check`
  - Run `tk show` to validate
  - Report errors
- [ ] Add to required checks
- [ ] Add badge to README
- [ ] Add workflow documentation

**Acceptance Criteria**:
- Workflow runs on PRs
- Invalid jsonnet is caught
- Errors are clear and actionable

**Labels**: ci, tanka, validation

**Dependencies**: Issue 6.1

---

## Summary

**Total Issues**: 23 issues across 6 workstreams

**Parallel Workstreams**:
- Workstream 1-2 can run first (foundation + infrastructure)
- Workstream 3-4 can run in parallel after 1-2
- Workstream 5 depends on 1-4
- Workstream 6 can run alongside 5

**Estimated Timeline**:
- Workstream 1: 1-2 days
- Workstream 2: 2-3 days
- Workstream 3: 2-3 days
- Workstream 4: 1-2 days
- Workstream 5: 2-3 days
- Workstream 6: 1-2 days

**Total**: ~2-3 weeks with parallel work

**Key Milestones**:
1. Tanka project structure complete (after WS1)
2. Infrastructure managed by Tanka (after WS2)
3. Applications deployable via Tanka (after WS3)
4. `tilt up` works end-to-end (after WS5)
5. Documentation complete (after WS6)
