use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE LXC containers
#[poise::command(
    slash_command,
    subcommands("list", "status", "create", "delete", "start", "stop", "shutdown", "clone"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn container(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all containers on a node
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>, #[description = "Node name"] node: String) -> Result<(), Error> {
    ctx.defer().await?;

    let containers = ctx.data().proxmox.list_containers(&node).await?;

    let mut desc = String::new();
    for ct in &containers {
        let status_icon = match ct.status.as_str() {
            "running" => "\u{1f7e2}",
            "stopped" => "\u{1f534}",
            _ => "\u{26aa}",
        };
        desc.push_str(&format!(
            "{status_icon} **CT {vmid}** \u{2014} {} ({status})\n",
            ct.name.as_deref().unwrap_or("unnamed"),
            vmid = ct.vmid,
            status = ct.status,
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Containers on {node}"))
        .description(if desc.is_empty() {
            "No containers found.".into()
        } else {
            desc
        })
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Get detailed status of a container
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Container ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let ct = ctx.data().proxmox.container_status(&node, vmid).await?;

    let color = match ct.status.as_str() {
        "running" => crate::colors::COLOR_SUCCESS,
        "stopped" => crate::colors::COLOR_ERROR,
        _ => crate::colors::COLOR_WARNING,
    };

    let mem_used = ct.mem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_max = ct.maxmem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_used = ct.swap.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_max = ct.maxswap.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;

    let embed = serenity::CreateEmbed::new()
        .title(format!(
            "CT {vmid} \u{2014} {}",
            ct.name.as_deref().unwrap_or("unnamed")
        ))
        .field("Status", &ct.status, true)
        .field("Node", &node, true)
        .field("CPU", format!("{:.1}%", ct.cpu.unwrap_or(0.0) * 100.0), true)
        .field("Cores", ct.maxcpu.map_or("?".into(), |c| c.to_string()), true)
        .field("Memory", format!("{mem_used:.1} GB / {mem_max:.1} GB"), true)
        .field("Swap", format!("{swap_used:.1} GB / {swap_max:.1} GB"), true)
        .field("Uptime", format_uptime(ct.uptime.unwrap_or(0)), true)
        .color(color);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Create a container from a template
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Container ID"] vmid: u64,
    #[description = "OS template (e.g. local:vztmpl/ubuntu-22.04)"] ostemplate: String,
    #[description = "Hostname"] hostname: String,
    #[description = "Target storage"] storage: Option<String>,
    #[description = "Root password"] password: Option<String>,
    #[description = "CPU cores"] cores: Option<u64>,
    #[description = "Memory in MiB"] memory: Option<u64>,
    #[description = "Swap in MiB"] swap: Option<u64>,
    #[description = "Network config (e.g. name=eth0,bridge=vmbr0)"] net0: Option<String>,
    #[description = "Root filesystem size (e.g. 8G)"] rootfs: Option<String>,
    #[description = "Start on boot"] onboot: Option<bool>,
    #[description = "Create unprivileged container"] unprivileged: Option<bool>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::LxcCreateOptions {
        node: node.clone(),
        vmid,
        ostemplate,
        hostname: hostname.clone(),
        storage,
        password,
        cores,
        memory,
        swap,
        net0,
        rootfs,
        onboot: onboot.map(|b| b as u8),
        description: None,
        nameserver: None,
        searchdomain: None,
        ssh_public_keys: None,
        unprivileged: unprivileged.map(|b| b as u8),
    };
    let task = ctx.data().proxmox.container_create(&node, &opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "container create".to_string(),
        details: format!("CT {vmid} '{hostname}' on {node} (task: {})", task.data),
    });

    let embed = serenity::CreateEmbed::new()
        .title("Creating Container")
        .field("CT ID", vmid.to_string(), true)
        .field("Hostname", &hostname, true)
        .field("Node", &node, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Delete a container (irreversible)
#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Container ID"] vmid: u64,
    #[description = "Confirm deletion (required to proceed)"] confirm: Option<bool>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if !confirm.unwrap_or(false) {
        let embed = serenity::CreateEmbed::new()
            .title("Confirm Container Deletion")
            .description(format!(
                "Are you sure you want to delete CT **{vmid}** on **{node}**?\n\
                 This action is irreversible.\n\n\
                 Run again with `confirm: True` to proceed."
            ))
            .color(crate::colors::COLOR_WARNING);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let task = ctx.data().proxmox.container_delete(&node, vmid).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "container delete".to_string(),
        details: format!("CT {vmid} on {node} (task: {})", task.data),
    });

    let embed = serenity::CreateEmbed::new()
        .title("Deleting Container")
        .field("CT ID", vmid.to_string(), true)
        .field("Node", &node, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_WARNING);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Start a container
#[poise::command(slash_command)]
pub async fn start(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Container ID"] vmid: u64,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.container_start(&node, vmid).await?;
    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "container start".to_string(),
        details: format!("CT {vmid} on {node} (task: {})", task.data),
    });
    ctx.say(format!("\u{2705} CT {vmid} is starting (task: {}).", task.data))
        .await?;
    Ok(())
}

/// Stop a container
#[poise::command(slash_command)]
pub async fn stop(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Container ID"] vmid: u64,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.container_stop(&node, vmid).await?;
    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "container stop".to_string(),
        details: format!("CT {vmid} on {node} (task: {})", task.data),
    });
    ctx.say(format!(
        "\u{23f9}\u{fe0f} CT {vmid} stop requested (task: {}).",
        task.data
    ))
    .await?;
    Ok(())
}

/// Gracefully shutdown a container
#[poise::command(slash_command)]
pub async fn shutdown(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Container ID"] vmid: u64,
    #[description = "Timeout in seconds before force-stop"] timeout: Option<u64>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;
    let task = ctx.data().proxmox.container_shutdown(&node, vmid, timeout).await?;
    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "container shutdown".to_string(),
        details: format!("CT {vmid} on {node} (task: {})", task.data),
    });
    ctx.say(format!("\u{23f3} CT {vmid} shutdown requested (task: {}).", task.data))
        .await?;
    Ok(())
}

/// Clone a container
#[poise::command(slash_command)]
pub async fn clone(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Source container ID"] vmid: u64,
    #[description = "New container ID"] newid: u64,
    #[description = "Hostname for the new container"] hostname: String,
    #[description = "Target storage pool"] storage: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::LxcCloneOptions {
        node: node.clone(),
        vmid,
        newid,
        hostname: hostname.clone(),
        storage,
        full: Some(1),
        target: None,
    };
    let task = ctx.data().proxmox.container_clone(&node, vmid, &opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "container clone".to_string(),
        details: format!("CT {vmid} on {node} cloned to {newid} (task: {})", task.data),
    });

    let embed = serenity::CreateEmbed::new()
        .title("Cloning Container")
        .field("Source CT", vmid.to_string(), true)
        .field("New CT ID", newid.to_string(), true)
        .field("Hostname", &hostname, true)
        .field("Node", &node, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_SUCCESS);

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
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn test_format_uptime_hours_only() {
        assert_eq!(format_uptime(3660), "1h 1m");
    }

    #[test]
    fn test_format_uptime_minutes_only() {
        assert_eq!(format_uptime(60), "1m");
    }

    #[test]
    fn test_format_uptime_seconds_rounded_down() {
        assert_eq!(format_uptime(59), "0m");
        assert_eq!(format_uptime(0), "0m");
    }

    #[test]
    fn test_format_uptime_exact_day() {
        assert_eq!(format_uptime(86400), "1d 0h 0m");
    }
}
