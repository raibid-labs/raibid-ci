# API Documentation

This directory contains API specifications and contracts for all raibid-ci APIs.

## Overview

raibid-ci provides three main APIs:

1. **REST API** - HTTP endpoints for job and agent management
2. **WebSocket API** - Real-time status updates and log streaming
3. **Redis Streams Protocol** - Job queue message format

## API Documentation

### [REST API Reference](./rest-api.md)
Complete HTTP API endpoint reference.

**Endpoints:**
- Job Management (`/api/v1/jobs/*`)
- Agent Management (`/api/v1/agents/*`)
- Repository Mirroring (`/api/v1/mirrors/*`)
- Configuration (`/api/v1/config/*`)
- Status (`/api/v1/status/*`)

### [WebSocket API](./websocket.md)
Real-time event streaming via WebSocket.

**Events:**
- Job status updates
- Agent status changes
- Build log streaming
- Queue metrics
- System events

### [Redis Streams Protocol](./redis-streams.md)
Job queue message format and consumer protocol.

**Topics:**
- Job message format
- Status update format
- Consumer group protocol
- Acknowledgment handling

### [Pipeline Format](./pipeline-format.md)
`.raibid.yaml` pipeline configuration format.

**Topics:**
- Pipeline schema
- Build steps
- Cache configuration
- Environment variables
- Secrets

## API Versions

### Current: v1 (Stable)
- All endpoints under `/api/v1/`
- Semantic versioning
- Backward compatibility guaranteed within major version

### Future: v2 (Planned)
- Enhanced filtering and pagination
- GraphQL endpoint
- Batch operations

## Authentication

All API endpoints require authentication via JWT token.

### Getting a Token
```bash
# Login to get token
curl -X POST http://api.example.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"secret"}'

# Response
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2025-11-02T15:00:00Z"
}
```

### Using the Token
```bash
# Include in Authorization header
curl http://api.example.com/api/v1/jobs \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## Error Handling

All API errors follow a consistent format:

```json
{
  "error": {
    "code": "job_not_found",
    "message": "Job with ID 'job-123' not found",
    "details": {
      "job_id": "job-123"
    }
  }
}
```

### HTTP Status Codes
- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid request
- `401 Unauthorized` - Missing or invalid token
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

### Error Codes
- `invalid_request` - Malformed request
- `authentication_failed` - Invalid credentials
- `token_expired` - JWT token expired
- `insufficient_permissions` - Missing required permissions
- `resource_not_found` - Resource doesn't exist
- `resource_conflict` - Resource already exists
- `rate_limit_exceeded` - Too many requests
- `internal_error` - Unexpected server error

## Rate Limiting

API requests are rate-limited to prevent abuse:

- **Anonymous**: 10 requests/minute
- **Authenticated**: 100 requests/minute
- **Privileged**: 1000 requests/minute

Rate limit headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1730476800
```

## Pagination

List endpoints support pagination via query parameters:

```bash
# Get second page of 20 items
curl "http://api.example.com/api/v1/jobs?page=2&per_page=20"
```

Response includes pagination metadata:
```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "per_page": 20,
    "total_pages": 5,
    "total_items": 95
  }
}
```

## Filtering

List endpoints support filtering:

```bash
# Filter jobs by status
curl "http://api.example.com/api/v1/jobs?status=running"

# Filter by multiple fields
curl "http://api.example.com/api/v1/jobs?status=running&repo=raibid/core"

# Filter with operators
curl "http://api.example.com/api/v1/jobs?created_after=2025-11-01&duration_lt=300"
```

### Filter Operators
- `eq` - Equals (default)
- `ne` - Not equals
- `gt` - Greater than
- `gte` - Greater than or equal
- `lt` - Less than
- `lte` - Less than or equal
- `in` - In list
- `contains` - String contains
- `starts_with` - String starts with

## Sorting

List endpoints support sorting:

```bash
# Sort by created_at descending (newest first)
curl "http://api.example.com/api/v1/jobs?sort=-created_at"

# Sort by multiple fields
curl "http://api.example.com/api/v1/jobs?sort=status,created_at"
```

Prefix with `-` for descending order.

## Field Selection

Select specific fields to reduce response size:

```bash
# Get only ID and status
curl "http://api.example.com/api/v1/jobs?fields=id,status"
```

## API Client Libraries

### Rust
```rust
use raibid_client::{Client, JobStatus};

let client = Client::new("http://api.example.com")
    .with_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");

let jobs = client.jobs()
    .list()
    .status(JobStatus::Running)
    .send()
    .await?;
```

### Python
```python
from raibid import Client

client = Client("http://api.example.com", token="...")
jobs = client.jobs.list(status="running")
```

### cURL Examples
See individual API documentation for comprehensive cURL examples.

## OpenAPI Specification

Download the complete OpenAPI 3.0 specification:
- [openapi.yaml](./openapi.yaml) - YAML format
- [openapi.json](./openapi.json) - JSON format

Use with tools like:
- Swagger UI
- Postman
- Insomnia
- OpenAPI Generator

## WebSocket Example

```javascript
const ws = new WebSocket('ws://api.example.com/api/v1/ws');

ws.onopen = () => {
  // Subscribe to job updates
  ws.send(JSON.stringify({
    type: 'subscribe',
    channel: 'jobs',
    filters: { status: 'running' }
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Job update:', data);
};
```

## Testing

### API Test Collection
Postman collection available: [raibid-ci.postman_collection.json](./raibid-ci.postman_collection.json)

### Integration Tests
```bash
# Run API integration tests
cargo test --package server --test api_integration_test
```

## Versioning Strategy

### Backward Compatibility
Within a major version (e.g., v1.x), we guarantee:
- No breaking changes to existing endpoints
- New fields are optional or have defaults
- Deprecated features have 6-month sunset period

### Breaking Changes
Breaking changes require a new major version:
- Removing endpoints
- Changing required fields
- Modifying response formats
- Changing authentication methods

### Deprecation Process
1. Mark as deprecated in documentation
2. Add `X-Deprecated` response header
3. Announce sunset date (6 months)
4. Remove in next major version

## Related Documentation

- [Server Component](../components/server/README.md)
- [Architecture](../architecture/system-overview.md)
- [User Guide](../USER_GUIDE.md)

---

*Last Updated: 2025-11-01*
*API Version: v1 (Planning Phase)*
