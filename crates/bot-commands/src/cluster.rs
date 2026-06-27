use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Show Proxmox cluster status and resources
#[poise::command(
    slash_command,
    subcommands("status", "resources"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn cluster(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// Show cluster status summary
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let nodes = ctx.data().proxmox.cluster_status().await?;
    let resources = ctx.data().proxmox.cluster_resources().await?;
    let (embed, components) = crate::interactions::build_cluster_status_embed(&nodes, &resources);
    ctx.send(CreateReply::default().embed(embed).components(components))
        .await?;
    Ok(())
}

/// List all resources across the cluster
#[poise::command(slash_command)]
pub async fn resources(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let resources = ctx.data().proxmox.cluster_resources().await?;
    let mut vms = String::new();
    let mut storage = String::new();

    for r in &resources {
        match r.kind.as_str() {
            "qemu" => {
                let icon = match r.status.as_deref() {
                    Some("running") => "🟢",
                    _ => "🔴",
                };
                vms.push_str(&format!(
                    "{icon} VM {vmid} — {name} on {node}\n",
                    vmid = r.vmid.unwrap_or(0),
                    name = r.name.as_deref().unwrap_or("unnamed"),
                    node = r.node.as_deref().unwrap_or("?"),
                ));
            }
            "storage" => {
                storage.push_str(&format!(
                    "📦 {name} — {used:.1}/{max:.1} GB\n",
                    name = r.storage.as_deref().unwrap_or("?"),
                    used = r.disk.unwrap_or(0) as f64 / 1_073_741_824.0,
                    max = r.maxdisk.unwrap_or(1) as f64 / 1_073_741_824.0,
                ));
            }
            _ => {}
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title("Cluster Resources")
        .field(
            "Virtual Machines",
            if vms.is_empty() { "None".into() } else { vms },
            false,
        )
        .field(
            "Storage",
            if storage.is_empty() { "None".into() } else { storage },
            false,
        )
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
