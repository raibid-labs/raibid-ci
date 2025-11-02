# [HTTP Method] /api/v1/[endpoint]

Brief description of what this endpoint does.

## Overview

**Endpoint:** `[METHOD] /api/v1/[endpoint]`

**Authentication:** Required | Optional | None

**Rate Limit:** X requests per minute

**Introduced:** v1.0.0

## Request

### HTTP Method
```
[GET|POST|PUT|DELETE|PATCH] /api/v1/[endpoint]
```

### Path Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `param1` | string | Yes | Description of param1 |
| `param2` | integer | No | Description of param2 |

### Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `status` | string | No | all | Filter by status |
| `page` | integer | No | 1 | Page number |
| `per_page` | integer | No | 20 | Items per page |

### Headers

| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes | Bearer token |
| `Content-Type` | Yes | application/json |

### Request Body

```json
{
  "field1": "value1",
  "field2": 42,
  "nested": {
    "field3": true
  }
}
```

**Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field1` | string | Yes | Description |
| `field2` | integer | No | Description |
| `nested.field3` | boolean | No | Description |

### Request Example

```bash
curl -X [METHOD] http://api.example.com/api/v1/[endpoint] \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "field1": "value1",
    "field2": 42
  }'
```

## Response

### Success Response

**Status Code:** `200 OK` | `201 Created` | `204 No Content`

**Response Body:**

```json
{
  "data": {
    "id": "123",
    "field1": "value1",
    "created_at": "2025-11-01T12:00:00Z"
  },
  "meta": {
    "request_id": "req-abc123"
  }
}
```

**Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `data.id` | string | Resource ID |
| `data.field1` | string | Description |
| `data.created_at` | string (ISO 8601) | Creation timestamp |
| `meta.request_id` | string | Request tracking ID |

### Error Responses

#### 400 Bad Request

Malformed request or validation error.

```json
{
  "error": {
    "code": "invalid_request",
    "message": "Field 'field1' is required",
    "details": {
      "field": "field1",
      "constraint": "required"
    }
  }
}
```

#### 401 Unauthorized

Missing or invalid authentication token.

```json
{
  "error": {
    "code": "authentication_failed",
    "message": "Invalid or expired token"
  }
}
```

#### 404 Not Found

Resource not found.

```json
{
  "error": {
    "code": "resource_not_found",
    "message": "Resource with ID '123' not found",
    "details": {
      "resource_id": "123"
    }
  }
}
```

#### 500 Internal Server Error

Unexpected server error.

```json
{
  "error": {
    "code": "internal_error",
    "message": "An unexpected error occurred"
  }
}
```

## Examples

### Example 1: Basic Request

```bash
curl http://api.example.com/api/v1/[endpoint] \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**
```json
{
  "data": {
    "id": "123",
    "status": "success"
  }
}
```

### Example 2: Advanced Request

```bash
curl -X POST http://api.example.com/api/v1/[endpoint] \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "field1": "value",
    "field2": 42
  }'
```

**Response:**
```json
{
  "data": {
    "id": "456",
    "field1": "value",
    "field2": 42,
    "created_at": "2025-11-01T12:00:00Z"
  }
}
```

## Client Library Examples

### Rust

```rust
use raibid_client::Client;

let client = Client::new("http://api.example.com")
    .with_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");

let response = client
    .endpoint()
    .field1("value")
    .field2(42)
    .send()
    .await?;

println!("ID: {}", response.id);
```

### Python

```python
from raibid import Client

client = Client("http://api.example.com", token="...")
response = client.endpoint.create(
    field1="value",
    field2=42
)
print(f"ID: {response.id}")
```

## Rate Limiting

This endpoint is subject to rate limiting:
- **Authenticated users:** 100 requests/minute
- **Anonymous users:** 10 requests/minute

Rate limit headers included in response:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1730476800
```

## Permissions

Required permissions:
- `endpoint:read` - For GET requests
- `endpoint:write` - For POST/PUT requests
- `endpoint:delete` - For DELETE requests

## Pagination

This endpoint supports pagination (for list endpoints):

```bash
curl "http://api.example.com/api/v1/[endpoint]?page=2&per_page=50"
```

Response includes pagination metadata:
```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "per_page": 50,
    "total_pages": 10,
    "total_items": 500
  }
}
```

## Filtering

Supported filters:

| Filter | Type | Description |
|--------|------|-------------|
| `status` | string | Filter by status |
| `created_after` | string (ISO 8601) | Items created after date |
| `created_before` | string (ISO 8601) | Items created before date |

Example:
```bash
curl "http://api.example.com/api/v1/[endpoint]?status=active&created_after=2025-11-01"
```

## Sorting

Supported sort fields:
- `created_at` - Creation timestamp
- `updated_at` - Last update timestamp
- `name` - Name (alphabetical)

Example:
```bash
# Sort by created_at descending (newest first)
curl "http://api.example.com/api/v1/[endpoint]?sort=-created_at"
```

## Changelog

### v1.1.0 (2025-XX-XX)
- Added field `new_field`
- Deprecated field `old_field`

### v1.0.0 (2025-XX-XX)
- Initial release

## See Also

- [API Overview](../api/)
- [Technology Research](../technology-research.md)

---

*Last Updated: YYYY-MM-DD*
*API Version: v1*
