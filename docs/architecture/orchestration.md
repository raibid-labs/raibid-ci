# Multi-Agent Orchestration Guide

This document provides instructions for orchestrating multiple AI agents to complete the raibid-ci project using parallel workstreams.

## Overview

The project is organized into **8 workstreams** with **59 issues** total. Multiple agents can work in parallel on different workstreams or within the same workstream (where dependencies allow).

## Quick Start

### Option 1: Automated Launch (Recommended)

Use Claude Code's Task tool to spawn all agents concurrently:

```bash
# From project root, use Claude Code to launch agents
# Claude will spawn agents using the Task tool

# Example: Launch all initial workstreams
Task("Infrastructure Agent", "Complete WS-01: Infrastructure Core workstream. Follow the workflow in docs/workstreams/01-infrastructure-core/README.md", "backend-architect")
Task("API Developer", "Complete WS-04: API Services workstream. Follow Rust TDD workflow in docs/workstreams/04-api-services/README.md", "rust-pro")
Task("TUI Developer", "Complete WS-05: Client TUI workstream. Follow Rust TDD workflow in docs/workstreams/05-client-tui/README.md", "rust-pro")
```

### Option 2: Manual MCP Coordination (Advanced)

Use MCP tools for advanced swarm coordination:

```bash
# Initialize swarm topology
npx claude-flow@alpha hooks pre-task --description "Initialize raibid-ci development swarm"
npx claude-flow@alpha swarm init --topology mesh --max-agents 6

# Define agent types and spawn
npx claude-flow@alpha agent spawn --type infrastructure --workstream WS-01
npx claude-flow@alpha agent spawn --type backend-dev --workstream WS-04
npx claude-flow@alpha agent spawn --type rust-pro --workstream WS-05
```

## Workstream Dependencies & Execution Phases

### Phase 1: Foundation (Start Immediately)
**Duration:** ~4-7 days | **Agents:** 3

Launch these workstreams in parallel on Day 1:

| Workstream | Agent Type | Priority | Can Start | Duration |
|-----------|-----------|----------|-----------|----------|
| **WS-01**: Infrastructure Core | `backend-architect` or `cloud-architect` | Critical | ✅ Immediately | 3-4 days |
| **WS-04**: API Services | `rust-pro` or `backend-dev` | Critical | ✅ Immediately | 4-6 days |
| **WS-05**: Client TUI | `rust-pro` or `frontend-developer` | High | ✅ Immediately | 5-7 days |

**Spawn Command:**
```bash
# Using Claude Code Task tool (recommended)
Task("Infra Agent", "Complete WS-01 following docs/workstreams/01-infrastructure-core/README.md. Use TDD workflow. Report progress via hooks.", "cloud-architect")
Task("API Agent", "Complete WS-04 following docs/workstreams/04-api-services/README.md. Rust TDD workflow. Coordinate via memory.", "rust-pro")
Task("TUI Agent", "Complete WS-05 following docs/workstreams/05-client-tui/README.md. Rust TDD workflow. Coordinate via memory.", "rust-pro")
```

**Coordination:**
- WS-01 blocks future workstreams - highest priority
- WS-04 and WS-05 are independent development work
- Agents should report progress every 2-4 hours
- Use shared memory for cross-workstream context

### Phase 2: Services & Core Development (After WS-01)
**Duration:** ~3-4 days | **Agents:** 3-5

Start after WS-01 k3s cluster is operational:

| Workstream | Agent Type | Priority | Depends On | Duration |
|-----------|-----------|----------|------------|----------|
| **WS-02**: Data Services | `backend-architect` or `database-admin` | Critical | WS-01 complete | 3-4 days |
| **WS-07**: Repository Management | `backend-dev` or `golang-pro` | Medium | None (strategy) | 3-4 days |
| Continue **WS-04**, **WS-05** | - | - | - | - |

**Spawn Command:**
```bash
# When WS-01 reports completion
Task("Data Services Agent", "Complete WS-02 following docs/workstreams/02-data-services/README.md. Deploy Gitea and Redis in parallel. Validation tests required.", "database-admin")
Task("Repo Mgmt Agent", "Complete WS-07 following docs/workstreams/07-repository-management/README.md. Start with strategy design (REPO-001). Build mirroring tools.", "golang-pro")
```

**Coordination:**
- WS-02 has internal parallelization: Gitea (DATA-001) ∥ Redis (DATA-004)
- Consider splitting WS-02 into 2 agents if possible
- WS-07 strategy design can start before Gitea is ready

### Phase 3: GitOps & Agents (After WS-02)
**Duration:** ~4-6 days | **Agents:** 3-4

Start after WS-02 Gitea and Redis are deployed:

| Workstream | Agent Type | Priority | Depends On | Duration |
|-----------|-----------|----------|------------|----------|
| **WS-03**: GitOps & Orchestration | `kubernetes-architect` or `deployment-engineer` | Critical | WS-02 (Gitea) | 2-3 days |
| **WS-06**: CI Agents | `rust-pro` or `backend-dev` | Critical | WS-02 (Redis) | 4-6 days |
| Continue **WS-04**, **WS-05**, **WS-07** | - | - | - | - |

