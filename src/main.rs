//! Main Entrypoint
//!
//! Initializes logging, loads environment configurations, constructs shared state,
//! and spins up the Actix HTTP server for the channel simulator stub.
//!
//! Responsibilities:
//! - Bootstrap environment variables from `.env`.
//! - Establish the async server engine.
//! - Wire health and messaging routes.

mod callbacks;
mod errors;
mod models;
mod routes;
mod simulator;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenvy::dotenv;
use routes::send::AppState;
use simulator::SimulatorConfig;
use std::env;

/// Read float value from environment, falling back to a default value if missing or invalid.
fn get_env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(default)
}

/// Main bootloader for the Actix web application.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Attempt loading environment variables from a local `.env` file
    let _ = dotenv();

    // Setup logging format and output to stdout
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting Xeno Channel Simulator backend...");

    // Retrieve port configuration
    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    // Load simulator configurations
    let config = SimulatorConfig {
        success_rate: get_env_f64("DELIVERY_SUCCESS_RATE", 0.70),
        failure_rate: get_env_f64("FAILURE_RATE", 0.10),
        open_rate: get_env_f64("OPEN_RATE", 0.40),
        click_rate: get_env_f64("CLICK_RATE", 0.25),
    };

    log::info!("Loaded Simulation Config: {:?}", config);

    // Initialize the HTTP client for CRM receipts callback
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let app_state = web::Data::new(AppState {
        http_client,
        config,
    });

    log::info!("Server listening on http://0.0.0.0:{}", port);

    // Instantiate and bind Actix-web server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .service(routes::health::health_check)
            .service(routes::send::send_campaign)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
