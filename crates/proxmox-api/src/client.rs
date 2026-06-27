use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::Client as HttpClient;

use crate::error::Error;
use crate::models::*;
use crate::resources::Resources;

/// A typed client for the Proxmox VE REST API v2.
///
/// ## Authentication
///
/// Use [`with_api_token`][Self::with_api_token] for long-lived bot access
/// (recommended) or [`with_ticket`][Self::with_ticket] for short-lived
/// sessions.
///
/// ## Example
///
/// ```rust,no_run
/// use jambon_proxmox_api::ProxmoxClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ProxmoxClient::with_api_token(
///     "https://pve.example.com:8006",
///     "root@pam!my-bot",
///     "my-secret",
///     true,  // accept_invalid_certs (self-signed certs)
/// )?;
///
/// let version = client.version().await?;
/// println!("Proxmox VE {}", version.version);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct ProxmoxClient {
    base_url: String,
    http: HttpClient,
    /// The `Authorization` header value, cached after construction.
    auth_header: HeaderValue,
}

impl ProxmoxClient {
    /// Create a client using an API token.
    ///
    /// The token should be created on the Proxmox host with
    /// `pveum user token add <user> <token-id> --privsep=0`.
    ///
    /// # Arguments
    ///
    /// * `base_url` – e.g. `"https://pve1:8006"`
    /// * `token_id` – e.g. `"root@pam!discord-bot"`
    /// * `token_secret` – the UUID secret returned by `pveum`
    /// * `accept_invalid_certs` – set to `true` if Proxmox uses a self-signed
    ///   certificate
    pub fn with_api_token(
        base_url: impl Into<String>,
        token_id: &str,
        token_secret: &str,
        accept_invalid_certs: bool,
    ) -> Result<Self, Error> {
        let base_url = base_url.into();
        let http = Self::build_http(accept_invalid_certs)?;
        let auth_value = format!("PVEAPIToken={token_id}={token_secret}");
        let auth_header =
            HeaderValue::from_str(&auth_value).map_err(|e| Error::Config(format!("invalid token value: {e}")))?;

        Ok(Self {
            base_url,
            http,
            auth_header,
        })
    }

    /// Create a client with username/password ticket auth.
    ///
    /// This is less convenient for bots than API-token auth because tickets
    /// expire (usually after 2 hours).
    ///
    /// You must call `login` before making other requests.
    pub fn with_ticket(base_url: impl Into<String>, accept_invalid_certs: bool) -> Result<Self, Error> {
        let base_url = base_url.into();
        let http = Self::build_http(accept_invalid_certs)?;
        let auth_header = HeaderValue::from_static(""); // placeholder, set by login()

        Ok(Self {
            base_url,
            http,
            auth_header,
        })
    }

    fn build_http(accept_invalid_certs: bool) -> Result<HttpClient, Error> {
        let hb = HttpClient::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .timeout(std::time::Duration::from_secs(30));
        // Content-Type is set automatically by .json() for POST/PUT — no
        // default needed (would cause empty-body POST to send wrong headers).
        hb.build().map_err(Error::Http)
    }

    // ── High-level methods ────────────────────────────────────────────

    pub async fn version(&self) -> Result<VersionInfo, Error> {
        self.get("version").await
    }

