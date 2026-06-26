use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE resource pools
#[poise::command(
    slash_command,
    subcommands("list", "status", "create"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn pool(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List all resource pools
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let pools = ctx.data().proxmox.list_pools().await?;

    let mut desc = String::new();
    for p in &pools {
        let member_count = p.members.unwrap_or(0);
        let comment = p.comment.as_deref().unwrap_or("");
        desc.push_str(&format!(
            "**{poolid}** \u{2014} {member_count} members{comment}\n",
            poolid = p.poolid,
            comment = if comment.is_empty() {
                String::new()
            } else {
                format!(" \u{2014} {comment}")
            },
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title("Resource Pools")
        .description(if desc.is_empty() {
            "No pools found.".into()
        } else {
            desc
        })
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Show detailed pool information
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>, #[description = "Pool ID"] poolid: String) -> Result<(), Error> {
    ctx.defer().await?;

    let detail = ctx.data().proxmox.pool_status(&poolid).await?;

    let members_desc = match &detail.members {
        Some(members) => {
            let mut s = String::new();
            for m in members {
                let icon = match m.kind.as_str() {
                    "qemu" => "\u{1f5b5}\u{fe0f}",
                    "lxc" => "\u{1f4e6}",
                    "storage" => "\u{1f4be}",
                    _ => "\u{26aa}",
                };
                s.push_str(&format!("{icon} **{id}** ({kind})\n", id = m.id, kind = m.kind,));
            }
            s
        }
        None => "No members.".into(),
    };

    let embed = serenity::CreateEmbed::new()
        .title(format!("Pool: {poolid}"))
        .field("Comment", detail.comment.as_deref().unwrap_or("(none)"), false)
        .field("Members", members_desc, false)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Create a new resource pool
#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Pool ID"] poolid: String,
    #[description = "Optional comment"] comment: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::PoolCreateOptions {
        poolid: poolid.clone(),
        comment,
    };
    ctx.data().proxmox.pool_create(&opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "pool create".to_string(),
        details: format!("pool '{poolid}' created"),
    });

    let embed = serenity::CreateEmbed::new()
        .title("Pool Created")
        .description(format!("Pool `{poolid}` has been created."))
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
