# Error Handling and Recovery Guide

## Overview

The raibid-cli infrastructure module provides comprehensive error handling and recovery mechanisms for all infrastructure operations. This guide explains how to use these features and troubleshoot common issues.

## Error Types

### InfraError Variants

The `InfraError` enum provides detailed error information with context and actionable suggestions:

#### Download Errors
```rust
InfraError::Download {
    component: String,
    url: String,
    reason: String,
    suggestion: String,
}
```

**Common Causes:**
- Network connectivity issues
- Invalid URLs
- Missing files (HTTP 404)
- DNS resolution failures

**Recovery:**
- Check network connection
- Verify URLs are correct and accessible
- Check firewall settings
- Retry the operation

#### Installation Errors
```rust
InfraError::Installation {
    component: String,
    phase: InstallPhase,
    reason: String,
    suggestion: String,
}
```

**Install Phases:**
- `PreFlight`: Pre-flight validation checks
- `Download`: Component download
- `Verification`: Checksum/signature verification
- `Installation`: Binary/package installation
- `Configuration`: Configuration setup
- `Bootstrap`: Service/cluster initialization
- `Validation`: Post-install validation
- `PostInstall`: Final setup steps

**Recovery:**
- Review the specific phase that failed
- Check logs for detailed error messages
- Ensure all prerequisites are met
- Verify disk space and permissions
- Retry after addressing the issue

#### Network Errors
```rust
InfraError::Network {
    operation: String,
    reason: String,
    suggestion: String,
}
```

**Common Causes:**
- Timeouts
- Connection refused
- DNS failures
- TLS/SSL issues

**Recovery:**
- Check network connectivity
- Verify firewall rules
- Check proxy settings
- Increase timeout values if needed

#### Timeout Errors
```rust
InfraError::Timeout {
    operation: String,
    duration: Duration,
    suggestion: String,
}
```

**Common Causes:**
- Operations taking longer than expected
- Slow network
- Resource constraints
- Service not starting

**Recovery:**
- Increase timeout values
- Check system resources
- Verify services are actually starting
- Check logs for underlying issues

#### Health Check Failures
```rust
InfraError::HealthCheck {
    component: String,
    check: String,
    reason: String,
    suggestion: String,
}
```

**Common Causes:**
- Services not fully initialized
- Configuration errors
- Resource constraints
- Dependency failures

**Recovery:**
- Wait for services to fully start
- Check component logs
- Verify configuration
- Check resource availability

## Retry Logic

### RetryConfig

Configure retry behavior with exponential backoff:

```rust
use raibid_cli::infrastructure::{RetryConfig, retry_with_backoff};

// Quick retries for network operations
let config = RetryConfig::quick();  // 5 attempts, 500ms-5s delays

// Slow retries for service readiness
let config = RetryConfig::slow();   // 10 attempts, 2s-60s delays

// Custom configuration
let config = RetryConfig {
    max_attempts: 3,
    initial_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(30),
    backoff_multiplier: 2.0,
    use_jitter: true,
};

// Use retry logic
let result = retry_with_backoff(&config, "my_operation", || {
    // Your operation here
    Ok(())
});
```

### Transient vs Fatal Errors

**Transient Errors** (will be retried):
- Network errors
- Timeouts
- Explicit `InfraError::Transient`

**Fatal Errors** (will NOT be retried):
- Invalid configuration
- Unsupported platforms
- Checksum mismatches
- Explicit `InfraError::Fatal`

## Pre-flight Validation

Pre-flight checks ensure system requirements are met before installation:

```rust
use raibid_cli::infrastructure::{
    SystemRequirements, PreFlightValidator,
    k3s_requirements, gitea_requirements
};

// Use predefined requirements
let requirements = k3s_requirements();

// Or create custom requirements
let requirements = SystemRequirements {
    min_disk_space_gb: 10,
    min_memory_gb: 4,
    required_commands: vec!["kubectl".to_string(), "helm".to_string()],
    optional_commands: vec!["flux".to_string()],
    required_directories: vec!["/data".to_string()],
    required_endpoints: vec!["https://github.com".to_string()],
};

// Validate
let validator = PreFlightValidator::new(requirements);
match validator.validate("gitea") {
    Ok(()) => println!("Pre-flight checks passed"),
    Err(e) => println!("Pre-flight checks failed: {}", e),
}
```

### Pre-flight Check Failures

