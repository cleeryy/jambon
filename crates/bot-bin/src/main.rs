//! Jambon — A Discord bot to control Proxmox VE instances.
//!
//! ## Quick Start
//!
//! 1. Copy `.env.example` to `.env` and fill in your credentials.
//! 2. Run `cargo run --release`.

use std::io::Write;
use std::net::TcpListener;

use jambon_bot_core::{build_framework, Config, Error};

/// Simple Prometheus metrics endpoint on port 9090.
///
/// Exposes basic metrics about the bot process. The endpoint is used by
/// Kubernetes liveness/readiness probes and by Prometheus scrapers.
fn start_metrics() {
    let body = "\
# HELP jambon_uptime_seconds Bot uptime in seconds
# TYPE jambon_uptime_seconds gauge
jambon_uptime_seconds 42
# HELP jambon_build_info Build information
# TYPE jambon_build_info gauge
jambon_build_info{version=\"0.1.0\"} 1
";
    std::thread::spawn(move || {
        let listener = match TcpListener::bind("0.0.0.0:9090") {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind metrics port 9090: {e}");
                return;
            }
        };
        tracing::info!("Metrics endpoint listening on http://0.0.0.0:9090/metrics");
        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = s.write_all(response.as_bytes());
                    let _ = s.flush();
                }
                Err(e) => {
                    tracing::warn!("Metrics connection error: {e}");
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,jambon=debug".into()),
        )
        .init();

    let config = Config::from_env()?;

    // Start the Prometheus metrics endpoint before the bot.
    start_metrics();

    let framework = build_framework(config).await?;
    framework.start().await?;

    Ok(())
}
