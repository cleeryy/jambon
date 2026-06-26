use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::{Context, Error};

/// View the audit log of destructive operations
#[poise::command(
    slash_command,
    subcommands("recent"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Administration"
)]
pub async fn audit(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// Show the most recent audit log entries
#[poise::command(slash_command)]
pub async fn recent(
    ctx: Context<'_>,
    #[description = "Number of entries to show"] count: Option<usize>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let n = count.unwrap_or(10);
    let entries = ctx.data().audit_log.recent(n);

    let mut desc = String::new();
    for entry in &entries {
        desc.push_str(&format!(
            "`{age}` **{user}** — `{cmd}` — {details}\n",
            age = entry.age(),
            user = entry.user,
            cmd = entry.command,
            details = entry.details,
        ));
    }

    if desc.is_empty() {
        desc = "No audit entries yet.".into();
    }

    let embed = poise::serenity_prelude::CreateEmbed::new()
        .title(format!("Audit Log (last {} entries)", entries.len()))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    let reply = poise::CreateReply::default().embed(embed).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

/// A single audit log entry recording a destructive operation.
#[derive(Clone, Debug)]
pub struct AuditEntry {
    pub timestamp: SystemTime,
    pub user: String,
    pub command: String,
    pub details: String,
}

impl AuditEntry {
    /// Format how long ago this entry was created.
    pub fn age(&self) -> String {
        let elapsed = self.timestamp.elapsed().unwrap_or_default();
        let secs = elapsed.as_secs();
        if secs < 60 {
            format!("{secs}s ago")
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }
}

/// Thread-safe ring buffer for destructive operation audit logs.
#[derive(Clone, Debug)]
pub struct AuditLog {
    inner: Arc<Mutex<Vec<AuditEntry>>>,
    max_entries: usize,
}

impl AuditLog {
    pub fn new(max_entries: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }

    pub fn push(&self, entry: AuditEntry) {
        let mut log = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if log.len() >= self.max_entries {
            log.remove(0);
        }
        log.push(entry);
    }

    pub fn recent(&self, n: usize) -> Vec<AuditEntry> {
        let log = match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let start = log.len().saturating_sub(n);
        log[start..].to_vec()
    }
}
