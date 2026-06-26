use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE nodes
#[poise::command(
    slash_command,
    subcommands("list", "status"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn node(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all nodes in the cluster
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let nodes = ctx.data().proxmox.list_nodes().await?;
    let mut desc = String::new();

    for n in &nodes {
        let icon = match n.status.as_deref() {
            Some("online") => "🟢",
            Some("offline") => "🔴",
            _ => "⚪",
        };
        desc.push_str(&format!(
            "{icon} **{name}** — CPU: {cpu:.1}%, Mem: {mem:.1}/{max:.1} GB, Uptime: {up}\n",
            name = n.node,
            cpu = n.cpu.unwrap_or(0.0) * 100.0,
            mem = n.mem.unwrap_or(0) as f64 / 1_073_741_824.0,
            max = n.maxmem.unwrap_or(1) as f64 / 1_073_741_824.0,
            up = format_uptime(n.uptime.unwrap_or(0)),
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Proxmox Cluster Nodes")
        .description(desc)
        .color(0x00aaff);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Get detailed status of a specific node
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>, #[description = "Node name"] node_name: String) -> Result<(), Error> {
    ctx.defer().await?;

    let status = ctx.data().proxmox.node_status(&node_name).await?;

    let color = if status.cpu > 0.9 { 0xff0000 } else { 0x00ff00 };

    let embed = serenity::CreateEmbed::new()
        .title(format!("Node: {node_name}"))
        .field("CPU", format!("{:.1}%", status.cpu * 100.0), true)
        .field(
            "Memory",
            format!(
                "{:.1} GB / {:.1} GB",
                status.memory.used as f64 / 1_073_741_824.0,
                status.memory.total as f64 / 1_073_741_824.0,
            ),
            true,
        )
        .field(
            "Swap",
            format!(
                "{:.1} GB / {:.1} GB",
                status.swap.used as f64 / 1_073_741_824.0,
                status.swap.total as f64 / 1_073_741_824.0,
            ),
            true,
        )
        .field("Uptime", format_uptime(status.uptime), true)
        .field("Kernel", status.kversion.as_deref().unwrap_or("unknown"), true)
        .color(color);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
