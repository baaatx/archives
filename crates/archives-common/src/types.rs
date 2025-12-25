//! Core types for Archives
//!
//! These types match the OpenTelemetry ClickHouse exporter schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Log severity levels matching OpenTelemetry specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogSeverity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogSeverity {
    /// Convert from OTEL severity number (1-24)
    pub fn from_severity_number(num: i32) -> Self {
        match num {
            1..=4 => LogSeverity::Trace,
            5..=8 => LogSeverity::Debug,
            9..=12 => LogSeverity::Info,
            13..=16 => LogSeverity::Warn,
            17..=20 => LogSeverity::Error,
            21..=24 => LogSeverity::Fatal,
            _ => LogSeverity::Info,
        }
    }

    /// Convert to OTEL severity number
    pub fn to_severity_number(&self) -> i32 {
        match self {
            LogSeverity::Trace => 1,
            LogSeverity::Debug => 5,
            LogSeverity::Info => 9,
            LogSeverity::Warn => 13,
            LogSeverity::Error => 17,
            LogSeverity::Fatal => 21,
        }
    }
}

impl std::fmt::Display for LogSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogSeverity::Trace => write!(f, "TRACE"),
            LogSeverity::Debug => write!(f, "DEBUG"),
            LogSeverity::Info => write!(f, "INFO"),
            LogSeverity::Warn => write!(f, "WARN"),
            LogSeverity::Error => write!(f, "ERROR"),
            LogSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

/// A log entry from ClickHouse otel_logs table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier
    pub id: Uuid,

    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,

    /// Observed timestamp (when collector received it)
    pub observed_timestamp: DateTime<Utc>,

    /// Trace ID for correlation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Span ID for correlation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,

    /// Severity level
    pub severity: LogSeverity,

    /// Severity text (original string)
    pub severity_text: String,

    /// Log body/message
    pub body: String,

    /// Resource attributes (service name, version, etc.)
    #[serde(default)]
    pub resource_attributes: serde_json::Value,

    /// Log attributes
    #[serde(default)]
    pub log_attributes: serde_json::Value,

    /// Service name (extracted from resource attributes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
}

/// Metric types matching OpenTelemetry specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Gauge,
    Sum,
    Histogram,
    ExponentialHistogram,
    Summary,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Sum => write!(f, "sum"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::ExponentialHistogram => write!(f, "exponential_histogram"),
            MetricType::Summary => write!(f, "summary"),
        }
    }
}

/// A metric data point from ClickHouse otel_metrics tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Metric unit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Metric type
    pub metric_type: MetricType,

    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,

    /// Metric value (for gauge/sum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,

    /// Resource attributes
    #[serde(default)]
    pub resource_attributes: serde_json::Value,

    /// Metric attributes/labels
    #[serde(default)]
    pub attributes: serde_json::Value,

    /// Service name (extracted from resource attributes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
}

/// Time range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (inclusive)
    pub start: DateTime<Utc>,

    /// End time (exclusive)
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Create a time range for the last N minutes
    pub fn last_minutes(minutes: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::minutes(minutes);
        Self { start, end }
    }

    /// Create a time range for the last N hours
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self { start, end }
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Number of items to skip
    #[serde(default)]
    pub offset: u64,

    /// Maximum number of items to return
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 {
    100
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: default_limit(),
        }
    }
}

/// Aggregation functions for metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aggregation {
    Avg,
    Min,
    Max,
    Sum,
    Count,
    P50,
    P90,
    P99,
}

impl std::fmt::Display for Aggregation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aggregation::Avg => write!(f, "avg"),
            Aggregation::Min => write!(f, "min"),
            Aggregation::Max => write!(f, "max"),
            Aggregation::Sum => write!(f, "sum"),
            Aggregation::Count => write!(f, "count"),
            Aggregation::P50 => write!(f, "p50"),
            Aggregation::P90 => write!(f, "p90"),
            Aggregation::P99 => write!(f, "p99"),
        }
    }
}
