use poise::serenity_prelude::RoleId;

use crate::{Context, Error};

/// Role names that grant access to destructive commands.
const DESTRUCTIVE_ROLES: &[&str] = &["Proxmox Admin", "Admin"];

/// Check that the user has one of the permitted roles in the current guild.
pub async fn require_destructive(ctx: Context<'_>) -> Result<(), Error> {
    let Some(guild_id) = ctx.guild_id() else {
        return Ok(());
    };

    // Collect allowed role IDs from cache (no await — safe for Send).
    let allowed_roles: Vec<RoleId> = ctx
        .cache()
        .guild(guild_id)
        .map(|g| {
            DESTRUCTIVE_ROLES
                .iter()
                .filter_map(|name| g.roles.iter().find(|(_, r)| r.name == *name).map(|(id, _)| *id))
                .collect()
        })
        .unwrap_or_default();

    // Fetch member over HTTP (this awaits — cache ref already dropped).
    let member = guild_id.member(ctx.http(), ctx.author().id).await?;
    let has_role = member.roles.iter().any(|rid| allowed_roles.contains(rid));

    if has_role {
        Ok(())
    } else {
        Err("You need the **Proxmox Admin** role to use destructive commands.".into())
    }
}
