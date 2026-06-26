use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::audit::AuditEntry;
use crate::{Context, Error};

fn audit_entry(user: &str, command: &str, details: String) -> AuditEntry {
    AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: user.to_string(),
        command: command.to_string(),
        details,
    }
}

/// Manage Proxmox VE nodes
#[poise::command(
    slash_command,
    subcommands("list", "status", "drain", "fence"),
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
            Some("online") => "\u{1f7e2}",
            Some("offline") => "\u{1f534}",
            _ => "\u{26aa}",
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
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Get detailed status of a specific node
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>, #[description = "Node name"] node_name: String) -> Result<(), Error> {
    ctx.defer().await?;

    let status = ctx.data().proxmox.node_status(&node_name).await?;

    let color = if status.cpu > 0.9 {
        crate::colors::COLOR_ERROR
    } else {
        crate::colors::COLOR_SUCCESS
    };

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

/// Manage node drain operations (migrate all VMs off a node)
#[poise::command(
    slash_command,
    subcommands("drain_start", "drain_status", "drain_cancel"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn drain(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("drain subcommand_required is set")
}

/// Migrate all VMs off a node
#[poise::command(slash_command, rename = "start")]
pub async fn drain_start(ctx: Context<'_>, #[description = "Node to drain"] node_name: String) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let vms = ctx.data().proxmox.list_vms(&node_name).await?;
    let running_vms: Vec<_> = vms.iter().filter(|v| v.status == "running").collect();

    if running_vms.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title("Node Drain")
            .description(format!("No running VMs on **{node_name}** to migrate."))
            .color(crate::colors::COLOR_INFO);
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let all_nodes = ctx.data().proxmox.list_nodes().await?;
    let target_nodes: Vec<&str> = all_nodes
        .iter()
        .filter(|n| n.node != node_name && n.status.as_deref() == Some("online"))
        .map(|n| n.node.as_str())
        .collect();

    if target_nodes.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title("Node Drain Failed")
            .description(format!(
                "No online target nodes available to migrate VMs from **{node_name}**."
            ))
            .color(crate::colors::COLOR_ERROR);
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    ctx.data().scheduler.add_drain_op(node_name.clone(), running_vms.len());

    let mut completed = 0usize;
    let mut failed: Vec<u64> = Vec::new();

    let progress_embed = serenity::CreateEmbed::new()
        .title(format!("Draining Node: {node_name}"))
        .description(format!(
            "Migrating {} VM(s) to {} target node(s)...",
            running_vms.len(),
            target_nodes.len()
        ))
        .field("Completed", format!("0 / {}", running_vms.len()), true)
        .field("Failed", "0", true)
        .color(crate::colors::COLOR_WARNING);

    let reply = ctx.send(CreateReply::default().embed(progress_embed)).await?;

    for (i, vm) in running_vms.iter().enumerate() {
        let target = target_nodes[i % target_nodes.len()];

        match ctx.data().proxmox.vm_migrate(&node_name, vm.vmid, target).await {
            Ok(task) => {
                completed += 1;
                ctx.data().audit_log.push(audit_entry(
                    ctx.author().name.as_ref(),
                    "node drain",
                    format!("VM {} from {} to {} (task: {})", vm.vmid, node_name, target, task.data),
                ));
            }
            Err(e) => {
                failed.push(vm.vmid);
                tracing::warn!("Drain: failed to migrate VM {}: {e}", vm.vmid);
            }
        }

        if (i + 1) % 3 == 0 || i + 1 == running_vms.len() {
            let progress_embed = serenity::CreateEmbed::new()
                .title(format!("Draining Node: {node_name}"))
                .description(format!(
                    "Migrating {} VM(s) — {} target(s) available.",
                    running_vms.len(),
                    target_nodes.len()
                ))
                .field("Completed", format!("{completed} / {}", running_vms.len()), true)
                .field("Failed", failed.len().to_string(), true)
                .color(if failed.is_empty() {
                    crate::colors::COLOR_WARNING
                } else {
                    crate::colors::COLOR_ERROR
                });
            let _ = reply.edit(ctx, CreateReply::default().embed(progress_embed)).await;
        }
    }

    let final_color = if failed.is_empty() {
        crate::colors::COLOR_SUCCESS
    } else {
        crate::colors::COLOR_ERROR
    };

    let mut desc = format!(
        "Drain of **{node_name}** completed.\n**{completed}** / **{}** VMs migrated successfully.",
        running_vms.len()
    );
    if !failed.is_empty() {
        desc.push_str(&format!(
            "\n\nFailed to migrate: VM {}",
            failed.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ")
        ));
    }

    let final_embed = serenity::CreateEmbed::new()
        .title(format!("Node Drain Complete: {node_name}"))
        .description(desc)
        .color(final_color);

    let _ = reply.edit(ctx, CreateReply::default().embed(final_embed)).await;
    Ok(())
}

/// Show ongoing drain operations
#[poise::command(slash_command, rename = "status")]
pub async fn drain_status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let ops = ctx.data().scheduler.get_drain_ops();

    if ops.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title("Drain Status")
            .description("No drain operations in progress or history.")
            .color(crate::colors::COLOR_INFO);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for op in &ops {
        let status_icon = if op.cancelled {
            "\u{23f9}\u{fe0f}"
        } else if op.completed_vms >= op.total_vms {
            "\u{2705}"
        } else {
            "\u{1f504}"
        };
        desc.push_str(&format!(
            "{status_icon} **{node}** — {completed}/{total} VMs migrated{extra}\n",
            node = op.node,
            completed = op.completed_vms,
            total = op.total_vms,
            extra = if op.failed_vms.is_empty() {
                String::new()
            } else {
                format!(
                    " (failed: VM {})",
                    op.failed_vms
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Drain Operations")
        .description(desc)
        .color(crate::colors::COLOR_INFO);
    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Cancel an active drain operation
#[poise::command(slash_command, rename = "cancel")]
pub async fn drain_cancel(
    ctx: Context<'_>,
    #[description = "Node to cancel drain for"] node_name: String,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if ctx.data().scheduler.cancel_drain(&node_name) {
        let embed = serenity::CreateEmbed::new()
            .title("Drain Cancelled")
            .description(format!("Drain operation for **{node_name}** has been cancelled."))
            .color(crate::colors::COLOR_WARNING);
        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("No Drain Found")
            .description(format!("No active drain operation for **{node_name}**."))
            .color(crate::colors::COLOR_INFO);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    }
    Ok(())
}

/// Manage node fencing
#[poise::command(
    slash_command,
    subcommands("fence_add", "fence_status"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn fence(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("fence subcommand_required is set")
}

/// Fence a node (mark it as isolated)
#[poise::command(slash_command, rename = "add")]
pub async fn fence_add(ctx: Context<'_>, #[description = "Node to fence"] node_name: String) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    ctx.data()
        .scheduler
        .add_fence(node_name.clone(), format!("manual:{}", ctx.author().name));

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "node fence",
        format!("fenced node {node_name}"),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Node Fenced")
        .description(format!("Node **{node_name}** has been fenced."))
        .color(crate::colors::COLOR_WARNING);
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show fenced nodes
#[poise::command(slash_command, rename = "status")]
pub async fn fence_status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let fences = ctx.data().scheduler.get_fenced_nodes();

    if fences.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title("Fence Status")
            .description("No nodes are currently fenced.")
            .color(crate::colors::COLOR_INFO);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for f in &fences {
        let ts = chrono::DateTime::from_timestamp(f.fenced_at as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| "unknown".to_string());
        desc.push_str(&format!(
            "\u{1f512} **{node}** — fenced at {ts} by {by}\n",
            node = f.node,
            ts = ts,
            by = f.fenced_by,
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Fenced Nodes ({})", fences.len()))
        .description(desc)
        .color(crate::colors::COLOR_WARNING);
    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_nodes_days() {
        assert_eq!(format_uptime(172800), "2d 0h 0m");
    }

    #[test]
    fn test_format_uptime_nodes_hours() {
        assert_eq!(format_uptime(7200), "2h 0m");
    }

    #[test]
    fn test_format_uptime_nodes_minutes() {
        assert_eq!(format_uptime(300), "5m");
    }

    #[test]
    fn test_format_uptime_nodes_zero() {
        assert_eq!(format_uptime(0), "0m");
    }
}
