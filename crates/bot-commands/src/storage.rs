use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

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

    let mut desc = String::new();
    for s in &storages {
        let status_icon = match s.status.as_deref() {
            Some("available") => "🟢",
            _ => "🔴",
        };
        let usage = s
            .used_fraction
            .map(|f| format!("{:.1}%", f * 100.0))
            .unwrap_or_default();
        let active = s
            .active
            .map(|a| if a == 1 { "active" } else { "inactive" })
            .unwrap_or("?");
        desc.push_str(&format!(
            "{status_icon} **{name}** — {kind} | {content} | {usage} used | {active}\n",
            name = s.storage,
            kind = s.kind.as_deref().unwrap_or("?"),
            content = s.content.as_deref().unwrap_or("?"),
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Storage Pools")
        .description(if desc.is_empty() {
            "No storage pools found.".into()
        } else {
            desc
        })
        .color(0x00aaff);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Show detailed status of a specific storage pool
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>, #[description = "Storage pool name"] pool: String) -> Result<(), Error> {
    ctx.defer().await?;

    let nodes = ctx.data().proxmox.list_nodes().await?;
    let mut desc = String::new();

    for n in &nodes {
        let storages = ctx.data().proxmox.node_storage(&n.node).await?;
        if let Some(s) = storages.iter().find(|s| s.storage == pool) {
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
                node = n.node,
            ));
        }
    }

    if desc.is_empty() {
        desc = format!("Pool `{pool}` not found on any node.");
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Storage Pool: {pool}"))
        .description(desc)
        .color(0x00aaff);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
