# raibid-ci Workstreams

This directory organizes the project work into parallel workstreams for multi-agent development.

## 🚀 Quick Start for Claude

**New to this project? Start here:**
1. Read [`START_HERE.md`](./START_HERE.md) for multi-agent launch instructions
2. Review [`docs/ORCHESTRATION.md`](../architecture/orchestration.md) for complete orchestration guide
3. Launch Phase 1 agents (WS-01, WS-04, WS-05) immediately

**TL;DR:** Use Claude Code's Task tool to spawn agents for each workstream. Each agent follows the TDD workflow in their workstream README.

## Workstream Overview

| ID | Workstream | Status | Dependencies | Parallelizable |
|----|-----------|--------|--------------|----------------|
| WS-01 | Infrastructure Core | Not Started | None | ✅ Start immediately |
| WS-02 | Data Services | Not Started | WS-01 | ⚠️ Partial (Redis ∥ Gitea after k3s) |
| WS-03 | GitOps & Orchestration | Not Started | WS-02 | ⚠️ Sequential within stream |
| WS-04 | API Services | Not Started | None | ✅ Start immediately |
| WS-05 | Client TUI | Not Started | None | ✅ Start immediately |
| WS-06 | CI Agents | Not Started | WS-02 (Redis) | ⚠️ Start after Redis ready |
| WS-07 | Repository Management | Not Started | WS-02 (Gitea) | ⚠️ Start after Gitea ready |
| WS-08 | Integration & Deployment | Not Started | All | ❌ Final integration phase |

## Parallelization Strategy

### Phase 1: Foundation (Week 1)
**Parallel Workstreams:**
- WS-01: Infrastructure Core (k3s cluster)
- WS-04: API Services (project scaffolding, code structure)
- WS-05: Client TUI (project scaffolding, UI prototypes)

**Goal:** Get infrastructure up while development work proceeds in parallel.

### Phase 2: Services & Development (Week 2)
**Parallel Workstreams:**
- WS-02: Data Services (Gitea + Redis deployment)
- WS-03: GitOps & Orchestration (Flux + KEDA setup)
- WS-04: API Services (webhook handlers, job tracking)
- WS-05: Client TUI (dashboard layout, data fetching)
- WS-07: Repository Management (mirroring strategy design)

**Dependencies:**
- WS-02 requires WS-01 complete
- WS-03 requires WS-02 in progress (Gitea ready for Flux)

### Phase 3: Agents & Integration (Week 3)
**Parallel Workstreams:**
- WS-06: CI Agents (container build, job consumer, build pipeline)
- WS-07: Repository Management (mirror configuration, org sync)
- WS-04: API Services (deployment)
- WS-05: Client TUI (interactive controls, deployment)

**Dependencies:**
- WS-06 requires WS-02 complete (Redis ready)
- WS-07 requires WS-02 complete (Gitea ready)

### Phase 4: Testing & Deployment (Week 4)
**Sequential Workstream:**
- WS-08: Integration & Deployment (end-to-end testing, performance tuning)

**Dependencies:**
- Requires all workstreams complete

## Critical Path

```
WS-01 (k3s) → WS-02 (Gitea) → WS-03 (Flux→KEDA) → WS-06 (Agent) → WS-08 (Integration)
                                                      ↑
                                         WS-02 (Redis) ───────────┘
```

**Critical Path Duration:** ~11-13 days

## Resource Allocation

### Agent Assignment Recommendations

**Infrastructure Specialists (2 agents):**
- WS-01: Infrastructure Core
- WS-02: Data Services
- WS-03: GitOps & Orchestration

**Backend Developers (2 agents):**
- WS-04: API Services
- WS-06: CI Agents

**Frontend/Client Developers (1 agent):**
- WS-05: Client TUI

**DevOps/Integration (1 agent):**
- WS-07: Repository Management
- WS-08: Integration & Deployment

## Progress Tracking

Each workstream directory contains:
- `README.md` - Workstream description, dependencies, issue list
- Issue placeholders organized by priority
- Dependency documentation
- Status tracking

## References

- [Project Plan](../work/plan.md) - Original milestone-based plan
- [Technology Research](../technology-research.md) - Technical references
- [Diagrams](../diagrams/) - Architecture visualizations
