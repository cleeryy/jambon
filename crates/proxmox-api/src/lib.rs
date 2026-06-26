//! Proxmox VE REST API client for the Jambon Discord bot.
//!
//! This crate provides typed access to the Proxmox VE API v2 at
//! `https://<host>:8006/api2/json/`.  It supports both API-token-based
//! authentication (recommended for bots) and ticket-based session auth.

pub mod client;
pub mod error;
pub mod models;
pub mod resources;

pub use client::ProxmoxClient;
pub use error::Error;
pub use models::{
    ApiResponse, BackupJob, ClusterResource, LxcSummary, NodeStatus, NodeSummary, SnapshotCreateOptions,
    SnapshotListItem, StorageContent, StorageSummary, TaskResponse, TaskStatus, VersionInfo, VmCloneOptions, VmConfig,
    VmCreateOptions, VmResizeDiskOptions, VmShutdownOptions, VmStatus, VmSummary,
};
