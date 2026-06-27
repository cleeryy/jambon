//! Utilities for iterating over cluster nodes with graceful error handling.
//!
//! When a cluster node is offline or unreachable, per-node API calls fail and
//! would abort the entire command. These helpers catch node-level errors,
//! log them, and continue processing remaining nodes so the command can still
//! produce partial results.

use std::future::Future;

use jambon_proxmox_api::ProxmoxClient;
use tracing::warn;

use crate::Error;

/// Call `f` for each online node, collecting successful results.
///
/// The closure receives an owned `String` so it can be moved into
/// async blocks without lifetime issues.
/// Nodes that fail are logged and skipped silently so the command
/// can still return partial data.
pub async fn try_for_each_node<T, Fut>(
    proxmox: &ProxmoxClient,
    f: impl Fn(String) -> Fut,
) -> Result<Vec<(String, T)>, Error>
where
    Fut: Future<Output = Result<T, Error>>,
{
    let nodes = proxmox.list_nodes().await?;
    let mut results = Vec::new();

    for node in &nodes {
        match f(node.node.clone()).await {
            Ok(val) => results.push((node.node.clone(), val)),
            Err(e) => {
                warn!("Skipping unreachable node {}: {e}", node.node);
                continue;
            }
        }
    }

    Ok(results)
}
