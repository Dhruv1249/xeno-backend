//! Delivery Simulator Module
//!
//! Simulates lifecycle event progression for communications sent via different channels.
//!
//! Responsibilities:
//! - Spawn asynchronous simulation flows for every communication.
//! - Progress communication through states: queued -> sent -> delivered/failed -> opened -> clicked.
//! - Generate randomized delays and apply configurable event probabilities.

use crate::callbacks::send_callback;
use crate::models::{CallbackPayload, Communication, SendRequest};
use chrono::Utc;
use rand::Rng;
use std::time::Duration;
use uuid::Uuid;

/// Simulator probability and latency settings.
#[derive(Debug, Clone)]
pub struct SimulatorConfig {
    /// Probability of a message transitioning to the 'delivered' state.
    pub success_rate: f64,
    /// Probability of a message transitioning to the 'failed' state.
    pub failure_rate: f64,
    /// Probability of a delivered message transitioning to the 'opened' state.
    pub open_rate: f64,
    /// Probability of an opened message transitioning to the 'clicked' state.
    pub click_rate: f64,
}

/// Dispatches asynchronous simulation tasks for all communications in a send request.
///
/// # Arguments
/// * `client` - Shared HTTP client
/// * `request` - Send request payload from the CRM
/// * `config` - Simulator configurations
pub fn start_simulation(
    client: reqwest::Client,
    request: SendRequest,
    config: SimulatorConfig,
) {
    let campaign_id = request.campaign_id;
    let callback_url = request.callback_url;

    for comm in request.communications {
        let client_clone = client.clone();
        let callback_url_clone = callback_url.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            if let Err(e) = simulate_communication(
                &client_clone,
                campaign_id,
                comm,
                &callback_url_clone,
                config_clone,
            )
            .await
            {
                log::error!("Simulation task failed: {}", e);
            }
        });
    }
}

/// Simulates the lifecycle of a single customer message.
///
/// Progresses the communication status from queued through to clicked or failed.
async fn simulate_communication(
    client: &reqwest::Client,
    campaign_id: Uuid,
    comm: Communication,
    callback_url: &str,
    config: SimulatorConfig,
) -> Result<(), crate::errors::AppError> {
    // 1. Queue -> Sent (0.5s - 3s delay)
    sleep_random(0.5, 3.0).await;
    fire_event(client, callback_url, campaign_id, comm.communication_id, "sent").await?;

    // 2. Sent -> Delivered or Failed
    sleep_random(1.0, 5.0).await;
    let roll: f64 = rand::thread_rng().gen();

    if roll < config.failure_rate {
        fire_event(client, callback_url, campaign_id, comm.communication_id, "failed").await?;
        return Ok(());
    } else {
        fire_event(client, callback_url, campaign_id, comm.communication_id, "delivered").await?;
    }

    // 3. Delivered -> Opened (2s - 8s delay)
    sleep_random(2.0, 8.0).await;
    let open_roll: f64 = rand::thread_rng().gen();
    if open_roll < config.open_rate {
        fire_event(client, callback_url, campaign_id, comm.communication_id, "opened").await?;

        // 4. Opened -> Clicked (1s - 4s delay)
        sleep_random(1.0, 4.0).await;
        let click_roll: f64 = rand::thread_rng().gen();
        if click_roll < config.click_rate {
            fire_event(client, callback_url, campaign_id, comm.communication_id, "clicked").await?;
        }
    }

    Ok(())
}

/// Helper function to sleep for a random duration between min and max seconds.
async fn sleep_random(min: f64, max: f64) {
    let delay = rand::thread_rng().gen_range(min..=max);
    tokio::time::sleep(Duration::from_millis((delay * 1000.0) as u64)).await;
}

/// Sends a webhook callback to CRM for the given state.
async fn fire_event(
    client: &reqwest::Client,
    callback_url: &str,
    campaign_id: Uuid,
    comm_id: Uuid,
    event: &str,
) -> Result<(), crate::errors::AppError> {
    let payload = CallbackPayload {
        communication_id: comm_id,
        campaign_id,
        event_type: event.to_string(),
        occurred_at: Utc::now().to_rfc3339(),
        metadata: serde_json::json!({}),
    };
    send_callback(client, callback_url, payload).await
}
