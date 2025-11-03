# Tanka + Tilt Deployment Project Tracking

**Status**: 🟡 In Progress
**Created**: 2025-11-02
**Target Completion**: 2-3 weeks

## Overview

This project migrates raibid-ci deployment to use Tanka (jsonnet + Helm) for configuration management and Tilt for local development orchestration. The goal is to achieve `tilt up` to start everything end-to-end.

## Objectives

✅ Single command development environment: `tilt up`
✅ Infrastructure as Code using Tanka + jsonnet
✅ Wrap existing Helm charts (Redis, Gitea, KEDA, Flux)
✅ Deploy raibid-server and raibid-agent via Tanka
✅ Live reload for Rust development
✅ Improved developer experience

## Project Organization

**23 Issues across 6 Workstreams**

### Workstream 1: Foundation (2 issues)
Foundation and base Tanka project structure.

| Issue | Title | Status | Assignee |
|-------|-------|--------|----------|
| #93 | feat: initialize Tanka project with base structure | 🟡 Open | - |
| #94 | feat: create reusable jsonnet libraries for common patterns | 🟡 Open | - |

**Dependencies**: None - can start immediately
**Estimated Time**: 1-2 days

---

### Workstream 2: Infrastructure (4 issues)
Tanka configurations for external dependencies using Helm charts.

| Issue | Title | Status | Assignee |
|-------|-------|--------|----------|
| #95 | feat: create Tanka configuration for Redis with Streams | 🟡 Open | - |
| #96 | feat: create Tanka configuration for Gitea with OCI registry | 🟡 Open | - |
| #97 | feat: create Tanka configuration for KEDA autoscaling | 🟡 Open | - |
| #98 | feat: create Tanka configuration for Flux GitOps | 🟡 Open | - |

**Dependencies**: Workstream 1 (Foundation)
**Estimated Time**: 2-3 days
**Parallelizable**: ✅ Yes - Redis, Gitea, KEDA can be done in parallel. Flux depends on Gitea.

---

### Workstream 3: Applications (3 issues)
Tanka configurations for raibid components (server, agent).

| Issue | Title | Status | Assignee |
|-------|-------|--------|----------|
| #111 | feat: create Tanka configuration for raibid-server deployment | 🟡 Open | - |
| #112 | feat: create Tanka configuration for raibid-agent ScaledJob | 🟡 Open | - |
| #113 | feat: create Tanka configuration for secrets and ConfigMaps | 🟡 Open | - |

**Dependencies**: Workstream 1 (Foundation), Workstream 4 (Docker images)
**Estimated Time**: 2-3 days
**Parallelizable**: ✅ Yes - Server and Agent configs can be done in parallel

---

### Workstream 4: Docker (3 issues)
Container images for server and agent with optimized builds.

| Issue | Title | Status | Assignee |
|-------|-------|--------|----------|
| #99 | feat: create optimized Dockerfile for raibid-server | 🟡 Open | - |
| #100 | feat: optimize agent Dockerfile with build stage | 🟡 Open | - |
| #101 | feat: create docker-compose.yml for local service testing | 🟡 Open | - |

**Dependencies**: None - can start immediately (parallel with Workstream 1)
**Estimated Time**: 1-2 days
**Parallelizable**: ✅ Yes - All can be done in parallel

---

### Workstream 5: Tilt Integration (5 issues)
Development orchestration with Tilt for the complete developer experience.

| Issue | Title | Status | Assignee |
|-------|-------|--------|----------|
| #102 | feat: create Tiltfile for k3s cluster management | 🟡 Open | - |
| #103 | feat: configure Docker image builds in Tiltfile | 🟡 Open | - |
| #104 | feat: integrate Tanka deployments in Tiltfile | 🟡 Open | - |
| #105 | feat: configure port forwards and shortcuts in Tiltfile | 🟡 Open | - |
| #106 | feat: configure live reload for Rust development in Tilt | 🟡 Open | - |

**Dependencies**: All Workstreams 1-4
**Estimated Time**: 2-3 days
**Parallelizable**: ⚠️ Partial - #102 first, then #103 and #104 in parallel, then #105 and #106

---

### Workstream 6: Documentation (4 issues)
Documentation and polish for excellent developer experience.

