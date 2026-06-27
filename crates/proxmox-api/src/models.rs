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

#[derive(Debug, Clone)]
pub struct TaskResponse {
    pub data: String,
}

/// Proxmox returns a raw UPID string for task-starting actions:
/// `{"data": "UPID:pve1:..."}`. After handle_response strips the outer
/// `{"data": ...}` envelope, our custom Deserialize builds the struct
/// from the bare string.
impl<'de> serde::Deserialize<'de> for TaskResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| TaskResponse { data: s })
    }
}

impl std::fmt::Display for TaskResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
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

#[derive(Debug, Deserialize)]
pub struct LxcStatus {
    pub status: String,
    pub vmid: u64,
    pub name: Option<String>,
    pub cpu: Option<f64>,
    pub maxcpu: Option<u64>,
    pub mem: Option<u64>,
    pub maxmem: Option<u64>,
    pub uptime: Option<u64>,
    pub swap: Option<u64>,
    pub maxswap: Option<u64>,
    pub pid: Option<u64>,
    pub cpus: Option<u64>,
    pub lock: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LxcCreateOptions {
    pub node: String,
    pub vmid: u64,
    pub ostemplate: String,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cores: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net0: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onboot: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameserver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub searchdomain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_public_keys: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "unprivileged")]
    pub unprivileged: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct LxcCloneOptions {
    pub node: String,
    pub vmid: u64,
    pub newid: u64,
    pub hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LxcResizeOptions {
    pub node: String,
    pub vmid: u64,
    pub disk: String,
    pub size: String,
}

#[derive(Debug, Serialize)]
pub struct LxcShutdownOptions {
    pub node: String,
    pub vmid: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

// ---------------------------------------------------------------------------
// Pools
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PoolSummary {
    pub poolid: String,
    pub comment: Option<String>,
    pub members: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct PoolDetail {
    pub poolid: String,
    pub comment: Option<String>,
    pub members: Option<Vec<PoolMember>>,
}

#[derive(Debug, Deserialize)]
pub struct PoolMember {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub node: Option<String>,
    pub vmid: Option<u64>,
    pub content: Option<String>,
    pub storage: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PoolCreateOptions {
    pub poolid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

// ---------------------------------------------------------------------------
// Users & ACL
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct UserSummary {
    pub userid: String,
    pub enable: Option<u8>,
    pub email: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub expire: Option<u64>,
    pub comment: Option<String>,
    pub keys: Option<String>,
    pub tokens: Option<serde_json::Value>,
    pub realm: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserCreateOptions {
    pub userid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firstname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AclEntry {
    pub path: String,
    pub roles: String,
    pub users: Option<String>,
    pub groups: Option<String>,
    pub propagate: Option<u8>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub ugid: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AclUpdateOptions {
    pub path: String,
    pub roles: String,
    pub users: Option<String>,
    pub groups: Option<String>,
    pub propagate: Option<u8>,
    /// Set to 1 to remove the ACL entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<u8>,
}

// ---------------------------------------------------------------------------
// Firewall
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct FwRule {
    pub pos: Option<u64>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub action: Option<String>,
    pub proto: Option<String>,
    pub source: Option<String>,
    pub dest: Option<String>,
    pub sport: Option<String>,
    pub dport: Option<String>,
    pub iface: Option<String>,
    pub log: Option<String>,
    pub comment: Option<String>,
    pub enable: Option<u8>,
    pub digest: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FwRuleOptions {
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dport: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pos")]
    pub position: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct FwLogEntry {
    pub n: Option<u64>,
    pub timestamp: Option<String>,
    pub line: Option<String>,
}

// ---------------------------------------------------------------------------
// QEMU Agent
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AgentInfo {
    pub supported: Option<bool>,
    pub version: Option<String>,
    pub command: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgentNetworkInterface {
    pub name: Option<String>,
    pub hardware_address: Option<String>,
    pub ip_addresses: Option<Vec<AgentIpAddress>>,
}

#[derive(Debug, Deserialize)]
pub struct AgentIpAddress {
    #[serde(rename = "ip-address")]
    pub ip_address: Option<String>,
    #[serde(rename = "ip-address-type")]
    pub ip_address_type: Option<String>,
    pub prefix: Option<u64>,
    #[serde(rename = "is-loopback")]
    pub is_loopback: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AgentExecResult {
    pub pid: Option<u64>,
    #[serde(rename = "out-data")]
    pub out_data: Option<String>,
    #[serde(rename = "err-data")]
    pub err_data: Option<String>,
    #[serde(rename = "exitcode")]
    pub exit_code: Option<i64>,
    #[serde(rename = "truncated")]
    pub truncated: Option<bool>,
    #[serde(rename = "out-truncated")]
    pub out_truncated: Option<bool>,
    #[serde(rename = "err-truncated")]
    pub err_truncated: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AgentExecOptions {
    pub command: Vec<String>,
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
