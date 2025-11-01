# Infrastructure Error Handling

This directory contains comprehensive error handling and recovery mechanisms for infrastructure operations.

## Modules

### `error.rs`
Defines comprehensive error types with detailed context and actionable suggestions.

**Key Types:**
- `InfraError`: Main error enum with variants for all failure scenarios
- `InstallPhase`: Installation phase tracking for detailed error reporting
- `HelmOperation`: Helm operation types
- `ValidationError`: Validation error details

**Features:**
- Detailed error messages with context
- Actionable recovery suggestions
- Transient vs fatal error classification
- Automatic retry delay calculation

### `retry.rs`
Implements retry logic with exponential backoff for transient failures.

**Key Types:**
- `RetryConfig`: Configurable retry behavior
- `retry_with_backoff`: Synchronous retry function
- `retry_with_backoff_async`: Asynchronous retry function
- `poll_until`: Poll for condition with timeout
- `poll_until_async`: Async polling

**Features:**
- Exponential backoff with jitter
- Configurable max attempts and delays
- Automatic transient error detection
- Quick, slow, and custom configurations

### `preflight.rs`
Provides pre-flight validation checks before installation.

**Key Types:**
- `SystemRequirements`: System prerequisites definition
- `PreFlightValidator`: Validation executor
- `PreFlightResult`: Validation results

**Checks:**
- Disk space availability
- Memory availability
- Required commands in PATH
- Optional commands
- Required directories
- Network connectivity

**Predefined Requirements:**
- `k3s_requirements()`
- `gitea_requirements()`
- `redis_requirements()`
- `keda_requirements()`
- `flux_requirements()`

### `rollback.rs`
Implements transaction-like rollback for infrastructure changes.

**Key Types:**
- `RollbackManager`: Manages rollback actions
- `RollbackContext`: Tracks installed resources
- `RollbackAction`: Cleanup function type

**Features:**
- Automatic rollback on failure (via Drop)
- LIFO action execution
- Resource tracking (files, directories, K8s resources, Helm releases)
- Partial cleanup reporting
- Commit to disable rollback on success

### `healthcheck.rs`
Provides health checking with timeout support.

**Key Types:**
- `HealthStatus`: Health status enum
- `HealthCheckResult`: Health check results
- `K3sHealthChecker`: K3s cluster health checker
- `HelmHealthChecker`: Helm release health checker

**Features:**
- Multiple check types
- Configurable timeouts
- Wait-until-healthy functionality
- Detailed check results

## Usage Examples

### Basic Error Handling

```rust
use raibid_cli::infrastructure::{InfraError, InfraResult, InstallPhase};

fn install_component() -> InfraResult<()> {
    // Return detailed errors
    Err(InfraError::installation(
        "k3s",
        InstallPhase::Download,
        "Failed to download binary",
    ))
}
```

### Retry Logic

```rust
use raibid_cli::infrastructure::{RetryConfig, retry_with_backoff};

let config = RetryConfig::quick();
let result = retry_with_backoff(&config, "download", || {
    download_file()
})?;
```

### Pre-flight Validation

```rust
use raibid_cli::infrastructure::{PreFlightValidator, k3s_requirements};

let validator = PreFlightValidator::new(k3s_requirements());
validator.validate("k3s")?;
```

### Rollback

```rust
use raibid_cli::infrastructure::RollbackManager;

let mut rollback = RollbackManager::new("k3s");

// Add cleanup actions
rollback.add_action("remove binary", Box::new(|| {
    std::fs::remove_file("/usr/local/bin/k3s")?;
    Ok(())
}));

// On success
rollback.commit();

// On failure, rollback is automatic
```

### Health Checks

```rust
use raibid_cli::infrastructure::K3sHealthChecker;
use std::time::Duration;

let checker = K3sHealthChecker::new("/path/to/kubeconfig")
    .with_timeout(Duration::from_secs(300));

checker.wait_until_healthy()?;
```

## Testing

Run the comprehensive error handling tests:

```bash
cargo test --test error_handling_test
```

Tests cover:
- Error type creation and formatting
- Retry logic with various scenarios
- Pre-flight validation
- Rollback execution
- Health check evaluation
- Integration workflows

## Documentation

See [docs/error-recovery.md](../../../docs/error-recovery.md) for:
- Detailed error type documentation
- Troubleshooting guide
- Best practices
- Recovery procedures
- Complete examples

## Design Principles

1. **Fail Fast with Context**: Provide immediate, actionable feedback
2. **Automatic Recovery**: Retry transient failures automatically
3. **Clean Failures**: Always clean up on failure
4. **Observable Operations**: Comprehensive logging and status reporting
5. **Type Safety**: Leverage Rust's type system for correctness

## Integration

All infrastructure installers support these error handling features:

- **k3s**: Pre-flight checks, retry downloads, automatic rollback
- **Gitea**: Helm operation retries, namespace cleanup
- **Redis**: Health checks, connection validation
- **KEDA**: CRD validation, operator health checks
- **Flux**: Binary verification, bootstrap retries
