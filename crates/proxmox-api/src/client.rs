use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
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
        let mut hb = HttpClient::builder()
            .danger_accept_invalid_certs(accept_invalid_certs)
            .timeout(std::time::Duration::from_secs(30));
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        hb = hb.default_headers(headers);
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
        format!("{}/api2/json/{path}", self.base_url)
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

        if status == reqwest::StatusCode::NO_CONTENT {
            // Some delete/stop operations return 204 — nothing to parse.
            // Return a default if T supports it, otherwise error.
            // For now we just attempt to deserialize from empty body.
            let body = resp.text().await.unwrap_or_default();
            serde_json::from_str(&body).map_err(Error::Json)
        } else {
            let body = resp.text().await?;
            serde_json::from_str(&body).map_err(Error::Json)
        }
    }

    // ── Cluster ──────────────────────────────────────────────────────

    pub async fn cluster_status(&self) -> Result<Vec<NodeSummary>, Error> {
        self.get("cluster/status").await
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

    pub async fn vm_migrate(&self, node: &str, vmid: u64, target: &str) -> Result<TaskResponse, Error> {
        let opts = VmMigrateOptions {
            node: node.to_string(),
            vmid,
            target: target.to_string(),
            online: Some(1),
        };
        self.post(&format!("nodes/{node}/qemu/{vmid}/migrate"), opts).await
    }

    // ── Containers (LXC) ─────────────────────────────────────────────

    pub async fn list_containers(&self, node: &str) -> Result<Vec<LxcSummary>, Error> {
        self.get(&format!("nodes/{node}/lxc")).await
    }

    pub async fn container_start(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/lxc/{vmid}/status/start")).await
    }

    pub async fn container_stop(&self, node: &str, vmid: u64) -> Result<TaskResponse, Error> {
        self.post_empty(&format!("nodes/{node}/lxc/{vmid}/status/stop")).await
    }

    // ── Storage ──────────────────────────────────────────────────────

    pub async fn list_storage(&self) -> Result<Vec<StorageSummary>, Error> {
        self.get("storage").await
    }

    pub async fn storage_content(&self, storage: &str) -> Result<Vec<StorageContent>, Error> {
        self.get(&format!("storage/{storage}/content")).await
    }

    // ── Backup ───────────────────────────────────────────────────────

    pub async fn list_backups(&self) -> Result<Vec<BackupJob>, Error> {
        self.get("cluster/backup").await
    }

    // ── Task tracking ─────────────────────────────────────────────────

    pub async fn task_status(&self, node: &str, upid: &str) -> Result<TaskStatus, Error> {
        self.get(&format!("nodes/{node}/tasks/{upid}/status")).await
    }
}
