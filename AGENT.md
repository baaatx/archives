# Archives - Agent Guide

## Project Overview

Archives is a self-hosted observability platform providing logging and metrics capabilities compatible with OpenTelemetry. It serves as a cost-effective alternative to cloud observability services.

## Tech Stack

- **Language**: Rust (2024 edition)
- **Storage**: ClickHouse
- **Ingestion**: OpenTelemetry Collector
- **Web Framework**: Axum
- **CLI**: Clap

## Architecture

```
OTEL SDK → OTEL Collector → ClickHouse ← archives-api/mcp/cli
```

Key crates:
- `archives-common` - Shared types, ClickHouse client, config
- `archives-api` - HTTP API server (port 8080)
- `archives-mcp` - MCP server for agent integration (port 8081)
- `archives-cli` - Command-line tool

## Session Startup Checklist

1. Check `signals/` for blocking issues
2. Review `backlog/` for prioritized work
3. Run `ops status` to verify infrastructure
4. Check `tasks/` for recurring task due dates

## Key Commands

```bash
# Source the ops framework
source scripts/ops.sh

# Infrastructure
ops start infra     # Start ClickHouse + OTEL Collector
ops stop infra      # Stop infrastructure
ops status          # Show all service status

# Development
ops dev api         # Run API in dev mode
ops dev mcp         # Run MCP server in dev mode
ops build           # Build all crates
ops test            # Run all tests

# Logs
ops logs clickhouse
ops logs otel-collector
```

## MCP Tools

The archives-mcp server exposes these tools:

| Tool | Description |
|------|-------------|
| `search_logs` | Search logs with time range, severity, text |
| `tail_logs` | Get most recent logs |
| `get_error_summary` | Get error patterns with counts |
| `query_metrics` | Query metrics with aggregation |
| `get_system_health` | Get overall system health |

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/health` | GET | Health check |
| `/v1/status` | GET | System status |
| `/v1/logs/search` | POST | Search logs |
| `/v1/metrics/query` | POST | Query metrics |
| `/v1/metrics/names` | GET | List metrics |

## ClickHouse Tables

Created automatically by OTEL Collector:
- `otel_logs` - Log entries
- `otel_metrics_gauge` - Gauge metrics
- `otel_metrics_sum` - Counter/sum metrics
- `otel_metrics_histogram` - Histogram metrics
- `otel_traces` - Trace spans (future)

## Configuration

Environment variables:
- `CLICKHOUSE_URL` - ClickHouse HTTP endpoint
- `CLICKHOUSE_DATABASE` - Database name
- `RUST_LOG` - Log level (e.g., `archives_api=debug`)

## Testing

```bash
# Unit tests
ops test unit

# Integration tests (requires infra running)
ops start infra
ops test integration

# Specific crate
cargo test -p archives-common
```

## Backlog Conventions

- Files in `backlog/` as JSON
- Format: `{id}-{timestamp}-{slug}.json`
- Status: TODO, IN_PROGRESS, DONE, BLOCKED
- Priority: P0 (critical) to P4 (backlog)

## Common Tasks

### Add new MCP tool

1. Define tool in `crates/archives-mcp/src/tools.rs` in `create_tool_registry()`
2. Implement handler function `execute_<tool_name>()`
3. Add match arm in `execute_tool()`
4. Test via MCP endpoint

### Add new API endpoint

1. Define handler in `crates/archives-api/src/main.rs`
2. Add route to router
3. Define request/response types
4. Add tests

### Query ClickHouse directly

```bash
# Connect to ClickHouse
docker exec -it archives-clickhouse clickhouse-client

# Example queries
SELECT count() FROM otel_logs;
SELECT * FROM otel_logs ORDER BY Timestamp DESC LIMIT 10;
SHOW TABLES;
```

## Error Handling

Use `archives_common::Error` for domain errors:
- `ClickHouseConnection` - Connection failures
- `ClickHouseQuery` - Query errors
- `Config` - Configuration issues
- `NotFound` - Resource not found
- `InvalidParameter` - Bad input

## Dependencies

External services:
- ClickHouse (required)
- OTEL Collector (for ingestion)

Cross-ecosystem:
- `llm_gateway` - Can aggregate archives MCP tools
- Other projects - Send telemetry via OTEL SDK
