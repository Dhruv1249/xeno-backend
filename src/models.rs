//! Models Module
//!
//! Defines the shared data structures for request validation, serialization,
//! and callback events sent back to the CRM API.
//!
//! Responsibilities:
//! - Map incoming JSON from Next.js campaign requests.
//! - Map outgoing JSON for delivery simulator webhooks.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Individual message recipient communication payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Communication {
    /// Unique identifier for this communication event.
    pub communication_id: Uuid,
    /// Recipient's phone number for SMS, WhatsApp, and RCS channels.
    pub recipient_phone: Option<String>,
    /// Recipient's email address for email channel.
    pub recipient_email: Option<String>,
    /// Channel name (whatsapp, sms, email, rcs).
    pub channel: String,
    /// Personalised text message body.
    pub message: String,
}

/// Incoming payload structure for POST /send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRequest {
    /// Unique campaign identifier.
    pub campaign_id: Uuid,
    /// List of communications to simulate delivery for.
    pub communications: Vec<Communication>,
    /// URL to callback on status changes.
    pub callback_url: String,
}

/// Outgoing callback webhook payload structure.
#[derive(Debug, Clone, Serialize)]
pub struct CallbackPayload {
    /// Reference to the specific communication event.
    pub communication_id: Uuid,
    /// Reference to the parent campaign.
    pub campaign_id: Uuid,
    /// Lifecycle transition (sent, delivered, opened, clicked, failed).
    pub event_type: String,
    /// Timestamp of when the simulated event occurred.
    pub occurred_at: String,
    /// Contextual metadata for debugging or extra tracking.
    pub metadata: serde_json::Value,
}

/// Standard success response structure.
#[derive(Debug, Clone, Serialize)]
pub struct SuccessResponse {
    /// Success or status message.
    pub message: String,
}
