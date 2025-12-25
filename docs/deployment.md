# Archives Deployment Guide

## Prerequisites

- Docker and Docker Compose
- 8GB+ RAM for ClickHouse
- 50GB+ disk space for data

## Local Development

### Quick Start

```bash
# Clone the repository
git clone https://github.com/baaatx/archives.git
cd archives

# Source the ops framework
source scripts/ops.sh

# Start infrastructure (ClickHouse + OTEL Collector)
ops start infra

# Run API in development mode
ops dev api
```

### Development Commands

```bash
ops build           # Build all crates
ops test            # Run tests
ops lint            # Run clippy
ops fmt fix         # Format code
ops check           # Run all CI checks
ops status          # Show service status
ops logs clickhouse # View ClickHouse logs
```

## Production Deployment

### Docker Compose

```bash
# Build and start all services
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Check status
docker compose ps

# View logs
docker compose logs -f archives-api
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| CLICKHOUSE_URL | http://localhost:8123 | ClickHouse HTTP URL |
| CLICKHOUSE_DATABASE | default | Database name |
| ARCHIVES__API__PORT | 8080 | API server port |
| ARCHIVES__MCP__PORT | 8081 | MCP server port |
| RUST_LOG | info | Log level |

### Ports

| Service | Port | Protocol |
|---------|------|----------|
| ClickHouse HTTP | 8123 | HTTP |
| ClickHouse Native | 9000 | TCP |
| OTEL Collector gRPC | 4317 | gRPC |
| OTEL Collector HTTP | 4318 | HTTP |
| Archives API | 8080 | HTTP |
| Archives MCP | 8081 | HTTP |

## Kubernetes Deployment

### ClickHouse

Use the official ClickHouse Operator or Helm chart:

```bash
helm repo add clickhouse https://docs.altinity.com/clickhouse-operator/
helm install clickhouse clickhouse/clickhouse-operator
```

### OTEL Collector

Deploy as DaemonSet for node-level collection:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: otel-collector
spec:
  selector:
    matchLabels:
      app: otel-collector
  template:
    spec:
      containers:
      - name: otel-collector
        image: otel/opentelemetry-collector-contrib:0.115.0
        ports:
        - containerPort: 4317
        - containerPort: 4318
```

### Archives API/MCP

Deploy as Deployment for horizontal scaling:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: archives-api
spec:
  replicas: 2
  selector:
    matchLabels:
      app: archives-api
  template:
    spec:
      containers:
      - name: archives-api
        image: archives-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: CLICKHOUSE_URL
          value: "http://clickhouse:8123"
```

## Data Retention

### Default TTL

- Logs: 30 days
- Metrics: 90 days
- Traces: 30 days

### Configuring TTL

1. Edit `config/clickhouse/init.sql`
2. Apply to ClickHouse:
   ```bash
   docker exec -i archives-clickhouse clickhouse-client < config/clickhouse/init.sql
   ```

## Monitoring Archives

Archives can monitor itself! Configure your apps to send telemetry:

```bash
# Set OTEL endpoint
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318

# Run your application with OTEL instrumentation
```

## Backup and Recovery

### Backup ClickHouse

```bash
# Backup using clickhouse-backup
docker exec archives-clickhouse clickhouse-backup create

# List backups
docker exec archives-clickhouse clickhouse-backup list
```

### Volume Backup

```bash
# Stop services
docker compose down

# Backup volume
docker run --rm -v archives_clickhouse_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/clickhouse-backup.tar.gz /data

# Restart services
docker compose up -d
```

## Troubleshooting

### ClickHouse Not Starting

Check logs:
```bash
docker compose logs clickhouse
```

Common issues:
- Insufficient memory (need 4GB+ free)
- Port conflicts (8123, 9000)

### OTEL Collector Not Receiving Data

Verify connectivity:
```bash
curl http://localhost:4318/v1/logs
```

Check collector logs:
```bash
docker compose logs otel-collector
```

### API Returns 503

ClickHouse connectivity issue. Check:
```bash
curl http://localhost:8123/ping
```
