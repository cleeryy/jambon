use std::str::FromStr;

use crate::error::Error;

/// Application configuration, populated from environment variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// Discord bot token.
    pub discord_token: String,

    /// Proxmox VE API base URL (e.g. `https://pve1:8006`).
    pub proxmox_url: String,

    /// Proxmox API token ID (e.g. `root@pam!discord-bot`).
    pub proxmox_token_id: String,

    /// Proxmox API token secret (UUID).
    pub proxmox_token_secret: String,

    /// Optional guild ID for instant slash-command registration during dev.
    pub dev_guild_id: Option<u64>,

    /// Channel to post health alerts to (optional).
    pub alert_channel_id: Option<u64>,

    /// Interval in seconds between health checks (default: 60).
    pub monitor_interval_secs: u64,

    /// Accept self-signed TLS certificates (default: true for Proxmox).
    pub accept_invalid_certs: bool,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Required variables
    ///
    /// - `DISCORD_TOKEN`
    /// - `PROXMOX_URL`
    /// - `PROXMOX_TOKEN_ID`
    /// - `PROXMOX_TOKEN_SECRET`
    ///
    /// # Optional variables
    ///
    /// - `DEV_GUILD_ID` — u64
    /// - `ALERT_CHANNEL_ID` — u64
    /// - `MONITOR_INTERVAL_SECS` — u64 (default: 60)
    /// - `ACCEPT_INVALID_CERTS` — `true` or `false` (default: true)
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            discord_token: get_env("DISCORD_TOKEN")?,
            proxmox_url: get_env("PROXMOX_URL")?,
            proxmox_token_id: get_env("PROXMOX_TOKEN_ID")?,
            proxmox_token_secret: get_env("PROXMOX_TOKEN_SECRET")?,
            dev_guild_id: opt_env("DEV_GUILD_ID"),
            alert_channel_id: opt_env("ALERT_CHANNEL_ID"),
            monitor_interval_secs: opt_env("MONITOR_INTERVAL_SECS").unwrap_or(60),
            accept_invalid_certs: opt_env("ACCEPT_INVALID_CERTS").unwrap_or(1) != 0,
        })
    }
}

fn get_env(key: &str) -> Result<String, Error> {
    std::env::var(key).map_err(|_| Error::Config(format!("Missing required env var: {key}")))
}

fn opt_env<T: FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}