    pub fn resources(&self) -> Resources<'_> {
        Resources { client: self }
    }

    // ── Internal helpers ──────────────────────────────────────────────

    fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        let base = self.base_url.trim_end_matches('/');
        format!("{base}/api2/json/{path}")
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        let resp = self
            .http
            .get(self.url(path))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;

        Self::handle_response(resp).await
    }

    async fn post<T: serde::de::DeserializeOwned>(&self, path: &str, body: impl serde::Serialize) -> Result<T, Error> {
        let resp = self
            .http
            .post(self.url(path))
            .header(AUTHORIZATION, self.auth_header.clone())
            .json(&body)
            .send()
            .await?;

        Self::handle_response(resp).await
    }

    async fn post_empty<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        let resp = self
            .http
            .post(self.url(path))
            .header(AUTHORIZATION, self.auth_header.clone())
            .json(&serde_json::Value::Object(serde_json::Map::new()))
            .send()
            .await?;

        Self::handle_response(resp).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T, Error> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return match status {
                reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => Err(Error::Unauthorized(body)),
                reqwest::StatusCode::NOT_FOUND => Err(Error::NotFound(body)),
                _ => Err(Error::Api { status, body }),
            };
        }

        // Proxmox VE wraps all API responses in {"data": ...} envelope.
        // We unwrap it here so every method gets the inner T directly.
        let body = resp.text().await?;
        serde_json::from_str::<ApiResponse<T>>(&body)
            .map(|envelope| envelope.data)
            .map_err(Error::Json)
    }

    // ── Cluster ──────────────────────────────────────────────────────

    pub async fn cluster_status(&self) -> Result<Vec<NodeSummary>, Error> {
        // The `/cluster/status` endpoint returns entries for the cluster
        // itself *and* each node.  Cluster-level entries lack a `node`
        // field, so we deserialise as untyped values first, then filter
        // and re-deserialise only entries that have one.
        let values: Vec<serde_json::Value> = self.get("cluster/status").await?;
        let mut nodes = Vec::with_capacity(values.len());
        for v in values {
            if v.get("node").and_then(|n| n.as_str()).is_some() {
                if let Ok(n) = serde_json::from_value(v) {
                    nodes.push(n);
                }
            }
        }
        Ok(nodes)
    }

    pub async fn cluster_resources(&self) -> Result<Vec<ClusterResource>, Error> {
        self.get("cluster/resources").await
    }

    // ── Nodes ────────────────────────────────────────────────────────

    pub async fn list_nodes(&self) -> Result<Vec<NodeSummary>, Error> {
        self.get("nodes").await
    }

    pub async fn node_status(&self, node: &str) -> Result<NodeStatus, Error> {
        self.get(&format!("nodes/{node}/status")).await
    }

    async fn put<T: serde::de::DeserializeOwned>(&self, path: &str, body: impl serde::Serialize) -> Result<T, Error> {
        let resp = self
            .http
            .put(self.url(path))
            .header(AUTHORIZATION, self.auth_header.clone())
            .json(&body)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        let resp = self
            .http
            .delete(self.url(path))
            .header(AUTHORIZATION, self.auth_header.clone())
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    // ── Cluster helpers ───────────────────────────────────────────────

    /// Get the next available VM ID from the cluster.
    /// GET /cluster/nextid
    pub async fn cluster_next_vmid(&self) -> Result<u64, Error> {
        let data: String = self.get("cluster/nextid").await?;
        data.parse()
            .map_err(|e| Error::Config(format!("invalid next VMID: {e}")))
    }

    // ── VMs (QEMU) ───────────────────────────────────────────────────

    pub async fn list_vms(&self, node: &str) -> Result<Vec<VmSummary>, Error> {
        self.get(&format!("nodes/{node}/qemu")).await
    }

    pub async fn vm_status(&self, node: &str, vmid: u64) -> Result<VmStatus, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/status/current")).await
    }

    pub async fn vm_config(&self, node: &str, vmid: u64) -> Result<VmConfig, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/config")).await
    }

    pub async fn vm_start(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/qemu/{vmid}/status/start")).await
    }

    pub async fn vm_shutdown(&self, node: &str, vmid: u64, timeout: Option<u64>) -> Result<TaskResponse, Error> {
        let opts = VmShutdownOptions {
            node: node.to_string(),
            vmid,
            timeout,
            keep_active: None,
        };
        self.post(&format!("nodes/{node}/qemu/{vmid}/status/shutdown"), opts)
            .await
    }

    pub async fn vm_stop(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/qemu/{vmid}/status/stop")).await
    }

    /// Reset (force-restart) a VM.
    pub async fn vm_reset(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/qemu/{vmid}/status/reset")).await
    }

    /// Suspend (hibernate) a VM.
    pub async fn vm_suspend(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/qemu/{vmid}/status/suspend"))
            .await
    }

    /// Convert a VM to a template.
    pub async fn vm_template(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/qemu/{vmid}/template")).await
    }

    pub async fn vm_migrate(&self, node: &str, vmid: u64, target: &str) -> Result<TaskResponse, Error> {
        let opts = VmMigrateOptions {
            node: node.to_string(),
            vmid,
            target: target.to_string(),
            online: Some(1),
        };
        self.post(&format!("nodes/{node}/qemu/{vmid}/migrate"), opts).await
    }

    // ── VM Lifecycle ─────────────────────────────────────────────────

    /// Create a VM (from scratch or from a template).
    /// POST /nodes/{node}/qemu
    pub async fn vm_create(&self, node: &str, opts: &VmCreateOptions) -> Result<TaskResponse, Error> {
        self.post(&format!("nodes/{node}/qemu"), opts).await
    }

    /// Delete a VM.
    /// DELETE /nodes/{node}/qemu/{vmid}
    pub async fn vm_delete(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.delete(&format!("nodes/{node}/qemu/{vmid}")).await
    }

    /// Clone a VM.
    /// POST /nodes/{node}/qemu/{vmid}/clone
    pub async fn vm_clone(&self, node: &str, vmid: u64, opts: &VmCloneOptions) -> Result<TaskResponse, Error> {
        self.post(&format!("nodes/{node}/qemu/{vmid}/clone"), opts).await
    }

    /// Resize a VM disk.
    /// PUT /nodes/{node}/qemu/{vmid}/resize
    pub async fn vm_resize_disk(
        &self,
        node: &str,
        vmid: u64,
        opts: &VmResizeDiskOptions,
    ) -> Result<TaskResponse, Error> {
        self.put(&format!("nodes/{node}/qemu/{vmid}/resize"), opts).await
    }

    /// Update VM configuration (CPU cores, memory, etc.).
    /// PUT /nodes/{node}/qemu/{vmid}/config
    pub async fn vm_config_set(
        &self,
        node: &str,
        vmid: u64,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value, Error> {
        self.put(&format!("nodes/{node}/qemu/{vmid}/config"), config).await
    }

    /// List snapshots for a VM.
    /// GET /nodes/{node}/qemu/{vmid}/snapshot
    pub async fn list_snapshots(&self, node: &str, vmid: u64) -> Result<Vec<SnapshotListItem>, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/snapshot")).await
    }

    /// Create a VM snapshot.
    /// POST /nodes/{node}/qemu/{vmid}/snapshot
    pub async fn snapshot_create(
        &self,
        node: &str,
        vmid: u64,
        opts: &SnapshotCreateOptions,
    ) -> Result<TaskResponse, Error> {
        self.post(&format!("nodes/{node}/qemu/{vmid}/snapshot"), opts).await
    }

    /// Roll back a VM to a snapshot.
    /// POST /nodes/{node}/qemu/{vmid}/snapshot/{snapname}/rollback
    pub async fn snapshot_rollback(&self, node: &str, vmid: u64, snapname: &str) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/qemu/{vmid}/snapshot/{snapname}/rollback"))
            .await
    }

    /// Delete a VM snapshot.
    /// DELETE /nodes/{node}/qemu/{vmid}/snapshot/{snapname}
    pub async fn snapshot_delete(&self, node: &str, vmid: u64, snapname: &str) -> Result<TaskResponse, Error> {
        self.delete(&format!("nodes/{node}/qemu/{vmid}/snapshot/{snapname}"))
            .await
    }

    // ── Containers (LXC) ─────────────────────────────────────────────

    pub async fn list_containers(&self, node: &str) -> Result<Vec<LxcSummary>, Error> {
        self.get(&format!("nodes/{node}/lxc")).await
    }

    pub async fn container_status(&self, node: &str, vmid: u64) -> Result<LxcStatus, Error> {
        self.get(&format!("nodes/{node}/lxc/{vmid}/status/current")).await
    }

    pub async fn container_create(&self, node: &str, opts: &LxcCreateOptions) -> Result<TaskResponse, Error> {
        self.post(&format!("nodes/{node}/lxc"), opts).await
    }

    pub async fn container_delete(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.delete(&format!("nodes/{node}/lxc/{vmid}")).await
    }

    pub async fn container_start(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/lxc/{vmid}/status/start")).await
    }

    pub async fn container_stop(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/lxc/{vmid}/status/stop")).await
    }

    pub async fn container_shutdown(&self, node: &str, vmid: u64, timeout: Option<u64>) -> Result<TaskResponse, Error> {
        let opts = LxcShutdownOptions {
            node: node.to_string(),
            vmid,
            timeout,
        };
        self.post(&format!("nodes/{node}/lxc/{vmid}/status/shutdown"), opts)
            .await
    }

    pub async fn container_clone(&self, node: &str, vmid: u64, opts: &LxcCloneOptions) -> Result<TaskResponse, Error> {
        self.post(&format!("nodes/{node}/lxc/{vmid}/clone"), opts).await
    }

    pub async fn container_resize(
        &self,
        node: &str,
        vmid: u64,
        opts: &LxcResizeOptions,
    ) -> Result<TaskResponse, Error> {
        self.put(&format!("nodes/{node}/lxc/{vmid}/resize"), opts).await
    }

    // ── Pools ────────────────────────────────────────────────────────

    pub async fn list_pools(&self) -> Result<Vec<PoolSummary>, Error> {
        self.get("pools").await
    }

    pub async fn pool_status(&self, poolid: &str) -> Result<PoolDetail, Error> {
        self.get(&format!("pools/{poolid}")).await
    }

    pub async fn pool_create(&self, opts: &PoolCreateOptions) -> Result<String, Error> {
        self.post("pools", opts).await
    }

    // ── Users & ACL ──────────────────────────────────────────────────

    pub async fn list_users(&self) -> Result<Vec<UserSummary>, Error> {
        self.get("access/users").await
    }

    pub async fn user_create(&self, opts: &UserCreateOptions) -> Result<String, Error> {
        self.post("access/users", opts).await
    }

    pub async fn list_acls(&self) -> Result<Vec<AclEntry>, Error> {
        self.get("access/acl").await
    }

    pub async fn acl_update(&self, opts: &AclUpdateOptions) -> Result<String, Error> {
        self.put("access/acl", opts).await
    }

    // ── Firewall ─────────────────────────────────────────────────────

    pub async fn fw_rules(&self, node: &str, vmid: u64) -> Result<Vec<FwRule>, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/firewall/rules")).await
    }

    pub async fn fw_add_rule(&self, node: &str, vmid: u64, opts: &FwRuleOptions) -> Result<TaskResponse, Error> {
        self.post(&format!("nodes/{node}/qemu/{vmid}/firewall/rules"), opts)
            .await
    }

    pub async fn fw_log(&self, node: &str, vmid: u64) -> Result<Vec<FwLogEntry>, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/firewall/log")).await
    }

    // ── QEMU Agent ───────────────────────────────────────────────────

    pub async fn vm_agent_info(&self, node: &str, vmid: u64) -> Result<AgentInfo, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/agent/info")).await
    }

    pub async fn vm_agent_network(&self, node: &str, vmid: u64) -> Result<Vec<AgentNetworkInterface>, Error> {
        self.get(&format!("nodes/{node}/qemu/{vmid}/agent/network-get-interfaces"))
            .await
    }

    pub async fn vm_agent_exec(
        &self,
        node: &str,
        vmid: u64,
        opts: &AgentExecOptions,
    ) -> Result<AgentExecResult, Error> {
        self.post(&format!("nodes/{node}/qemu/{vmid}/agent/exec"), opts).await
    }

    // ── Storage ──────────────────────────────────────────────────────

    /// List all storages configured on the cluster.
    pub async fn list_storage(&self) -> Result<Vec<StorageSummary>, Error> {
        self.get("storage").await
    }

    /// List storages available on a specific node (includes usage).
    pub async fn node_storage(&self, node: &str) -> Result<Vec<StorageSummary>, Error> {
        self.get(&format!("nodes/{node}/storage")).await
    }

    /// Get detailed status of a specific storage on a node.
    pub async fn storage_status(&self, node: &str, storage: &str) -> Result<StorageSummary, Error> {
        self.get(&format!("nodes/{node}/storage/{storage}/status")).await
    }

    /// List content (volumes/VMs/backups) in a storage.
    pub async fn storage_content(&self, storage: &str) -> Result<Vec<StorageContent>, Error> {
        self.get(&format!("storage/{storage}/content")).await
    }

    // ── Backup ───────────────────────────────────────────────────────

    pub async fn list_backups(&self) -> Result<Vec<BackupJob>, Error> {
        self.get("cluster/backup").await
    }

    pub async fn create_backup(
        &self,
        node: &str,
        vmid: &str,
        storage: &str,
        mode: Option<&str>,
        compress: Option<&str>,
    ) -> Result<TaskResponse, Error> {
        let opts = BackupCreateOptions {
            vmid: vmid.to_string(),
            storage: storage.to_string(),
            mode: mode.map(|m| m.to_string()),
            compress: compress.map(|c| c.to_string()),
        };
        self.post(&format!("nodes/{node}/vzdump"), opts).await
    }

    // ── Task tracking ─────────────────────────────────────────────────

    pub async fn task_status(&self, node: &str, upid: &str) -> Result<TaskStatus, Error> {
        self.get(&format!("nodes/{node}/tasks/{upid}/status")).await
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let client = ProxmoxClient::with_api_token("https://pve1:8006", "root@pam!test", "test-secret", true)
            .expect("should build client");
        assert_eq!(client.url("version"), "https://pve1:8006/api2/json/version");
        assert_eq!(client.url("/version"), "https://pve1:8006/api2/json/version");
        assert_eq!(
            client.url("nodes/pve1/qemu"),
            "https://pve1:8006/api2/json/nodes/pve1/qemu"
        );
    }

    #[test]
    fn test_url_with_trailing_slash_in_base() {
        let client = ProxmoxClient::with_api_token("https://pve1:8006/", "root@pam!test", "test-secret", true)
            .expect("should build client");
        assert_eq!(client.url("version"), "https://pve1:8006/api2/json/version");
    }

    #[test]
    fn test_with_api_token_sets_auth_header() {
        let client = ProxmoxClient::with_api_token("https://pve1:8006", "root@pam!discord-bot", "uuid-secret", true)
            .expect("should build client");
        let expected = "PVEAPIToken=root@pam!discord-bot=uuid-secret";
        assert_eq!(
            client.auth_header,
            HeaderValue::from_static(expected),
            "auth header should be PVEAPIToken=<token_id>=<token_secret>"
        );
    }

    #[test]
    fn test_with_api_token_rejects_invalid_token() {
        let result = ProxmoxClient::with_api_token("https://pve1:8006", "root@pam!test", "\0invalid", true);
        assert!(result.is_err(), "token with null byte should fail");
    }

    #[test]
    fn test_with_ticket_creates_empty_auth() {
        let client = ProxmoxClient::with_ticket("https://pve1:8006", true).expect("should build ticket client");
        assert_eq!(client.auth_header, HeaderValue::from_static(""));
    }
}
