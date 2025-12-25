//! Configuration for Archives services

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// ClickHouse configuration
    #[serde(default)]
    pub clickhouse: ClickHouseConfig,

    /// API server configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// MCP server configuration
    #[serde(default)]
    pub mcp: McpConfig,

    /// Retention configuration
    #[serde(default)]
    pub retention: RetentionConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clickhouse: ClickHouseConfig::default(),
            api: ApiConfig::default(),
            mcp: McpConfig::default(),
            retention: RetentionConfig::default(),
        }
    }
}

/// ClickHouse connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickHouseConfig {
    /// ClickHouse URL (e.g., "http://localhost:8123")
    #[serde(default = "default_clickhouse_url")]
    pub url: String,

    /// Database name
    #[serde(default = "default_database")]
    pub database: String,

    /// Username (optional)
    #[serde(default)]
    pub username: Option<String>,

    /// Password (optional)
    #[serde(default)]
    pub password: Option<String>,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

fn default_clickhouse_url() -> String {
    std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string())
}

fn default_database() -> String {
    std::env::var("CLICKHOUSE_DATABASE").unwrap_or_else(|_| "default".to_string())
}

fn default_pool_size() -> u32 {
    10
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: default_clickhouse_url(),
            database: default_database(),
            username: std::env::var("CLICKHOUSE_USERNAME").ok(),
            password: std::env::var("CLICKHOUSE_PASSWORD").ok(),
            pool_size: default_pool_size(),
        }
    }
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_api_port")]
    pub port: u16,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_api_port() -> u16 {
    8080
}

fn default_timeout() -> u64 {
    30
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_api_port(),
            timeout_secs: default_timeout(),
        }
    }
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_mcp_port")]
    pub port: u16,

    /// Whether MCP server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_mcp_port() -> u16 {
    8081
}

fn default_true() -> bool {
    true
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_mcp_port(),
            enabled: default_true(),
        }
    }
}

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Log retention in days
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u32,

    /// Metrics retention in days
    #[serde(default = "default_metrics_retention_days")]
    pub metrics_retention_days: u32,
}

fn default_log_retention_days() -> u32 {
    30
}

fn default_metrics_retention_days() -> u32 {
    90
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            log_retention_days: default_log_retention_days(),
            metrics_retention_days: default_metrics_retention_days(),
        }
    }
}

impl Config {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self, crate::Error> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::File::with_name("config.local").required(false))
            .add_source(config::Environment::with_prefix("ARCHIVES").separator("__"))
            .build()
            .map_err(|e| crate::Error::Config(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| crate::Error::Config(e.to_string()))
    }

    /// Load configuration with defaults (for when config file doesn't exist)
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }
}