**Spawn Command:**
```bash
# When WS-02 Gitea is ready
Task("GitOps Agent", "Complete WS-03 following docs/workstreams/03-gitops-orchestration/README.md. Sequential: Flux → KEDA → ScaledJob. Validation tests.", "kubernetes-architect")

# When WS-02 Redis is ready
Task("CI Agent Developer", "Complete WS-06 following docs/workstreams/06-ci-agents/README.md. Rust TDD workflow. Focus on build pipeline and caching.", "rust-pro")
```

**Coordination:**
- WS-03 is sequential within workstream (Flux → KEDA → ScaledJob)
- WS-06 can start as soon as Redis is deployed
- WS-06 has internal parallelization: AGENT-003 ∥ AGENT-004

### Phase 4: Integration & Testing (After All Workstreams)
**Duration:** ~3-5 days | **Agents:** 1-2

Start after all workstreams complete:

| Workstream | Agent Type | Priority | Depends On | Duration |
|-----------|-----------|----------|------------|----------|
| **WS-08**: Integration & Deployment | `incident-responder` or `tester` | Critical | All workstreams | 3-5 days |

**Spawn Command:**
```bash
# When all workstreams report completion
Task("Integration Agent", "Complete WS-08 following docs/workstreams/08-integration-deployment/README.md. End-to-end testing, performance validation, production readiness.", "tester")
```

## Agent Workflow (All Agents Follow This)

Each agent working on a workstream must follow this TDD-based workflow:

### 1. Initialization
```bash
# Agent reads workstream README
# Example: docs/workstreams/01-infrastructure-core/README.md

# Set up hooks for coordination
npx claude-flow@alpha hooks pre-task --description "Starting work on WS-XX"
npx claude-flow@alpha hooks session-restore --session-id "raibid-ci-ws-XX"
```

### 2. Issue Loop
For each issue in workstream:
1. **Select next issue** (highest priority, not blocked)
2. **Checkout branch** (`git checkout -b <issue-id>-description`)
3. **Write tests first** (TDD - tests should fail initially)
4. **Commit tests** (`git commit -m "test: add tests for <issue-id>"`)
5. **Implement functionality** (make tests pass)
6. **Commit implementation** (`git commit -m "feat(<issue-id>): ..."`)
7. **Create PR** (with test results, docs, issue links)
8. **Verify PR** (tests passing, docs updated, edge cases handled)
9. **Continue to next issue**

### 3. Coordination Hooks
Throughout work, agents should:
```bash
# After each significant step
npx claude-flow@alpha hooks post-edit --file "<file>" --memory-key "raibid-ci/ws-XX/<issue-id>"
npx claude-flow@alpha hooks notify --message "Completed <issue-id>"

# At completion
npx claude-flow@alpha hooks post-task --task-id "ws-XX"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## PR Acceptance Criteria (All PRs Must Meet)

Every PR must satisfy:
- [ ] **Tests passing**: All tests execute successfully (unit, integration, validation)
- [ ] **Documentation updated**: README, runbooks, API docs, code comments
- [ ] **Issue comments added**: Link PR to issue, document decisions, note blockers
- [ ] **Code quality**: No warnings, formatted, no hardcoded secrets, proper error handling
- [ ] **Success criteria met**: All criteria from issue description satisfied

## Monitoring Progress

### Via GitHub
```bash
# Check PR status
gh pr list --state open

# Check CI status
gh run list

# Check specific workstream
gh issue list --label "WS-01"
```

### Via Claude Flow Hooks
```bash
# Check swarm status
npx claude-flow@alpha swarm status

# Check agent metrics
npx claude-flow@alpha agent metrics

# View memory context
npx claude-flow@alpha memory usage
```

### Via TUI (Once Deployed)
```bash
# Launch TUI for real-time monitoring
./raibid-tui
```

## Handling Blockers

### Agent is Blocked
If an agent encounters a blocker:

1. **Comment on issue** with blocker details
2. **Notify coordinator** via hooks or direct message
3. **Switch to another issue** in same workstream (if available)
4. **Offer help** to blocking workstream if no other work available

Example:
```bash
# Comment on issue
gh issue comment <issue-number> --body "Blocked by: WS-02 Redis not yet deployed. Switching to INFRA-002."

# Notify via hooks
npx claude-flow@alpha hooks notify --message "Agent blocked on INFRA-003, switching to INFRA-002"
```

### Workstream is Blocked
If entire workstream is blocked:

1. **Document blocker** in workstream README
2. **Notify all agents** via shared memory
3. **Agent switches workstreams** or assists blocking workstream

## Cross-Workstream Collaboration

### Shared Memory Pattern
Agents use shared memory for cross-workstream context:

```bash
# Store decision/context
npx claude-flow@alpha memory store \
  --key "raibid-ci/shared/gitea-url" \
  --value "gitea.dgx.local:3000"

