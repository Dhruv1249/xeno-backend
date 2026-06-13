//! Routes Module
//!
//! Bundles and exports route endpoint submodules for the Actix web server.
//!
//! Responsibilities:
//! - Re-export health probes and messaging dispatch routes.

pub mod health;
pub mod send;
