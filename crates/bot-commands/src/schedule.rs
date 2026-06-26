use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::audit::AuditEntry;
use crate::{Context, Error};

fn audit_entry(user: &str, command: &str, details: String) -> AuditEntry {
    AuditEntry {
        timestamp: std::time::SystemTime::now(),
        user: user.to_string(),
        command: command.to_string(),
        details,
    }
}

#[poise::command(
    slash_command,
    subcommands(
        "schedule_vm_start",
        "schedule_vm_stop",
        "schedule_backup",
        "schedule_list",
        "schedule_remove",
        "schedule_pause",
        "schedule_resume",
    ),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Automation"
)]
pub async fn schedule(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

#[poise::command(slash_command, rename = "vm_start")]
pub async fn schedule_vm_start(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Cron expression (e.g. \"0 9 * * 1-5\")"] cron: String,
    #[description = "Optional label for this schedule"] name: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let label = name.unwrap_or_else(|| format!("VM {vmid} start"));

    let id = ctx.data().scheduler.add_job(
        label.clone(),
        crate::scheduler::ScheduleAction::VmStart {
            node: node.clone(),
            vmid,
        },
        cron.clone(),
    );

    match id {
        Ok(job_id) => {
            ctx.data().audit_log.push(audit_entry(
                ctx.author().name.as_ref(),
                "schedule vm start",
                format!("VM {vmid} on {node} (cron: {cron})"),
            ));

            let embed = serenity::CreateEmbed::new()
                .title("Schedule Created")
                .field("Name", &label, true)
                .field("Action", "VM Start", true)
                .field("VM ID", vmid.to_string(), true)
                .field("Node", &node, true)
                .field("Cron", &cron, true)
                .field("Job ID", job_id.to_string(), true)
                .color(crate::colors::COLOR_SUCCESS);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed = serenity::CreateEmbed::new()
                .title("Invalid Schedule")
                .description(format!("```\n{e}\n```"))
                .color(crate::colors::COLOR_ERROR);

            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, rename = "vm_stop")]
pub async fn schedule_vm_stop(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID"] vmid: u64,
    #[description = "Cron expression (e.g. \"0 18 * * 1-5\")"] cron: String,
    #[description = "Optional label for this schedule"] name: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let label = name.unwrap_or_else(|| format!("VM {vmid} stop"));

    let id = ctx.data().scheduler.add_job(
        label.clone(),
        crate::scheduler::ScheduleAction::VmStop {
            node: node.clone(),
            vmid,
        },
        cron.clone(),
    );

    match id {
        Ok(job_id) => {
            ctx.data().audit_log.push(audit_entry(
                ctx.author().name.as_ref(),
                "schedule vm stop",
                format!("VM {vmid} on {node} (cron: {cron})"),
            ));

            let embed = serenity::CreateEmbed::new()
                .title("Schedule Created")
                .field("Name", &label, true)
                .field("Action", "VM Stop", true)
                .field("VM ID", vmid.to_string(), true)
                .field("Node", &node, true)
                .field("Cron", &cron, true)
                .field("Job ID", job_id.to_string(), true)
                .color(crate::colors::COLOR_SUCCESS);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed = serenity::CreateEmbed::new()
                .title("Invalid Schedule")
                .description(format!("```\n{e}\n```"))
                .color(crate::colors::COLOR_ERROR);

            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, rename = "backup")]
pub async fn schedule_backup(
    ctx: Context<'_>,
    #[description = "Node name"] node: String,
    #[description = "VM ID(s) (comma-separated, or \"all\")"] vmid: String,
    #[description = "Target storage"] storage: String,
    #[description = "Cron expression (e.g. \"0 2 * * 0\")"] cron: String,
    #[description = "Optional label for this schedule"] name: Option<String>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let label = name.unwrap_or_else(|| format!("Backup VM {vmid}"));

    let id = ctx.data().scheduler.add_job(
        label.clone(),
        crate::scheduler::ScheduleAction::VmBackup {
            node: node.clone(),
            vmid: vmid.clone(),
            storage: storage.clone(),
        },
        cron.clone(),
    );

    match id {
        Ok(job_id) => {
            ctx.data().audit_log.push(audit_entry(
                ctx.author().name.as_ref(),
                "schedule backup",
                format!("VM {vmid} -> {storage} (cron: {cron})"),
            ));

            let embed = serenity::CreateEmbed::new()
                .title("Backup Schedule Created")
                .field("Name", &label, true)
                .field("VM ID(s)", &vmid, true)
                .field("Node", &node, true)
                .field("Storage", &storage, true)
                .field("Cron", &cron, true)
                .field("Job ID", job_id.to_string(), true)
                .color(crate::colors::COLOR_SUCCESS);

            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed = serenity::CreateEmbed::new()
                .title("Invalid Schedule")
                .description(format!("```\n{e}\n```"))
                .color(crate::colors::COLOR_ERROR);

            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, rename = "list")]
pub async fn schedule_list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let jobs = ctx.data().scheduler.get_jobs();

    if jobs.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title("Scheduled Jobs")
            .description("No scheduled jobs.")
            .color(crate::colors::COLOR_INFO);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for job in &jobs {
        let status_icon = if job.enabled { "\u{1f7e2}" } else { "\u{23f8}\u{fe0f}" };
        let action_str = match &job.action {
            crate::scheduler::ScheduleAction::VmStart { node, vmid } => {
                format!("VM {vmid} start on {node}")
            }
            crate::scheduler::ScheduleAction::VmStop { node, vmid } => {
                format!("VM {vmid} stop on {node}")
            }
            crate::scheduler::ScheduleAction::VmBackup { node, vmid, storage } => {
                format!("Backup VM {vmid} on {node} -> {storage}")
            }
        };
        desc.push_str(&format!(
            "{status_icon} **#{id}** — {name} — `{cron}`\n  {action}\n",
            id = job.id,
            name = job.name,
            cron = job.cron_expr,
            action = action_str,
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Scheduled Jobs ({})", jobs.len()))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "remove")]
pub async fn schedule_remove(ctx: Context<'_>, #[description = "Job ID to remove"] job_id: u64) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if ctx.data().scheduler.remove_job(job_id) {
        ctx.data().audit_log.push(audit_entry(
            ctx.author().name.as_ref(),
            "schedule remove",
            format!("removed job {job_id}"),
        ));

        let embed = serenity::CreateEmbed::new()
            .title("Job Removed")
            .description(format!("Scheduled job **#{job_id}** has been removed."))
            .color(crate::colors::COLOR_SUCCESS);

        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("Job Not Found")
            .description(format!("No scheduled job with ID **#{job_id}**."))
            .color(crate::colors::COLOR_WARNING);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    }

    Ok(())
}

#[poise::command(slash_command, rename = "pause")]
pub async fn schedule_pause(ctx: Context<'_>, #[description = "Job ID to pause"] job_id: u64) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if ctx.data().scheduler.set_job_enabled(job_id, false) {
        ctx.data().audit_log.push(audit_entry(
            ctx.author().name.as_ref(),
            "schedule pause",
            format!("paused job {job_id}"),
        ));

        let embed = serenity::CreateEmbed::new()
            .title("Job Paused")
            .description(format!("Scheduled job **#{job_id}** has been paused."))
            .color(crate::colors::COLOR_WARNING);

        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("Job Not Found")
            .description(format!("No scheduled job with ID **#{job_id}**."))
            .color(crate::colors::COLOR_WARNING);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    }

    Ok(())
}

#[poise::command(slash_command, rename = "resume")]
pub async fn schedule_resume(ctx: Context<'_>, #[description = "Job ID to resume"] job_id: u64) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if ctx.data().scheduler.set_job_enabled(job_id, true) {
        ctx.data().audit_log.push(audit_entry(
            ctx.author().name.as_ref(),
            "schedule resume",
            format!("resumed job {job_id}"),
        ));

        let embed = serenity::CreateEmbed::new()
            .title("Job Resumed")
            .description(format!("Scheduled job **#{job_id}** has been resumed."))
            .color(crate::colors::COLOR_SUCCESS);

        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("Job Not Found")
            .description(format!("No scheduled job with ID **#{job_id}**."))
            .color(crate::colors::COLOR_WARNING);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    }

    Ok(())
}

#[poise::command(
    slash_command,
    subcommands(
        "autoscale_rule_add",
        "autoscale_rule_list",
        "autoscale_rule_remove",
        "autoscale_status"
    ),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    category = "Automation"
)]
pub async fn autoscale(_ctx: Context<'_>) -> Result<(), Error> {
    unreachable!("subcommand_required is set")
}

#[poise::command(slash_command, rename = "rule_add")]
#[allow(clippy::too_many_arguments)]
pub async fn autoscale_rule_add(
    ctx: Context<'_>,
    #[description = "VM ID"] vmid: u64,
    #[description = "Node name"] node: String,
    #[description = "Metric to monitor (cpu or memory)"] metric: String,
    #[description = "Threshold (0.0-1.0, e.g. 0.8 for 80%)"] threshold: f64,
    #[description = "Direction: up or down"] direction: String,
    #[description = "Number of cores to add/remove"] adjustment: u64,
    #[description = "Cooldown in seconds between actions (default: 300)"] cooldown: Option<u64>,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    let metric_enum = match metric.to_lowercase().as_str() {
        "cpu" => crate::scheduler::AutoscaleMetric::Cpu,
        "memory" | "mem" => crate::scheduler::AutoscaleMetric::Memory,
        _ => {
            let embed = serenity::CreateEmbed::new()
                .title("Invalid Metric")
                .description("Metric must be `cpu` or `memory`.")
                .color(crate::colors::COLOR_ERROR);

            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
            return Ok(());
        }
    };

    let direction_enum = match direction.to_lowercase().as_str() {
        "up" => crate::scheduler::AutoscaleDirection::Up,
        "down" => crate::scheduler::AutoscaleDirection::Down,
        _ => {
            let embed = serenity::CreateEmbed::new()
                .title("Invalid Direction")
                .description("Direction must be `up` or `down`.")
                .color(crate::colors::COLOR_ERROR);

            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
            return Ok(());
        }
    };

    if !(0.0..=1.0).contains(&threshold) {
        let embed = serenity::CreateEmbed::new()
            .title("Invalid Threshold")
            .description("Threshold must be between 0.0 and 1.0.")
            .color(crate::colors::COLOR_ERROR);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let rule_id = ctx.data().scheduler.add_autoscale_rule(
        vmid,
        node.clone(),
        metric_enum,
        threshold,
        direction_enum,
        adjustment,
        cooldown.unwrap_or(300),
    );

    ctx.data().audit_log.push(audit_entry(
        ctx.author().name.as_ref(),
        "autoscale rule add",
        format!("VM {vmid} metric={metric} threshold={threshold} direction={direction} adjustment={adjustment}"),
    ));

    let embed = serenity::CreateEmbed::new()
        .title("Auto-Scale Rule Added")
        .field("Rule ID", rule_id.to_string(), true)
        .field("VM ID", vmid.to_string(), true)
        .field("Node", &node, true)
        .field("Metric", &metric, true)
        .field("Threshold", format!("{:.0}%", threshold * 100.0), true)
        .field("Direction", &direction, true)
        .field("Adjustment", adjustment.to_string(), true)
        .field("Cooldown", format!("{}s", cooldown.unwrap_or(300)), true)
        .color(crate::colors::COLOR_SUCCESS);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "rule_list")]
pub async fn autoscale_rule_list(
    ctx: Context<'_>,
    #[description = "VM ID (optional \u{2014} shows all if omitted)"] vmid: Option<u64>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let rules = if let Some(vm_id) = vmid {
        ctx.data().scheduler.get_autoscale_rules_for_vm(vm_id)
    } else {
        ctx.data().scheduler.get_autoscale_rules()
    };

    if rules.is_empty() {
        let label = vmid
            .map(|id| format!("VM {id}"))
            .unwrap_or_else(|| "any VM".to_string());
        let embed = serenity::CreateEmbed::new()
            .title("Auto-Scale Rules")
            .description(format!("No auto-scaling rules for {label}."))
            .color(crate::colors::COLOR_INFO);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut desc = String::new();
    for rule in &rules {
        let metric_str = match rule.metric {
            crate::scheduler::AutoscaleMetric::Cpu => "CPU",
            crate::scheduler::AutoscaleMetric::Memory => "Memory",
        };
        let dir_str = match rule.direction {
            crate::scheduler::AutoscaleDirection::Up => "\u{2b06}\u{fe0f} up",
            crate::scheduler::AutoscaleDirection::Down => "\u{2b07}\u{fe0f} down",
        };
        desc.push_str(&format!(
            "**#{id}** — VM {vmid} on {node}\n  \
             Metric: {metric} | Threshold: {pct:.0}% | Direction: {dir} | Adjustment: {adj} cores\n",
            id = rule.id,
            vmid = rule.vmid,
            node = rule.node,
            metric = metric_str,
            pct = rule.threshold * 100.0,
            dir = dir_str,
            adj = rule.adjustment,
        ));
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Auto-Scale Rules ({})", rules.len()))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "rule_remove")]
pub async fn autoscale_rule_remove(
    ctx: Context<'_>,
    #[description = "Rule ID to remove"] rule_id: u64,
) -> Result<(), Error> {
    crate::permissions::require_destructive(ctx).await?;
    ctx.defer().await?;

    if ctx.data().scheduler.remove_autoscale_rule(rule_id) {
        ctx.data().audit_log.push(audit_entry(
            ctx.author().name.as_ref(),
            "autoscale rule remove",
            format!("removed rule {rule_id}"),
        ));

        let embed = serenity::CreateEmbed::new()
            .title("Rule Removed")
            .description(format!("Auto-scale rule **#{rule_id}** has been removed."))
            .color(crate::colors::COLOR_SUCCESS);

        ctx.send(CreateReply::default().embed(embed)).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("Rule Not Found")
            .description(format!("No auto-scale rule with ID **#{rule_id}**."))
            .color(crate::colors::COLOR_WARNING);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    }

    Ok(())
}

#[poise::command(slash_command, rename = "status")]
pub async fn autoscale_status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let rules = ctx.data().scheduler.get_autoscale_rules();

    if rules.is_empty() {
        let embed = serenity::CreateEmbed::new()
            .title("Auto-Scale Status")
            .description("No auto-scaling rules configured.")
            .color(crate::colors::COLOR_INFO);

        ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        return Ok(());
    }

    let mut by_vm: std::collections::BTreeMap<u64, Vec<&crate::scheduler::AutoscaleRule>> =
        std::collections::BTreeMap::new();
    for rule in &rules {
        by_vm.entry(rule.vmid).or_default().push(rule);
    }

    let mut desc = String::new();
    for (vmid, vm_rules) in &by_vm {
        desc.push_str(&format!("**VM {vmid}** — {} rule(s)\n", vm_rules.len()));
        for rule in vm_rules {
            let metric_str = match rule.metric {
                crate::scheduler::AutoscaleMetric::Cpu => "CPU",
                crate::scheduler::AutoscaleMetric::Memory => "Memory",
            };
            let dir_str = match rule.direction {
                crate::scheduler::AutoscaleDirection::Up => "\u{2b06}\u{fe0f}",
                crate::scheduler::AutoscaleDirection::Down => "\u{2b07}\u{fe0f}",
            };
            let cooldown_str = match rule.cooldown_until {
                Some(ts) => {
                    let remaining = ts.saturating_sub(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    );
                    if remaining > 0 {
                        format!(" (cooldown: {remaining}s)")
                    } else {
                        String::new()
                    }
                }
                None => String::new(),
            };
            desc.push_str(&format!(
                "  #{id} {dir} {metric} > {pct:.0}% adjust {adj} cores{cooldown}\n",
                id = rule.id,
                dir = dir_str,
                metric = metric_str,
                pct = rule.threshold * 100.0,
                adj = rule.adjustment,
                cooldown = cooldown_str,
            ));
        }
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Auto-Scale Status ({} rules active)", rules.len()))
        .description(desc)
        .color(crate::colors::COLOR_INFO);

    ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
