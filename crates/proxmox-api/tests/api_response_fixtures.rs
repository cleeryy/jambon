use jambon_proxmox_api::{
    ApiResponse, BackupJob, ClusterResource, LxcSummary, NodeStatus, NodeSummary, StorageContent, StorageSummary,
    TaskStatus, VersionInfo, VmConfig, VmStatus, VmSummary,
};

#[allow(clippy::expect_used)]
fn parse_envelope<T: serde::de::DeserializeOwned>(json: &str) -> T {
    let resp: ApiResponse<T> = serde_json::from_str(json).expect("envelope deserialization failed");
    resp.data
}

#[test]
fn version_info_fixture() {
    let json = r#"{"data":{"version":"8.2.4","release":"8.2","repoid":"abc123"}}"#;
    let v: VersionInfo = parse_envelope(json);
    assert_eq!(v.version, "8.2.4");
    assert_eq!(v.release, "8.2");
    assert_eq!(v.repoid, "abc123");
}

#[test]
fn node_summary_fixture() {
    let json = r#"{
        "data": [{
            "node": "pve1",
            "status": "online",
            "type": "node",
            "cpu": 0.23,
            "maxcpu": 16,
            "mem": 8589934592,
            "maxmem": 34359738368,
            "uptime": 1234567,
            "disk": 500000000000,
            "maxdisk": 1000000000000,
            "id": "node/pve1"
        }]
    }"#;
    let nodes: Vec<NodeSummary> = parse_envelope(json);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].node, "pve1");
    assert_eq!(nodes[0].status.as_deref(), Some("online"));
}

#[test]
fn node_status_fixture() {
    let json = r#"{
        "data": {
            "cpu": 0.42,
            "memory": { "used": 12884901888, "total": 34359738368, "free": 21474836480 },
            "swap": { "used": 1073741824, "total": 8589934592, "free": 7516192768 },
            "uptime": 12345678,
            "kversion": "Linux 6.8.8-1-pve",
            "loadavg": [0.5, 0.3, 0.1],
            "pveversion": "pve-manager/8.2.4/abc123",
            "currentkernel": "6.8.8-1-pve"
        }
    }"#;
    let s: NodeStatus = parse_envelope(json);
    assert!((s.cpu - 0.42).abs() < 1e-10);
    assert_eq!(s.uptime, 12345678);
}

#[test]
fn cluster_resource_fixture() {
    let json = r#"{
        "data": [{
            "id": "qemu/100",
            "type": "qemu",
            "node": "pve1",
            "vmid": 100,
            "name": "web-prod-01",
            "status": "running",
            "cpu": 0.15,
            "mem": 4294967296,
            "maxmem": 8589934592,
            "disk": 53687091200,
            "maxdisk": 107374182400,
            "uptime": 604800
        }]
    }"#;
    let resources: Vec<ClusterResource> = parse_envelope(json);
    assert_eq!(resources[0].kind, "qemu");
    assert_eq!(resources[0].vmid, Some(100));
}

#[test]
fn vm_summary_fixture() {
    let json = r#"{
        "data": [{
            "vmid": 100,
            "name": "web-prod-01",
            "status": "running",
            "node": "pve1",
            "mem": 4294967296,
            "maxmem": 8589934592,
            "cpu": 0.12,
            "uptime": 3600,
            "disk": 53687091200,
            "maxdisk": 107374182400,
            "pid": 1234,
            "tags": "production;web",
            "template": 0,
            "lock": null
        }]
    }"#;
    let vms: Vec<VmSummary> = parse_envelope(json);
    assert_eq!(vms[0].vmid, 100);
    assert_eq!(vms[0].status, "running");
}

#[test]
fn vm_status_fixture() {
    let json = r#"{
        "data": {
            "status": "running",
            "vmid": 100,
            "name": "web-prod-01",
            "cpu": 0.15,
            "maxcpu": 4,
            "mem": 4294967296,
            "maxmem": 8589934592,
            "uptime": 7200,
            "qmpstatus": "running",
            "pid": 1234
        }
    }"#;
    let s: VmStatus = parse_envelope(json);
    assert_eq!(s.status, "running");
    assert_eq!(s.maxcpu, Some(4));
}

#[test]
fn vm_config_fixture() {
    let json = r#"{
        "data": {
            "cores": 4,
            "memory": 8192,
            "name": "web-prod-01",
            "ostype": "l26",
            "tags": "production;web"
        }
    }"#;
    let config: VmConfig = parse_envelope(json);
    assert_eq!(config.properties.get("cores").and_then(|v| v.as_u64()), Some(4));
}

#[test]
fn lxc_summary_fixture() {
    let json = r#"{
        "data": [{
            "vmid": 200,
            "name": "db-dev",
            "status": "running",
            "node": "pve1",
            "mem": 1073741824,
            "maxmem": 2147483648,
            "cpu": 0.05,
            "uptime": 86400,
            "disk": 10737418240,
            "maxdisk": 21474836480,
            "lock": null
        }]
    }"#;
    let containers: Vec<LxcSummary> = parse_envelope(json);
    assert_eq!(containers[0].vmid, 200);
}

#[test]
fn storage_summary_fixture() {
    let json = r#"{
        "data": [{
            "storage": "local",
            "type": "dir",
            "status": "available",
            "used": 500000000000,
            "avail": 1500000000000,
            "used_fraction": 0.25,
            "content": "vztmpl;iso;backup",
            "shared": 0,
            "active": 1
        }]
    }"#;
    let storages: Vec<StorageSummary> = parse_envelope(json);
    assert_eq!(storages[0].storage, "local");
    assert_eq!(storages[0].kind.as_deref(), Some("dir"));
}

#[test]
fn storage_content_fixture() {
    let json = r#"{
        "data": [{
            "volid": "local:iso/debian-12.iso",
            "format": "iso",
            "size": 1200000000,
            "content": "iso",
            "name": "debian-12.iso",
            "ctime": 1700000000
        }]
    }"#;
    let contents: Vec<StorageContent> = parse_envelope(json);
    assert_eq!(contents[0].volid, "local:iso/debian-12.iso");
}

#[test]
fn backup_job_fixture() {
    let json = r#"{
        "data": [{
            "id": "backup-daily",
            "vmid": "100,101,102",
            "mode": "snapshot",
            "storage": "backup-pool",
            "compress": "zstd",
            "schedule": "0 2 * * *",
            "enabled": 1,
            "starttime": "2024-01-01 02:00:00",
            "repeat_missed": 1
        }]
    }"#;
    let jobs: Vec<BackupJob> = parse_envelope(json);
    assert_eq!(jobs[0].id, "backup-daily");
    assert_eq!(jobs[0].enabled, Some(1));
}

#[test]
fn task_response_fixture() {
    // The Proxmox API returns {"data": "UPID:..."} for task-starting endpoints.
    let json = r#"{"data":"UPID:pve1:00001234:..."}"#;
    let t: String = parse_envelope(json);
    assert_eq!(t, "UPID:pve1:00001234:...");
}

#[test]
fn task_status_fixture() {
    let json = r#"{
        "data": {
            "pid": 1234,
            "status": "running",
            "exitstatus": null,
            "starttime": 1700000000,
            "endtime": null,
            "upid": "UPID:pve1:00001234:...",
            "node": "pve1",
            "user": "root@pam"
        }
    }"#;
    let t: TaskStatus = parse_envelope(json);
    assert_eq!(t.status.as_deref(), Some("running"));
    assert_eq!(t.node.as_deref(), Some("pve1"));
}
