# Archives

Self-hosted logging and metrics platform compatible with OpenTelemetry. Archives provides a cost-effective alternative to AWS CloudWatch and Datadog for basic log viewing, searching, and metrics visualization.

## Features

- **OpenTelemetry Native**: Ingest logs and metrics via OTLP (gRPC and HTTP)
- **ClickHouse Backend**: Fast columnar storage with 90% compression
- **MCP Integration**: Expose log/metrics search to ecosystem AI agents
- **REST API**: HTTP API for programmatic access
- **CLI Tool**: Command-line interface for log searching and metrics queries

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                         ARCHIVES                               │
├────────────────────────────────────────────────────────────────┤
│   Applications (OTEL SDK)          Ecosystem Agents            │
│         │                                │                     │
│         │ OTLP (gRPC/HTTP)               │ MCP Protocol        │
│         ▼                                ▼                     │
│   ┌──────────────────┐         ┌──────────────────┐           │
│   │  OTEL Collector  │         │  archives-mcp    │           │
│   └────────┬─────────┘         └────────┬─────────┘           │
│            │                            │                      │
│            ▼                            ▼                      │
│   ┌─────────────────────────────────────────────────┐         │
│   │              ClickHouse Database                │         │
│   │   otel_logs  │  otel_metrics  │  otel_traces    │         │
│   └─────────────────────────────────────────────────┘         │
└────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust 1.75+ (for development)

### Start Infrastructure

```bash
# Source the ops framework
source scripts/ops.sh

# Start ClickHouse and OTEL Collector
ops start infra

# Check status
ops status
```

### Send Test Logs

```bash
# Using curl to send OTLP/HTTP logs
curl -X POST http://localhost:4318/v1/logs \
  -H "Content-Type: application/json" \
  -d '{
    "resourceLogs": [{
      "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "test-service"}}]},
      "scopeLogs": [{
        "logRecords": [{
          "timeUnixNano": "'$(date +%s)000000000'",
          "severityNumber": 9,
          "severityText": "INFO",
          "body": {"stringValue": "Hello from Archives!"}
        }]
      }]
    }]
  }'
```

### Run API Server

```bash
# Development mode
ops dev api

# Or build and run
ops build api
cargo run -p archives-api
```

### Search Logs

```bash
# Using CLI
cargo run -p archives-cli -- logs search "error" --hours 24

# Using API
curl -X POST http://localhost:8080/v1/logs/search \
  -H "Content-Type: application/json" \
  -d '{
    "start": "2024-01-01T00:00:00Z",
    "end": "2024-12-31T23:59:59Z",
    "query": "error",
    "limit": 50
  }'
```

## MCP Integration

Archives exposes search capabilities via MCP for ecosystem agents:

```json
POST http://localhost:8081/mcp
{
  "tool": "search_logs",
  "params": {
    "query": "error",
    "hours": 24,
    "min_severity": "WARN",
    "limit": 50
  }
}
```

Available MCP tools:
- `search_logs` - Search logs with filters
- `tail_logs` - Get recent logs
- `get_error_summary` - Get error patterns
- `query_metrics` - Query metrics with aggregation
- `get_system_health` - Get overall health summary

## Configuration

Copy `config.example.toml` to `config.toml` and adjust:

```toml
[clickhouse]
url = "http://localhost:8123"
database = "default"

[api]
port = 8080

[mcp]
port = 8081

[retention]
log_retention_days = 30
metrics_retention_days = 90
```

Environment variables (override config):
- `CLICKHOUSE_URL` - ClickHouse HTTP URL
- `CLICKHOUSE_DATABASE` - Database name
- `ARCHIVES__API__PORT` - API server port
- `ARCHIVES__MCP__PORT` - MCP server port

## Development

```bash
# Source ops framework
source scripts/ops.sh

# Build all crates
ops build

# Run tests
ops test

# Start dev environment
ops start infra
ops dev api
```

## Project Structure

```
archives/
├── crates/
│   ├── archives-common/   # Shared library
│   ├── archives-api/      # HTTP API server
│   ├── archives-mcp/      # MCP server
│   └── archives-cli/      # CLI tool
├── config/
│   └── otel-collector.yaml
├── docker-compose.yml
├── backlog/               # Work items
├── tasks/                 # Recurring tasks
├── signals/               # Blockers
└── scripts/
    └── ops.sh             # Operations framework
```

## Ports

| Service | Port | Protocol |
|---------|------|----------|
| ClickHouse HTTP | 8123 | HTTP |
| ClickHouse Native | 9000 | TCP |
| OTEL Collector gRPC | 4317 | gRPC |
| OTEL Collector HTTP | 4318 | HTTP |
| Archives API | 8080 | HTTP |
| Archives MCP | 8081 | HTTP |

## License

MIT
