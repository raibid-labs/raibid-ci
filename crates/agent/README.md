# raibid-agent

CI agent runner that executes build pipelines from the job queue.

## Overview

The raibid-agent crate provides the core build pipeline execution engine for the raibid-ci system. It handles:

- Complete Rust build pipeline execution
- Job polling from Redis Streams (planned)
- Build cache management with sccache
- Docker image building and publishing
- Real-time log streaming to Redis
- Artifact metadata tracking

## Features

### Build Pipeline

The agent executes a comprehensive Rust build pipeline:

1. **Check** - Verify code compilation (`cargo check`)
2. **Format** - Check code formatting (`cargo fmt --check`)
3. **Clippy** - Run lints (`cargo clippy`)
4. **Test** - Run test suite (`cargo test`)
5. **Build** - Build release binary (`cargo build --release`)
6. **Audit** - Security audit (`cargo audit`)
7. **Docker Build** - Build container image (optional)
8. **Docker Push** - Push to registry (optional)

### Timeouts

- **Step timeout**: 5 minutes per step
- **Pipeline timeout**: 30 minutes total

Steps that exceed the timeout are automatically terminated.

### Log Streaming

Build logs are streamed in real-time to Redis Streams for consumption by the TUI and API:

```
raibid:logs:{job_id}
```

Each log entry includes:
- `timestamp` - ISO 8601 timestamp
- `message` - Log line from stdout/stderr

### Job Status Updates

The pipeline updates job status in Redis at key points:

```
raibid:job:{job_id}
```

Status values:
- `running` - Pipeline is executing
- `success` - Pipeline completed successfully
- `failed` - Pipeline failed

### Artifact Metadata

Build artifacts are tracked in Redis with metadata:

```
raibid:artifacts:{job_id}
```

Metadata includes:
- Docker image name and tag
- Binary artifacts produced
- Build timestamp

## Usage

### Basic Pipeline Execution

```rust
use raibid_agent::{PipelineConfig, PipelineExecutor};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure pipeline
    let config = PipelineConfig {
        job_id: "job-abc123".to_string(),
        repo_path: PathBuf::from("/workspace/repo"),
        use_sccache: true,
        registry_url: Some("https://gitea.example.com".to_string()),
        image_tag: Some("myapp:v1.0.0".to_string()),
        redis_url: Some("redis://localhost:6379".to_string()),
    };

    // Create executor
    let executor = PipelineExecutor::new(config)?;

    // Execute pipeline
    let result = executor.execute().await?;

    // Check results
    if result.success {
        println!("Pipeline succeeded in {}s", result.total_duration_secs);

        if let Some(artifacts) = result.artifacts {
            println!("Built image: {}", artifacts.image.unwrap());
        }
    } else {
        println!("Pipeline failed");
        for step in result.steps {
            if !step.success {
                println!("Failed step: {}", step.step);
                println!("Output: {}", step.output);
            }
        }
    }

    Ok(())
}
```

### With sccache Optimization

Enable sccache for faster builds:

```rust
let config = PipelineConfig {
    job_id: "job-123".to_string(),
    repo_path: PathBuf::from("/workspace/repo"),
    use_sccache: true,  // Enable sccache
    // ... other config
    redis_url: None,
};
```

### Docker Image Building

Build and push Docker images:

```rust
let config = PipelineConfig {
    job_id: "job-123".to_string(),
    repo_path: PathBuf::from("/workspace/repo"),
    use_sccache: false,
    registry_url: Some("https://gitea.example.com".to_string()),
    image_tag: Some("myorg/myapp:latest".to_string()),
    redis_url: None,
};

let executor = PipelineExecutor::new(config)?;
let result = executor.execute().await?;

if let Some(artifacts) = result.artifacts {
    println!("Image: {}", artifacts.image.unwrap());
    println!("Binaries: {:?}", artifacts.binaries);
}
```

### Manual Job Status Updates

Update job status independently:

```rust
executor.update_job_status("running", None).await?;
// ... execute work ...
executor.update_job_status("success", Some(0)).await?;
```

### Store Artifact Metadata

```rust
use raibid_agent::ArtifactMetadata;

let artifacts = ArtifactMetadata {
    image: Some("myapp:v1.0.0".to_string()),
    binaries: vec!["myapp".to_string()],
    built_at: chrono::Utc::now().to_rfc3339(),
};

executor.store_artifacts(&artifacts).await?;
```

## Pipeline Result Structure

```rust
pub struct PipelineResult {
    pub job_id: String,
    pub success: bool,
    pub steps: Vec<StepResult>,
    pub total_duration_secs: u64,
    pub artifacts: Option<ArtifactMetadata>,
}

pub struct StepResult {
    pub step: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub duration_secs: u64,
    pub output: String,  // First 10KB of output
}
```

## Environment Variables

The pipeline respects these environment variables:

- `RUSTC_WRAPPER` - Set to `sccache` when `use_sccache` is enabled
- Standard Rust/Cargo environment variables

## Testing

### Unit Tests

```bash
cargo test --package raibid-agent --lib
```

### Integration Tests

Integration tests require cargo to be installed:

```bash
cargo test --package raibid-agent --test pipeline_integration -- --ignored
```

### Redis Integration Tests

Tests requiring Redis are marked as ignored:

```bash
REDIS_URL=redis://localhost:6379 \
cargo test --package raibid-agent -- --ignored test_pipeline_with_redis
```

## Architecture

### Pipeline Execution Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         Pipeline Executor                        │
└─────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│ Step 1: Check    │ cargo check --all-features                   │
│ Step 2: Format   │ cargo fmt -- --check                         │
│ Step 3: Clippy   │ cargo clippy --all-features -- -D warnings   │
│ Step 4: Test     │ cargo test --all-features                    │
│ Step 5: Build    │ cargo build --release                        │
│ Step 6: Audit    │ cargo audit                                  │
│ Step 7: Docker   │ docker build -t <tag> .                      │
│ Step 8: Push     │ docker push <tag>                            │
└─────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────┐
│                           Redis                                  │
│  • raibid:logs:{job_id}     - Build logs (stream)              │
│  • raibid:job:{job_id}      - Job status (hash)                │
│  • raibid:artifacts:{job_id} - Artifact metadata (string)       │
└─────────────────────────────────────────────────────────────────┘
```

### Error Handling

The pipeline stops on the first failed step. Each step result includes:
- Exit code
- Success/failure status
- Captured output (stdout + stderr)
- Duration

## Future Enhancements

- [ ] Incremental build support
- [ ] Custom pipeline steps from `.raibid.yaml`
- [ ] Parallel test execution
- [ ] Build artifact uploading to S3/MinIO
- [ ] Cache statistics and optimization
- [ ] Multi-stage Docker builds
- [ ] Cross-compilation support

## Related Documentation

- [CI Agent Component](../../docs/components/agent/README.md)
- [Redis Usage Guide](../../docs/components/infrastructure/redis-usage.md)
- [WS-02: CI Agent Core](../../docs/workstreams/02-ci-agent-core/README.md)

## License

MIT OR Apache-2.0