# Retrieve context
npx claude-flow@alpha memory retrieve --key "raibid-ci/shared/gitea-url"
```

**Common shared keys:**
- `raibid-ci/shared/cluster-ready`: WS-01 completion flag
- `raibid-ci/shared/gitea-url`: Gitea URL and credentials
- `raibid-ci/shared/redis-url`: Redis connection string
- `raibid-ci/shared/api-url`: API endpoint URL
- `raibid-ci/shared/blockers`: Current blockers list

### Building Off Previous Branches
When issues are sequential, agents can build off previous branches:

```bash
# If INFRA-002 builds on INFRA-001
git checkout infra-001-k3s-setup
git checkout -b infra-002-storage-config

# Continue work on new issue
# PR will show changes from previous branch + new work
```

## Example: Full Multi-Agent Launch

### Using Claude Code Task Tool (Recommended)

```javascript
// Single message with all agent spawning
[Parallel Agent Execution in Claude Code]:
  Task("Infrastructure Specialist",
       "Complete WS-01: Infrastructure Core. Follow docs/workstreams/01-infrastructure-core/README.md. Use validation tests. Report progress every 2 hours.",
       "cloud-architect")

  Task("API Backend Developer",
       "Complete WS-04: API Services. Follow docs/workstreams/04-api-services/README.md. Rust TDD workflow. Write tests first, then implement.",
       "rust-pro")

  Task("TUI Frontend Developer",
       "Complete WS-05: Client TUI. Follow docs/workstreams/05-client-tui/README.md. Rust TDD workflow. Focus on usability and real-time updates.",
       "rust-pro")
```

### Using MCP Tools (Advanced)

```bash
# Step 1: Initialize coordination
npx claude-flow@alpha swarm init --topology mesh --max-agents 6 --session-id raibid-ci-main

# Step 2: Spawn agents with workstream assignments
npx claude-flow@alpha agent spawn \
  --type cloud-architect \
  --workstream WS-01 \
  --instructions "Follow TDD workflow in docs/workstreams/01-infrastructure-core/README.md"

npx claude-flow@alpha agent spawn \
  --type rust-pro \
  --workstream WS-04 \
  --instructions "Follow Rust TDD workflow in docs/workstreams/04-api-services/README.md"

npx claude-flow@alpha agent spawn \
  --type rust-pro \
  --workstream WS-05 \
  --instructions "Follow Rust TDD workflow in docs/workstreams/05-client-tui/README.md"

# Step 3: Monitor progress
npx claude-flow@alpha swarm monitor --session-id raibid-ci-main
```

## Workstream Completion Checklist

When a workstream completes, verify:
- [ ] All issues in workstream have PRs
- [ ] All PRs merged to main
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Deliverables met (see workstream README)
- [ ] Dependent workstreams notified
- [ ] Shared memory updated with completion status

## Timeline & Milestones

| Milestone | Workstreams | Duration | Agents |
|-----------|-------------|----------|---------|
| **M1**: Foundation | WS-01, WS-04, WS-05 | 4-7 days | 3 |
| **M2**: Services | WS-02, WS-07 | 3-4 days | 2-3 |
| **M3**: GitOps & Agents | WS-03, WS-06 | 4-6 days | 2-3 |
| **M4**: Integration | WS-08 | 3-5 days | 1-2 |
| **Total** | All workstreams | **21-31 days** | **3-6 parallel** |

## Troubleshooting

### Agent Not Following TDD Workflow
- Verify agent read workstream README
- Check if tests were committed before implementation
- Review PR for test coverage

### Agent Creating Files in Wrong Location
- Verify agent read CLAUDE.md project guidelines
- Check file paths in commits
- Ensure tests/ and docs/ directories used correctly

### Agent Blocked by Missing Dependency
- Check workstream README dependencies section
- Verify blocking workstream completion status
- Assign agent to different issue or workstream

### Multiple Agents Conflicting
- Use separate branches per agent/issue
- Coordinate via shared memory
- Stagger work on interdependent issues

## Success Metrics

Track these metrics for orchestration success:

- **Parallelization Efficiency**: 3+ agents working concurrently
- **Idle Time**: <10% agent idle time
- **Blocker Resolution**: <4 hours average blocker resolution
- **PR Cycle Time**: <24 hours from creation to merge
- **Test Coverage**: >80% for Rust code, 100% validation for infrastructure
- **Rework Rate**: <15% PRs requiring significant changes

## Additional Resources

- **Workstream READMEs**: `docs/workstreams/*/README.md`
- **Dependency Diagram**: `docs/diagrams/workstream-dependencies.md`
- **Technology Research**: `docs/technology-research.md`
- **Project Plan**: `docs/work/plan.md`
- **Claude Flow Docs**: https://github.com/ruvnet/claude-flow

## Quick Reference Commands

```bash
# List all workstreams
ls docs/workstreams/

# Check workstream status
gh issue list --label "WS-01"

# View agent activity
npx claude-flow@alpha swarm status

# Check PR status
gh pr list --state open --json number,title,state,headRefName

# Run all tests
cargo test --all-features  # For Rust workstreams
./tests/*-validation.sh     # For infrastructure workstreams

# View shared context
npx claude-flow@alpha memory list --prefix "raibid-ci/shared/"
```
