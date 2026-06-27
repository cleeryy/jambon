use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tokio::time::Instant;
use tracing::{error, info, warn};

use jambon_proxmox_api::ProxmoxClient;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Handle Discord gateway events.
pub async fn handle_event(
    ctx: &serenity::Context,
    event: &poise::serenity_prelude::FullEvent,
    _framework: poise::FrameworkContext<'_, jambon_bot_commands::Data, Error>,
    data: &jambon_bot_commands::Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { .. } => {
            info!("Bot is ready! Logged in as {}", ctx.cache.current_user().name);

            let ctx = Arc::new(ctx.clone());
            let interval = tokio::time::Duration::from_secs(data.monitor_interval_secs);
            let alert_channel = data.alert_channel_id.map(serenity::ChannelId::new);
            let proxmox = data.proxmox.clone();

            tokio::spawn(async move {
                health_monitor_loop(ctx, interval, alert_channel, proxmox).await;
            });

            let scheduler = data.scheduler.clone();
            let scheduler_proxmox = data.proxmox.clone();
            scheduler.start(scheduler_proxmox);
        }
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(component),
        } => {
            let _ = jambon_bot_commands::interactions::handle_component(ctx, component, data).await;
        }
        _ => {}
    }

    Ok(())
}

/// Periodic background task that checks Proxmox connectivity and alerts.
async fn health_monitor_loop(
    ctx: Arc<serenity::Context>,
    interval: tokio::time::Duration,
    alert_channel: Option<serenity::ChannelId>,
    proxmox: ProxmoxClient,
) {
    info!("Health monitor started (interval: {interval:?})");

    let mut was_healthy = true;
    let mut degraded_since: Option<Instant> = None;

    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    loop {
        tokio::time::sleep(interval).await;

        let health = check_proxmox_health(&proxmox).await;

        match (&health, was_healthy) {
            // Healthy → healthy: nothing to do.
            (Ok(_), true) => {
                degraded_since = None;
            }
            // Healthy → degraded (recovery): alert.
            (Ok(_), false) => {
                info!("Proxmox connectivity restored");
                if let Some(duration) = degraded_since {
                    let downtime = format_duration(duration.elapsed());
                    send_alert(
                        &ctx,
                        alert_channel,
                        "🟢 Proxmox Connectivity Restored",
                        &format!("Proxmox is reachable again after **{downtime}** of degraded state."),
                        0x00ff00,
                    )
                    .await;
                }
                was_healthy = true;
                degraded_since = None;
            }
            // Degraded → healthy (new failure): alert.
            (Err(e), true) => {
                warn!("Proxmox health check failed: {e}");
                degraded_since = Some(Instant::now());

                send_alert(
                    &ctx,
                    alert_channel,
                    "🔴 Proxmox Connectivity Lost",
                    &format!(
                        "Cannot reach Proxmox VE:\n```{e}```\nThe health monitor will keep checking every {interval:?} and alert when restored.",
                    ),
                    0xff0000,
                )
                .await;

                was_healthy = false;
            }
            // Degraded → degraded: no alert (cooldown).
            (Err(_), false) => {
                // Log at trace level only to avoid noise.
                tracing::trace!(
                    "Proxmox still unreachable (degraded since ~{:.0?})",
                    degraded_since.map_or(std::time::Duration::ZERO, |t| t.elapsed())
                );
            }
        }
    }
}

/// Ping the Proxmox API `/version` endpoint to verify connectivity.
async fn check_proxmox_health(client: &ProxmoxClient) -> Result<String, String> {
    let version = client.version().await.map_err(|e| e.to_string())?;
    Ok(version.version)
}

/// Send an embed alert to the configured channel (or log if not configured).
async fn send_alert(
    ctx: &serenity::Context,
    channel: Option<serenity::ChannelId>,
    title: &str,
    description: &str,
    color: u32,
) {
    let Some(channel_id) = channel else {
        info!("[ALERT] {title}: {description}");
        return;
    };

    let msg = serenity::CreateMessage::new().embed(
        serenity::CreateEmbed::new()
            .title(title)
            .description(description)
            .color(color),
    );

    if let Err(e) = channel_id.send_message(&ctx.http, msg).await {
        error!("Failed to send alert to channel {channel_id}: {e}");
    }
}

fn format_duration(d: tokio::time::Duration) -> String {
    let secs = d.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
