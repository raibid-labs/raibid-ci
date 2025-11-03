# Testing Guide

Comprehensive testing documentation for raibid-ci.

## Test Organization

```
tests/
├── e2e/              # End-to-end pipeline tests
├── performance/      # Load and performance tests
├── cache/            # Build cache tests
├── failure/          # Chaos engineering tests
├── integration/      # Component integration tests
├── fixtures/         # Shared test data
└── helpers/          # Test utilities
```

## Test Categories

### Unit Tests
Located in each crate's `src/` directory alongside the code.

```bash
# Run all unit tests
just test-unit

# Run unit tests for specific crate
just test-server
just test-agent
just test-cli
```

### Integration Tests
Test interactions between components.

```bash
# Run all integration tests
just test-integration

# Run specific integration test
cargo test --test redis_test
```

### E2E Tests
Validate complete CI pipeline workflows.

**Requirements:** k3s, Redis, Gitea, Docker

```bash
# Run E2E tests
just test-e2e

# Run with cleanup
just test-e2e-clean

# View E2E documentation
cat tests/e2e/README.md
```

**Covered scenarios:**
- Webhook to job queueing
- KEDA autoscaling
- Build execution
- Log streaming
- Image publishing
- Agent scale-down

### Performance Tests
Measure system behavior under load.

**Requirements:** Running raibid-server, Redis, k8s with KEDA

```bash
# Run performance tests
just test-performance
```

**Test scenarios:**
- Burst load (20 concurrent jobs)
- Sustained load (constant rate)
- Ramp-up load (gradual increase)
- Resource usage monitoring
- Autoscaling responsiveness

**Performance targets:**
- Response time P95: < 2s
- Success rate: > 95%
- Throughput: > 5 req/s sustained
- First agent spawn: < 30s

### Cache Tests
Validate build cache functionality.

**Requirements:** Cargo, test fixture project

```bash
# Run cache tests
just test-cache
```

**Test scenarios:**
- Cold cache build (no cache)
- Warm cache build (full cache hit)
- Incremental build (source change)
- Dependency change
- Cache invalidation
- Cache cleanup

**Performance targets:**
- Warm build speedup: 2-5x
- Cache hit rate: > 70%
- Incremental speedup: 5-10x

### Failure Recovery Tests
Chaos engineering and resilience testing.

**Requirements:** k8s cluster, kubectl access

```bash
# Run failure tests
just test-failure
```

**Test scenarios:**
- Redis pod deletion
- Gitea restart
- Agent termination
- API server crash
- Network partition
- Resource exhaustion
- KEDA scaler failure
- Cascading failures

**Recovery targets:**
- Redis recovery: < 60s
- API server recovery: < 60s
- Gitea recovery: < 120s
- Cascading recovery: < 180s

## Running Tests

### Quick Start
```bash
# Install dependencies
just install-tools

# Run all unit and integration tests
just test

# Run all tests with coverage
just test-coverage
```

### External Service Tests
Tests marked with `#[ignore]` require external services.

```bash
# Set environment variable
export TEST_EXTERNAL=1

# Run specific external test category
just test-e2e
just test-performance
just test-cache
just test-failure

# Run all external tests
just test-all-external
```

### Environment Variables

#### Server URLs
- `TEST_SERVER_URL` - raibid-server endpoint (default: `http://localhost:8080`)
- `TEST_REDIS_URL` - Redis URL (default: `redis://localhost:6379`)
- `TEST_GITEA_URL` - Gitea URL (default: `http://localhost:3000`)
- `TEST_REGISTRY_URL` - OCI registry (default: `localhost:5000`)

#### Test Configuration
- `TEST_EXTERNAL` - Enable external service tests (set to `1`)
- `RAIBID_NAMESPACE` - Kubernetes namespace (default: `raibid-ci`)
- `CACHE_DIR` - Cache test directory (default: `/tmp/raibid-cache-test`)

### With Tilt Development Environment

```bash
# Terminal 1: Start all services
just dev

# Terminal 2: Run tests
just test-e2e
just test-performance
```

## CI Integration

### GitHub Actions Workflows

#### PR Tests (`.github/workflows/test.yml`)
Runs on every pull request:
- Unit tests
- Integration tests
- Linting and formatting
- Code coverage

#### E2E Tests (`.github/workflows/e2e-tests.yml`)
Runs on:
- Main branch commits
- Weekly schedule
- Manual dispatch

#### Performance Tests (`.github/workflows/performance.yml`)
Runs on:
- Weekly schedule
- Release tags
- Manual dispatch

### Test Reporting
- Coverage reports uploaded to Codecov
- Performance metrics tracked over time
- Failure recovery times documented in runbook

## Writing Tests

### Test Structure
```rust
#[tokio::test]
#[ignore] // For tests requiring external services
async fn test_feature() {
    // Arrange
    let config = setup_test_config();

    // Act
    let result = perform_operation(&config).await;

    // Assert
    assert!(result.is_ok(), "Operation should succeed");
}
```

### Test Fixtures
Shared test data in `tests/fixtures/`:
- `rust-project/` - Minimal Rust project for build tests
- `sample_config.yaml` - Example configuration

### Test Helpers
Utilities in `tests/helpers/`:
- `test_env.rs` - Temporary environment setup
- `generators.rs` - Test data generators
- `mod.rs` - Shared test utilities

## Troubleshooting

### Tests Fail to Connect to Services
```bash
# Verify services are running
just dev-status

# Check service health
curl http://localhost:8080/health
redis-cli PING
kubectl get pods -n raibid-ci
```

### Tests Timeout
- Increase timeout values in test code
- Check system resource availability
- Verify network connectivity

### Cache Tests Fail
```bash
# Clean cache directory
rm -rf /tmp/raibid-cache-test

# Verify Cargo version
cargo --version  # Should be 1.70+
```

### Failure Tests Fail
- Verify kubectl access
- Check cluster has sufficient resources
- Ensure pods can be deleted (RBAC permissions)

## Coverage Goals

- **Unit tests**: > 80% code coverage
- **Integration tests**: All API endpoints
- **E2E tests**: Critical user workflows
- **Performance tests**: Load scenarios
- **Failure tests**: Recovery procedures

## Best Practices

### Do
- Write tests before fixing bugs
- Use descriptive test names
- Test edge cases and error conditions
- Clean up test artifacts
- Run tests locally before pushing

### Don't
- Run failure tests in production
- Skip external service tests without verification
- Commit test artifacts or temporary files
- Ignore flaky tests

## Operational Runbook

After running failure recovery tests, refer to the operational runbook for production scenarios:

```bash
cat docs/RUNBOOK.md
```

The runbook includes:
- Common failure scenarios
- Recovery procedures
- Expected recovery times
- Monitoring and alerts
- Backup and restore procedures

## Future Improvements

- [ ] Implement contract testing for API
- [ ] Add mutation testing
- [ ] Property-based testing for core logic
- [ ] Visual regression testing for TUI
- [ ] Distributed cache testing
- [ ] Multi-cluster testing
- [ ] Security/penetration testing

## Resources

- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [Kubernetes Testing Best Practices](https://kubernetes.io/docs/tasks/debug/debug-cluster/)

---

**Last Updated:** 2025-11-03
**Maintained By:** raibid-ci Team
