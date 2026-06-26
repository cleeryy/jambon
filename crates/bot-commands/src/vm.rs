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

/// Manage Proxmox VE virtual machines
#[poise::command(
    slash_command,
    subcommands("list", "status", "start", "stop", "shutdown", "migrate"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn vm(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all VMs on a specific node (or all nodes)
#[poise::command(slash_command)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "Node name (optional — shows all nodes if omitted)"] node: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let node_label = node.as_deref().unwrap_or("all nodes").to_string();

    let vms = if let Some(ref node_name) = node {
        ctx.data().proxmox.list_vms(node_name).await?
    } else {
        let resources = ctx.data().proxmox.resources().vms().await?;
        let mut all = Vec::new();
        for r in &resources {
            if let (Some(rnode), Some(vmid)) = (&r.node, r.vmid) {
                if let Ok(vm) = ctx.data().proxmox.vm_status(rnode, vmid).await {
                    all.push(vm);
                }
            }
        }
        // Build VmSummary-compatible display
        return show_cluster_vms(ctx, &resources).await;
    };

    let mut desc = String::new();
    for vm in &vms {
        let status_icon = match vm.status.as_str() {
            "running" => "🟢",
            "stopped" => "🔴",
            _ => "⚪",
        };
        desc.push_str(&format!(
            "{status_icon} **VM {vmid}** — {} ({status})\n",
            vm.name.as_deref().unwrap_or("unnamed"),
            vmid = vm.vmid,
            status = vm.status,
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("VMs on {node_label}"))
        .description(if desc.is_empty() { "No VMs found.".into() } else { desc })
        .color(0x00aaff);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

async fn show_cluster_vms(ctx: Context<'_>, resources: &[jambon_proxmox_api::ClusterResource]) -> Result<(), Error> {
    let mut desc = String::new();
    for r in resources {
        let status_icon = match r.status.as_deref() {
            Some("running") => "🟢",
            Some("stopped") => "🔴",
            _ => "⚪",
        };
        desc.push_str(&format!(
            "{status_icon} **VM {vmid}** — {name} on {node} ({status})\n",
            vmid = r.vmid.unwrap_or(0),
            name = r.name.as_deref().unwrap_or("unnamed"),
            node = r.node.as_deref().unwrap_or("?"),
            status = r.status.as_deref().unwrap_or("?"),
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Cluster VMs")
        .description(if desc.is_empty() { "No VMs found.".into() } else { desc })
        .color(0x00aaff);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Get detailed status of a VM
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let status = ctx.data().proxmox.vm_status(&node, vmid).await?;

    let color = match status.status.as_str() {
        "running" => 0x00ff00,
        "stopped" => 0xff0000,
        _ => 0xffaa00,
    };

    let embed = serenity::CreateEmbed::new()
        .title(format!("VM {vmid} — {}", status.name.as_deref().unwrap_or("unnamed")))
        .field("Status", &status.status, true)
        .field("Node", &node, true)
        .field("CPU", format!("{:.1}%", status.cpu.unwrap_or(0.0) * 100.0), true)
        .field(
            "Memory",
            format!(
                "{:.1} GB / {:.1} GB",
                status.mem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
                status.maxmem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
            ),
            true,
        )
        .field("Uptime", format_uptime(status.uptime.unwrap_or(0)), true)
        .color(color);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Start a VM
#[poise::command(slash_command)]
pub async fn start(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.vm_start(&node, vmid).await?;
    let entry = audit_entry(
        ctx.author().name.as_ref(),
        "vm start",
        format!("VM {vmid} on {node} (task: {})", task.data),
    );
    ctx.data().audit_log.push(entry);
    ctx.say(format!("✅ VM {vmid} is starting (task: {}).", task.data))
        .await?;
    Ok(())
}

/// Force-stop a VM
#[poise::command(slash_command)]
pub async fn stop(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.vm_stop(&node, vmid).await?;
    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm stop",
        format!("VM {vmid} on {node} (task: {})", task.data),
    ));
    ctx.say(format!("⏹️ VM {vmid} stop requested (task: {}).", task.data))
        .await?;
    Ok(())
}

/// Gracefully shutdown a VM
#[poise::command(slash_command)]
pub async fn shutdown(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Timeout in seconds before force-stop"] timeout: Option<u64>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.vm_shutdown(&node, vmid, timeout).await?;
    let entry = audit_entry(
        ctx.author().name.as_ref(),
        "vm shutdown",
        format!("VM {vmid} on {node} (task: {})", task.data),
    );
    ctx.data().audit_log.push(entry);
    ctx.say(format!("⏳ VM {vmid} shutdown requested (task: {}).", task.data))
        .await?;
    Ok(())
}

/// Migrate a VM to another node
#[poise::command(slash_command)]
pub async fn migrate(
    ctx: Context<'_>,
    #[description = "Source node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Target node name"] target: String,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.vm_migrate(&node, vmid, &target).await?;
    let entry = audit_entry(
        ctx.author().name.as_ref(),
        "vm migrate",
        format!("VM {vmid} from {node} to {target} (task: {})", task.data),
    );
    ctx.data().audit_log.push(entry);
    ctx.say(format!(
        "🔄 VM {vmid} migrating from {node} to {target} (task: {}).",
        task.data
    ))
    .await?;
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
