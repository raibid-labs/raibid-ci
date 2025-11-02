# 🚀 Multi-Agent Workstream Execution - START HERE

## For Claude: How to Orchestrate This Project

You are about to orchestrate multiple AI agents to build the **raibid-ci** project using parallel workstreams. This document tells you exactly how to do it.

## 📋 Quick Start

### Step 1: Read the Orchestration Guide
**First, read this:** [`docs/ORCHESTRATION.md`](../architecture/orchestration.md)

It contains:
- Complete multi-agent launch instructions
- Phase-by-phase execution plan
- Dependency management
- Coordination patterns

### Step 2: Launch Initial Workstreams

Use Claude Code's Task tool to spawn 3 agents immediately:

```javascript
// Copy and execute this in Claude Code:
Task("Infrastructure Agent",
     "Complete WS-01: Infrastructure Core. Follow the TDD workflow in docs/workstreams/01-infrastructure-core/README.md. Run validation tests before each commit. Report progress every 2 hours via hooks.",
     "cloud-architect")

Task("API Developer",
     "Complete WS-04: API Services. Follow the Rust TDD workflow in docs/workstreams/04-api-services/README.md. Write tests first, then implement. Use cargo watch for continuous testing.",
     "rust-pro")

Task("TUI Developer",
     "Complete WS-05: Client TUI. Follow the Rust TDD workflow in docs/workstreams/05-client-tui/README.md. Focus on usability and real-time updates. Mock external services in tests.",
     "rust-pro")
```

### Step 3: Monitor and Launch Next Phase

**Wait for WS-01 to complete**, then launch:

```javascript
Task("Data Services Agent",
     "Complete WS-02: Data Services. Follow docs/workstreams/02-data-services/README.md. Deploy Gitea and Redis in parallel (DATA-001 ∥ DATA-004). Validation tests required.",
     "database-admin")

Task("Repo Management Agent",
     "Complete WS-07: Repository Management. Follow docs/workstreams/07-repository-management/README.md. Start with REPO-001 strategy design immediately.",
     "golang-pro")
```

### Step 4: Continue Through Phases

Follow the phase-by-phase plan in `docs/ORCHESTRATION.md` sections:
- Phase 2: Services & Core Development
- Phase 3: GitOps & Agents
- Phase 4: Integration & Testing

## 📂 Workstream Structure

Each workstream has a README with:
- Issue list (prioritized)
- TDD workflow (test-first development)
- Dependencies (what blocks this work)
- Parallelization notes (what can run in parallel)
- Success criteria

**Workstream READMEs:**
```
docs/workstreams/
├── 01-infrastructure-core/README.md    ✅ Can start immediately
├── 02-data-services/README.md          ⏳ Blocked by WS-01
├── 03-gitops-orchestration/README.md   ⏳ Blocked by WS-02
├── 04-api-services/README.md           ✅ Can start immediately
├── 05-client-tui/README.md             ✅ Can start immediately
├── 06-ci-agents/README.md              ⏳ Blocked by WS-02
├── 07-repository-management/README.md  ✅ Strategy can start immediately
└── 08-integration-deployment/README.md ⏳ Blocked by all workstreams
```

## 🔄 TDD Workflow (Every Agent Follows This)

For EVERY issue, agents must:

