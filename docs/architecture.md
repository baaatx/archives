# Archives Architecture

## Overview

Archives is a self-hosted observability platform for logs and metrics, compatible with OpenTelemetry. It provides a cost-effective alternative to cloud observability services.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐     │
│   │  Applications    │    │  Ecosystem       │    │  CLI Tool        │     │
│   │  (OTEL SDK)      │    │  Agents (Claude) │    │  (archives)      │     │
│   └────────┬─────────┘    └────────┬─────────┘    └────────┬─────────┘     │
│            │                       │                        │               │
│            │ OTLP                  │ MCP                    │ HTTP          │
│            │ (gRPC/HTTP)           │ Protocol               │ API           │
│            │                       │                        │               │
└────────────┼───────────────────────┼────────────────────────┼───────────────┘
             │                       │                        │
             ▼                       ▼                        ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             INGESTION LAYER                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────────────┐    ┌────────────────────────────────────┐   │
│   │   OTEL Collector         │    │  Archives API + MCP                │   │
│   │   ━━━━━━━━━━━━━━━━━━━━   │    │  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │   │
│   │   • OTLP Receiver        │    │  • HTTP API (port 8080)            │   │
│   │   • Batch Processor      │    │  • MCP Server (port 8081)          │   │
│   │   • ClickHouse Exporter  │    │  • Query Processor                 │   │
│   └────────────┬─────────────┘    └────────────────┬───────────────────┘   │
│                │                                   │                        │
└────────────────┼───────────────────────────────────┼────────────────────────┘
                 │                                   │
                 ▼                                   ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             STORAGE LAYER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                     ClickHouse Database                               │  │
│   │   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━   │  │
│   │                                                                       │  │
│   │   ┌─────────────┐   ┌─────────────────┐   ┌─────────────────────┐   │  │
│   │   │  otel_logs  │   │ otel_metrics_*  │   │  otel_traces        │   │  │
│   │   │  ───────────│   │ ─────────────── │   │  ─────────────────  │   │  │
│   │   │  TTL: 30d   │   │ TTL: 90d        │   │  TTL: 30d           │   │  │
│   │   └─────────────┘   └─────────────────┘   └─────────────────────┘   │  │
│   │                                                                       │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Component Details

### OTEL Collector
- **Purpose**: Ingest telemetry data using OpenTelemetry standards
- **Receivers**: OTLP (gRPC on 4317, HTTP on 4318)
- **Processors**: Batch (for efficient inserts), Memory Limiter
- **Exporters**: ClickHouse with async inserts

### Archives API
- **Purpose**: HTTP API for querying logs and metrics
- **Framework**: Rust + Axum
- **Features**: Search, aggregation, health checks

### Archives MCP
- **Purpose**: Expose search capabilities to AI agents via MCP
- **Tools**: search_logs, tail_logs, query_metrics, get_error_summary, get_system_health

### ClickHouse
- **Purpose**: High-performance columnar storage
- **Benefits**: 90% compression, fast analytical queries
- **Tables**: Created automatically by OTEL Collector

## Data Flow

### Ingestion Path
1. Application sends telemetry via OTEL SDK
2. OTEL Collector receives OTLP data
3. Batch processor groups data for efficiency
4. ClickHouse exporter writes to database

### Query Path
1. User/agent sends query to API/MCP
2. Archives builds ClickHouse query
3. ClickHouse executes and returns results
4. Archives formats response

## Deployment Models

### Local Development
```bash
source scripts/ops.sh
ops start infra     # Start ClickHouse + OTEL Collector
ops dev api         # Run API in dev mode
```

### Production (Docker)
```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Production (Kubernetes)
- OTEL Collector as DaemonSet (one per node)
- Archives API/MCP as Deployment (replicated)
- ClickHouse as StatefulSet (single or cluster)

## Scaling Considerations

### Single Node (up to 50 customers)
- 8 vCPU, 32GB RAM, 500GB SSD
- Handles ~5TB logs/day
- Sufficient for most use cases

### Cluster Mode (50+ customers)
- ClickHouse Cluster with sharding
- Multiple OTEL Collectors with load balancing
- Archives API behind load balancer
