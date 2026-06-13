//! Campaign Send Route Module
//!
//! Receives communications requests from CRM and starts simulation.
//!
//! Responsibilities:
//! - Validate incoming `SendRequest` payload.
//! - Instantly return 202 Accepted.
//! - Dispatch asynchronous background worker tasks.

use crate::errors::AppError;
use crate::models::{SendRequest, SuccessResponse};
use crate::simulator::{start_simulation, SimulatorConfig};
use actix_web::{post, web, HttpResponse, Responder};

/// Shared state container for Actix endpoints.
#[derive(Clone)]
pub struct AppState {
    /// Shared HTTP client for callbacks.
    pub http_client: reqwest::Client,
    /// Simulator settings loaded from environment.
    pub config: SimulatorConfig,
}

/// Receives a list of messages to simulate, triggers background tasks, and returns 202.
///
/// # Arguments
/// * `payload` - Validated request body
/// * `state` - Shared app state containing HTTP client and simulator configs
#[post("/send")]
pub async fn send_campaign(
    payload: web::Json<SendRequest>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    log::info!(
        "Received send request for Campaign: {} containing {} communications",
        payload.campaign_id,
        payload.communications.len()
    );

    // Start simulation in background tasks (tokio::spawn)
    start_simulation(
        state.http_client.clone(),
        payload.into_inner(),
        state.config.clone(),
    );

    // Return 202 Accepted immediately to decoupled caller
    Ok(HttpResponse::Accepted().json(SuccessResponse {
        message: "Simulation dispatched".to_string(),
    }))
}
