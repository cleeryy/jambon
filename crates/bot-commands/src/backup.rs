use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE backups and backup jobs
#[poise::command(
    slash_command,
    subcommands("list", "create", "status"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn backup(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all configured backup jobs
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let jobs = ctx.data().proxmox.list_backups().await?;

    let mut desc = String::new();
    for job in &jobs {
        let status_icon = match job.enabled {
            Some(1) => "🟢",
            _ => "🔴",
        };
        let schedule = job.schedule.as_deref().unwrap_or("manual");
        let compress = job.compress.as_deref().unwrap_or("none");
        desc.push_str(&format!(
            "{status_icon} **{id}**\n  └ VMs: `{vms}` | Mode: {mode} | \u{2192} {storage} | {compress} | `{schedule}`\n",
            id = job.id,
            vms = job.vmid.as_deref().unwrap_or("all"),
            mode = job.mode.as_deref().unwrap_or("?"),
            storage = job.storage.as_deref().unwrap_or("?"),
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Backup Jobs")
        .description(if desc.is_empty() {
            "No backup jobs configured.".into()
        } else {
            desc
        })
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Create an ad-hoc backup for a VM
#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID (or comma-separated list)"] vmid: String,
    #[description = "Target storage pool"] storage: String,
    #[description = "Backup mode (snapshot/suspend/stop)"] mode: Option<String>,
    #[description = "Compression (lzo/gzip/zstd)"] compress: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let task = ctx
        .data()
        .proxmox
        .create_backup(&node, &vmid, &storage, mode.as_deref(), compress.as_deref())
        .await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "backup create".to_string(),
        details: format!("VM(s) {vmid} on {node} → {storage}"),
    });

    let embed = serenity::CreateEmbed::new()
        .title("Backup Started")
        .description(format!(
            "Backup of VM(s) `{vmid}` on **{node}** \u{2192} `{storage}`\nTask: `{}`",
            task.data
        ))
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Check status of a backup task by UPID
#[poise::command(slash_command)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "Task UPID"] upid: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let task = ctx.data().proxmox.task_status(&node, &upid).await?;

    let status_str = task.status.as_deref().unwrap_or("unknown");
    let exit = task.exitstatus.as_deref().unwrap_or("?");
    let status_icon = match exit {
        "OK" => "\u{2705}",
        "RUNNING" => "\u{23F3}",
        _ => "\u{274C}",
    };

    let embed = serenity::CreateEmbed::new()
        .title("Backup Task Status")
        .description(format!(
            "{status_icon} Status: {status_str} | Exit: {exit}\nUPID: `{upid}`",
        ))
        .color(match exit {
            "OK" => crate::colors::COLOR_SUCCESS,
            "RUNNING" => crate::colors::COLOR_WARNING,
            _ => crate::colors::COLOR_ERROR,
        });

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
