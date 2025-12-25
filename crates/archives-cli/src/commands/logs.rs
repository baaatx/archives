//! Logs commands

use crate::{LogsCommands, OutputFormat};
use chrono::{Duration, Utc};
use serde_json::Value;

pub async fn handle(api_url: &str, command: LogsCommands, format: OutputFormat) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    match command {
        LogsCommands::Search { query, hours, severity, service, limit } => {
            let now = Utc::now();
            let start = now - Duration::hours(hours as i64);

            let mut body = serde_json::json!({
                "start": start.to_rfc3339(),
                "end": now.to_rfc3339(),
                "limit": limit
            });

            if let Some(q) = query {
                body["query"] = Value::String(q);
            }
            if let Some(s) = severity {
                body["min_severity"] = Value::String(s.to_uppercase());
            }
            if let Some(s) = service {
                body["service"] = Value::String(s);
            }

            let resp = client
                .post(format!("{}/v1/logs/search", api_url))
                .json(&body)
                .send()
                .await?
                .json::<Value>()
                .await?;

            print_logs(&resp, format);
        }

        LogsCommands::Tail { count, severity, service } => {
            let now = Utc::now();
            let start = now - Duration::minutes(10);

            let mut body = serde_json::json!({
                "start": start.to_rfc3339(),
                "end": now.to_rfc3339(),
                "limit": count
            });

            if let Some(s) = severity {
                body["min_severity"] = Value::String(s.to_uppercase());
            }
            if let Some(s) = service {
                body["service"] = Value::String(s);
            }

            let resp = client
                .post(format!("{}/v1/logs/search", api_url))
                .json(&body)
                .send()
                .await?
                .json::<Value>()
                .await?;

            print_logs(&resp, format);
        }

        LogsCommands::Errors { hours, limit } => {
            // Use the MCP endpoint for error summary
            let body = serde_json::json!({
                "tool": "get_error_summary",
                "params": {
                    "hours": hours,
                    "limit": limit
                }
            });

            let resp = client
                .post(format!("{}/../:8081/mcp", api_url).replace(":8080/../:8081", ":8081"))
                .json(&body)
                .send()
                .await?
                .json::<Value>()
                .await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                }
                _ => {
                    if let Some(data) = resp.get("data") {
                        if let Some(patterns) = data.get("top_patterns").and_then(|p| p.as_array()) {
                            println!("Top {} error patterns (last {} hours):\n", patterns.len(), hours);
                            for (i, pattern) in patterns.iter().enumerate() {
                                let count = pattern.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
                                let msg = pattern.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
                                println!("{}. [{}x] {}", i + 1, count, msg);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_logs(resp: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
        }
        OutputFormat::Compact => {
            if let Some(logs) = resp.get("logs").and_then(|l| l.as_array()) {
                for log in logs {
                    let ts = log.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
                    let sev = log.get("severity").and_then(|s| s.as_str()).unwrap_or("");
                    let msg = log.get("body").and_then(|b| b.as_str()).unwrap_or("");
                    println!("{} [{}] {}", &ts[11..19], sev, msg);
                }
            }
        }
        OutputFormat::Table => {
            if let Some(logs) = resp.get("logs").and_then(|l| l.as_array()) {
                println!("{:<20} {:<8} {:<20} {}", "TIMESTAMP", "SEVERITY", "SERVICE", "MESSAGE");
                println!("{}", "-".repeat(100));
                for log in logs {
                    let ts = log.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
                    let sev = log.get("severity").and_then(|s| s.as_str()).unwrap_or("");
                    let svc = log.get("service_name").and_then(|s| s.as_str()).unwrap_or("-");
                    let msg = log.get("body").and_then(|b| b.as_str()).unwrap_or("");
                    let msg_short = if msg.len() > 60 { &msg[..60] } else { msg };
                    println!("{:<20} {:<8} {:<20} {}", &ts[..19], sev, svc, msg_short);
                }
            }
        }
    }
}