**Common Issues:**
- Missing commands in PATH
- Insufficient disk space
- Insufficient memory
- Network connectivity issues

**Recovery:**
- Install missing dependencies
- Free up disk space
- Check system resources
- Verify network access

## Rollback Mechanisms

Automatic rollback ensures clean cleanup on failures:

```rust
use raibid_cli::infrastructure::{RollbackManager, RollbackContext};

// Create rollback manager
let mut rollback_manager = RollbackManager::new("gitea");

// Add rollback actions
rollback_manager.add_action("remove binary", Box::new(|| {
    std::fs::remove_file("/usr/local/bin/gitea")?;
    Ok(())
}));

rollback_manager.add_action("delete namespace", Box::new(|| {
    // Kubernetes cleanup
    Ok(())
}));

// On success, commit to prevent rollback
rollback_manager.commit();

// On failure, rollback is automatic (via Drop)
// Or manual: rollback_manager.rollback()?;
```

### Rollback Context

Track installed resources for comprehensive cleanup:

```rust
let mut context = RollbackContext::new();

// Track various resources
context.add_file("/usr/local/bin/k3s");
context.add_directory("/var/lib/rancher/k3s");
context.add_k8s_resource("Pod", "gitea-0", Some("gitea".to_string()));
context.add_helm_release("gitea", "gitea");
context.add_systemd_service("k3s");

// Generate rollback actions
context.to_rollback_actions(&mut rollback_manager, Some("/path/to/kubeconfig"));
```

## Health Checks

Monitor component health with timeout support:

```rust
use raibid_cli::infrastructure::{K3sHealthChecker, HelmHealthChecker};
use std::time::Duration;

// K3s health check
let checker = K3sHealthChecker::new("/path/to/kubeconfig")
    .with_timeout(Duration::from_secs(600));

match checker.check() {
    Ok(result) => {
        println!("Status: {}", result.status);
        println!("Message: {}", result.message);
        for check in result.checks {
            println!("  - {}: {}", check.name, check.message);
        }
    }
    Err(e) => println!("Health check failed: {}", e),
}

// Wait for healthy status
checker.wait_until_healthy()?;

// Helm release health check
let checker = HelmHealthChecker::new(
    "/path/to/kubeconfig",
    "gitea",
    "gitea"
).with_timeout(Duration::from_secs(600));

checker.wait_until_healthy()?;
```

## Troubleshooting Guide

### Installation Failures

1. **Check Prerequisites**
   ```bash
   # Verify required commands
   which kubectl helm

   # Check disk space
   df -h

   # Check memory
   free -h
   ```

2. **Review Logs**
   ```bash
   # Infrastructure logs
   kubectl logs -n <namespace> <pod-name>

   # System logs
   journalctl -u k3s
   ```

3. **Verify Network**
   ```bash
   # Test connectivity
   curl -I https://github.com

   # Check DNS
   nslookup github.com
   ```

4. **Check Resources**
   ```bash
   # Kubernetes resources
   kubectl get pods --all-namespaces
   kubectl describe pod <pod-name> -n <namespace>

   # Helm releases
   helm list --all-namespaces
   ```

### Rollback Failures

If automatic rollback fails, manually clean up:

1. **Helm Releases**
   ```bash
   helm uninstall <release> -n <namespace>
   ```

2. **Kubernetes Resources**
   ```bash
   kubectl delete namespace <namespace>
   kubectl delete <resource-type> <name> -n <namespace>
   ```

3. **Files and Directories**
   ```bash
   sudo rm -f /usr/local/bin/<binary>
   sudo rm -rf /var/lib/<component>
   ```

4. **Services**
   ```bash
   sudo systemctl stop <service>
   sudo systemctl disable <service>
   ```

### Timeout Issues

If operations timeout:

1. **Increase Timeout**
   ```rust
   let checker = K3sHealthChecker::new(kubeconfig)
       .with_timeout(Duration::from_secs(1200)); // 20 minutes
   ```

2. **Check Progress**
   ```bash
   # Watch pod status
   kubectl get pods -w -n <namespace>

   # Check events
   kubectl get events -n <namespace> --sort-by='.lastTimestamp'
   ```

3. **Verify Resources**
   ```bash
   # CPU and memory
   kubectl top nodes
   kubectl top pods -A
   ```

### Network Errors

For persistent network issues:

1. **Configure Proxy**
   ```bash
   export HTTP_PROXY=http://proxy:port
   export HTTPS_PROXY=http://proxy:port
   export NO_PROXY=localhost,127.0.0.1
   ```

