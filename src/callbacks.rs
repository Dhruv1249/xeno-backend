//! Webhook Callbacks Module
//!
//! Handles transmission of lifecycle event updates back to the CRM API.
//!
//! Responsibilities:
//! - Send status updates (sent, delivered, opened, clicked, failed).
//! - Implement automatic retry backoff on delivery failure.

use crate::errors::AppError;
use crate::models::CallbackPayload;
use std::time::Duration;

/// Sends a status update payload to the CRM server callback endpoint.
///
/// If transmission fails, retries up to 3 times with exponential backoff (1s, 2s, 4s).
///
/// # Arguments
/// * `client` - Shared HTTP client instance
/// * `callback_url` - Target webhook endpoint URL
/// * `payload` - Status callback payload to transmit
pub async fn send_callback(
    client: &reqwest::Client,
    callback_url: &str,
    payload: CallbackPayload,
) -> Result<(), AppError> {
    let mut retry_count = 0;
    let mut backoff = Duration::from_secs(1);
    let max_retries = 3;

    loop {
        log::info!(
            "Attempting to send callback for communication ID {} (Event: {}). Attempt {}/{}",
            payload.communication_id,
            payload.event_type,
            retry_count + 1,
            max_retries + 1
        );

        let secret = std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| "some-shared-secret".to_string());
        let response = client
            .post(callback_url)
            .header("x-webhook-secret", secret)
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                log::info!(
                    "Callback successfully delivered for communication ID {} (Event: {})",
                    payload.communication_id,
                    payload.event_type
                );
                return Ok(());
            }
            Ok(resp) => {
                let status = resp.status();
                log::warn!(
                    "CRM returned status {} for communication ID {}. Retrying...",
                    status,
                    payload.communication_id
                );
            }
            Err(e) => {
                log::warn!(
                    "Network error sending callback for communication ID {}: {}. Retrying...",
                    payload.communication_id,
                    e
                );
            }
        }

        if retry_count >= max_retries {
            log::error!(
                "All retry attempts failed to send callback for communication ID {} (Event: {})",
                payload.communication_id,
                payload.event_type
            );
            return Err(AppError::CallbackFailed(format!(
                "Failed to send callback after {} retries",
                max_retries
            )));
        }

        // Wait before trying again
        tokio::time::sleep(backoff).await;
        retry_count += 1;
        backoff *= 2;
    }
}
