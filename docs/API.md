# API Documentation

Complete API reference for raibid-ci server endpoints.

## Table of Contents

- [Overview](#overview)
- [Base URL](#base-url)
- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)
- [Error Handling](#error-handling)
- [API Versioning](#api-versioning)
- [Endpoints](#endpoints)
  - [Health Checks](#health-checks)
  - [Jobs](#jobs)
  - [Webhooks](#webhooks)
- [Server-Sent Events (SSE)](#server-sent-events-sse)
- [Request/Response Examples](#requestresponse-examples)
- [Client Libraries](#client-libraries)

## Overview

The raibid-ci API server provides RESTful HTTP endpoints for managing CI/CD jobs, monitoring agent status, and receiving webhooks from Git providers.

**Architecture**: Built with Axum (Rust web framework)
**Data Format**: JSON
**Real-time Updates**: Server-Sent Events (SSE)
**Job Queue**: Redis Streams

**Key Features**:
- Job status and management
- Real-time log streaming via SSE
- GitHub and Gitea webhook integration
- Health and readiness probes for Kubernetes
- Prometheus-compatible metrics endpoint

## Base URL

**Local Development**:
```
http://localhost:8080
```

**Production** (cluster internal):
```
http://raibid-server.raibid-system.svc.cluster.local:8080
```

**Production** (external, if exposed):
```
https://raibid-ci.example.com
```

## Authentication

**Current Status**: Authentication is not yet implemented in MVP.

**Future**: Authentication will use one of the following methods:
- API Keys (via `Authorization` header)
- JWT tokens
- mTLS for inter-service communication

**Webhook Authentication**: Webhooks use HMAC-SHA256 signature verification.
- GitHub: `X-Hub-Signature-256` header
- Gitea: `X-Gitea-Signature` header

## Rate Limiting

**Current Status**: Rate limiting is not yet implemented.

**Future**: Rate limits will be enforced per client IP or API key.

**Recommended Limits**:
- API requests: 100 requests/minute
- Webhook requests: 50 requests/minute
- SSE connections: 10 concurrent connections per client

## Error Handling

### Error Response Format

All errors follow a consistent JSON format:

```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Job not found: job-123",
    "details": {
      "resource": "job",
      "id": "job-123"
    }
  }
}
```

### HTTP Status Codes

| Status Code | Meaning | When Used |
|-------------|---------|-----------|
| 200 OK | Success | Successful GET requests |
| 201 Created | Resource created | Successful POST creating new resource |
| 202 Accepted | Request accepted | Webhook accepted, job queued |
| 204 No Content | Success, no body | Successful DELETE |
| 400 Bad Request | Invalid input | Malformed JSON, missing fields |
| 401 Unauthorized | Authentication failed | Invalid or missing credentials |
| 403 Forbidden | Insufficient permissions | Valid auth but not allowed |
| 404 Not Found | Resource not found | Job/agent doesn't exist |
| 409 Conflict | Resource conflict | Duplicate resource creation |
| 429 Too Many Requests | Rate limit exceeded | Too many requests |
| 500 Internal Server Error | Server error | Unexpected server error |
| 503 Service Unavailable | Service unavailable | Redis/dependency unavailable |

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `BAD_REQUEST` | 400 | Invalid request format or parameters |
| `UNAUTHORIZED` | 401 | Missing or invalid authentication |
| `FORBIDDEN` | 403 | Valid auth but insufficient permissions |
| `RESOURCE_NOT_FOUND` | 404 | Requested resource doesn't exist |
| `CONFLICT` | 409 | Resource already exists or conflict |
| `INTERNAL_ERROR` | 500 | Unexpected internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Dependency unavailable (Redis, etc.) |

## API Versioning

**Current Version**: v1 (implicit, no version in URL)

**Future**: API versioning will be added when breaking changes are introduced.

**Planned Format**:
```
/api/v1/jobs
/api/v2/jobs
```

**Compatibility**: The server will support multiple API versions simultaneously during transition periods.

## Endpoints

### Health Checks

#### GET /health

Basic health check endpoint.

**Description**: Returns server health status, uptime, and basic metrics.

**Request**:
```bash
curl http://localhost:8080/health
```

**Response**: `200 OK`
```json
{
  "status": "ok",
  "uptime_seconds": 3600,
  "requests_total": 1234,
  "active_connections": 5,
  "timestamp": "2025-11-03T12:00:00Z"
}
```

**Use Case**: Basic monitoring, load balancer health checks.

---

#### GET /health/ready

Readiness probe with detailed component checks.

**Description**: Returns detailed health status of all dependencies. Used by Kubernetes readiness probes.

**Request**:
```bash
curl http://localhost:8080/health/ready
```

**Response**: `200 OK` (ready) or `503 Service Unavailable` (not ready)
```json
{
  "status": "ready",
  "uptime_seconds": 3600,
  "requests_total": 1234,
  "active_connections": 5,
  "timestamp": "2025-11-03T12:00:00Z",
  "checks": {
    "database": {
      "healthy": true,
      "message": "Connected"
    },
    "redis": {
      "healthy": true,
      "message": "Connected to redis://redis:6379"
    },
    "kubernetes": {
      "healthy": true,
      "message": "API server reachable"
    }
  }
}
```

**Use Case**: Kubernetes readiness probe, detailed health monitoring.

---

#### GET /health/live

Liveness probe.

**Description**: Simple liveness check. Returns 200 if server is running.

**Request**:
```bash
curl http://localhost:8080/health/live
```

**Response**: `200 OK`
```json
{
  "status": "alive",
  "timestamp": "2025-11-03T12:00:00Z"
}
```

**Use Case**: Kubernetes liveness probe.

---

### Jobs

#### GET /jobs

List jobs with filtering and pagination.

**Description**: Retrieve a list of CI jobs with optional filtering by status, repository, or branch.

**Query Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter by job status (`pending`, `running`, `success`, `failed`) |
| `repo` | string | No | Filter by repository name (e.g., `owner/repo`) |
| `branch` | string | No | Filter by branch name |
| `limit` | integer | No | Number of results per page (default: 20, max: 100) |
| `offset` | integer | No | Offset for pagination (default: 0) |
| `cursor` | string | No | Cursor for cursor-based pagination |

**Request**:
```bash
# List all jobs
curl http://localhost:8080/jobs

# Filter by status
curl http://localhost:8080/jobs?status=failed

# Filter by repository and branch
curl "http://localhost:8080/jobs?repo=raibid-labs/raibid-ci&branch=main"

# Pagination
curl "http://localhost:8080/jobs?limit=10&offset=20"

# Cursor-based pagination
curl "http://localhost:8080/jobs?cursor=eyJpZCI6ImFiYzEyMyJ9"
```

**Response**: `200 OK`
```json
{
  "jobs": [
    {
      "id": "job-abc123",
      "repo": "raibid-labs/raibid-ci",
      "branch": "main",
      "commit": "7838242f9d8e1234567890abcdef",
      "status": "success",
      "started_at": "2025-11-03T12:00:00Z",
      "finished_at": "2025-11-03T12:05:30Z",
      "duration": 330,
      "agent_id": "agent-xyz789",
      "exit_code": 0
    },
    {
      "id": "job-def456",
      "repo": "raibid-labs/example",
      "branch": "develop",
      "commit": "9c2b627a1b2c3d4e5f6g7h8i9j0k",
      "status": "failed",
      "started_at": "2025-11-03T11:50:00Z",
      "finished_at": "2025-11-03T11:52:15Z",
      "duration": 135,
      "agent_id": "agent-xyz789",
      "exit_code": 1
    }
  ],
  "total": 42,
  "offset": 0,
  "limit": 20,
  "next_cursor": "eyJpZCI6ImRlZjQ1NiJ9"
}
```

**Job Status Values**:
- `pending`: Job queued, waiting for agent
- `running`: Job currently executing
- `success`: Job completed successfully (exit code 0)
- `failed`: Job failed (non-zero exit code)

---

#### GET /jobs/{id}

Get details of a specific job.

**Description**: Retrieve full details of a job by its ID.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Job ID |

**Request**:
```bash
curl http://localhost:8080/jobs/job-abc123
```

**Response**: `200 OK`
```json
{
  "id": "job-abc123",
  "repo": "raibid-labs/raibid-ci",
  "branch": "main",
  "commit": "7838242f9d8e1234567890abcdef",
  "status": "success",
  "started_at": "2025-11-03T12:00:00Z",
  "finished_at": "2025-11-03T12:05:30Z",
  "duration": 330,
  "agent_id": "agent-xyz789",
  "exit_code": 0
}
```

**Error Response**: `404 Not Found`
```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Job not found: job-abc123"
  }
}
```

---

#### GET /jobs/{id}/logs

Stream job logs in real-time via Server-Sent Events.

**Description**: Subscribe to real-time log output for a running or completed job.

**Path Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Job ID |

**Request**:
```bash
curl -N http://localhost:8080/jobs/job-abc123/logs
```

**Response**: `200 OK` (Server-Sent Events stream)
```
Content-Type: text/event-stream

data: [{"id":"1234567890-0","timestamp":"2025-11-03T12:00:01Z","level":"info","message":"Starting build..."}]

data: [{"id":"1234567891-0","timestamp":"2025-11-03T12:00:02Z","level":"info","message":"Cloning repository..."}]

data: [{"id":"1234567892-0","timestamp":"2025-11-03T12:00:05Z","level":"info","message":"Running cargo build..."}]

: keepalive

data: [{"id":"1234567893-0","timestamp":"2025-11-03T12:05:30Z","level":"info","message":"Build completed successfully"}]
```

**Log Entry Format**:
```json
{
  "id": "1234567890-0",           // Redis Stream message ID
  "timestamp": "2025-11-03T12:00:01Z",
  "level": "info",                 // info, warn, error
  "message": "Build started"
}
```

**SSE Events**:
- `data`: Log entries (JSON array)
- `: keepalive`: Keepalive comment (no data)

**Connection Behavior**:
- Connection stays open until job completes
- Sends keepalive comments every 500ms when no new logs
- Automatically closes when job finishes

**Error Response**: `404 Not Found`
```json
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Job not found: job-abc123"
  }
}
```

---

### Webhooks

#### POST /webhooks/gitea

Receive webhooks from Gitea.

**Description**: Handle push events from Gitea, validate signature, and queue CI jobs.

**Headers**:
| Header | Required | Description |
|--------|----------|-------------|
| `Content-Type` | Yes | Must be `application/json` |
| `X-Gitea-Event` | Yes | Event type (e.g., `push`) |
| `X-Gitea-Signature` | Conditional | HMAC-SHA256 signature (if secret configured) |

**Request**:
```bash
curl -X POST http://localhost:8080/webhooks/gitea \
  -H "Content-Type: application/json" \
  -H "X-Gitea-Event: push" \
  -H "X-Gitea-Signature: sha256=abc123..." \
  -d @gitea-webhook-payload.json
```

**Request Body** (Gitea push webhook payload):
```json
{
  "ref": "refs/heads/main",
  "before": "0000000000000000000000000000000000000000",
  "after": "7838242f9d8e1234567890abcdef",
  "repository": {
    "id": 1,
    "name": "raibid-ci",
    "full_name": "raibid-labs/raibid-ci",
    "owner": {
      "username": "raibid-labs"
    },
    "clone_url": "https://gitea.example.com/raibid-labs/raibid-ci.git"
  },
  "pusher": {
    "username": "johndoe",
    "email": "john@example.com"
  },
  "commits": [
    {
      "id": "7838242f9d8e1234567890abcdef",
      "message": "Add new feature",
      "author": {
        "name": "John Doe",
        "email": "john@example.com"
      }
    }
  ]
}
```

**Response**: `202 Accepted`
```json
{
  "job_id": "job-abc123",
  "message": "Job job-abc123 queued successfully"
}
```

**Error Response**: `401 Unauthorized` (invalid signature)
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid signature"
  }
}
```

**Error Response**: `400 Bad Request` (invalid payload)
```json
{
  "error": {
    "code": "BAD_REQUEST",
    "message": "Invalid webhook payload: missing field 'repository'"
  }
}
```

**Signature Verification**:
```python
# Gitea sends HMAC-SHA256 signature
import hmac
import hashlib

secret = "your-webhook-secret"
payload = request.body
signature = request.headers["X-Gitea-Signature"]

expected = hmac.new(secret.encode(), payload, hashlib.sha256).hexdigest()
valid = hmac.compare_digest(signature, expected)
```

---

#### POST /webhooks/github

Receive webhooks from GitHub.

**Description**: Handle push events from GitHub, validate signature, and queue CI jobs.

**Headers**:
| Header | Required | Description |
|--------|----------|-------------|
| `Content-Type` | Yes | Must be `application/json` |
| `X-GitHub-Event` | Yes | Event type (e.g., `push`) |
| `X-Hub-Signature-256` | Conditional | HMAC-SHA256 signature (if secret configured) |

**Request**:
```bash
curl -X POST http://localhost:8080/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: push" \
  -H "X-Hub-Signature-256: sha256=abc123..." \
  -d @github-webhook-payload.json
```

**Request Body** (GitHub push webhook payload):
```json
{
  "ref": "refs/heads/main",
  "before": "0000000000000000000000000000000000000000",
  "after": "7838242f9d8e1234567890abcdef",
  "repository": {
    "id": 123456,
    "name": "raibid-ci",
    "full_name": "raibid-labs/raibid-ci",
    "owner": {
      "name": "raibid-labs",
      "login": "raibid-labs"
    },
    "clone_url": "https://github.com/raibid-labs/raibid-ci.git"
  },
  "pusher": {
    "name": "johndoe",
    "email": "john@example.com"
  },
  "commits": [
    {
      "id": "7838242f9d8e1234567890abcdef",
      "message": "Add new feature",
      "author": {
        "name": "John Doe",
        "email": "john@example.com"
      }
    }
  ]
}
```

**Response**: `202 Accepted`
```json
{
  "job_id": "job-def456",
  "message": "Job job-def456 queued successfully"
}
```

**Error Response**: `401 Unauthorized` (invalid signature)
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid signature"
  }
}
```

**Signature Verification**:
```python
# GitHub sends HMAC-SHA256 with "sha256=" prefix
import hmac
import hashlib

secret = "your-webhook-secret"
payload = request.body
signature = request.headers["X-Hub-Signature-256"]

expected = "sha256=" + hmac.new(secret.encode(), payload, hashlib.sha256).hexdigest()
valid = hmac.compare_digest(signature, expected)
```

---

## Server-Sent Events (SSE)

### Overview

Server-Sent Events provide real-time, unidirectional streaming from server to client over HTTP.

**Use Cases**:
- Real-time job log streaming
- Live job status updates
- Agent status monitoring

**Protocol**: `text/event-stream`

### SSE Format

```
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive

event: message
data: {"key": "value"}

event: update
data: {"status": "running"}

: keepalive comment
```

### Client Examples

#### JavaScript (Browser)

```javascript
const eventSource = new EventSource('http://localhost:8080/jobs/job-abc123/logs');

eventSource.onmessage = (event) => {
  const logs = JSON.parse(event.data);
  logs.forEach(log => {
    console.log(`[${log.level}] ${log.message}`);
  });
};

eventSource.onerror = (error) => {
  console.error('SSE error:', error);
  eventSource.close();
};

// Close connection when done
eventSource.close();
```

#### curl

```bash
curl -N http://localhost:8080/jobs/job-abc123/logs
```

#### Python

```python
import requests
import json

url = 'http://localhost:8080/jobs/job-abc123/logs'
response = requests.get(url, stream=True)

for line in response.iter_lines():
    if line:
        line = line.decode('utf-8')
        if line.startswith('data:'):
            data = line[5:].strip()
            logs = json.loads(data)
            for log in logs:
                print(f"[{log['level']}] {log['message']}")
```

---

## Request/Response Examples

### Complete Job Lifecycle

#### 1. Webhook triggers job

```bash
# Gitea sends webhook
curl -X POST http://localhost:8080/webhooks/gitea \
  -H "Content-Type: application/json" \
  -H "X-Gitea-Event: push" \
  -d '{
    "ref": "refs/heads/main",
    "after": "abc123",
    "repository": {"full_name": "owner/repo"},
    "pusher": {"username": "johndoe"}
  }'

# Response: 202 Accepted
{
  "job_id": "job-abc123",
  "message": "Job job-abc123 queued successfully"
}
```

#### 2. Check job status

```bash
# Poll job status
curl http://localhost:8080/jobs/job-abc123

# Response: 200 OK
{
  "id": "job-abc123",
  "status": "running",
  "started_at": "2025-11-03T12:00:00Z",
  ...
}
```

#### 3. Stream logs in real-time

```bash
# Connect to SSE endpoint
curl -N http://localhost:8080/jobs/job-abc123/logs

# Response: Server-Sent Events
data: [{"id":"123-0","message":"Starting build..."}]
data: [{"id":"124-0","message":"Running tests..."}]
...
```

#### 4. Check completion

```bash
# Get final status
curl http://localhost:8080/jobs/job-abc123

# Response: 200 OK
{
  "id": "job-abc123",
  "status": "success",
  "started_at": "2025-11-03T12:00:00Z",
  "finished_at": "2025-11-03T12:05:30Z",
  "duration": 330,
  "exit_code": 0
}
```

### Error Handling Examples

#### Invalid Job ID

```bash
curl http://localhost:8080/jobs/invalid-job-id

# Response: 404 Not Found
{
  "error": {
    "code": "RESOURCE_NOT_FOUND",
    "message": "Job not found: invalid-job-id"
  }
}
```

#### Invalid Webhook Signature

```bash
curl -X POST http://localhost:8080/webhooks/github \
  -H "X-Hub-Signature-256: sha256=wrong-signature" \
  -d '{...}'

# Response: 401 Unauthorized
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid signature"
  }
}
```

#### Service Unavailable (Redis Down)

```bash
curl http://localhost:8080/jobs

# Response: 503 Service Unavailable
{
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Failed to connect to Redis"
  }
}
```

---

## Client Libraries

### Official Clients

**Status**: No official client libraries yet.

**Planned**:
- Rust client (using `reqwest`)
- Python client (using `requests`)
- JavaScript/TypeScript client (using `fetch`)

### Community Clients

None available yet. Contributions welcome!

### DIY Client Examples

#### Bash/curl

```bash
#!/bin/bash
# raibid-api.sh - Simple Bash client

API_URL="http://localhost:8080"

# List jobs
list_jobs() {
  curl -s "$API_URL/jobs" | jq .
}

# Get job status
get_job() {
  local job_id=$1
  curl -s "$API_URL/jobs/$job_id" | jq .
}

# Stream logs
stream_logs() {
  local job_id=$1
  curl -N "$API_URL/jobs/$job_id/logs"
}

# Usage
case "$1" in
  list) list_jobs ;;
  get) get_job "$2" ;;
  logs) stream_logs "$2" ;;
  *) echo "Usage: $0 {list|get|logs} [job_id]" ;;
esac
```

#### Python

```python
# raibid_client.py - Simple Python client

import requests
import json

class RaibidClient:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
        self.session = requests.Session()

    def list_jobs(self, status=None, repo=None, limit=20):
        params = {"limit": limit}
        if status:
            params["status"] = status
        if repo:
            params["repo"] = repo

        response = self.session.get(f"{self.base_url}/jobs", params=params)
        response.raise_for_status()
        return response.json()

    def get_job(self, job_id):
        response = self.session.get(f"{self.base_url}/jobs/{job_id}")
        response.raise_for_status()
        return response.json()

    def stream_logs(self, job_id, callback):
        response = self.session.get(
            f"{self.base_url}/jobs/{job_id}/logs",
            stream=True
        )
        response.raise_for_status()

        for line in response.iter_lines():
            if line:
                line = line.decode('utf-8')
                if line.startswith('data:'):
                    data = line[5:].strip()
                    logs = json.loads(data)
                    for log in logs:
                        callback(log)

# Usage
client = RaibidClient()

# List failed jobs
jobs = client.list_jobs(status="failed")
for job in jobs["jobs"]:
    print(f"{job['id']}: {job['repo']} - {job['status']}")

# Get job details
job = client.get_job("job-abc123")
print(f"Duration: {job['duration']}s")

# Stream logs
def print_log(log):
    print(f"[{log['level']}] {log['message']}")

client.stream_logs("job-abc123", print_log)
```

#### JavaScript/TypeScript

```typescript
// raibid-client.ts - Simple TypeScript client

class RaibidClient {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  async listJobs(filters?: {
    status?: string;
    repo?: string;
    limit?: number;
  }): Promise<any> {
    const params = new URLSearchParams();
    if (filters?.status) params.set('status', filters.status);
    if (filters?.repo) params.set('repo', filters.repo);
    if (filters?.limit) params.set('limit', filters.limit.toString());

    const response = await fetch(`${this.baseUrl}/jobs?${params}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  }

  async getJob(jobId: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/jobs/${jobId}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  }

  streamLogs(jobId: string, callback: (log: any) => void): EventSource {
    const eventSource = new EventSource(
      `${this.baseUrl}/jobs/${jobId}/logs`
    );

    eventSource.onmessage = (event) => {
      const logs = JSON.parse(event.data);
      logs.forEach((log: any) => callback(log));
    };

    eventSource.onerror = (error) => {
      console.error('SSE error:', error);
      eventSource.close();
    };

    return eventSource;
  }
}

// Usage
const client = new RaibidClient();

// List jobs
const jobs = await client.listJobs({ status: 'failed' });
console.log(jobs);

// Stream logs
const eventSource = client.streamLogs('job-abc123', (log) => {
  console.log(`[${log.level}] ${log.message}`);
});

// Close when done
eventSource.close();
```

---

## Additional Resources

### OpenAPI/Swagger Spec

**Status**: Not yet generated.

**Future**: An OpenAPI 3.0 specification will be generated using `utoipa` (Rust library).

**Planned Location**: `docs/api/openapi.yaml`

### Interactive API Documentation

**Status**: Not yet implemented.

**Future**: Swagger UI or ReDoc will be deployed for interactive API exploration.

**Planned Access**: `http://localhost:8080/docs`

### Postman Collection

**Status**: Not yet created.

**Future**: A Postman collection will be provided for easy API testing.

---

## Support

For questions or issues with the API:

- **GitHub Issues**: https://github.com/raibid-labs/raibid-ci/issues
- **Documentation**: [docs/](../docs/)
- **Source Code**: [crates/server/src/routes/](../crates/server/src/routes/)

---

**API Documentation Version**: 0.1.0
**Last Updated**: 2025-11-03
**Server Implementation**: [raibid-server](../crates/server/)
