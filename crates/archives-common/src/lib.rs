//! Archives Common Library
//!
//! Shared types, utilities, and ClickHouse client for the Archives observability platform.

pub mod clickhouse;
pub mod config;
pub mod error;
pub mod types;

#[cfg(test)]
mod config_test;
#[cfg(test)]
mod error_test;
#[cfg(test)]
mod types_test;

pub use config::Config;
pub use error::{Error, Result};
pub use types::{LogEntry, LogSeverity, Metric, MetricType};
