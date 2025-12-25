//! Metrics commands

use crate::{MetricsCommands, OutputFormat};
use chrono::{Duration, Utc};
use serde_json::Value;

pub async fn handle(
    api_url: &str,
    command: MetricsCommands,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    match command {
        MetricsCommands::List => {
            let resp = client
                .get(format!("{}/v1/metrics/names", api_url))
                .send()
                .await?
                .json::<Value>()
                .await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                }
                _ => {
                    if let Some(names) = resp.get("names").and_then(|n| n.as_array()) {
                        println!("Available metrics ({}):\n", names.len());
                        for name in names {
                            if let Some(n) = name.as_str() {
                                println!("  {}", n);
                            }
                        }
                    }
                }
            }
        }

        MetricsCommands::Query {
            name,
            hours,
            aggregation,
            interval,
        } => {
            let now = Utc::now();
            let start = now - Duration::hours(hours as i64);

            let body = serde_json::json!({
                "metric_name": name,
                "start": start.to_rfc3339(),
                "end": now.to_rfc3339(),
                "aggregation": aggregation,
                "interval_seconds": interval
            });

            let resp = client
                .post(format!("{}/v1/metrics/query", api_url))
                .json(&body)
                .send()
                .await?
                .json::<Value>()
                .await?;

            match format {
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                }
                OutputFormat::Compact => {
                    if let Some(data) = resp.get("data").and_then(|d| d.as_array()) {
                        for point in data {
                            let ts = point
                                .get("timestamp")
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            let val = point.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("{} {:.4}", &ts[11..19], val);
                        }
                    }
                }
                OutputFormat::Table => {
                    println!("Metric: {} ({})", name, aggregation);
                    println!("{:<25} {:>15}", "TIMESTAMP", "VALUE");
                    println!("{}", "-".repeat(42));
                    if let Some(data) = resp.get("data").and_then(|d| d.as_array()) {
                        for point in data {
                            let ts = point
                                .get("timestamp")
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            let val = point.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!("{:<25} {:>15.4}", ts, val);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
