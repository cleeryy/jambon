use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Show the interactive main menu with category buttons.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn menu(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let proxmox = &ctx.data().proxmox;

    // Fetch diagnostic data in parallel.
    let (nodes_res, resources_res, storages_res) = tokio::join!(
        proxmox.cluster_status(),
        proxmox.cluster_resources(),
        proxmox.list_storage(),
    );

    // Graceful degradation: each block is independent.
    let node_count = nodes_res
        .as_ref()
        .map(|nodes| {
            let online = nodes.iter().filter(|n| n.status.as_deref() == Some("online")).count();
            format!("{online}/{}", nodes.len())
        })
        .unwrap_or_else(|_| "?/?".into());

    let running_vms = resources_res
        .as_ref()
        .map(|resources| {
            resources
                .iter()
                .filter(|r| r.kind == "qemu" && r.status.as_deref() == Some("running"))
                .count()
                .to_string()
        })
        .unwrap_or_else(|_| "?".into());

    let storage_available = storages_res
        .as_ref()
        .map(|s| {
            let avail = s.iter().filter(|st| st.status.as_deref() == Some("available")).count();
            format!("{avail}/{}", s.len())
        })
        .unwrap_or_else(|_| "?/?".into());

    let embed = serenity::CreateEmbed::new()
        .title("🏠 Jambon Dashboard")
        .field("🌐 Nodes", node_count, true)
        .field("🖥️ Running VMs", running_vms, true)
        .field("💾 Storage Available", storage_available, true)
        .color(crate::colors::COLOR_INFO);

    let buttons = vec![
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("menu:vm")
                .label("🖥️ Virtual Machines")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("menu:container")
                .label("📦 Containers")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("menu:storage")
                .label("💾 Storage")
                .style(serenity::ButtonStyle::Primary),
        ]),
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("menu:cluster")
                .label("🌐 Cluster")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("menu:node")
                .label("🖥️ Nodes")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("nav:close")
                .label("❌ Close")
                .style(serenity::ButtonStyle::Danger),
        ]),
    ];

    ctx.send(CreateReply::default().embed(embed).components(buttons).ephemeral(true))
        .await?;

    Ok(())
}
