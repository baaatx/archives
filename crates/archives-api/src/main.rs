//! Archives API Server
//!
//! HTTP API server for querying logs and metrics from ClickHouse.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing::{error, info};

use archives_common::{
    clickhouse::{ClickHouseClient, LogSearchParams, MetricDataPoint, MetricQueryParams},
    types::{Aggregation, LogSeverity, Pagination, TimeRange},
    Config,
};

/// Application state shared across handlers
struct AppState {
    clickhouse: ClickHouseClient,
    config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("archives_api=debug".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap()),
        )
        .json()
        .init();

    info!("Starting Archives API server");

    // Load configuration
    let config = Config::load_or_default();
    info!(
        clickhouse_url = %config.clickhouse.url,
        api_port = config.api.port,
        "Configuration loaded"
    );

    // Create ClickHouse client
    let clickhouse = ClickHouseClient::new(&config.clickhouse)?;

    // Check ClickHouse connectivity
    match clickhouse.health_check().await {
        Ok(true) => info!("ClickHouse connection established"),
        Ok(false) => error!("ClickHouse health check returned false"),
        Err(e) => error!(error = %e, "ClickHouse connection failed - continuing anyway"),
    }

    let state = Arc::new(AppState { clickhouse, config: config.clone() });

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/status", get(status_handler))
        .route("/v1/logs/search", post(search_logs_handler))
        .route("/v1/logs/{id}", get(get_log_handler))
        .route("/v1/metrics/query", post(query_metrics_handler))
        .route("/v1/metrics/names", get(list_metrics_handler))
        .layer(TimeoutLayer::new(Duration::from_secs(config.api.timeout_secs)))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = SocketAddr::new(
        config.api.host.parse().unwrap_or([0, 0, 0, 0].into()),
        config.api.port,
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(address = %addr, "Archives API server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Archives API server stopped");
    Ok(())
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

/// Health check endpoint
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.clickhouse.health_check().await {
        Ok(true) => (StatusCode::OK, Json(HealthResponse { status: "healthy", clickhouse: true })),
        _ => (StatusCode::SERVICE_UNAVAILABLE, Json(HealthResponse { status: "unhealthy", clickhouse: false })),
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    clickhouse: bool,
}

/// System status endpoint
async fn status_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.clickhouse.get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(StatusResponse {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
            log_count: stats.log_count,
            log_bytes: stats.log_bytes,
            metric_count: stats.metric_count,
            metric_bytes: stats.metric_bytes,
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StatusResponse {
            status: "error",
            version: env!("CARGO_PKG_VERSION"),
            log_count: 0,
            log_bytes: 0,
            metric_count: 0,
            metric_bytes: 0,
        })),
    }
}

#[derive(Serialize)]
struct StatusResponse {
    status: &'static str,
    version: &'static str,
    log_count: u64,
    log_bytes: u64,
    metric_count: u64,
    metric_bytes: u64,
}

/// Search logs endpoint
async fn search_logs_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LogSearchRequest>,
) -> impl IntoResponse {
    let params = LogSearchParams {
        time_range: TimeRange {
            start: request.start,
            end: request.end,
        },
        min_severity: request.min_severity,
        text_query: request.query,
        service_name: request.service,
        pagination: Pagination {
            offset: request.offset.unwrap_or(0),
            limit: request.limit.unwrap_or(100),
        },
    };

    match state.clickhouse.search_logs(&params).await {
        Ok(logs) => (StatusCode::OK, Json(LogSearchResponse { logs, error: None })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(LogSearchResponse {
            logs: vec![],
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Deserialize)]
struct LogSearchRequest {
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    query: Option<String>,
    min_severity: Option<LogSeverity>,
    service: Option<String>,
    offset: Option<u64>,
    limit: Option<u64>,
}

#[derive(Serialize)]
struct LogSearchResponse {
    logs: Vec<archives_common::types::LogEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Get single log by ID (placeholder - logs don't have stable IDs in OTEL schema)
async fn get_log_handler(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Note: OTEL logs don't have stable IDs, this would need trace_id + timestamp
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "error": "Log retrieval by ID not implemented - use search with trace_id filter"
    })))
}

/// Query metrics endpoint
async fn query_metrics_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MetricQueryRequest>,
) -> impl IntoResponse {
    let params = MetricQueryParams {
        metric_name: request.metric_name,
        time_range: TimeRange {
            start: request.start,
            end: request.end,
        },
        aggregation: request.aggregation.unwrap_or(Aggregation::Avg),
        interval_seconds: request.interval_seconds,
        labels: request.labels,
    };

    match state.clickhouse.query_metrics(&params).await {
        Ok(data) => (StatusCode::OK, Json(MetricQueryResponse { data, error: None })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MetricQueryResponse {
            data: vec![],
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Deserialize)]
struct MetricQueryRequest {
    metric_name: String,
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>,
    aggregation: Option<Aggregation>,
    interval_seconds: Option<u32>,
    labels: Option<std::collections::HashMap<String, String>>,
}

#[derive(Serialize)]
struct MetricQueryResponse {
    data: Vec<MetricDataPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// List metric names endpoint
async fn list_metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.clickhouse.list_metric_names().await {
        Ok(names) => (StatusCode::OK, Json(MetricNamesResponse { names, error: None })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MetricNamesResponse {
            names: vec![],
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Serialize)]
struct MetricNamesResponse {
    names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}
