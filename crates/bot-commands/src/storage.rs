use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{node_utils, Context, Error};

/// Manage Proxmox VE storage pools
#[poise::command(
    slash_command,
    subcommands("list", "status"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn storage(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all storage pools with usage
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let storages = ctx.data().proxmox.list_storage().await?;
    const PAGE_SIZE: usize = 5;
    let total_pages = (storages.len().max(1) - 1) / PAGE_SIZE + 1;
    let (embed, components) = crate::interactions::build_storage_list_embed(&storages, 0, total_pages);
    ctx.send(CreateReply::default().embed(embed).components(components).ephemeral(true)).await?;
    Ok(())
}

/// Show detailed status of a specific storage pool
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>, #[description = "Storage pool name"] pool: String) -> Result<(), Error> {
    ctx.defer().await?;

    let proxmox = &ctx.data().proxmox;
    let results = node_utils::try_for_each_node(proxmox, |node| {
        let proxmox = proxmox.clone();
        async move {
            let storages = proxmox.node_storage(&node).await?;
            Ok::<_, Error>(storages)
        }
    })
    .await?;

    let mut desc = String::new();

    for (node, storages) in &results {
        if let Some(s) = storages.iter().find(|s| s.storage == *pool) {
            let used_gb = s.used.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
            let avail_gb = s.avail.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
            let usage = s
                .used_fraction
                .map(|f| format!("{:.1}%", f * 100.0))
                .unwrap_or_default();
            let status_icon = match s.status.as_deref() {
                Some("available") => "🟢",
                _ => "🔴",
            };
            desc.push_str(&format!(
                "{status_icon} **{node}**: {used_gb:.1} GB / {avail_gb:.1} GB ({usage})\n",
            ));
        }
    }

    if desc.is_empty() {
        if results.is_empty() {
            desc = "No reachable nodes in cluster.".into();
        } else {
            desc = format!("Pool `{pool}` not found on any reachable node.");
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Storage Pool: {pool}"))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
