use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tracing::info;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Handle Discord gateway events.
///
/// This is called by Poise's `event_handler` for every gateway event.
pub async fn handle_event(
    ctx: &serenity::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, jambon_bot_commands::Data, Error>,
    data: &jambon_bot_commands::Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about: _ } => {
            info!("Bot is ready! Logged in as {}", ctx.cache.current_user().name);

            // Spawn background health monitor.
            let ctx = Arc::new(ctx.clone());
            let interval = tokio::time::Duration::from_secs(data.monitor_interval_secs);
            let alert_channel = data.alert_channel_id.map(serenity::ChannelId::new);
            let proxmox_url = data.proxmox_url.clone();

            tokio::spawn(async move {
                health_monitor_loop(ctx, interval, alert_channel, proxmox_url).await;
            });
        }
        _ => {}
    }

    Ok(())
}

/// Periodic background task that checks Proxmox connectivity and alerts.
async fn health_monitor_loop(
    _ctx: Arc<serenity::Context>,
    _interval: tokio::time::Duration,
    _alert_channel: Option<serenity::ChannelId>,
    _proxmox_url: String,
) {
    info!("Health monitor started (placeholder)");
    loop {
        tokio::time::sleep(_interval).await;
    }
}
