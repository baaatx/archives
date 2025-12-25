//! MCP Server implementation

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};

use archives_common::{clickhouse::ClickHouseClient, Config};

use crate::tools::{self, McpTool, ToolRegistry};

/// MCP Server state
pub struct McpServer {
    clickhouse: ClickHouseClient,
    config: Config,
    tools: ToolRegistry,
}

impl McpServer {
    pub fn new(clickhouse: ClickHouseClient, config: Config) -> Self {
        Self {
            clickhouse,
            config,
            tools: tools::create_tool_registry(),
        }
    }

    pub async fn run(self, addr: SocketAddr) -> anyhow::Result<()> {
        let state = Arc::new(AppState {
            clickhouse: self.clickhouse,
            config: self.config,
            tools: self.tools,
        });

        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/ping", get(ping_handler))
            .route("/mcp", post(mcp_handler))
            .route("/tools", get(list_tools_handler))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

struct AppState {
    clickhouse: ClickHouseClient,
    config: Config,
    tools: ToolRegistry,
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received");
}

// ============================================================================
// Handlers
// ============================================================================

async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.clickhouse.health_check().await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "healthy"})),
        ),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "unhealthy"})),
        ),
    }
}

async fn ping_handler() -> impl IntoResponse {
    Json(serde_json::json!({"pong": true}))
}

/// List available MCP tools
async fn list_tools_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tools: Vec<&McpTool> = state.tools.list();
    Json(serde_json::json!({
        "tools": tools
    }))
}

/// Main MCP endpoint for tool invocation
async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    info!(tool = %request.tool, "MCP tool invocation");

    match tools::execute_tool(&state.clickhouse, &request.tool, request.params).await {
        Ok(result) => (
            StatusCode::OK,
            Json(McpResponse {
                success: true,
                data: Some(result),
                error: None,
            }),
        ),
        Err(e) => {
            error!(tool = %request.tool, error = %e, "Tool execution failed");
            (
                StatusCode::OK,
                Json(McpResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
struct McpRequest {
    tool: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}
