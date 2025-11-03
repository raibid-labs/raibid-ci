# Next Steps - Post Wave 3

**Current Status**: Wave 3 Complete (Tilt Integration) ✅
**Date**: 2025-11-03

---

## Wave 3 Completion Summary

✅ **All 5 issues complete**:
- Issue #102: Base Tiltfile with k3s management
- Issue #103: Docker image builds
- Issue #104: Tanka deployments
- Issue #105: Port forwards and shortcuts
- Issue #106: Live reload evaluation

✅ **4 files created** (54KB total):
- `Tiltfile` (15KB) - Main orchestration
- `TILT.md` (13KB) - Usage documentation
- `docs/TILT_SETUP.md` (8.2KB) - Setup checklist
- `docs/work/WAVE3_COMPLETION_SUMMARY.md` (18KB) - Completion summary

---

## Immediate Actions Required

### 1. Test the Tilt Integration

**Prerequisites to install**:
```bash
# Check current status
kubectl cluster-info  # Should connect to k3s
tilt version          # Should show version
docker ps             # Should show running daemon
tk --version          # Should show Tanka version

# If missing, install:
# 1. k3s
cd /home/beengud/raibid-labs/raibid-ci/infra/k3s
sudo ./install.sh

# 2. Tilt
curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash

# 3. Docker (if needed)
sudo systemctl start docker

# 4. Vendor Helm charts
cd /home/beengud/raibid-labs/raibid-ci/tanka
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add gitea-charts https://dl.gitea.io/charts/
helm repo add kedacore https://kedacore.github.io/charts
helm repo add fluxcd-community https://fluxcd-community.github.io/helm-charts
helm pull bitnami/redis --untar -d vendor/
helm pull gitea-charts/gitea --untar -d vendor/
helm pull kedacore/keda --untar -d vendor/
helm pull fluxcd-community/flux2 --untar -d vendor/
```

**Test workflow**:
```bash
# Start Tilt
cd /home/beengud/raibid-labs/raibid-ci
tilt up

# Verify in Tilt UI (http://localhost:10350):
# - All resources green
# - No error logs
# - Port forwards working

# Test services:
curl http://localhost:8080/health      # Server API
curl http://localhost:8081/metrics     # Metrics
open http://localhost:3000             # Gitea UI
redis-cli -h localhost -p 6379 ping    # Redis

# Stop Tilt
tilt down
```

### 2. Document Test Results

Create file: `docs/work/WAVE3_TEST_RESULTS.md`

Include:
- Prerequisites installation status
- Tilt startup output
- Any errors encountered
- Resolution steps taken
- Final working status

### 3. Create GitHub Issues

Convert these Wave 3 issues to GitHub issues:

```bash
# Use GitHub CLI or web interface
gh issue create --title "Issue #102: Create base Tiltfile with k3s management" --body-file docs/work/issue-102.md
gh issue create --title "Issue #103: Configure Docker image builds in Tiltfile" --body-file docs/work/issue-103.md
gh issue create --title "Issue #104: Integrate Tanka deployments in Tiltfile" --body-file docs/work/issue-104.md
gh issue create --title "Issue #105: Configure port forwards and shortcuts in Tiltfile" --body-file docs/work/issue-105.md
gh issue create --title "Issue #106: Configure live reload for Rust development in Tilt" --body-file docs/work/issue-106.md
```

Then close them with completion notes.

---

## Short-term Enhancements (Wave 3.5?)

### Priority 1: Complete Test Job Trigger

**Issue**: Implement `trigger-test-job` manual trigger

**Tasks**:
1. Create `scripts/send-test-job.sh`:
   ```bash
   #!/usr/bin/env bash
   # Send a test job to Redis Streams queue

   redis-cli -h localhost -p 6379 XADD raibid:jobs '*' \
     type 'test' \
     command 'cargo test --all' \
     repository 'raibid-labs/raibid-ci' \
     ref 'main'
   ```

2. Update Tiltfile:
   ```python
   local_resource(
       name='trigger-test-job',
       cmd='./scripts/send-test-job.sh',
       auto_init=False,
       trigger_mode=TRIGGER_MODE_MANUAL,
       labels=['tools'],
   )
   ```

3. Test:
   - Click button in Tilt UI
   - Verify job in Redis: `redis-cli XLEN raibid:jobs`
   - Watch agent pod scale up

### Priority 2: Add Log Filtering

**Issue**: Improve log readability in Tilt UI

**Tasks**:
1. Filter health check noise
2. Highlight errors in red
3. Show job context in agent logs

