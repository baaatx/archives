//! ClickHouse client wrapper for Archives

use crate::{
    config::ClickHouseConfig,
    error::{Error, Result},
    types::{LogEntry, LogSeverity, Pagination, TimeRange},
};
use clickhouse::{Client, Row};
use serde::Deserialize;
use tracing::{debug, instrument};

/// ClickHouse client wrapper with connection pooling
#[derive(Clone)]
pub struct ClickHouseClient {
    client: Client,
    database: String,
}

impl ClickHouseClient {
    /// Create a new ClickHouse client from configuration
    pub fn new(config: &ClickHouseConfig) -> Result<Self> {
        let mut client = Client::default().with_url(&config.url);

        if let Some(ref username) = config.username {
            client = client.with_user(username);
        }

        if let Some(ref password) = config.password {
            client = client.with_password(password);
        }

        client = client.with_database(&config.database);

        Ok(Self {
            client,
            database: config.database.clone(),
        })
    }

    /// Check if ClickHouse is reachable
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool> {
        self.client
            .query("SELECT 1")
            .fetch_one::<u8>()
            .await
            .map(|_| true)
            .map_err(|e| Error::ClickHouseConnection(e.to_string()))
    }

    /// Get database statistics
    #[instrument(skip(self))]
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        #[derive(Row, Deserialize)]
        struct TableStats {
            table: String,
            rows: u64,
            bytes: u64,
        }

        let stats: Vec<TableStats> = self
            .client
            .query(
                r#"
                SELECT
                    table,
                    sum(rows) as rows,
                    sum(bytes) as bytes
                FROM system.parts
                WHERE database = ? AND active = 1
                GROUP BY table
                "#,
            )
            .bind(&self.database)
            .fetch_all()
            .await
            .map_err(|e| Error::ClickHouseQuery(e.to_string()))?;

        let mut db_stats = DatabaseStats::default();
        for stat in stats {
            match stat.table.as_str() {
                "otel_logs" => {
                    db_stats.log_count = stat.rows;
                    db_stats.log_bytes = stat.bytes;
                }
                t if t.starts_with("otel_metrics") => {
                    db_stats.metric_count += stat.rows;
                    db_stats.metric_bytes += stat.bytes;
                }
                _ => {}
            }
        }

