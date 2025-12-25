# Archives MCP Tools Reference

Archives exposes search capabilities via MCP (Model Context Protocol) for integration with AI agents.

Base URL: `http://localhost:8081`

## Endpoints

### GET /health
Health check endpoint.

### GET /tools
List available tools.

### POST /mcp
Invoke a tool.

**Request Format**
```json
{
  "tool": "tool_name",
  "params": { ... }
}
```

**Response Format**
```json
{
  "success": true,
  "data": { ... }
}
```

## Available Tools

### search_logs

Search logs with time range, severity filter, and text query.

**Parameters**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| query | string | - | Text to search for in log messages |
| hours | integer | 1 | Number of hours to search back |
| min_severity | string | - | Minimum severity: TRACE, DEBUG, INFO, WARN, ERROR, FATAL |
| service | string | - | Filter by service name |
| limit | integer | 50 | Maximum results |

**Example**
```json
{
  "tool": "search_logs",
  "params": {
    "query": "connection refused",
    "hours": 24,
    "min_severity": "ERROR",
    "limit": 20
  }
}
```

**Response**
```json
{
  "success": true,
  "data": {
    "count": 5,
    "logs": [
      {
        "timestamp": "2024-01-01T12:00:00Z",
        "severity": "ERROR",
        "service": "api",
        "message": "Connection refused to database"
      }
    ]
  }
}
```

### tail_logs

Get the most recent log entries.

**Parameters**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| count | integer | 20 | Number of recent logs to return |
| min_severity | string | - | Minimum severity level |
| service | string | - | Filter by service name |

**Example**
```json
{
  "tool": "tail_logs",
  "params": {
    "count": 10,
    "min_severity": "WARN"
  }
}
```

### get_error_summary

Get a summary of error patterns in the system.

**Parameters**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| hours | integer | 24 | Number of hours to analyze |
| limit | integer | 10 | Number of top patterns to return |

**Example**
```json
{
  "tool": "get_error_summary",
  "params": {
    "hours": 12,
    "limit": 5
  }
}
```

**Response**
```json
{
  "success": true,
  "data": {
    "total_errors": 150,
    "time_range_hours": 12,
    "top_patterns": [
      {
        "pattern": "Connection refused to database...",
        "count": 45,
        "example": "Connection refused to database at 192.168.1.5:5432"
      }
    ]
  }
}
```

### query_metrics

Query metrics with aggregation over time.

**Parameters**
| Name | Type | Default | Description |
|------|------|---------|-------------|
| metric_name | string | required | Name of the metric to query |
| hours | integer | 1 | Number of hours to query |
| aggregation | string | avg | Aggregation: avg, min, max, sum, count, p50, p90, p99 |
| interval_seconds | integer | 60 | Time bucket size in seconds |

**Example**
```json
{
  "tool": "query_metrics",
  "params": {
    "metric_name": "http_request_duration_seconds",
    "hours": 6,
    "aggregation": "p99",
    "interval_seconds": 300
  }
}
```

### get_system_health

Get overall system health summary.

**Parameters**: None

**Example**
```json
{
  "tool": "get_system_health",
  "params": {}
}
```

**Response**
```json
{
  "success": true,
  "data": {
    "status": "operational",
    "storage": {
      "log_count": 1234567,
      "log_bytes": 123456789,
      "log_bytes_human": "117.74 MB",
      "metric_count": 987654,
      "metric_bytes": 98765432,
      "metric_bytes_human": "94.19 MB"
    },
    "last_hour": {
      "total_logs": 5432,
      "error_count": 12
    }
  }
}
```

## Integration with LLM Gateway

To add Archives MCP server to LLM Gateway, add to `llm_gateway.toml`:

```toml
[[mcp.servers]]
name = "archives"
transport = "http"
url = "http://localhost:8081/mcp"
```
