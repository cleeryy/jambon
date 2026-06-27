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
    subcommands(
        "list", "status", "start", "stop", "shutdown", "migrate", "create", "delete", "resize", "snapshot", "clone",
        "agent"
    ),
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
    #[description = "Node name (optional \u{2014} shows all nodes if omitted)"] node: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    if let Some(ref node_name) = node {
        let vms = ctx.data().proxmox.list_vms(node_name).await?;
        const PAGE_SIZE: usize = 6;
        let total_pages = (vms.len().max(1) - 1) / PAGE_SIZE + 1;
        let (embed, components) = crate::interactions::build_vm_list_embed(&vms, 0, node_name, total_pages);
        ctx.send(CreateReply::default().embed(embed).components(components).ephemeral(true)).await?;
    } else {
        let resources = ctx.data().proxmox.resources().vms().await?;
        return show_cluster_vms(ctx, &resources).await;
    }

    Ok(())
}

async fn show_cluster_vms(ctx: Context<'_>, resources: &[jambon_proxmox_api::ClusterResource]) -> Result<(), Error> {
    let mut desc = String::new();
    for r in resources {
        let status_icon = match r.status.as_deref() {
            Some("running") => "\u{1f7e2}",
            Some("stopped") => "\u{1f534}",
            _ => "\u{26aa}",
        };
        desc.push_str(&format!(
            "{status_icon} **VM {vmid}** \u{2014} {name} on {node} ({status})\n",
            vmid = r.vmid.unwrap_or(0),
            name = r.name.as_deref().unwrap_or("unnamed"),
            node = r.node.as_deref().unwrap_or("?"),
            status = r.status.as_deref().unwrap_or("?"),
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Cluster VMs")
        .description(if desc.is_empty() { "No VMs found.".into() } else { desc })
        .color(crate::colors::COLOR_INFO);

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
    let (embed, components) = crate::interactions::build_vm_detail_embed(&status, &node, vmid);
    ctx.send(CreateReply::default().embed(embed).components(components)).await?;
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
    ctx.say(format!("\u{2705} VM {vmid} is starting (task: {}).", task.data))
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
    ctx.say(format!(
        "\u{23f9}\u{fe0f} VM {vmid} stop requested (task: {}).",
        task.data
    ))
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
    ctx.say(format!("\u{23f3} VM {vmid} shutdown requested (task: {}).", task.data))
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
        "\u{1f504} VM {vmid} migrating from {node} to {target} (task: {}).",
        task.data
    ))
    .await?;
    Ok(())
}

/// Create a VM from a template
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Name for the new VM"] name: String,
    #[description = "Template VM ID to clone from"] template: u64,
    #[description = "New VM ID (auto-assigned if omitted)"] vmid: Option<u64>,
    #[description = "Number of CPU cores"] cores: Option<u64>,
    #[description = "Memory in MiB"] memory: Option<u64>,
    #[description = "Target storage pool"] storage: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::VmCreateOptions {
        node: node.clone(),
        name: name.clone(),
        vmid,
        template: Some(template),
        cores,
        memory,
        storage,
        full: Some(1),
    };
    let task = ctx.data().proxmox.vm_create(&node, &opts).await?;

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm create",
        format!("VM '{name}' from template {template} on {node} (task: {})", task.data),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Creating VM")
        .field("Name", &name, true)
        .field("Node", &node, true)
        .field("Template", template.to_string(), true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Delete a VM (irreversible)
#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Confirm deletion (required to proceed)"] confirm: Option<bool>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if !confirm.unwrap_or(false) {
        let embed = serenity::CreateEmbed::new()
            .title("Confirm VM Deletion")
            .description(format!(
                "Are you sure you want to delete VM **{vmid}** on **{node}**?\n\
                 This action is irreversible.\n\n\
                 Run again with `confirm: True` to proceed."
            ))
            .color(crate::colors::COLOR_WARNING);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let task = ctx.data().proxmox.vm_delete(&node, vmid).await?;

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm delete",
        format!("VM {vmid} on {node} (task: {})", task.data),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Deleting VM")
        .field("VM ID", vmid.to_string(), true)
        .field("Node", &node, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_WARNING);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Resize a VM (CPU, memory, and/or disk)
#[poise::command(slash_command)]
pub async fn resize(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "CPU cores"] cores: Option<u64>,
    #[description = "Memory in MiB"] memory: Option<u64>,
    #[description = "Disk to resize (e.g. scsi0, virtio0)"] disk: Option<String>,
    #[description = "New disk size (e.g. +10G, 32G)"] size: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let mut changes: Vec<String> = Vec::new();

    if cores.is_some() || memory.is_some() {
        let mut config = serde_json::Map::new();
        if let Some(c) = cores {
            config.insert("cores".to_string(), serde_json::Value::Number(c.into()));
            changes.push(format!("cores={c}"));
        }
        if let Some(m) = memory {
            config.insert("memory".to_string(), serde_json::Value::Number(m.into()));
            changes.push(format!("memory={m}MiB"));
        }
        let payload = serde_json::Value::Object(config);
        ctx.data().proxmox.vm_config_set(&node, vmid, &payload).await?;
    }

    if let Some(ref disk_name) = disk {
        let resize_opts = jambon_proxmox_api::VmResizeDiskOptions {
            disk: disk_name.clone(),
            size: size.clone().unwrap_or_default(),
        };
        if !resize_opts.size.is_empty() {
            ctx.data().proxmox.vm_resize_disk(&node, vmid, &resize_opts).await?;
            changes.push(format!("{disk_name}={}", resize_opts.size));
        }
    }

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm resize",
        format!("VM {vmid} on {node}: {}", changes.join(", ")),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Resizing VM")
        .field("VM ID", vmid.to_string(), true)
        .field("Node", &node, true)
        .field(
            "Changes",
            if changes.is_empty() {
                "No changes requested.".to_string()
            } else {
                changes.join(", ")
            },
            false,
        )
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Manage VM snapshots (list, create, rollback)
#[poise::command(
    slash_command,
    subcommands("snapshot_list", "snapshot_create", "snapshot_rollback"),
    subcommand_required
)]
pub async fn snapshot(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("snapshot subcommand_required is set")
}

