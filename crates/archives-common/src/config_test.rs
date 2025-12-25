//! Tests for config module

use crate::config::{ApiConfig, ClickHouseConfig, Config, McpConfig, RetentionConfig};

#[test]
fn test_default_config() {
    let config = Config::default();

    // Check ClickHouse defaults
    assert_eq!(config.clickhouse.database, "default");
    assert_eq!(config.clickhouse.pool_size, 10);

    // Check API defaults
    assert_eq!(config.api.host, "0.0.0.0");
    assert_eq!(config.api.port, 8080);
    assert_eq!(config.api.timeout_secs, 30);

    // Check MCP defaults
    assert_eq!(config.mcp.host, "0.0.0.0");
    assert_eq!(config.mcp.port, 8081);
    assert!(config.mcp.enabled);

    // Check retention defaults
    assert_eq!(config.retention.log_retention_days, 30);
    assert_eq!(config.retention.metrics_retention_days, 90);
}

#[test]
fn test_clickhouse_config_default() {
    let config = ClickHouseConfig::default();
    assert!(config.url.contains("localhost") || config.url.contains("8123"));
    assert_eq!(config.database, "default");
    assert_eq!(config.pool_size, 10);
}

#[test]
fn test_api_config_default() {
    let config = ApiConfig::default();
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert_eq!(config.timeout_secs, 30);
}

#[test]
fn test_mcp_config_default() {
    let config = McpConfig::default();
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8081);
    assert!(config.enabled);
}

#[test]
fn test_retention_config_default() {
    let config = RetentionConfig::default();
    assert_eq!(config.log_retention_days, 30);
    assert_eq!(config.metrics_retention_days, 90);
}

#[test]
fn test_load_or_default() {
    // Should not panic even if config file doesn't exist
    let config = Config::load_or_default();
    assert_eq!(config.api.port, 8080);
}
