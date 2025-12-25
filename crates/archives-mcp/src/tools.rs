//! MCP Tool implementations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use archives_common::{
    clickhouse::{ClickHouseClient, LogSearchParams},
    types::{Aggregation, LogSeverity, Pagination, TimeRange},
    Error, Result,
};

/// MCP Tool definition
#[derive(Debug, Clone, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Registry of available tools
pub struct ToolRegistry {
    tools: HashMap<String, McpTool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: McpTool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&McpTool> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<&McpTool> {
        self.tools.values().collect()
    }
}

/// Create the default tool registry with all available tools
pub fn create_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // search_logs tool
    registry.register(McpTool {
        name: "search_logs".to_string(),
        description: "Search logs with time range, severity filter, and text query. Returns matching log entries.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Text to search for in log messages"
                },
                "hours": {
                    "type": "integer",
                    "description": "Number of hours to search back (default: 1)",
                    "default": 1
                },
                "min_severity": {
                    "type": "string",
                    "enum": ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"],
                    "description": "Minimum severity level to include"
                },
                "service": {
                    "type": "string",
                    "description": "Filter by service name"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 50)",
                    "default": 50
                }
            }
        }),
    });

    // tail_logs tool
    registry.register(McpTool {
        name: "tail_logs".to_string(),
        description:
            "Get the most recent log entries. Useful for seeing what's happening right now."
                .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "count": {
                    "type": "integer",
                    "description": "Number of recent logs to return (default: 20)",
                    "default": 20
                },
                "min_severity": {
                    "type": "string",
                    "enum": ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"],
                    "description": "Minimum severity level to include"
                },
                "service": {
                    "type": "string",
                    "description": "Filter by service name"
                }
            }
        }),
    });

    // get_error_summary tool
    registry.register(McpTool {
        name: "get_error_summary".to_string(),
        description: "Get a summary of errors in the system. Groups errors by message pattern and shows counts.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "hours": {
                    "type": "integer",
                    "description": "Number of hours to analyze (default: 24)",
                    "default": 24
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of error patterns to return (default: 10)",
                    "default": 10
                }
            }
        }),
    });

    // query_metrics tool
    registry.register(McpTool {
        name: "query_metrics".to_string(),
        description: "Query metrics with aggregation over time. Returns time series data."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "required": ["metric_name"],
            "properties": {
                "metric_name": {
                    "type": "string",
                    "description": "Name of the metric to query"
                },
                "hours": {
                    "type": "integer",
                    "description": "Number of hours to query (default: 1)",
                    "default": 1
                },
                "aggregation": {
                    "type": "string",
                    "enum": ["avg", "min", "max", "sum", "count", "p50", "p90", "p99"],
                    "description": "Aggregation function (default: avg)",
                    "default": "avg"
                },
                "interval_seconds": {
                    "type": "integer",
                    "description": "Time bucket size in seconds (default: 60)",
                    "default": 60
                }
            }
        }),
    });

    // get_system_health tool
    registry.register(McpTool {
        name: "get_system_health".to_string(),
        description: "Get overall system health summary including error rates, log volume, and storage usage.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    });

    registry
}

/// Execute a tool by name
pub async fn execute_tool(
    clickhouse: &ClickHouseClient,
    tool_name: &str,
    params: Value,
) -> Result<Value> {
    match tool_name {
        "search_logs" => execute_search_logs(clickhouse, params).await,
        "tail_logs" => execute_tail_logs(clickhouse, params).await,
        "get_error_summary" => execute_get_error_summary(clickhouse, params).await,
        "query_metrics" => execute_query_metrics(clickhouse, params).await,
        "get_system_health" => execute_get_system_health(clickhouse, params).await,
        _ => Err(Error::NotFound(format!("Tool not found: {}", tool_name))),
    }
}

// ============================================================================
// Tool implementations
// ============================================================================

#[derive(Debug, Deserialize)]
struct SearchLogsParams {
    query: Option<String>,
    hours: Option<i64>,
    min_severity: Option<String>,
    service: Option<String>,
    limit: Option<u64>,
}

async fn execute_search_logs(clickhouse: &ClickHouseClient, params: Value) -> Result<Value> {
    let p: SearchLogsParams = serde_json::from_value(params)?;

    let hours = p.hours.unwrap_or(1);
    let limit = p.limit.unwrap_or(50);

    let min_severity = p
        .min_severity
        .and_then(|s| match s.to_uppercase().as_str() {
            "TRACE" => Some(LogSeverity::Trace),
            "DEBUG" => Some(LogSeverity::Debug),
            "INFO" => Some(LogSeverity::Info),
            "WARN" => Some(LogSeverity::Warn),
            "ERROR" => Some(LogSeverity::Error),
            "FATAL" => Some(LogSeverity::Fatal),
            _ => None,
        });

    let search_params = LogSearchParams {
        time_range: TimeRange::last_hours(hours),
        min_severity,
        text_query: p.query,
        service_name: p.service,
        pagination: Pagination { offset: 0, limit },
    };

    let logs = clickhouse.search_logs(&search_params).await?;

    // Format for LLM consumption
    let formatted: Vec<Value> = logs
        .iter()
        .map(|log| {
            serde_json::json!({
                "timestamp": log.timestamp.to_rfc3339(),
                "severity": log.severity.to_string(),
                "service": log.service_name,
                "message": log.body,
                "trace_id": log.trace_id,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "count": formatted.len(),
        "logs": formatted
    }))
}

#[derive(Debug, Deserialize)]
struct TailLogsParams {
    count: Option<u64>,
    min_severity: Option<String>,
    service: Option<String>,
}

async fn execute_tail_logs(clickhouse: &ClickHouseClient, params: Value) -> Result<Value> {
    let p: TailLogsParams = serde_json::from_value(params)?;

    let count = p.count.unwrap_or(20);

    let min_severity = p
        .min_severity
        .and_then(|s| match s.to_uppercase().as_str() {
            "TRACE" => Some(LogSeverity::Trace),
            "DEBUG" => Some(LogSeverity::Debug),
            "INFO" => Some(LogSeverity::Info),
            "WARN" => Some(LogSeverity::Warn),
            "ERROR" => Some(LogSeverity::Error),
            "FATAL" => Some(LogSeverity::Fatal),
            _ => None,
        });

    let search_params = LogSearchParams {
        time_range: TimeRange::last_minutes(10),
        min_severity,
        text_query: None,
        service_name: p.service,
        pagination: Pagination {
            offset: 0,
            limit: count,
        },
    };

    let logs = clickhouse.search_logs(&search_params).await?;

    let formatted: Vec<Value> = logs
        .iter()
        .map(|log| {
            serde_json::json!({
                "timestamp": log.timestamp.to_rfc3339(),
                "severity": log.severity.to_string(),
                "service": log.service_name,
                "message": log.body,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "count": formatted.len(),
        "logs": formatted
    }))
}

#[derive(Debug, Deserialize)]
struct ErrorSummaryParams {
    hours: Option<i64>,
    limit: Option<u64>,
}

async fn execute_get_error_summary(clickhouse: &ClickHouseClient, params: Value) -> Result<Value> {
    let p: ErrorSummaryParams = serde_json::from_value(params)?;

    let hours = p.hours.unwrap_or(24);
    let limit = p.limit.unwrap_or(10);

    // Get errors from the time range
    let search_params = LogSearchParams {
        time_range: TimeRange::last_hours(hours),
        min_severity: Some(LogSeverity::Error),
        text_query: None,
        service_name: None,
        pagination: Pagination {
            offset: 0,
            limit: 1000, // Get more logs for aggregation
        },
    };

    let logs = clickhouse.search_logs(&search_params).await?;

    // Group by message pattern (first 100 chars)
    let mut error_counts: HashMap<String, (u64, String)> = HashMap::new();
    for log in &logs {
        let pattern = if log.body.len() > 100 {
            format!("{}...", &log.body[..100])
        } else {
            log.body.clone()
        };
        let entry = error_counts
            .entry(pattern.clone())
            .or_insert((0, log.body.clone()));
        entry.0 += 1;
    }

    // Sort by count and take top N
    let mut sorted: Vec<_> = error_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    sorted.truncate(limit as usize);

    let patterns: Vec<Value> = sorted
        .into_iter()
        .map(|(pattern, (count, example))| {
            serde_json::json!({
                "pattern": pattern,
                "count": count,
                "example": example
            })
        })
        .collect();

    Ok(serde_json::json!({
        "total_errors": logs.len(),
        "time_range_hours": hours,
        "top_patterns": patterns
    }))
}

#[derive(Debug, Deserialize)]
struct QueryMetricsParams {
    metric_name: String,
    hours: Option<i64>,
    aggregation: Option<String>,
    interval_seconds: Option<u32>,
}

async fn execute_query_metrics(clickhouse: &ClickHouseClient, params: Value) -> Result<Value> {
    let p: QueryMetricsParams = serde_json::from_value(params)?;

    let hours = p.hours.unwrap_or(1);
    let interval = p.interval_seconds.unwrap_or(60);

    let aggregation = p
        .aggregation
        .and_then(|s| match s.to_lowercase().as_str() {
            "avg" => Some(Aggregation::Avg),
            "min" => Some(Aggregation::Min),
            "max" => Some(Aggregation::Max),
            "sum" => Some(Aggregation::Sum),
            "count" => Some(Aggregation::Count),
            "p50" => Some(Aggregation::P50),
            "p90" => Some(Aggregation::P90),
            "p99" => Some(Aggregation::P99),
            _ => None,
        })
        .unwrap_or(Aggregation::Avg);

    let query_params = archives_common::clickhouse::MetricQueryParams {
        metric_name: p.metric_name.clone(),
        time_range: TimeRange::last_hours(hours),
        aggregation,
        interval_seconds: Some(interval),
        labels: None,
    };

    let data = clickhouse.query_metrics(&query_params).await?;

    let points: Vec<Value> = data
        .iter()
        .map(|p| {
            serde_json::json!({
                "timestamp": p.timestamp.to_rfc3339(),
                "value": p.value
            })
        })
        .collect();

    Ok(serde_json::json!({
        "metric_name": p.metric_name,
        "aggregation": aggregation.to_string(),
        "interval_seconds": interval,
        "data_points": points.len(),
        "data": points
    }))
}

async fn execute_get_system_health(clickhouse: &ClickHouseClient, _params: Value) -> Result<Value> {
    // Get database stats
    let stats = clickhouse.get_stats().await?;

    // Get recent error count
    let error_params = LogSearchParams {
        time_range: TimeRange::last_hours(1),
        min_severity: Some(LogSeverity::Error),
        text_query: None,
        service_name: None,
        pagination: Pagination {
            offset: 0,
            limit: 1,
        },
    };
    let recent_errors = clickhouse
        .count_logs(&error_params.time_range)
        .await
        .unwrap_or(0);

    // Get total log count for last hour
    let total_logs = clickhouse
        .count_logs(&TimeRange::last_hours(1))
        .await
        .unwrap_or(0);

    Ok(serde_json::json!({
        "status": "operational",
        "storage": {
            "log_count": stats.log_count,
            "log_bytes": stats.log_bytes,
            "log_bytes_human": format_bytes(stats.log_bytes),
            "metric_count": stats.metric_count,
            "metric_bytes": stats.metric_bytes,
            "metric_bytes_human": format_bytes(stats.metric_bytes),
        },
        "last_hour": {
            "total_logs": total_logs,
            "error_count": recent_errors,
        }
    }))
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_tool_registry_register() {
        let mut registry = ToolRegistry::new();
        registry.register(McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({}),
        });
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_tool_registry_get() {
        let mut registry = ToolRegistry::new();
        registry.register(McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({}),
        });

        let tool = registry.get("test_tool");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "test_tool");

        let missing = registry.get("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_create_tool_registry() {
        let registry = create_tool_registry();
        let tools = registry.list();

        // Should have 5 tools
        assert_eq!(tools.len(), 5);

        // Check all expected tools exist
        assert!(registry.get("search_logs").is_some());
        assert!(registry.get("tail_logs").is_some());
        assert!(registry.get("get_error_summary").is_some());
        assert!(registry.get("query_metrics").is_some());
        assert!(registry.get("get_system_health").is_some());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 bytes");
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.00 GB");
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "Test description".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"description\":\"Test description\""));
    }
}
