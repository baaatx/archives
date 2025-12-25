# Archives API Reference

Base URL: `http://localhost:8080`

## Health & Status

### GET /health

Check service health and ClickHouse connectivity.

**Response**
```json
{
  "status": "healthy",
  "clickhouse": true
}
```

### GET /v1/status

Get system status including storage statistics.

**Response**
```json
{
  "status": "ok",
  "version": "0.1.0",
  "log_count": 1234567,
  "log_bytes": 123456789,
  "metric_count": 987654,
  "metric_bytes": 98765432
}
```

## Logs

### POST /v1/logs/search

Search logs with filters.

**Request**
```json
{
  "start": "2024-01-01T00:00:00Z",
  "end": "2024-01-02T00:00:00Z",
  "query": "error",
  "min_severity": "WARN",
  "service": "my-service",
  "offset": 0,
  "limit": 100
}
```

**Parameters**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| start | ISO8601 | Yes | Start of time range |
| end | ISO8601 | Yes | End of time range |
| query | string | No | Text search in log body |
| min_severity | string | No | Minimum severity: TRACE, DEBUG, INFO, WARN, ERROR, FATAL |
| service | string | No | Filter by service name |
| offset | integer | No | Pagination offset (default: 0) |
| limit | integer | No | Max results (default: 100) |

**Response**
```json
{
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2024-01-01T12:00:00Z",
      "observed_timestamp": "2024-01-01T12:00:01Z",
      "trace_id": "abc123",
      "span_id": "def456",
      "severity": "ERROR",
      "severity_text": "ERROR",
      "body": "Connection refused to database",
      "resource_attributes": {"service.name": "api"},
      "log_attributes": {},
      "service_name": "api"
    }
  ]
}
```

## Metrics

### GET /v1/metrics/names

List available metric names.

**Response**
```json
{
  "names": [
    "http_request_duration_seconds",
    "process_cpu_seconds_total",
    "go_goroutines"
  ]
}
```

### POST /v1/metrics/query

Query metrics with aggregation.

**Request**
```json
{
  "metric_name": "http_request_duration_seconds",
  "start": "2024-01-01T00:00:00Z",
  "end": "2024-01-01T01:00:00Z",
  "aggregation": "avg",
  "interval_seconds": 60
}
```

**Parameters**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| metric_name | string | Yes | Name of metric to query |
| start | ISO8601 | Yes | Start of time range |
| end | ISO8601 | Yes | End of time range |
| aggregation | string | No | avg, min, max, sum, count, p50, p90, p99 (default: avg) |
| interval_seconds | integer | No | Time bucket size (default: 60) |

**Response**
```json
{
  "data": [
    {"timestamp": "2024-01-01T00:00:00Z", "value": 0.125},
    {"timestamp": "2024-01-01T00:01:00Z", "value": 0.130}
  ]
}
```

## Error Responses

All endpoints return errors in this format:

```json
{
  "error": "Error message describing the issue"
}
```

**HTTP Status Codes**
| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request - Invalid parameters |
| 500 | Internal Server Error |
| 503 | Service Unavailable - ClickHouse not connected |
