//! Slash command handlers for Jambon.
//!
//! Commands are grouped by domain: Proxmox VM operations, node operations,
//! cluster operations, and administrative commands.

pub mod admin;
pub mod audit;
pub mod backup;
pub mod cluster;
pub mod r#mod;
pub mod node;
pub mod permissions;
pub mod storage;
pub mod vm;

pub use audit::AuditLog;

/// Shared error type for all command handlers.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Shared context alias for all command handlers.
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Application data shared across all commands.
pub struct Data {
    pub proxmox: jambon_proxmox_api::ProxmoxClient,
    pub alert_channel_id: Option<u64>,
    pub monitor_interval_secs: u64,
    pub proxmox_url: String,
    pub audit_log: AuditLog,
}

/// Collect all commands into a single flat Vec for the framework.
pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        vm::vm(),
        node::node(),
        cluster::cluster(),
        storage::storage(),
        backup::backup(),
        audit::audit(),
        r#mod::r#mod(),
        admin::register(),
        admin::ping(),
    ]
}
