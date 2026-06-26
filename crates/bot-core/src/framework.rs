use std::collections::HashSet;

use poise::serenity_prelude as serenity;

use crate::config::Config;
use crate::error::Error as CoreError;
use crate::events;

/// Framework-level error type (matches command error type).
type Error = Box<dyn std::error::Error + Send + Sync>;

/// Poise framework instance, already built and ready to start.
pub struct BotFramework {
    client: serenity::Client,
}

impl BotFramework {
    /// Start the bot's gateway connection (blocks on signal).
    pub async fn start(self) -> Result<(), CoreError> {
        self.client.start().await.map_err(CoreError::Discord)
    }
}

/// Build the Poise framework from configuration.
pub async fn build_framework(config: Config) -> Result<BotFramework, CoreError> {
    // Guild- or global-command registration based on config.
    let dev_guild_id = config.dev_guild_id;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: jambon_bot_commands::all_commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(events::handle_event(ctx, event, framework, data))
            },
            on_error: |error| Box::pin(on_error::<jambon_bot_commands::Data>(error)),
            owners: HashSet::new(),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                // Register commands: guild-only for dev, global for prod.
                if let Some(guild_id) = dev_guild_id {
                    tracing::info!("Registering commands in guild {guild_id}");
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        serenity::GuildId::new(guild_id),
                    )
                    .await?;
                } else {
                    tracing::info!("Registering commands globally");
                    poise::builtins::register_globally(ctx, &framework.options().commands)
                        .await?;
                }

                // Wrap config into shared user data.
                let proxmox = jambon_proxmox_api::ProxmoxClient::with_api_token(
                    &config.proxmox_url,
                    &config.proxmox_token_id,
                    &config.proxmox_token_secret,
                    config.accept_invalid_certs,
                )?;

                Ok(jambon_bot_commands::Data {
                    proxmox,
                    alert_channel_id: config.alert_channel_id,
                    monitor_interval_secs: config.monitor_interval_secs,
                    proxmox_url: config.proxmox_url.clone(),
                })
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(&config.discord_token, intents)
        .framework(framework)
        .await
        .map_err(CoreError::Discord)?;

    Ok(BotFramework { client })
}

/// Global error handler for unhandled command errors.
async fn on_error<Data: Send + Sync>(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            tracing::error!("Bot setup failed: {error}");
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!("Command `{}` failed: {error}", ctx.command().name);
            let msg = format!("An error occurred: {error}");
            if let Err(e) = ctx.say(msg).await {
                tracing::error!("Failed to send error message: {e}");
            }
        }
        poise::FrameworkError::ArgumentParse { error, ctx, .. } => {
            tracing::warn!("Argument parse error: {error}");
            if let Err(e) = ctx.say(format!("Invalid argument: {error}")).await {
                tracing::error!("Failed to send error message: {e}");
            }
        }
        poise::FrameworkError::MissingBotPermissions { missing_permissions, ctx, .. } => {
            let msg = format!("I need the `{missing_permissions}` permission to do that.");
            if let Err(e) = ctx.say(msg).await {
                tracing::error!("Failed to send error message: {e}");
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("Error while handling error: {e}");
            }
        }
    }
}