**Example**:
```python
# In Tiltfile, for server resource:
k8s_resource(
    workload='raibid-server',
    new_name='server',
    # ... other config ...
    # TODO: Add log filtering
    # pod_readiness='wait'
)
```

### Priority 3: Performance Tuning

**Issue**: Optimize build times for different environments

**Tasks**:
1. Profile build stages:
   ```bash
   docker build --progress=plain -f crates/server/Dockerfile . 2>&1 | tee build.log
   ```

2. Adjust parallel builds based on machine:
   - DGX Spark: `max_parallel_updates=4`
   - Desktop: `max_parallel_updates=2`
   - Laptop: `max_parallel_updates=1`

3. Document optimal settings in TILT.md

---

## Long-term Work

### Wave 4: Observability (Future)

**Potential Issues**:
- Prometheus deployment
- Grafana dashboards
- Alert rules
- Log aggregation (Loki?)

### Wave 5: CI/CD Integration

**Potential Issues**:
- GitHub Actions workflow
- Tilt CI mode in pipeline
- Automated testing
- Release automation

### Wave 6: Multi-Environment Support

**Potential Issues**:
- Dev/staging/prod Tanka environments
- Environment-specific Tiltfiles
- Secret management across environments
- Production deployment strategy

---

## Technical Debt

### Documentation

- ✅ Tilt usage guide (TILT.md)
- ✅ Setup checklist (TILT_SETUP.md)
- ⚠️ Need: Troubleshooting playbook with real issues encountered
- ⚠️ Need: Video walkthrough or GIF demonstrations

### Code Quality

- ✅ Tiltfile well-commented
- ✅ Organized into sections
- ⚠️ Need: Unit tests for helper functions (if possible)
- ⚠️ Need: Linting/validation for Tiltfile

### Testing

- ⚠️ Need: Full end-to-end test on DGX Spark
- ⚠️ Need: Test on x86_64 machine
- ⚠️ Need: Test with k3d instead of k3s
- ⚠️ Need: Load testing of agent scaling

---

## Questions to Answer

### Architecture

1. **Agent scaling**: What's the optimal min/max replica count for KEDA?
2. **Resource quotas**: Are current k3s quotas appropriate for DGX Spark?
3. **Cache strategy**: Should we use external cache volume for cargo?

### Development Workflow

1. **Local vs Tilt**: When should devs use cargo watch vs Tilt?
2. **Team usage**: How do multiple devs share a single k3s cluster?
3. **Debugging**: What's the best way to debug agent jobs?

### Operations

1. **Monitoring**: What metrics should we track in production?
2. **Scaling limits**: What's the max number of concurrent agents?
3. **Storage**: How much disk space for Docker images and build cache?

---

## Success Criteria for Wave 3 Sign-off

Before considering Wave 3 truly complete:

- [ ] Tilt successfully runs on DGX Spark
- [ ] All services accessible via port forwards
- [ ] Docker builds complete without errors
- [ ] Tanka deploys all resources successfully
- [ ] Resource dependencies enforced correctly
- [ ] Manual triggers work as expected
- [ ] Documentation validated by user testing
- [ ] At least one complete development cycle tested (edit → build → deploy → test)

---

## Metrics to Track

### Build Performance

- First build time: _____ minutes
- Dependency change rebuild: _____ minutes
- Source change rebuild: _____ seconds
- Average rebuild time: _____ seconds

### Resource Usage

- Docker build peak memory: _____ GB
- Tilt memory overhead: _____ MB
- k3s base memory: _____ MB
- Total cluster memory: _____ GB

### Developer Experience

- Time to `tilt up` completion: _____ minutes
- Time to see code changes live: _____ seconds
- Number of manual steps required: _____ (goal: 1)
- Documentation clarity: ___/10

---

## Contact / Support

For questions or issues:

1. **Documentation**: Review TILT.md and TILT_SETUP.md
2. **Tiltfile comments**: Check inline documentation
3. **Completion summary**: See WAVE3_COMPLETION_SUMMARY.md
4. **GitHub**: Open issue with `tilt` label
5. **Tilt Slack**: https://slack.tilt.dev/

---

## Conclusion

Wave 3 has established a solid foundation for Tilt-based development. The next critical step is **testing** on actual hardware with all prerequisites installed.

Once tested and validated, Wave 3 can be considered production-ready, and we can move forward with:
- Wave 4: Observability
- Wave 5: CI/CD Integration
- Or tackle enhancements listed above

**Current blocker**: Need working k3s cluster and Tilt installation to validate implementation.

---

**Status**: Wave 3 Complete, Awaiting Testing ✅
**Next Action**: Install prerequisites and test `tilt up`
**Documentation**: Complete and ready for use
