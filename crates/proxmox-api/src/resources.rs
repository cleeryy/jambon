use crate::{models::ClusterResource, Error, ProxmoxClient};

/// Builder-style interface for the cluster resources endpoint.
pub struct Resources<'a> {
    pub(super) client: &'a ProxmoxClient,
}

impl Resources<'_> {
    /// Fetch all resources across the cluster.
    pub async fn list(&self) -> Result<Vec<ClusterResource>, Error> {
        self.client.cluster_resources().await
    }

    /// Fetch only VMs.
    pub async fn vms(&self) -> Result<Vec<ClusterResource>, Error> {
        let all = self.list().await?;
        Ok(all.into_iter().filter(|r| r.kind == "qemu").collect())
    }

    /// Fetch only containers.
    pub async fn containers(&self) -> Result<Vec<ClusterResource>, Error> {
        let all = self.list().await?;
        Ok(all.into_iter().filter(|r| r.kind == "lxc").collect())
    }

    /// Fetch only nodes.
    pub async fn nodes(&self) -> Result<Vec<ClusterResource>, Error> {
        let all = self.list().await?;
        Ok(all.into_iter().filter(|r| r.kind == "node").collect())
    }

    /// Fetch only storage items.
    pub async fn storage(&self) -> Result<Vec<ClusterResource>, Error> {
        let all = self.list().await?;
        Ok(all.into_iter().filter(|r| r.kind == "storage").collect())
    }
}
