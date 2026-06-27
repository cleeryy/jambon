//! Slash command handlers for Jambon.

pub mod acl;
pub mod admin;
pub mod audit;
pub mod autocomplete;
pub mod backup;
pub mod cluster;
pub mod colors;
pub mod container;
pub mod firewall;
pub mod interactions;
pub mod menu;
pub mod r#mod;
pub mod node;
pub mod node_utils;
pub mod permissions;
pub mod pool;
pub mod schedule;
pub mod scheduler;
pub mod storage;
pub mod vm;

pub use audit::AuditLog;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub proxmox: jambon_proxmox_api::ProxmoxClient,
    pub alert_channel_id: Option<u64>,
    pub monitor_interval_secs: u64,
    pub proxmox_url: String,
    pub audit_log: AuditLog,
    pub scheduler: std::sync::Arc<crate::scheduler::Scheduler>,
}

pub fn all_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        vm::vm(),
        node::node(),
        cluster::cluster(),
        storage::storage(),
        container::container(),
        pool::pool(),
        acl::acl(),
        firewall::fw(),
        backup::backup(),
        audit::audit(),
        r#mod::r#mod(),
        schedule::schedule(),
        schedule::autoscale(),
        admin::register(),
        admin::ping(),
        menu::menu(),
    ]
}
