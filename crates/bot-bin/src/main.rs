//! Jambon — A Discord bot to control Proxmox VE instances.
//!
//! ## Quick Start
//!
//! 1. Copy `.env.example` to `.env` and fill in your credentials.
//! 2. Run `cargo run --release`.

use jambon_bot_core::{build_framework, Config, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load .env before anything else.
    dotenvy::dotenv().ok();

    // Structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,jambon=debug".into()),
        )
        .init();

    // Read config from environment.
    let config = Config::from_env()?;

    // Build and start the framework.
    let framework = build_framework(config).await?;
    framework.start().await?;

    Ok(())
}
