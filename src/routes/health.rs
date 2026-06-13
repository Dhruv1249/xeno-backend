//! Health Check Route Module
//!
//! Exposes a simple health assessment endpoint for container orchestrators or monitoring.
//!
//! Responsibilities:
//! - Respond to GET /health requests with a 200 OK.

use actix_web::{get, HttpResponse, Responder};

/// Handles health check requests.
///
/// Returns HTTP 200 OK with a status payload.
#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "xeno-channel-simulator"
    }))
}
