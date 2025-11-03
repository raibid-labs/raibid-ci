# Build Cache Testing

Test suite for validating build cache functionality and performance.

## Test Scenarios

### Cold Cache Build
- **Purpose**: Measure baseline build time without cache
- **Expected**: Full compilation, all dependencies downloaded
- **Metrics**: Build time, cache size created

### Warm Cache Build
- **Purpose**: Measure build time with full cache hit
- **Expected**: Minimal compilation, dependencies from cache
- **Metrics**: Build time, speedup factor, cache hit rate
- **Target**: 2-5x speedup vs cold build

### Incremental Build
- **Purpose**: Test cache behavior after source changes
- **Expected**: Only changed modules recompiled
- **Metrics**: Build time, partial cache usage

### Dependency Change
- **Purpose**: Test cache invalidation on Cargo.toml changes
- **Expected**: New dependencies downloaded, existing cached
- **Metrics**: Build time, network savings

### Cache Invalidation
- **Purpose**: Verify cache properly invalidates when needed
- **Expected**: Stale cache detected and rebuilt
- **Metrics**: Invalidation correctness

### Cache Performance
- **Purpose**: Comprehensive cache performance measurement
- **Expected**: Meet all performance targets
- **Metrics**: Hit rate, size, network savings, speedup

### Cache Cleanup
- **Purpose**: Test cache pruning functionality
- **Expected**: Old artifacts removed, size reduced
- **Metrics**: Size before/after cleanup

## Performance Targets

### Build Time Speedup
- Warm cache: 2-5x faster than cold
- Incremental: 5-10x faster than cold

### Cache Hit Rate
- Warm build: >70%
- Incremental: >50%

### Storage
- Cache size: <2GB for typical project
- Cleanup: Should reduce size

### Network Savings
- Warm build: >100MB saved
- Dependencies: Cached across builds

## Running Tests

```bash
# Run all cache tests
just test-cache

# Run specific test
TEST_EXTERNAL=1 cargo test --test cache_test test_warm_cache_build -- --ignored --nocapture

# Run with custom cache location
export CACHE_DIR=/custom/path
just test-cache
```

## Cache Configuration

The tests use the test fixture project at `tests/fixtures/rust-project/`.

### Cache Locations
- Default: `/tmp/raibid-cache-test`
- Cargo cache: `~/.cargo/registry/`, `~/.cargo/git/`
- Target dir: `target/` or `$CARGO_TARGET_DIR`

### Cache Layers
1. **Dependency cache** - crates.io downloads
2. **Build cache** - Compiled dependencies
3. **Incremental cache** - Partial compilation results

## Expected Results

### Cold Build
```
Build time: ~20-30s
Cache created: ~500MB
Dependencies: Downloaded from crates.io
```

### Warm Build
```
Build time: ~5-10s (2-5x speedup)
Cache hit rate: >70%
Network saved: ~100MB
```

### Incremental Build
```
Build time: ~2-5s (5-10x speedup)
Changed files: Recompiled
Unchanged: From cache
```

## Troubleshooting

### Low Speedup
- Verify cache directory exists
- Check CARGO_TARGET_DIR is set
- Ensure dependencies haven't changed

### Cache Not Working
- Check cache permissions
- Verify Cargo version (1.70+)
- Review cache invalidation triggers

### Large Cache Size
- Run `cargo clean` to reset
- Check for duplicate artifacts
- Review cache retention policies

## CI Integration

Cache tests run in CI to:
- Verify cache correctness
- Detect cache regressions
- Validate performance targets
- Test cleanup procedures

## Future Enhancements

- Distributed cache (S3, registry)
- Cache warming strategies
- Multi-project cache sharing
- Cache size limits and LRU eviction
