//! Core bot framework integration.
//!
//! This crate ties the Discord gateway (poise / serenity) to the Proxmox
//! API client and hosts the framework setup, configuration, error handling,
//! background tasks, and shared types.

pub mod config;
pub mod error;
pub mod events;
pub mod framework;

pub use config::Config;
pub use error::Error;
pub use framework::build_framework;
