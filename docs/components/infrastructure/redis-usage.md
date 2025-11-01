# Redis Streams Job Queue - Usage Guide

## Quick Start

### 1. Install Redis

Ensure k3s is installed first, then install Redis:

```bash
# Install k3s if not already installed
raibid-cli setup k3s

# Install Redis
raibid-cli setup redis
```

### 2. Verify Installation

Check Redis is running:

```bash
kubectl get pods -n raibid-redis
```

Expected output:
```
NAME                         READY   STATUS    RESTARTS   AGE
raibid-redis-master-0        1/1     Running   0          2m
```

### 3. Access Credentials

Credentials are saved to `~/.raibid/redis-credentials.json`:

```bash
cat ~/.raibid/redis-credentials.json
```

## Using Redis Streams for Job Queue

### Connecting to Redis

From within the k3s cluster (e.g., from a pod):

```rust
use redis::Client;
use std::fs;

// Read credentials
let creds = fs::read_to_string(
    dirs::home_dir().unwrap().join(".raibid/redis-credentials.json")
)?;
let config: serde_json::Value = serde_json::from_str(&creds)?;

// Create connection
let redis_url = format!(
    "redis://:{}@{}:{}",
    config["password"].as_str().unwrap(),
    config["host"].as_str().unwrap(),
    config["port"].as_u64().unwrap()
);

let client = Client::open(redis_url)?;
let mut con = client.get_connection()?;
```

### Publishing Jobs to the Queue

```rust
use redis::Commands;

// Add a job to the stream
let job_data = vec![
    ("job_id", "job-123"),
    ("job_type", "build"),
    ("repository", "raibid-labs/raibid-cli"),
    ("branch", "main"),
    ("commit", "abc123"),
];

let job_id: String = con.xadd(
    "raibid:jobs",     // Stream name
    "*",                // Auto-generate ID
    &job_data
)?;

println!("Job published with ID: {}", job_id);
```

### Consuming Jobs from the Queue

```rust
use redis::{Commands, streams::StreamReadOptions, streams::StreamReadReply};

// Read from consumer group
let opts = StreamReadOptions::default()
    .count(1)                    // Read 1 message at a time
    .group("raibid-workers", "worker-1");  // Consumer group and consumer ID

let results: StreamReadReply = con.xread_options(
    &["raibid:jobs"],
    &[">"],              // Read only new messages
    &opts
)?;

// Process messages
for stream_key in results.keys {
    for stream_id in stream_key.ids {
        println!("Processing job {}", stream_id.id);

        for (field, value) in stream_id.map.iter() {
            println!("  {}: {:?}", field, value);
        }

        // Acknowledge the message after processing
        let _: i32 = con.xack(
            "raibid:jobs",
            "raibid-workers",
            &[&stream_id.id]
        )?;
    }
}
```

### Monitoring Queue Status

```rust
use redis::Commands;

// Get stream information
let stream_info: redis::Value = con.xinfo_stream("raibid:jobs")?;
println!("Stream info: {:?}", stream_info);

// Get consumer group information
let group_info: redis::Value = con.xinfo_groups("raibid:jobs")?;
println!("Consumer groups: {:?}", group_info);

// Get pending messages
let pending: redis::Value = con.xpending_count(
    "raibid:jobs",
    "raibid-workers",
    "-",
    "+",
    10
)?;
println!("Pending messages: {:?}", pending);
```

## Job Data Structure

### Recommended Job Schema

```json
{
  "job_id": "unique-job-identifier",
  "job_type": "build|test|deploy",
  "repository": "org/repo",
  "branch": "main",
  "commit": "commit-sha",
  "config": "{json-encoded-config}",
  "priority": "high|normal|low",
  "created_at": "2025-10-29T12:00:00Z"
}
```

### Example: Build Job

```rust
let job = serde_json::json!({
    "job_id": format!("build-{}", uuid::Uuid::new_v4()),
    "job_type": "build",
    "repository": "raibid-labs/raibid-cli",
    "branch": "main",
    "commit": "7629e52",
    "config": {
        "build_steps": [
            "cargo build --release",
            "cargo test"
        ],
        "cache_enabled": true,
        "timeout_minutes": 30
    },
    "priority": "normal",
    "created_at": chrono::Utc::now().to_rfc3339()
});

// Convert to Redis field-value pairs
let fields: Vec<(&str, String)> = vec![
    ("job_id", job["job_id"].as_str().unwrap().to_string()),
    ("job_type", job["job_type"].as_str().unwrap().to_string()),
    ("repository", job["repository"].as_str().unwrap().to_string()),
    ("branch", job["branch"].as_str().unwrap().to_string()),
    ("commit", job["commit"].as_str().unwrap().to_string()),
    ("config", serde_json::to_string(&job["config"]).unwrap()),
    ("priority", job["priority"].as_str().unwrap().to_string()),
    ("created_at", job["created_at"].as_str().unwrap().to_string()),
];

con.xadd("raibid:jobs", "*", &fields)?;
```

