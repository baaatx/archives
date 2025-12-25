//! Status command

use crate::OutputFormat;
use serde_json::Value;

pub async fn handle(api_url: &str, format: OutputFormat) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/v1/status", api_url))
        .send()
        .await?
        .json::<Value>()
        .await?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        _ => {
            println!("Archives System Status");
            println!("======================\n");

            let status = resp.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
            let version = resp.get("version").and_then(|v| v.as_str()).unwrap_or("unknown");
            println!("Status:  {}", status);
            println!("Version: {}\n", version);

            let log_count = resp.get("log_count").and_then(|c| c.as_u64()).unwrap_or(0);
            let log_bytes = resp.get("log_bytes").and_then(|b| b.as_u64()).unwrap_or(0);
            let metric_count = resp.get("metric_count").and_then(|c| c.as_u64()).unwrap_or(0);
            let metric_bytes = resp.get("metric_bytes").and_then(|b| b.as_u64()).unwrap_or(0);

            println!("Storage:");
            println!("  Logs:    {} entries ({:.2} MB)", log_count, log_bytes as f64 / 1024.0 / 1024.0);
            println!("  Metrics: {} entries ({:.2} MB)", metric_count, metric_bytes as f64 / 1024.0 / 1024.0);
        }
    }

    Ok(())
}
