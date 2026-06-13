//! Error Handling Module
//!
//! Declarative error definitions for the Actix web server and simulator.
//!
//! Responsibilities:
//! - Define custom `AppError` variants.
//! - Implement Actix-web response representation for error types.

use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;

/// Custom error enum representing failures within the simulator.
#[derive(Error, Debug)]
pub enum AppError {
    /// Error parsing environment variables or configuration.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error during HTTP callback transmission.
    #[error("Callback delivery failed: {0}")]
    CallbackFailed(String),

    /// Network client errors.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Serialization/deserialization errors.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl ResponseError for AppError {
    /// Maps the internal error to an HTTP status code.
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::CallbackFailed(_) => StatusCode::BAD_GATEWAY,
            AppError::Network(_) => StatusCode::BAD_GATEWAY,
            AppError::Serialization(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Builds the HTTP response payload for the error.
    fn error_response(&self) -> actix_web::HttpResponse {
        log::error!("Error occurred: {}", self);
        actix_web::HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}
