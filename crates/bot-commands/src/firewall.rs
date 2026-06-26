use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Manage Proxmox VE firewall rules
#[poise::command(
    slash_command,
    subcommands("rules", "add_rule", "log"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Proxmox"
)]
pub async fn fw(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

/// List firewall rules for a VM
#[poise::command(slash_command, rename = "rules")]
pub async fn rules(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let fw_rules = ctx.data().proxmox.fw_rules(&node, vmid).await?;

    if fw_rules.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title(format!("Firewall Rules for VM {vmid}"))
            .description("No firewall rules found.")
            .color(crate::colors::COLOR_INFO);
        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for rule in &fw_rules {
        let action_icon = match rule.action.as_deref() {
            Some("ACCEPT") => "\u{2705}",
            Some("DROP") => "\u{274c}",
            Some("REJECT") => "\u{26d4}",
            _ => "\u{2795}",
        };
        desc.push_str(&format!(
            "{action_icon} #{pos} {action} {proto} {source} \u{2192} {dest}:{dport} [{comment}]\n",
            pos = rule.pos.unwrap_or(0),
            action = rule.action.as_deref().unwrap_or("?"),
            proto = rule.proto.as_deref().unwrap_or("any"),
            source = rule.source.as_deref().unwrap_or("any"),
            dest = rule.dest.as_deref().unwrap_or("any"),
            dport = rule.dport.as_deref().unwrap_or("any"),
            comment = rule.comment.as_deref().unwrap_or(""),
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Firewall Rules for VM {vmid} on {node}"))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Add a firewall rule to a VM
#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, rename = "add_rule")]
pub async fn add_rule(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Action (ACCEPT, DROP, REJECT)"] action: String,
    #[description = "Protocol (tcp, udp, icmp, any)"] proto: Option<String>,
    #[description = "Source IP/CIDR"] source: Option<String>,
    #[description = "Destination IP/CIDR"] dest: Option<String>,
    #[description = "Source port"] sport: Option<String>,
    #[description = "Destination port"] dport: Option<String>,
    #[description = "Network interface"] iface: Option<String>,
    #[description = "Log level"] log: Option<String>,
    #[description = "Comment"] comment: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let opts = jambon_proxmox_api::FwRuleOptions {
        kind: None,
        action: Some(action.clone()),
        proto,
        source,
        dest,
        sport,
        dport,
        iface,
        log,
        comment,
        enable: Some(1),
        position: None,
    };
    let task = ctx.data().proxmox.fw_add_rule(&node, vmid, &opts).await?;

    ctx.data().audit_log.push(crate::audit::AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: ctx.author().name.clone(),
        command: "fw add_rule".to_string(),
        details: format!("VM {vmid} on {node} action '{action}' (task: {})", task.data),
    });

    let embed = serenity::CreateEmbed::new()
        .title("Firewall Rule Added")
        .description(format!(
            "Rule `{action}` added for VM {vmid} on {node}.\nTask: `{}`",
            task.data
        ))
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

/// Show recent firewall log entries for a VM
#[poise::command(slash_command)]
pub async fn log(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
) -> Result<(), Error> {
    ctx.defer().await?;

    let entries = ctx.data().proxmox.fw_log(&node, vmid).await?;

    let mut desc = String::new();
    for entry in &entries {
        desc.push_str(&format!(
            "#{n} {ts} {line}\n",
            n = entry.n.unwrap_or(0),
            ts = entry.timestamp.as_deref().unwrap_or("?"),
            line = entry.line.as_deref().unwrap_or(""),
        ));
    }

    if desc.is_empty() {
        desc = "No firewall log entries found.".into();
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Firewall Log for VM {vmid} on {node}"))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
