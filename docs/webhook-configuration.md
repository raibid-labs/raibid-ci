# Webhook Configuration Guide

This guide explains how to configure webhooks for raibid-ci to receive events from GitHub and Gitea.

## Overview

The raibid-ci server exposes two webhook endpoints:

- **Gitea**: `POST /webhooks/gitea`
- **GitHub**: `POST /webhooks/github`

Both endpoints support HMAC-SHA256 signature verification for security and return `202 Accepted` with a job ID when successful.

## Configuration

### Environment Variables

Configure webhook secrets using environment variables:

```bash
# Gitea webhook secret
export RAIBID_GITEA_WEBHOOK_SECRET="your-gitea-secret"

# GitHub webhook secret
export RAIBID_GITHUB_WEBHOOK_SECRET="your-github-secret"

# Redis connection URL
export RAIBID_REDIS_URL="redis://127.0.0.1:6379"

# Rate limiting (requests per minute)
export RAIBID_RATE_LIMIT_RPM=100
```

### Server Configuration

When initializing the server programmatically:

```rust
use raibid_server::{Server, ServerConfig};

let config = ServerConfig {
    host: "0.0.0.0".to_string(),
    port: 8080,
    redis_url: "redis://127.0.0.1:6379".to_string(),
    gitea_webhook_secret: Some("your-gitea-secret".to_string()),
    github_webhook_secret: Some("your-github-secret".to_string()),
    rate_limit_rpm: 100,
    ..Default::default()
};

let server = Server::new(config);
server.run().await?;
```

## Gitea Webhook Setup

### 1. Navigate to Repository Settings

1. Go to your Gitea repository
2. Click **Settings** > **Webhooks**
3. Click **Add Webhook** > **Gitea**

### 2. Configure Webhook

- **Target URL**: `http://your-server:8080/webhooks/gitea`
- **HTTP Method**: `POST`
- **POST Content Type**: `application/json`
- **Secret**: Enter the same secret as `RAIBID_GITEA_WEBHOOK_SECRET`
- **Trigger On**: Select `Push events`
- **Active**: Check this box

### 3. Test Webhook

Click **Test Delivery** to verify the configuration.

## GitHub Webhook Setup

### 1. Navigate to Repository Settings

1. Go to your GitHub repository
2. Click **Settings** > **Webhooks**
3. Click **Add webhook**

### 2. Configure Webhook

- **Payload URL**: `http://your-server:8080/webhooks/github`
- **Content type**: `application/json`
- **Secret**: Enter the same secret as `RAIBID_GITHUB_WEBHOOK_SECRET`
- **Which events**: Select `Just the push event`
- **Active**: Check this box

### 3. Test Webhook

After saving, GitHub will send a ping event. Check the webhook delivery status.

## Webhook Payload

### Request

Both Gitea and GitHub send POST requests with JSON payloads containing:

```json
{
  "ref": "refs/heads/main",
  "before": "abc123...",
  "after": "def456...",
  "repository": {
    "id": 1,
    "name": "repo-name",
    "full_name": "owner/repo-name",
    "clone_url": "https://git.example.com/owner/repo-name.git",
    "default_branch": "main"
  },
  "pusher": {
    "username": "pusher-name",
    "email": "pusher@example.com"
  }
}
```

### Response

Successful webhook processing returns `202 Accepted`:

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Job 550e8400-e29b-41d4-a716-446655440000 queued successfully"
}
```

## Signature Verification

### Gitea

Gitea sends the HMAC-SHA256 signature in the `X-Gitea-Signature` header as a hex string:

```
X-Gitea-Signature: a1b2c3d4e5f6...
```

### GitHub

GitHub sends the HMAC-SHA256 signature in the `X-Hub-Signature-256` header with `sha256=` prefix:

```
X-Hub-Signature-256: sha256=a1b2c3d4e5f6...
```

## Job Queuing

When a webhook is received:

1. **Signature Verification**: If a secret is configured, the HMAC signature is verified
2. **Payload Parsing**: JSON payload is parsed and validated
3. **Metadata Extraction**: Repository, branch, commit, and author information is extracted
4. **Job Creation**: A UUID job ID is generated
5. **Redis Streams**: Job is queued to Redis Streams using `XADD ci:jobs`
6. **Response**: `202 Accepted` with job ID is returned

### Job Metadata

Jobs are stored in Redis Streams with the following metadata:

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "repository": "owner/repo-name",
  "branch": "main",
  "commit": "def456...",
  "author": "pusher-name",
  "event_type": "push",
  "created_at": "2024-01-01T00:00:00Z"
}
```

## Error Handling

### 400 Bad Request

Returned when:
- Webhook payload is malformed
- Required fields are missing
- JSON parsing fails

```json
{
  "error": "Invalid webhook payload: missing field 'repository'",
  "status": 400
}
```

### 401 Unauthorized

Returned when:
- Signature verification fails
- Signature header is missing when secret is configured

```json
{
  "error": "Invalid signature",
  "status": 401
}
```

### 429 Too Many Requests

Returned when rate limit is exceeded:

```json
{
  "error": "Rate limit exceeded",
  "status": 429
}
```

### 500 Internal Server Error

Returned when:
- Redis connection fails
- Job queuing fails

```json
{
  "error": "Failed to queue job: connection error",
  "status": 500
}
```

## Rate Limiting

By default, webhook endpoints are rate-limited to 100 requests per minute. This can be configured using `RAIBID_RATE_LIMIT_RPM`.

## Security Best Practices

1. **Always use secrets**: Configure webhook secrets in production
2. **Use HTTPS**: Deploy behind a reverse proxy with TLS
3. **Validate payloads**: Server validates all incoming payloads
4. **Rate limiting**: Configured by default to prevent abuse
5. **IP whitelisting**: Consider restricting webhook sources at the firewall level

## Testing

### Using curl

Test Gitea webhook:

```bash
# Generate signature
PAYLOAD='{"ref":"refs/heads/main","repository":{"full_name":"owner/repo"},"pusher":{"username":"test"}}'
SECRET="your-secret"
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$SECRET" | cut -d' ' -f2)

# Send webhook
curl -X POST http://localhost:8080/webhooks/gitea \
  -H "Content-Type: application/json" \
  -H "X-Gitea-Signature: $SIGNATURE" \
  -d "$PAYLOAD"
```

Test GitHub webhook:

```bash
# Generate signature
PAYLOAD='{"ref":"refs/heads/main","repository":{"full_name":"owner/repo"},"pusher":{"name":"test"}}'
SECRET="your-secret"
SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$SECRET" | cut -d' ' -f2)

# Send webhook
curl -X POST http://localhost:8080/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=$SIGNATURE" \
  -d "$PAYLOAD"
```

## Monitoring

Monitor webhook processing through:

1. **Server logs**: Check for webhook reception and job queuing
2. **Redis Streams**: Monitor `ci:jobs` stream length
3. **Health endpoint**: Check `/health/ready` for Redis connectivity
4. **Metrics**: Request count and error rates in application state

## Troubleshooting

### Webhook not triggering

- Verify webhook is active in repository settings
- Check server logs for incoming requests
- Ensure server is accessible from Git hosting platform

### Signature verification failures

- Verify secrets match on both sides
- Check for whitespace in secret configuration
- Ensure payload is not modified in transit

### Jobs not appearing in queue

- Verify Redis connection (`RAIBID_REDIS_URL`)
- Check Redis Streams with `redis-cli XLEN ci:jobs`
- Review server logs for queuing errors
