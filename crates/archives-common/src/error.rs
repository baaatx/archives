//! Error types for Archives

use thiserror::Error;

/// Result type alias using Archives Error
pub type Result<T> = std::result::Result<T, Error>;

/// Archives error types
#[derive(Error, Debug)]
pub enum Error {
    #[error("ClickHouse connection error: {0}")]
    ClickHouseConnection(String),

    #[error("ClickHouse query error: {0}")]
    ClickHouseQuery(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid query parameter: {0}")]
    InvalidParameter(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound(_))
    }

    pub fn is_connection_error(&self) -> bool {
        matches!(self, Error::ClickHouseConnection(_))
    }
}
