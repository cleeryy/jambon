/// Top-level errors for the Jambon bot.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Proxmox API error: {0}")]
    Proxmox(#[from] jambon_proxmox_api::Error),

    #[error("Discord error: {0}")]
    Discord(Box<poise::serenity_prelude::Error>),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
