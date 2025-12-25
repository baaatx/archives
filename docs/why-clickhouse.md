# Why ClickHouse?

This document explains the architectural decision to use ClickHouse as the storage backend for Archives, rather than building a custom solution.

## The Decision

Archives uses ClickHouse for log and metrics storage. This was a deliberate build-vs-buy decision.

## Why Not Build Our Own?

### Effort Comparison

| Component | Build Ourselves | ClickHouse Provides |
|-----------|-----------------|---------------------|
| Storage engine | 6+ months | Mature, battle-tested |
| Compression | 2+ months | LZ4, ZSTD, Delta, Gorilla |
| Query engine | 6+ months | Full SQL, optimized |
| Indexing | 3+ months | Skip indexes, bloom filters |
| Replication | 3+ months | Built-in HA |
| TTL/retention | 1 month | Single config line |
| **Total** | **20+ months** | **0** |

### Scale Reality

- ClickHouse handles 5TB+ logs/day on a single node
- Sub-second queries over billions of rows
- 90% compression on log data
- We need to serve 50 customers now, not after building a database

### Philosophy

> "Build what differentiates you, buy/use what doesn't"

ClickHouse is commodity infrastructure. MCP integration for AI agents is our differentiation.

## What We Built Instead

Our thin Rust layer (~2000 lines) adds unique value:

- **MCP integration** - AI agents can query logs (unique to us)
- **Unified API** - Consistent interface over OTEL schema
- **Ecosystem integration** - Works with our other projects
- **Self-hosted simplicity** - Docker Compose one-liner

Building equivalent storage would require 200,000+ lines.

## ClickHouse Technology Stack

### Core Technologies

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Language** | C++ | Performance-critical, low-level control |
| **Storage** | Columnar format | Store each column separately on disk |
| **Compression** | LZ4, ZSTD, Delta, Gorilla | 90%+ compression ratios |
| **Query** | Vectorized execution + JIT | SIMD operations, runtime code generation |
| **Protocol** | Custom binary + HTTP | High-throughput ingestion |

### Columnar Storage

Unlike row-based databases, ClickHouse stores data by column:

```
Row-based storage:
┌─────────────────────────────────────────────────────────┐
│ [id, timestamp, message, severity]                      │
│ [id, timestamp, message, severity]                      │
│ [id, timestamp, message, severity]                      │
└─────────────────────────────────────────────────────────┘

Columnar storage:
┌─────────────┬─────────────┬─────────────┬─────────────┐
│ id          │ timestamp   │ message     │ severity    │
│ id          │ timestamp   │ message     │ severity    │
│ id          │ timestamp   │ message     │ severity    │
└─────────────┴─────────────┴─────────────┴─────────────┘
```

Benefits:
- Only reads columns needed for query
- Same-type data compresses better (10x improvement)
- SIMD can process entire columns at once

### MergeTree Engine

ClickHouse's primary storage engine uses concepts from LSM-trees (like LevelDB, RocksDB):

- Append-only writes for speed
- Background merges optimize storage layout
- Sparse primary indexes (not every row indexed)
- Data parts sorted by primary key

### Vectorized Query Execution

- Processes data in blocks (64K rows default)
- Uses CPU SIMD instructions (AVX2, AVX-512)
- Column-wise operations instead of row-by-row
- 10-100x faster than traditional row processing

### JIT Compilation

- Compiles hot query paths to native machine code
- Uses LLVM for code generation
- Fuses multiple operations together
- Eliminates interpretation overhead

### Compression Codecs

| Codec | Best For | Compression |
|-------|----------|-------------|
| LZ4 | General purpose | Fast, moderate |
| ZSTD | Cold data | High compression |
| Delta | Timestamps, sequences | Excellent for time series |
| Gorilla | Float metrics | Facebook's algorithm |
| T64 | Integer sequences | Bit-packing |

## What We'd Have To Build

If building custom storage:

```
┌─────────────────────────────────────────────────────────┐
│                    QUERY LAYER                          │
│  SQL Parser → Query Planner → Optimizer → Executor      │
│  (months of work each)                                  │
├─────────────────────────────────────────────────────────┤
│                   EXECUTION LAYER                       │
│  Vectorized ops, SIMD, JIT compilation, parallelism     │
│  (requires deep systems expertise)                      │
├─────────────────────────────────────────────────────────┤
│                   STORAGE LAYER                         │
│  Columnar format, compression, indexing, merging        │
│  (this alone is a multi-year project)                   │
├─────────────────────────────────────────────────────────┤
│                   INFRASTRUCTURE                        │
│  Replication, sharding, backups, TTL, monitoring        │
│  (ongoing maintenance forever)                          │
└─────────────────────────────────────────────────────────┘
```

We'd essentially be recreating:
- Apache Kafka (ingestion)
- Apache Parquet (columnar storage)
- Apache Arrow (query engine)

...which is what ClickHouse already combines.

## Performance

According to benchmarks, ClickHouse is:
- **100x faster** than Hive for OLAP queries
- **100x faster** than MySQL for analytical workloads
- Capable of scanning billions of rows per second

## OpenTelemetry Integration

Additional benefit: OTEL Collector has a maintained ClickHouse exporter.

- Schema already designed and tested
- Zero custom ingestion code needed
- Community maintains compatibility

## When To Reconsider

The right time to build custom storage is when ClickHouse's model doesn't fit:

- Need ACID transactions (ClickHouse is eventually consistent)
- Need row-level updates (ClickHouse is append-optimized)
- Need sub-millisecond point lookups (ClickHouse optimizes for scans)

For logs and metrics, ClickHouse fits perfectly.

## References

- [ClickHouse Architecture Overview](https://clickhouse.com/docs/development/architecture)
- [Deep Dive into ClickHouse Internals](https://medium.com/@ShivIyer/deep-dive-into-clickhouse-internals-architectural-insights-and-performance-optimization-for-olap-b2c571f0c40d)
- [ClickHouse Architecture 101](https://www.chaosgenius.io/blog/clickhouse-architecture/)
- [ClickHouse Cloud Stateless Compute](https://clickhouse.com/blog/clickhouse-cloud-stateless-compute)
