# Performance and Load Tests

Performance testing suite for raibid-ci to validate system behavior under load.

## Test Scenarios

### Burst Load
- **Purpose**: Test system under sudden high load
- **Configuration**: 20 concurrent jobs, 100 total jobs
- **Metrics**: Response times, success rate, throughput

### Sustained Load
- **Purpose**: Test system under constant load
- **Configuration**: 5 jobs/second sustained rate
- **Metrics**: Requests/second, latency, reliability

### Ramp-Up Load
- **Purpose**: Test system scaling behavior
- **Configuration**: Gradually increase from 0 to peak over 30s
- **Metrics**: Scaling response time, stability

### Resource Usage
- **Purpose**: Monitor CPU, memory, and Redis usage
- **Metrics**: Baseline vs. under-load resource consumption

### Autoscaling Responsiveness
- **Purpose**: Measure KEDA scaling performance
- **Metrics**: Time to spawn agents, time to reach target count

## Performance Targets

### Response Times
- Average: < 500ms
- P95: < 2s
- P99: < 5s

### Throughput
- Minimum: 5 requests/second sustained
- Burst: 20 concurrent requests

### Success Rate
- Burst load: > 95%
- Sustained load: > 98%

### Resource Usage
- Memory increase: < 1GB under load
- CPU: Scales with load

### Autoscaling
- First agent spawn: < 30s
- Scale to 5 agents: < 90s

## Running Tests

```bash
# Run all performance tests
just test-performance

# Run specific test
TEST_EXTERNAL=1 cargo test --test performance_test test_burst_load -- --ignored --nocapture

# Run with custom configuration
export CONCURRENT_JOBS=50
export TOTAL_JOBS=200
just test-performance
```

## Requirements

- Running raibid-server
- Redis available
- Kubernetes cluster with KEDA
- kubectl configured

## Interpreting Results

### Good Performance
- Success rate > 98%
- P95 latency < 2s
- Consistent throughput
- Agents spawn within 30s

### Performance Issues
- High failure rate (> 5%)
- High P95/P99 latency (> 5s)
- Declining throughput over time
- Slow or no autoscaling

## Troubleshooting

### High Latency
- Check server logs for bottlenecks
- Verify Redis performance
- Check network latency

### Low Throughput
- Increase server resources
- Optimize database queries
- Check for connection pool exhaustion

### Failed Requests
- Check server error logs
- Verify service availability
- Check rate limiting configuration

## CI Integration

Performance tests run in CI:
- Weekly on schedule
- On release tags
- Manual workflow dispatch

Regression detection:
- Compare against baseline metrics
- Alert on >20% performance degradation
