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
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub status: Option<String>,
    pub used: Option<u64>,
    pub avail: Option<u64>,
    pub used_fraction: Option<f64>,
    pub content: Option<String>,
    pub shared: Option<u8>,
    pub active: Option<u8>,
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
pub struct BackupCreateOptions {
    pub vmid: String,
    pub storage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compress: Option<String>,
}

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
    #[serde(skip_serializing_if = "Option::is_none", rename = "keepActive")]
    pub keep_active: Option<bool>,
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

// ---------------------------------------------------------------------------
// VM Lifecycle
// ---------------------------------------------------------------------------

/// Options for creating a VM (POST /nodes/{node}/qemu).
#[derive(Debug, Serialize)]
pub struct VmCreateOptions {
    pub node: String,
    pub name: String,
    /// Desired VM ID (auto-assigned if omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vmid: Option<u64>,
    /// Source template VM ID to clone from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores: Option<u64>,
    /// Memory in MiB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    /// Target storage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    /// 1 for full clone (defaults to 1 for non-linked clones).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<u8>,
}

/// Options for cloning a VM (POST /nodes/{node}/qemu/{vmid}/clone).
#[derive(Debug, Serialize)]
pub struct VmCloneOptions {
    pub node: String,
    pub vmid: u64,
    pub newid: u64,
    pub name: String,
    /// Target storage for the clone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    /// 1 for full clone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<u8>,
    /// Target node for the clone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Options for resizing a VM disk (PUT /nodes/{node}/qemu/{vmid}/resize).
#[derive(Debug, Serialize)]
pub struct VmResizeDiskOptions {
    /// Disk identifier (e.g. "scsi0", "virtio0").
    pub disk: String,
    /// New size (e.g. "+10G", "32G").
    pub size: String,
}

/// A snapshot entry returned by the list-snapshots endpoint.
#[derive(Debug, Deserialize)]
pub struct SnapshotListItem {
    pub name: String,
    pub description: Option<String>,
    /// Unix epoch timestamp of when the snapshot was taken.
    pub snaptime: Option<u64>,
    /// Parent snapshot name (empty string for root / "current").
    pub parent: Option<String>,
    /// VM ID this snapshot belongs to.
    pub vmid: Option<u64>,
}

/// Options for creating a VM snapshot (POST /nodes/{node}/qemu/{vmid}/snapshot).
#[derive(Debug, Serialize)]
pub struct SnapshotCreateOptions {
    /// Snapshot name.
    pub snapname: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
