//! Tests for types module

use crate::types::{Aggregation, LogSeverity, MetricType, Pagination, TimeRange};

#[test]
fn test_log_severity_from_severity_number() {
    assert_eq!(LogSeverity::from_severity_number(1), LogSeverity::Trace);
    assert_eq!(LogSeverity::from_severity_number(4), LogSeverity::Trace);
    assert_eq!(LogSeverity::from_severity_number(5), LogSeverity::Debug);
    assert_eq!(LogSeverity::from_severity_number(8), LogSeverity::Debug);
    assert_eq!(LogSeverity::from_severity_number(9), LogSeverity::Info);
    assert_eq!(LogSeverity::from_severity_number(12), LogSeverity::Info);
    assert_eq!(LogSeverity::from_severity_number(13), LogSeverity::Warn);
    assert_eq!(LogSeverity::from_severity_number(16), LogSeverity::Warn);
    assert_eq!(LogSeverity::from_severity_number(17), LogSeverity::Error);
    assert_eq!(LogSeverity::from_severity_number(20), LogSeverity::Error);
    assert_eq!(LogSeverity::from_severity_number(21), LogSeverity::Fatal);
    assert_eq!(LogSeverity::from_severity_number(24), LogSeverity::Fatal);
}

#[test]
fn test_log_severity_from_severity_number_out_of_range() {
    // Out of range defaults to Info
    assert_eq!(LogSeverity::from_severity_number(0), LogSeverity::Info);
    assert_eq!(LogSeverity::from_severity_number(25), LogSeverity::Info);
    assert_eq!(LogSeverity::from_severity_number(-1), LogSeverity::Info);
}

#[test]
fn test_log_severity_to_severity_number() {
    assert_eq!(LogSeverity::Trace.to_severity_number(), 1);
    assert_eq!(LogSeverity::Debug.to_severity_number(), 5);
    assert_eq!(LogSeverity::Info.to_severity_number(), 9);
    assert_eq!(LogSeverity::Warn.to_severity_number(), 13);
    assert_eq!(LogSeverity::Error.to_severity_number(), 17);
    assert_eq!(LogSeverity::Fatal.to_severity_number(), 21);
}

#[test]
fn test_log_severity_display() {
    assert_eq!(format!("{}", LogSeverity::Trace), "TRACE");
    assert_eq!(format!("{}", LogSeverity::Debug), "DEBUG");
    assert_eq!(format!("{}", LogSeverity::Info), "INFO");
    assert_eq!(format!("{}", LogSeverity::Warn), "WARN");
    assert_eq!(format!("{}", LogSeverity::Error), "ERROR");
    assert_eq!(format!("{}", LogSeverity::Fatal), "FATAL");
}

#[test]
fn test_log_severity_roundtrip() {
    for sev in [
        LogSeverity::Trace,
        LogSeverity::Debug,
        LogSeverity::Info,
        LogSeverity::Warn,
        LogSeverity::Error,
        LogSeverity::Fatal,
    ] {
        let num = sev.to_severity_number();
        assert_eq!(LogSeverity::from_severity_number(num), sev);
    }
}

#[test]
fn test_time_range_last_minutes() {
    let range = TimeRange::last_minutes(30);
    let now = chrono::Utc::now();

    // End should be close to now
    assert!((range.end - now).num_seconds().abs() < 2);

    // Duration should be approximately 30 minutes
    let duration = range.end - range.start;
    assert!((duration.num_minutes() - 30).abs() < 1);
}

#[test]
fn test_time_range_last_hours() {
    let range = TimeRange::last_hours(24);
    let now = chrono::Utc::now();

    // End should be close to now
    assert!((range.end - now).num_seconds().abs() < 2);

    // Duration should be approximately 24 hours
    let duration = range.end - range.start;
    assert!((duration.num_hours() - 24).abs() < 1);
}

#[test]
fn test_pagination_default() {
    let pagination = Pagination::default();
    assert_eq!(pagination.offset, 0);
    assert_eq!(pagination.limit, 100);
}

#[test]
fn test_aggregation_display() {
    assert_eq!(format!("{}", Aggregation::Avg), "avg");
    assert_eq!(format!("{}", Aggregation::Min), "min");
    assert_eq!(format!("{}", Aggregation::Max), "max");
    assert_eq!(format!("{}", Aggregation::Sum), "sum");
    assert_eq!(format!("{}", Aggregation::Count), "count");
    assert_eq!(format!("{}", Aggregation::P50), "p50");
    assert_eq!(format!("{}", Aggregation::P90), "p90");
    assert_eq!(format!("{}", Aggregation::P99), "p99");
}

#[test]
fn test_metric_type_display() {
    assert_eq!(format!("{}", MetricType::Gauge), "gauge");
    assert_eq!(format!("{}", MetricType::Sum), "sum");
    assert_eq!(format!("{}", MetricType::Histogram), "histogram");
    assert_eq!(
        format!("{}", MetricType::ExponentialHistogram),
        "exponential_histogram"
    );
    assert_eq!(format!("{}", MetricType::Summary), "summary");
}