/// List snapshots for a VM
#[poise::command(slash_command, rename = "list")]
pub async fn snapshot_list(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let snapshots = ctx.data().proxmox.list_snapshots(&node, vmid).await?;

    if snapshots.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title(format!("Snapshots for VM {vmid}"))
            .description("No snapshots found.")
            .color(crate::colors::COLOR_INFO);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for snap in &snapshots {
        let ts_human = snap.snaptime.map_or_else(
            || "unknown".to_string(),
            |t| {
                chrono::DateTime::from_timestamp(t as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            },
        );

        let desc_text = snap.description.as_deref().unwrap_or("");
        let parent_text = snap.parent.as_deref().unwrap_or("");
        desc.push_str(&format!(
            "**{name}** \u{2014} {ts}  parent: {parent}\n  {extra}\n",
            name = snap.name,
            ts = ts_human,
            parent = parent_text,
            extra = desc_text,
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Snapshots for VM {vmid} on {node}"))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Create a VM snapshot
#[poise::command(slash_command, rename = "create")]
pub async fn snapshot_create(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Snapshot name"] name: String,
    #[description = "Snapshot description"] description: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::SnapshotCreateOptions {
        snapname: name.clone(),
        description,
    };
    let task = ctx.data().proxmox.snapshot_create(&node, vmid, &opts).await?;

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm snapshot create",
        format!("snapshot '{name}' of VM {vmid} on {node} (task: {})", task.data),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Creating Snapshot")
        .field("Snapshot", &name, true)
        .field("VM ID", vmid.to_string(), true)
        .field("Node", &node, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Roll back a VM to a snapshot
#[poise::command(slash_command, rename = "rollback")]
pub async fn snapshot_rollback(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Snapshot name to roll back to"] name: String,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let task = ctx.data().proxmox.snapshot_rollback(&node, vmid, &name).await?;

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm snapshot rollback",
        format!("VM {vmid} on {node} to snapshot '{name}' (task: {})", task.data),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Rolling Back Snapshot")
        .field("VM ID", vmid.to_string(), true)
        .field("Node", &node, true)
        .field("Snapshot", &name, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_WARNING);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Clone a VM
#[poise::command(slash_command)]
pub async fn clone(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Source VM ID"] vmid: u64,
    #[description = "Name for the new VM"] name: String,
    #[description = "New VM ID (auto-assigned if omitted)"] newid: Option<u64>,
    #[description = "Target storage pool"] storage: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::VmCloneOptions {
        node: node.clone(),
        vmid,
        newid: newid.unwrap_or(0),
        name: name.clone(),
        storage,
        full: Some(1),
        target: None,
    };
    let task = ctx.data().proxmox.vm_clone(&node, vmid, &opts).await?;

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "vm clone",
        format!("VM {vmid} on {node} cloned as '{name}' (task: {})", task.data),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Cloning VM")
        .field("Source VM", vmid.to_string(), true)
        .field("New Name", &name, true)
        .field("Node", &node, true)
        .field("Task", &task.data, false)
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Interact with the QEMU guest agent on a VM
#[poise::command(
    slash_command,
    subcommands("agent_info", "agent_network", "agent_exec"),
    subcommand_required
)]
pub async fn agent(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("agent subcommand_required is set")
}

/// Show guest agent info (hostname, OS, version)
#[poise::command(slash_command, rename = "info")]
pub async fn agent_info(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let info = ctx.data().proxmox.vm_agent_info(&node, vmid).await?;

    let embed = serenity::CreateEmbed::new()
        .title(format!("QEMU Agent Info for VM {vmid}"))
        .field("Version", info.version.as_deref().unwrap_or("unknown"), true)
        .field(
            "Supported",
            if info.supported.unwrap_or(false) {
                "\u{2705}"
            } else {
                "\u{274c}"
            },
            true,
        )
        .field("Command", info.command.as_deref().unwrap_or("unknown"), true)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show guest network interfaces
#[poise::command(slash_command, rename = "network")]
pub async fn agent_network(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let interfaces = ctx.data().proxmox.vm_agent_network(&node, vmid).await?;

    let mut desc = String::new();
    for iface in &interfaces {
        let addrs: Vec<String> = iface
            .ip_addresses
            .as_ref()
            .map(|ips| ips.iter().filter_map(|ip| ip.ip_address.clone()).collect())
            .unwrap_or_default();
        desc.push_str(&format!(
            "**{name}** ({mac})\n  IPs: {ips}\n",
            name = iface.name.as_deref().unwrap_or("?"),
            mac = iface.hardware_address.as_deref().unwrap_or("?"),
            ips = if addrs.is_empty() {
                "none".into()
            } else {
                addrs.join(", ")
            },
        ));
    }

    if desc.is_empty() {
        desc = "No network interfaces reported.".into();
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Guest Network Interfaces for VM {vmid}"))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Execute a command on the guest via QEMU agent
#[poise::command(slash_command, rename = "exec")]
pub async fn agent_exec(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Command to execute"] command: String,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    let opts = jambon_proxmox_api::AgentExecOptions { command: parts };
    let result = ctx.data().proxmox.vm_agent_exec(&node, vmid, &opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "vm agent exec".to_string(),
        details: format!("VM {vmid} on {node}: '{command}'"),
    });

    let stdout = result.out_data.as_deref().unwrap_or("");
    let stderr = result.err_data.as_deref().unwrap_or("");
    let exit_code = result.exit_code.unwrap_or(-1);

    let mut desc = String::new();
    if !stdout.is_empty() {
        let truncated = if result.out_truncated.unwrap_or(false) {
            " (truncated)"
        } else {
            ""
        };
        desc.push_str(&format!("**stdout**{truncated}:\n```\n{stdout}\n```\n"));
    }
    if !stderr.is_empty() {
        let truncated = if result.err_truncated.unwrap_or(false) {
            " (truncated)"
        } else {
            ""
        };
        desc.push_str(&format!("**stderr**{truncated}:\n```\n{stderr}\n```\n"));
    }
    if stdout.is_empty() && stderr.is_empty() {
        desc = "Command produced no output.".into();
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Guest Exec on VM {vmid}"))
        .field("Command", &command, false)
        .field("PID", result.pid.map_or("?".into(), |p| p.to_string()), true)
        .field("Exit Code", exit_code.to_string(), true)
        .field("Output", desc, false)
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}


