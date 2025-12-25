//! Tests for error module

use crate::error::Error;

#[test]
fn test_is_not_found() {
    let err = Error::NotFound("test".to_string());
    assert!(err.is_not_found());

    let err = Error::Internal("test".to_string());
    assert!(!err.is_not_found());
}

#[test]
fn test_is_connection_error() {
    let err = Error::ClickHouseConnection("test".to_string());
    assert!(err.is_connection_error());

    let err = Error::ClickHouseQuery("test".to_string());
    assert!(!err.is_connection_error());
}

#[test]
fn test_error_display() {
    let err = Error::ClickHouseConnection("connection refused".to_string());
    assert_eq!(
        format!("{}", err),
        "ClickHouse connection error: connection refused"
    );

    let err = Error::NotFound("log entry".to_string());
    assert_eq!(format!("{}", err), "Resource not found: log entry");
}
