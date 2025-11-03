# End-to-End Tests

E2E tests validate complete CI pipeline workflows from webhook to deployment.

## Overview

The E2E test suite validates:
- Webhook reception and job queueing
- Redis Streams integration
- KEDA autoscaling and agent spawning
- Build execution and log streaming
- Docker image publishing
- Agent termination and scale-to-zero

## Test Files

- `ci_pipeline_test.rs` - Complete CI pipeline E2E test
- `cleanup.sh` - Test artifact cleanup script

## Test Fixtures

- `fixtures/rust-project/` - Minimal Rust project for testing
  - Simple library with tests
  - Dockerfile for image building
  - `.raibid.yml` configuration

## Requirements

### External Services
- **k3s/kubectl** - Kubernetes cluster (v1.28+)
- **Redis** - Job queue and log streaming (v7.0+)
- **Gitea** - Git server and OCI registry (v1.21+)
- **Docker** - Image building and registry access

### Environment Variables
- `TEST_SERVER_URL` - raibid-server URL (default: http://localhost:8080)
- `TEST_REDIS_URL` - Redis URL (default: redis://localhost:6379)
- `TEST_GITEA_URL` - Gitea URL (default: http://localhost:3000)
- `TEST_REGISTRY_URL` - OCI registry URL (default: localhost:5000)

## Running E2E Tests

### Full Test Suite
```bash
# Run all E2E tests (requires external services)
just test-e2e

# Or directly with cargo
TEST_EXTERNAL=1 cargo test --test ci_pipeline_test -- --ignored --nocapture
```

### Individual Tests
```bash
# Run specific test
TEST_EXTERNAL=1 cargo test --test ci_pipeline_test test_complete_ci_pipeline -- --ignored --nocapture
```

### With Tilt (Recommended)
```bash
# Start development environment with all services
just dev

# In another terminal, run E2E tests
just test-e2e
```

## Test Flow

1. **Setup** - Verify all external services are available
2. **Trigger Webhook** - POST to `/webhooks/gitea` with test payload
3. **Verify Job Queued** - Check Redis Streams for job entry
4. **Wait for Agent** - Poll Kubernetes for agent pod creation (KEDA)
5. **Monitor Status** - Poll `/jobs/{id}` endpoint for status transitions
6. **Verify Logs** - Check `/jobs/{id}/logs` SSE endpoint
7. **Verify Image** - Inspect Docker registry for published image
8. **Verify Scale-Down** - Poll Kubernetes for agent pod termination
9. **Cleanup** - Remove test artifacts

## Cleanup

```bash
# Manual cleanup after tests
./tests/e2e/cleanup.sh

# Or run cleanup test
TEST_EXTERNAL=1 cargo test --test ci_pipeline_test cleanup_test_artifacts -- --ignored
```

## Timeouts

- **Overall Test**: 5 minutes (300s)
- **Agent Spawn**: 60 seconds
- **Job Completion**: 3 minutes (180s)
- **Agent Scale-Down**: 2 minutes (120s)

## Expected Behavior

### Success Criteria
- All services respond to health checks
- Webhook returns 202 Accepted with job_id
- Job appears in Redis Streams within 1s
- Agent pod spawns within 60s
- Job transitions: pending -> running -> success
- Logs are captured and accessible
- Docker image is pushed (optional in MVP)
- Agent pod terminates within 120s after job completion

### Acceptable Failures
- Docker registry image verification (registry may not be configured)
- Agent scale-down timeout (KEDA cooldown may be longer than test timeout)

## Troubleshooting

### Services Not Available
```bash
# Check service health
curl http://localhost:8080/health
redis-cli -u redis://localhost:6379 PING
kubectl get pods -n raibid-ci
```

### Agent Not Spawning
```bash
# Check KEDA scaler
kubectl describe scaledjob raibid-agent -n raibid-ci

# Check Redis stream
redis-cli -u redis://localhost:6379 XLEN ci:jobs
```

### Job Not Completing
```bash
# Check agent logs
kubectl logs -n raibid-ci -l app=raibid-agent --tail=100

# Check job status in Redis
redis-cli -u redis://localhost:6379 HGETALL job:YOUR_JOB_ID
```

## CI Integration

E2E tests run in CI on:
- Pull requests (if `TEST_EXTERNAL=1` environment variable is set)
- Main branch commits
- Manual workflow dispatch

See `.github/workflows/e2e-tests.yml` for CI configuration.