## Manual Testing

### From kubectl

```bash
# Get Redis password
REDIS_PASSWORD=$(cat ~/.raibid/redis-credentials.json | jq -r '.password')

# Port forward Redis
kubectl port-forward -n raibid-redis svc/raibid-redis-master 6379:6379 &

# Test connection
redis-cli -h localhost -p 6379 -a "$REDIS_PASSWORD" PING

# Add a test job
redis-cli -h localhost -p 6379 -a "$REDIS_PASSWORD" XADD raibid:jobs "*" \
  job_id test-1 \
  job_type build \
  repository raibid-labs/raibid-cli

# Read from consumer group
redis-cli -h localhost -p 6379 -a "$REDIS_PASSWORD" XREADGROUP GROUP raibid-workers worker-1 COUNT 1 STREAMS raibid:jobs ">"

# View stream info
redis-cli -h localhost -p 6379 -a "$REDIS_PASSWORD" XINFO STREAM raibid:jobs

# View consumer groups
redis-cli -h localhost -p 6379 -a "$REDIS_PASSWORD" XINFO GROUPS raibid:jobs
```

### From Pod

```bash
# Get pod name
POD=$(kubectl get pod -n raibid-redis -l app.kubernetes.io/component=master -o jsonpath='{.items[0].metadata.name}')

# Get password
REDIS_PASSWORD=$(cat ~/.raibid/redis-credentials.json | jq -r '.password')

# Execute redis-cli in pod
kubectl exec -n raibid-redis $POD -- redis-cli -a "$REDIS_PASSWORD" PING

# Add test job
kubectl exec -n raibid-redis $POD -- redis-cli -a "$REDIS_PASSWORD" XADD raibid:jobs "*" job_id test-1 job_type build

# Read from stream
kubectl exec -n raibid-redis $POD -- redis-cli -a "$REDIS_PASSWORD" XREADGROUP GROUP raibid-workers worker-1 COUNT 1 STREAMS raibid:jobs ">"
```

## Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
redis = { version = "0.24", features = ["streams"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
```

## Best Practices

### 1. Message Acknowledgment

Always acknowledge messages after successful processing:

```rust
// Process job
process_job(&job)?;

// Acknowledge after success
con.xack("raibid:jobs", "raibid-workers", &[&job_id])?;
```

### 2. Error Handling

Handle failed jobs appropriately:

```rust
match process_job(&job) {
    Ok(_) => {
        // Acknowledge success
        con.xack("raibid:jobs", "raibid-workers", &[&job_id])?;
    }
    Err(e) => {
        // Log error, potentially re-queue or move to dead-letter queue
        eprintln!("Job {} failed: {}", job_id, e);
        // Don't ack - message stays in pending
    }
}
```

### 3. Consumer IDs

Use unique consumer IDs per worker:

```rust
let consumer_id = format!("worker-{}", hostname);
```

### 4. Graceful Shutdown

Claim pending messages before shutdown:

```rust
// Claim messages from dead consumers
let pending = con.xpending_count("raibid:jobs", "raibid-workers", "-", "+", 100)?;
// Process pending messages...
```

### 5. Monitoring

Regularly check for:
- Pending message count
- Consumer group lag
- Dead consumers
- Stream length

## Troubleshooting

### Connection Issues

```bash
# Check if pod is running
kubectl get pods -n raibid-redis

# Check pod logs
kubectl logs -n raibid-redis raibid-redis-master-0

# Test connectivity from another pod
kubectl run redis-test --rm -it --image=redis:7 -- redis-cli -h raibid-redis-master.raibid-redis.svc.cluster.local -p 6379 -a <password> PING
```

### Stream Issues

```bash
# View stream length
redis-cli XLEN raibid:jobs

# View pending messages
redis-cli XPENDING raibid:jobs raibid-workers

# View consumer group info
redis-cli XINFO CONSUMERS raibid:jobs raibid-workers

# Clear dead consumers
redis-cli XGROUP DELCONSUMER raibid:jobs raibid-workers <dead-consumer-id>
```

### Performance Issues

- Check stream length (max 10,000 by default)
- Monitor memory usage
- Consider adding read replicas
- Enable connection pooling

## References

- [Redis Streams Introduction](https://redis.io/docs/data-types/streams/)
- [Redis Streams Tutorial](https://redis.io/docs/data-types/streams-tutorial/)
- [redis-rs Documentation](https://docs.rs/redis/latest/redis/)