1. **Checkout branch** (named after issue)
2. **Write tests FIRST** (they will fail - that's expected!)
3. **Commit tests** (push to remote)
4. **Implement functionality** (make tests pass)
5. **Commit implementation** (incremental commits)
6. **Create PR** (with test results, docs, issue link)
7. **Verify** (tests passing, docs updated, edge cases handled)
8. **Continue** to next issue

**Critical:** Tests must be written BEFORE implementation. This is non-negotiable.

## 🧪 Test Requirements by Workstream Type

### Rust Code (WS-04, WS-05, WS-06)
```rust
// tests/feature_test.rs
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_feature_works() {
        // Test implementation
    }
}
```

**Required checks:**
- `cargo test --all-features`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Infrastructure (WS-01, WS-02, WS-03)
```bash
#!/bin/bash
# tests/infra-001-validation.sh

# Test deployment
kubectl get deployment <service> -n <namespace>
kubectl get pods -n <namespace> | grep Running
# etc.
```

**Required checks:**
- All validation scripts pass
- Services are healthy
- Manifests are valid YAML

## 📊 Tracking Progress

### View All Issues
```bash
# List all issues across workstreams
gh issue list --label "WS-01,WS-02,WS-03,WS-04,WS-05,WS-06,WS-07,WS-08"

# Check specific workstream
gh issue list --label "WS-01"
```

### View PRs
```bash
# List open PRs
gh pr list --state open

# Check PR status
gh pr status
```

### Monitor Agents (if using MCP)
```bash
# Check swarm status
npx claude-flow@alpha swarm status

# View agent metrics
npx claude-flow@alpha agent metrics
```

## 🚧 Handling Blockers

If an agent is blocked:

1. **Comment on issue** with blocker details
2. **Switch to another issue** in same workstream
3. **Report to coordinator** via hooks or direct message
4. **Offer help** to blocking workstream if idle

Example:
```bash
gh issue comment 123 --body "Blocked: waiting for WS-01 k3s cluster. Switching to INFRA-002."
```

## ✅ PR Acceptance Criteria

Every PR must have:
- [ ] **Tests passing** (unit, integration, or validation)
- [ ] **Documentation updated** (README, runbooks, code comments)
- [ ] **Issue linked** (use "Closes #123" in PR description)
- [ ] **No warnings** (cargo clippy, YAML validation)
- [ ] **No secrets** (no hardcoded credentials)
- [ ] **Success criteria met** (from issue description)

## 🎯 Success Metrics

Aim for:
- **3+ agents working in parallel**
- **<10% agent idle time**
- **<24 hours PR cycle time**
- **>80% test coverage** (Rust)
- **100% validation coverage** (infrastructure)

## 📚 Key Documents

| Document | Purpose |
|----------|---------|
| [`ORCHESTRATION.md`](../architecture/orchestration.md) | Complete orchestration guide |
| [`workstreams/README.md`](./README.md) | Workstream overview |
| [`STRUCTURE.md`](./STRUCTURE.md) | Project structure |
| [`diagrams/workstream-dependencies.md`](../diagrams/workstream-dependencies.md) | Dependency diagram |
| [`technology-research.md`](../technology-research.md) | Technical references |

## 🚀 Agent Launch Templates

### Phase 1 Launch (Day 1)
```javascript
// Launch 3 agents immediately
Task("Infra Agent", "WS-01 from docs/workstreams/01-infrastructure-core/README.md", "cloud-architect")
Task("API Agent", "WS-04 from docs/workstreams/04-api-services/README.md", "rust-pro")
Task("TUI Agent", "WS-05 from docs/workstreams/05-client-tui/README.md", "rust-pro")
```

### Phase 2 Launch (After WS-01)
```javascript
// Launch 2 more agents
Task("Data Agent", "WS-02 from docs/workstreams/02-data-services/README.md", "database-admin")
Task("Mirror Agent", "WS-07 from docs/workstreams/07-repository-management/README.md", "golang-pro")
```

### Phase 3 Launch (After WS-02)
```javascript
// Launch 2 more agents
Task("GitOps Agent", "WS-03 from docs/workstreams/03-gitops-orchestration/README.md", "kubernetes-architect")
Task("CI Agent", "WS-06 from docs/workstreams/06-ci-agents/README.md", "rust-pro")
```

### Phase 4 Launch (After All Complete)
```javascript
// Launch final integration agent
Task("Integration Agent", "WS-08 from docs/workstreams/08-integration-deployment/README.md", "tester")
```

## 🔍 Quick Health Checks

### Before Starting
```bash
# Verify git repo is clean
git status

# Check current branch
git branch --show-current  # Should be 'main'

# Verify you have access
gh auth status
```

### During Execution
```bash
# Check PR status
gh pr list --state open

# Check test status
gh run list --limit 5

# View recent commits
git log --oneline -n 10
```

### After Completion
```bash
# Verify all workstreams done
gh issue list --state closed --label "WS-01,WS-02,WS-03,WS-04,WS-05,WS-06,WS-07,WS-08"

# Check main branch
git checkout main
git pull
cargo test --all-features  # Should pass
```

## 💡 Pro Tips

1. **Start with 3 agents** (WS-01, WS-04, WS-05) - don't overload
2. **Wait for WS-01** before launching WS-02 - critical path
3. **Monitor PR queue** - merge quickly to unblock others
4. **Use shared memory** - avoid duplicate work
5. **Document blockers** - communicate early and often

## 🆘 Getting Help

If stuck:
1. Read the relevant workstream README
2. Check `docs/ORCHESTRATION.md` for coordination patterns
3. Review `docs/technology-research.md` for technical details
4. Check `docs/work/plan.md` for original milestone plan

## ⚡ Common Commands Reference

```bash
# Agent workflow
git checkout -b <issue-id>-description  # Start work
cargo test                               # Run tests
git commit -m "test: add tests..."      # Commit tests
cargo test                               # Verify implementation
gh pr create                             # Create PR

# Coordination
npx claude-flow@alpha hooks pre-task    # Start coordination
npx claude-flow@alpha hooks post-edit   # Report progress
npx claude-flow@alpha hooks post-task   # Complete coordination

# Monitoring
gh pr list --state open                 # Check PRs
gh run list                              # Check CI
npx claude-flow@alpha swarm status      # Check agents
```

---

**Ready to start?** Launch Phase 1 agents now! 🚀
