//! Archives MCP Server
//!
//! Model Context Protocol server exposing log and metrics search to ecosystem agents.

mod server;
mod tools;

use std::{net::SocketAddr, sync::Arc};

use tracing::info;

use archives_common::{clickhouse::ClickHouseClient, Config};

use server::McpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("archives_mcp=debug".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting Archives MCP server");

    // Load configuration
    let config = Config::load_or_default();
    info!(
        clickhouse_url = %config.clickhouse.url,
        mcp_port = config.mcp.port,
        "Configuration loaded"
    );

    // Create ClickHouse client
    let clickhouse = ClickHouseClient::new(&config.clickhouse)?;

    // Check connectivity
    match clickhouse.health_check().await {
        Ok(true) => info!("ClickHouse connection established"),
        Ok(false) => tracing::error!("ClickHouse health check returned false"),
        Err(e) => tracing::error!(error = %e, "ClickHouse connection failed - continuing anyway"),
    }

    // Create and run MCP server
    let server = McpServer::new(clickhouse, config.clone());

    let addr = SocketAddr::new(
        config.mcp.host.parse().unwrap_or([0, 0, 0, 0].into()),
        config.mcp.port,
    );

    info!(address = %addr, "Archives MCP server listening");
    server.run(addr).await?;

    info!("Archives MCP server stopped");
    Ok(())
}
