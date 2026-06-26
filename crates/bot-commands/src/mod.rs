use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE modules (top-level command group)
#[poise::command(
    slash_command,
    subcommands("list"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn r#mod(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all Proxmox modules/plugins
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("Available Modules")
        .description("Active Proxmox VE integration modules")
        .field("VM Management", "Start, stop, shutdown, migrate VMs", false)
        .field("Container Management", "Start, stop containers", false)
        .field("Node Management", "List and inspect nodes", false)
        .field("Cluster Monitoring", "Cluster status and resources", false)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
