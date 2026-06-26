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
            "{status_icon} **{id}**\n  └ VMs: `{vms}` | Mode: {mode} | → {storage} | {compress} | `{schedule}`\n",
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
        .color(0x00aaff);

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

    let embed = serenity::CreateEmbed::new()
        .title("Backup Started")
        .description(format!(
            "Backup of VM(s) `{vmid}` on **{node}** → `{storage}`\nTask: `{}`",
            task.data
        ))
        .color(0x00ff00);

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

    let status = task.status.as_deref().unwrap_or("unknown");
    let exit = task.exitstatus.as_deref().unwrap_or("?");
    let status_icon = match exit {
        "OK" => "✅",
        "RUNNING" => "⏳",
        _ => "❌",
    };

    let embed = serenity::CreateEmbed::new()
        .title("Backup Task Status")
        .description(format!("{status_icon} Status: {status} | Exit: {exit}\nUPID: `{upid}`",))
        .color(match exit {
            "OK" => 0x00ff00,
            "RUNNING" => 0xffaa00,
            _ => 0xff0000,
        });

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
