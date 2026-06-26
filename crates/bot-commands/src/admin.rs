use std::time::Instant;

use poise::CreateReply;

use crate::{Context, Error};

/// Register slash commands in this guild
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Check if the bot is responsive (Discord + Proxmox latency)
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let discord_latency = ctx.ping().await;

    let proxmox_start = Instant::now();
    let proxmox_result = ctx.data().proxmox.version().await;
    let proxmox_ms = proxmox_start.elapsed().as_millis();

    let content = match proxmox_result {
        Ok(ver) => {
            format!(
                "**Pong!** — {} (PVE {})\n\
                 ─────────────────────\n\
                 🟢 Discord WS · `{}ms`\n\
                 🟢 Proxmox API · `{}ms`",
                ctx.data().proxmox_url,
                ver.version,
                discord_latency.as_millis(),
                proxmox_ms,
            )
        }
        Err(e) => {
            format!(
                "**Pong!**\n\
                 ─────────────────────\n\
                 🟢 Discord WS · `{discord}ms`\n\
                 🔴 Proxmox API · `{proxmox}ms` (error: {e})",
                discord = discord_latency.as_millis(),
                proxmox = proxmox_ms,
            )
        }
    };

    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}