        Ok(db_stats)
    }

    /// Search logs with filters
    #[instrument(skip(self))]
    pub async fn search_logs(&self, params: &LogSearchParams) -> Result<Vec<LogEntry>> {
        let mut query = String::from(
            r#"
            SELECT
                generateUUIDv4() as id,
                Timestamp as timestamp,
                ObservedTimestamp as observed_timestamp,
                TraceId as trace_id,
                SpanId as span_id,
                SeverityNumber as severity_number,
                SeverityText as severity_text,
                Body as body,
                ResourceAttributes as resource_attributes,
                LogAttributes as log_attributes,
                ServiceName as service_name
            FROM otel_logs
            WHERE Timestamp >= ? AND Timestamp < ?
            "#,
        );

        // Add severity filter
        if let Some(min_severity) = &params.min_severity {
            query.push_str(&format!(
                " AND SeverityNumber >= {}",
                min_severity.to_severity_number()
            ));
        }

        // Add text search
        if let Some(ref text) = params.text_query {
            query.push_str(" AND Body ILIKE ?");
        }

        // Add service filter
        if let Some(ref service) = params.service_name {
            query.push_str(" AND ServiceName = ?");
        }

        query.push_str(" ORDER BY Timestamp DESC");
        query.push_str(&format!(
            " LIMIT {} OFFSET {}",
            params.pagination.limit, params.pagination.offset
        ));

        let mut q = self
            .client
            .query(&query)
            .bind(params.time_range.start)
            .bind(params.time_range.end);

        if let Some(ref text) = params.text_query {
            q = q.bind(format!("%{}%", text));
        }

        if let Some(ref service) = params.service_name {
            q = q.bind(service);
        }

        #[derive(Row, Deserialize)]
        struct LogRow {
            id: uuid::Uuid,
            timestamp: time::OffsetDateTime,
            observed_timestamp: time::OffsetDateTime,
            trace_id: String,
            span_id: String,
            severity_number: i32,
            severity_text: String,
            body: String,
            resource_attributes: String,
            log_attributes: String,
            service_name: String,
        }

        let rows: Vec<LogRow> = q
            .fetch_all()
            .await
            .map_err(|e| Error::ClickHouseQuery(e.to_string()))?;

        let entries: Vec<LogEntry> = rows
            .into_iter()
            .map(|row| LogEntry {
                id: row.id,
                timestamp: chrono::DateTime::from_timestamp(
                    row.timestamp.unix_timestamp(),
                    row.timestamp.nanosecond(),
                )
                .unwrap_or_default(),
                observed_timestamp: chrono::DateTime::from_timestamp(
                    row.observed_timestamp.unix_timestamp(),
                    row.observed_timestamp.nanosecond(),
                )
                .unwrap_or_default(),
                trace_id: if row.trace_id.is_empty() {
                    None
                } else {
                    Some(row.trace_id)
                },
                span_id: if row.span_id.is_empty() {
                    None
                } else {
                    Some(row.span_id)
                },
                severity: LogSeverity::from_severity_number(row.severity_number),
                severity_text: row.severity_text,
                body: row.body,
                resource_attributes: serde_json::from_str(&row.resource_attributes)
                    .unwrap_or_default(),
                log_attributes: serde_json::from_str(&row.log_attributes).unwrap_or_default(),
                service_name: if row.service_name.is_empty() {
                    None
                } else {
                    Some(row.service_name)
                },
            })
            .collect();

        debug!(count = entries.len(), "Found log entries");
        Ok(entries)
    }

    /// Get log count for time range
    #[instrument(skip(self))]
    pub async fn count_logs(&self, time_range: &TimeRange) -> Result<u64> {
        #[derive(Row, Deserialize)]
        struct CountRow {
            count: u64,
        }

        let row: CountRow = self
            .client
            .query("SELECT count() as count FROM otel_logs WHERE Timestamp >= ? AND Timestamp < ?")
            .bind(time_range.start)
            .bind(time_range.end)
            .fetch_one()
            .await
            .map_err(|e| Error::ClickHouseQuery(e.to_string()))?;

        Ok(row.count)
    }

    /// List available metric names
    #[instrument(skip(self))]
    pub async fn list_metric_names(&self) -> Result<Vec<String>> {
        #[derive(Row, Deserialize)]
        struct NameRow {
            name: String,
        }

        let rows: Vec<NameRow> = self
            .client
            .query("SELECT DISTINCT MetricName as name FROM otel_metrics_gauge ORDER BY name")
            .fetch_all()
            .await
            .map_err(|e| Error::ClickHouseQuery(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.name).collect())
    }

    /// Query metrics with aggregation
    #[instrument(skip(self))]
    pub async fn query_metrics(&self, params: &MetricQueryParams) -> Result<Vec<MetricDataPoint>> {
        let agg_fn = match params.aggregation {
            crate::types::Aggregation::Avg => "avg(Value)",
            crate::types::Aggregation::Min => "min(Value)",
            crate::types::Aggregation::Max => "max(Value)",
            crate::types::Aggregation::Sum => "sum(Value)",
            crate::types::Aggregation::Count => "count()",
            crate::types::Aggregation::P50 => "quantile(0.5)(Value)",
            crate::types::Aggregation::P90 => "quantile(0.9)(Value)",
            crate::types::Aggregation::P99 => "quantile(0.99)(Value)",
        };

        let interval_seconds = params.interval_seconds.unwrap_or(60);

        let query = format!(
            r#"
            SELECT
                toStartOfInterval(TimeUnix, INTERVAL {} SECOND) as bucket,
                {} as value
            FROM otel_metrics_gauge
            WHERE MetricName = ?
              AND TimeUnix >= ?
              AND TimeUnix < ?
            GROUP BY bucket
            ORDER BY bucket
            "#,
            interval_seconds, agg_fn
        );

        #[derive(Row, Deserialize)]
        struct MetricRow {
            bucket: time::OffsetDateTime,
            value: f64,
        }

        let rows: Vec<MetricRow> = self
            .client
            .query(&query)
            .bind(&params.metric_name)
            .bind(params.time_range.start)
            .bind(params.time_range.end)
            .fetch_all()
            .await
            .map_err(|e| Error::ClickHouseQuery(e.to_string()))?;

        let points = rows
            .into_iter()
            .map(|row| MetricDataPoint {
                timestamp: chrono::DateTime::from_timestamp(
                    row.bucket.unix_timestamp(),
                    row.bucket.nanosecond(),
                )
                .unwrap_or_default(),
                value: row.value,
            })
            .collect();

        Ok(points)
    }
}

/// Database statistics
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub log_count: u64,
    pub log_bytes: u64,
    pub metric_count: u64,
    pub metric_bytes: u64,
}

/// Parameters for log search
#[derive(Debug, Clone)]
pub struct LogSearchParams {
    pub time_range: TimeRange,
    pub min_severity: Option<LogSeverity>,
    pub text_query: Option<String>,
    pub service_name: Option<String>,
    pub pagination: Pagination,
}

impl Default for LogSearchParams {
    fn default() -> Self {
        Self {
            time_range: TimeRange::last_hours(1),
            min_severity: None,
            text_query: None,
            service_name: None,
            pagination: Pagination::default(),
        }
    }
}

/// Parameters for metric query
#[derive(Debug, Clone)]
pub struct MetricQueryParams {
    pub metric_name: String,
    pub time_range: TimeRange,
    pub aggregation: crate::types::Aggregation,
    pub interval_seconds: Option<u32>,
    pub labels: Option<std::collections::HashMap<String, String>>,
}

/// A single metric data point in a time series
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricDataPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}
