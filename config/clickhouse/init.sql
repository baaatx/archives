-- Archives ClickHouse initialization script
-- Applies TTL policies for data retention

-- Note: OTEL Collector creates tables automatically, but we can alter them for TTL

-- Apply 30-day TTL to logs table (if it exists)
ALTER TABLE otel_logs
    MODIFY TTL Timestamp + INTERVAL 30 DAY
    SETTINGS allow_nullable_key = 1;

-- Apply 90-day TTL to metrics tables (if they exist)
ALTER TABLE otel_metrics_gauge
    MODIFY TTL TimeUnix + INTERVAL 90 DAY
    SETTINGS allow_nullable_key = 1;

ALTER TABLE otel_metrics_sum
    MODIFY TTL TimeUnix + INTERVAL 90 DAY
    SETTINGS allow_nullable_key = 1;

ALTER TABLE otel_metrics_histogram
    MODIFY TTL TimeUnix + INTERVAL 90 DAY
    SETTINGS allow_nullable_key = 1;

-- Create a materialized view for error summary (optional optimization)
CREATE MATERIALIZED VIEW IF NOT EXISTS error_summary_mv
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMMDD(Timestamp)
ORDER BY (ServiceName, SeverityText, toStartOfHour(Timestamp))
AS SELECT
    ServiceName,
    SeverityText,
    toStartOfHour(Timestamp) as Hour,
    count() as Count
FROM otel_logs
WHERE SeverityNumber >= 17  -- ERROR and above
GROUP BY ServiceName, SeverityText, Hour;
