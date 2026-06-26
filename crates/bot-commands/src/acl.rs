use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE ACL permissions
#[poise::command(
    slash_command,
    subcommands("list", "set", "remove"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn acl(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all ACL entries
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let entries = ctx.data().proxmox.list_acls().await?;

    let mut desc = String::new();
    for e in &entries {
        desc.push_str(&format!(
            "**{path}** \u{2192} roles: `{roles}`",
            path = e.path,
            roles = e.roles,
        ));
        if let Some(ref users) = e.users {
            desc.push_str(&format!(" | users: `{users}`"));
        }
        if let Some(ref groups) = e.groups {
            desc.push_str(&format!(" | groups: `{groups}`"));
        }
        if e.propagate == Some(0) {
            desc.push_str(" | no-prop");
        }
        desc.push('\n');
    }

    let embed = serenity::CreateEmbed::new()
        .title("ACL Entries")
        .description(if desc.is_empty() {
            "No ACL entries found.".into()
        } else {
            desc
        })
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Add or update an ACL entry
#[poise::command(slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "Permission path (e.g. /vms/100)"] path: String,
    #[description = "Comma-separated roles (e.g. PVEVMAdmin,PVEVMUser)"] roles: String,
    #[description = "Comma-separated users (e.g. root@pam,user@pve)"] users: Option<String>,
    #[description = "Comma-separated groups"] groups: Option<String>,
    #[description = "Propagate to child paths"] propagate: Option<bool>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::AclUpdateOptions {
        path: path.clone(),
        roles: roles.clone(),
        users,
        groups,
        propagate: propagate.map(|b| b as u8),
        delete: None,
    };
    ctx.data().proxmox.acl_update(&opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "acl set".to_string(),
        details: format!("path '{path}' roles '{roles}'"),
    });

    let embed = serenity::CreateEmbed::new()
        .title("ACL Updated")
        .description(format!("Permissions set for path `{path}` with roles `{roles}`."))
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Remove an ACL entry
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Permission path (e.g. /vms/100)"] path: String,
    #[description = "Comma-separated roles (e.g. PVEVMAdmin)"] roles: String,
    #[description = "Comma-separated users"] users: Option<String>,
    #[description = "Comma-separated groups"] groups: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::AclUpdateOptions {
        path: path.clone(),
        roles: roles.clone(),
        users,
        groups,
        propagate: None,
        delete: Some(1),
    };
    ctx.data().proxmox.acl_update(&opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "acl remove".to_string(),
        details: format!("path '{path}' roles '{roles}'"),
    });

    let embed = serenity::CreateEmbed::new()
        .title("ACL Removed")
        .description(format!("Permissions removed from path `{path}` for roles `{roles}`."))
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
