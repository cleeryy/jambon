use poise::CreateReply;

use crate::{Context, Error};

/// Register slash commands in this guild
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Check if the bot is responsive
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await;
    ctx.send(CreateReply::default()
        .content(format!("🏓 Pong! Latency: {latency}ms"))
        .ephemeral(true))
        .await?;
    Ok(())
}