| Issue | Title | Status | Assignee |
|-------|-------|--------|----------|
| #107 | docs: create comprehensive Tanka project documentation | 🟡 Open | - |
| #108 | docs: create Tilt development workflow documentation | 🟡 Open | - |
| #109 | feat: add Tanka and Tilt commands to justfile | 🟡 Open | - |
| #110 | ci: add GitHub Actions workflow for Tanka validation | 🟡 Open | - |

**Dependencies**: Workstreams 1-5
**Estimated Time**: 1-2 days
**Parallelizable**: ✅ Yes - Can all be done in parallel once dependencies are complete

---

## Critical Path

The critical path for completing the project:

```
1. Foundation (WS1) → 2. Infrastructure (WS2) → 5. Tilt Integration (WS5) → 6. Documentation (WS6)
                   → 3. Applications (WS3) →
   4. Docker (WS4) →
```

**Parallel Execution Plan**:
- **Week 1**: Start WS1 (Foundation) and WS4 (Docker) in parallel
- **Week 2**: Start WS2 (Infrastructure) and WS3 (Applications) in parallel once WS1 is done
- **Week 3**: Start WS5 (Tilt Integration) once WS2, WS3, WS4 are complete
- **Week 3-4**: Complete WS6 (Documentation) alongside final WS5 tasks

## Key Milestones

| Milestone | Description | Issues | Target |
|-----------|-------------|--------|--------|
| 🏗️ Foundation Complete | Tanka project structure ready | #93-94 | Day 2 |
| 📦 Infrastructure Managed | All infra via Tanka | #95-98 | Day 5 |
| 🚀 Apps Deployable | Server & Agent via Tanka | #111-113 | Day 7 |
| 🐳 Docker Optimized | Production-ready images | #99-101 | Day 4 |
| ⚡ Tilt Working | `tilt up` works end-to-end | #102-106 | Day 12 |
| 📚 Docs Complete | Full documentation | #107-110 | Day 14 |

## Success Criteria

**Must Have** (MVP):
- [ ] `tilt up` starts k3s, builds images, deploys everything
- [ ] Server and Agent deploy successfully
- [ ] KEDA autoscaling works (0-10 agents)
- [ ] Changes trigger fast rebuilds
- [ ] Basic documentation exists

**Should Have** (Enhanced):
- [ ] Live reload for Rust code changes
- [ ] Tilt UI with port forwards and shortcuts
- [ ] Comprehensive documentation with examples
- [ ] CI validation for Tanka configs

**Nice to Have** (Future):
- [ ] Multi-environment support (dev, staging, prod)
- [ ] Secrets management integration
- [ ] Observability stack (metrics, logs)

## Getting Started (Once Complete)

Prerequisites:
```bash
# Install required tools
cargo install just
brew install tilt-dev/tap/tilt  # or appropriate package manager
brew install tanka jsonnet-bundler
```

One command to rule them all:
```bash
tilt up
```

This will:
1. ✅ Start k3s cluster (if not running)
2. ✅ Build server and agent Docker images
3. ✅ Deploy Redis, Gitea, KEDA, Flux via Tanka
4. ✅ Deploy raibid-server and raibid-agent via Tanka
5. ✅ Set up port forwards and live reload
6. ✅ Open Tilt UI in browser

## Progress Tracking

**Overall Progress**: 0 / 23 issues (0%)

### By Workstream:
- WS1 Foundation: 0 / 2 (0%)
- WS2 Infrastructure: 0 / 4 (0%)
- WS3 Applications: 0 / 3 (0%)
- WS4 Docker: 0 / 3 (0%)
- WS5 Tilt: 0 / 5 (0%)
- WS6 Documentation: 0 / 4 (0%)

---

## Notes

### Reference Material
- **Tanka Docs**: https://tanka.dev
- **Tilt Docs**: https://docs.tilt.dev
- **mop-core Reference**: https://github.com/gudo11y/mop-core (structural reference)

### Technical Decisions
- **Tanka over raw Helm**: Better composition, type safety, reusability
- **Single environment (local)**: Simplifies MVP, can expand later
- **k3s for local**: Lightweight, fast, production-like
- **Tilt for orchestration**: Best-in-class dev experience for K8s

### Known Challenges
- Live reload for compiled Rust code (slower than interpreted languages)
- Managing Helm chart versions in vendor/
- Testing without a real k3s cluster in CI

---

Last Updated: 2025-11-02