2. **Adjust Timeouts**
   ```rust
   let config = RetryConfig {
       max_attempts: 5,
       initial_delay: Duration::from_secs(10),
       max_delay: Duration::from_secs(120),
       backoff_multiplier: 2.0,
       use_jitter: true,
   };
   ```

3. **Use Local Mirrors**
   - Configure Helm chart repositories
   - Use local container registries
   - Cache downloaded binaries

## Best Practices

### Error Handling

1. **Use Specific Error Types**
   ```rust
   // Good
   return Err(InfraError::installation(
       "k3s",
       InstallPhase::Download,
       "HTTP 404 Not Found",
   ));

   // Avoid generic errors
   return Err(anyhow!("Download failed"));
   ```

2. **Provide Context**
   ```rust
   // Include helpful suggestions
   InfraError::PrerequisiteMissing {
       component: "gitea".to_string(),
       prerequisite: "kubectl".to_string(),
       suggestion: "Install kubectl: https://kubernetes.io/docs/tasks/tools/".to_string(),
   }
   ```

3. **Log at Appropriate Levels**
   ```rust
   use tracing::{debug, info, warn, error};

   debug!("Attempting download from {}", url);
   info!("Installation completed successfully");
   warn!("Retrying after transient failure");
   error!("Fatal error: {}", err);
   ```

### Retry Logic

1. **Use Appropriate Configs**
   - `RetryConfig::quick()` for network operations
   - `RetryConfig::slow()` for service readiness
   - `RetryConfig::none()` for operations that shouldn't retry

2. **Mark Errors Correctly**
   - Use `InfraError::Transient` for retryable errors
   - Use `InfraError::Fatal` for non-retryable errors

3. **Set Reasonable Timeouts**
   - Consider operation complexity
   - Account for slow networks
   - Balance between responsiveness and reliability

### Rollback

1. **Add Rollback Actions Early**
   ```rust
   // Add rollback immediately after action
   installer.install_binary()?;
   rollback_manager.add_action("remove binary", Box::new(|| {
       fs::remove_file("/usr/local/bin/k3s")?;
       Ok(())
   }));
   ```

2. **Track All Changes**
   - Use `RollbackContext` to track resources
   - Record all state changes
   - Document cleanup procedures

3. **Test Rollback**
   - Verify rollback actions work
   - Test partial failure scenarios
   - Ensure idempotency

### Health Checks

1. **Appropriate Timeouts**
   - 5 minutes for simple services
   - 10+ minutes for complex deployments
   - Consider cluster size and resources

2. **Multiple Check Types**
   - Service existence
   - Pod readiness
   - Endpoint accessibility
   - Functional tests

3. **Poll with Backoff**
   - Don't overwhelm the system
   - Use exponential backoff
   - Add jitter to prevent thundering herd

## Examples

### Complete Installation with Error Handling

```rust
use raibid_cli::infrastructure::*;

pub async fn install_gitea() -> InfraResult<()> {
    // 1. Pre-flight validation
    let requirements = gitea_requirements();
    let validator = PreFlightValidator::new(requirements);
    validator.validate("gitea")?;

    // 2. Setup rollback manager
    let mut rollback_manager = RollbackManager::new("gitea");
    let mut rollback_context = RollbackContext::new();

    // 3. Configure installer
    let config = GiteaConfig::default();
    let installer = GiteaInstaller::with_config(config)?;

    // 4. Install with retry
    let retry_config = RetryConfig::slow();

    retry_with_backoff_async(&retry_config, "gitea installation", || async {
        installer.install().await
    }).await?;

    // Track for rollback
    rollback_context.add_helm_release("gitea", "gitea");
    rollback_context.add_k8s_resource("namespace", "gitea", None);

    // 5. Health check
    let health_checker = HelmHealthChecker::new(
        config.kubeconfig_path.to_str().unwrap(),
        "gitea",
        "gitea"
    ).with_timeout(Duration::from_secs(600));

    health_checker.wait_until_healthy()?;

    // 6. Commit successful installation
    rollback_context.to_rollback_actions(&mut rollback_manager, None);
    rollback_manager.commit();

    Ok(())
}
```

## Support

For additional help:
- Check component logs
- Review Kubernetes events
- Consult component documentation
- File issues on GitHub with:
  - Error messages
  - Component versions
  - System information
  - Steps to reproduce
