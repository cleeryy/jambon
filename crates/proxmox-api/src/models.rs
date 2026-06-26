use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Generic API response wrapper
// ---------------------------------------------------------------------------

/// Proxmox VE API responses are wrapped in a `{ data: ... }` envelope.
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub release: String,
    pub repoid: String,
}

// ---------------------------------------------------------------------------
// Nodes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NodeSummary {
    pub node: String,
    pub status: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub cpu: Option<f64>,
    pub maxcpu: Option<u64>,
    pub mem: Option<u64>,
    pub maxmem: Option<u64>,
    pub uptime: Option<u64>,
    pub disk: Option<u64>,
    pub maxdisk: Option<u64>,
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeStatus {
    pub cpu: f64,
    pub memory: NodeMemoryInfo,
    pub swap: NodeMemoryInfo,
    pub uptime: u64,
    pub kversion: Option<String>,
    pub loadavg: Option<Vec<f64>>,
    pub pveversion: Option<String>,
    pub currentkernel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NodeMemoryInfo {
    pub used: u64,
    pub total: u64,
    pub free: Option<u64>,
}

// ---------------------------------------------------------------------------
// Cluster Resources
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ClusterResource {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub node: Option<String>,
    pub vmid: Option<u64>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub cpu: Option<f64>,
    pub mem: Option<u64>,
    pub maxmem: Option<u64>,
    pub disk: Option<u64>,
    pub maxdisk: Option<u64>,
    pub uptime: Option<u64>,
    pub storage: Option<String>,
    pub content: Option<String>,
}

// ---------------------------------------------------------------------------
// QEMU (VM)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VmSummary {
    pub vmid: u64,
    pub name: Option<String>,
    pub status: String,
    pub node: Option<String>,
    pub mem: Option<u64>,
    pub maxmem: Option<u64>,
    pub cpu: Option<f64>,
    pub uptime: Option<u64>,
    pub disk: Option<u64>,
    pub maxdisk: Option<u64>,
    pub pid: Option<u64>,
    pub tags: Option<String>,
    pub template: Option<u8>,
    pub lock: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VmConfig {
    #[serde(flatten)]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct VmStatus {
    pub status: String,
    pub vmid: u64,
    pub name: Option<String>,
    pub cpu: Option<f64>,
    pub maxcpu: Option<u64>,
    pub mem: Option<u64>,
    pub maxmem: Option<u64>,
    pub uptime: Option<u64>,
    pub qmpstatus: Option<String>,
    pub pid: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TaskResponse {
    pub data: String, // UPID
}

// ---------------------------------------------------------------------------
// LXC (Container)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct LxcSummary {
    pub vmid: u64,
    pub name: Option<String>,
    pub status: String,
    pub node: Option<String>,
    pub mem: Option<u64>,
    pub maxmem: Option<u64>,
    pub cpu: Option<f64>,
    pub uptime: Option<u64>,
    pub disk: Option<u64>,
    pub maxdisk: Option<u64>,
    pub lock: Option<String>,
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct StorageSummary {
    pub storage: String,
    pub status: Option<String>,
    pub used: Option<u64>,
    pub avail: Option<u64>,
    pub used_fraction: Option<f64>,
    pub content: Option<String>,
    pub shared: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct StorageContent {
    pub volid: String,
    pub format: Option<String>,
    pub size: Option<u64>,
    pub content: Option<String>,
    pub vmid: Option<u64>,
    pub name: Option<String>,
    pub ctime: Option<u64>,
    pub parent: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

// ---------------------------------------------------------------------------
// Backup
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BackupJob {
    pub id: String,
    pub vmid: Option<String>,
    pub mode: Option<String>,
    pub storage: Option<String>,
    pub compress: Option<String>,
    pub schedule: Option<String>,
    pub enabled: Option<u8>,
    pub starttime: Option<String>,
    pub repeat_missed: Option<u8>,
    pub mailto: Option<String>,
}

// ---------------------------------------------------------------------------
// Task / UPID helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TaskStatus {
    pub pid: Option<u64>,
    pub status: Option<String>,
    pub exitstatus: Option<String>,
    pub starttime: Option<u64>,
    pub endtime: Option<u64>,
    pub upid: Option<String>,
    pub node: Option<String>,
    pub user: Option<String>,
}

// ---------------------------------------------------------------------------
// Request types (serializable)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct VmStartOptions {
    pub node: String,
    pub vmid: u64,
    /// Timeout in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct VmShutdownOptions {
    pub node: String,
    pub vmid: u64,
    /// Timeout in seconds before force-stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Skip HA checks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepActive: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct VmMigrateOptions {
    pub node: String,
    pub vmid: u64,
    pub target: String,
    /// 1 for online/live migration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<u8>,
}
