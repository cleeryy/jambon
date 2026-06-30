use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::{Context, Error};

/// Show the interactive main menu with category buttons.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn menu(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let proxmox = &ctx.data().proxmox;

    // Fetch diagnostic data in parallel.
    let (nodes_res, resources_res, storages_res) = tokio::join!(
        proxmox.cluster_status(),
        proxmox.cluster_resources(),
        proxmox.list_storage(),
    );

    // Graceful degradation: each block is independent.
    let nodes = nodes_res.as_ref().ok();
    let resources = resources_res.as_ref().ok();
    let storages = storages_res.as_ref().ok();

    let online = nodes
        .map(|n| n.iter().filter(|n| n.status.as_deref() == Some("online")).count())
        .unwrap_or(0);
    let node_count = nodes.map(|n| n.len()).unwrap_or(0);

    let total_vms = resources
        .map(|r| r.iter().filter(|r| r.kind == "qemu").count())
        .unwrap_or(0);
    let running_vms = resources
        .map(|r| {
            r.iter()
                .filter(|r| r.kind == "qemu" && r.status.as_deref() == Some("running"))
                .count()
        })
        .unwrap_or(0);

    let total_cts = resources
        .map(|r| r.iter().filter(|r| r.kind == "lxc").count())
        .unwrap_or(0);
    let running_cts = resources
        .map(|r| {
            r.iter()
                .filter(|r| r.kind == "lxc" && r.status.as_deref() == Some("running"))
                .count()
        })
        .unwrap_or(0);

    let stor_avail = storages
        .map(|s| s.iter().filter(|s| s.status.as_deref() == Some("available")).count())
        .unwrap_or(0);
    let stor_total = storages.map(|s| s.len()).unwrap_or(0);

    // CPU — average of online nodes.
    let cpu_pct = nodes
        .map(|n| {
            let online_nodes: Vec<_> = n.iter().filter(|n| n.status.as_deref() == Some("online")).collect();
            if online_nodes.is_empty() {
                "?".into()
            } else {
                let sum: f64 = online_nodes.iter().filter_map(|n| n.cpu).sum();
                format!("{:.1}%", sum / online_nodes.len() as f64 * 100.0)
            }
        })
        .unwrap_or_else(|| "?".into());

    // Memory — sum across nodes.
    let mem_used = nodes.map(|n| n.iter().filter_map(|n| n.mem).sum::<u64>()).unwrap_or(0);
    let mem_max = nodes
        .map(|n| n.iter().filter_map(|n| n.maxmem).sum::<u64>())
        .unwrap_or(1);
    let mem_gb_used = mem_used as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_gb_max = mem_max as f64 / 1024.0 / 1024.0 / 1024.0;
    let mem_line = format!("{:.1} GB / {:.1} GB", mem_gb_used, mem_gb_max);

    // Disk — sum across nodes.
    let disk_used = nodes.map(|n| n.iter().filter_map(|n| n.disk).sum::<u64>()).unwrap_or(0);
    let disk_max = nodes
        .map(|n| n.iter().filter_map(|n| n.maxdisk).sum::<u64>())
        .unwrap_or(1);
    let disk_gb_used = disk_used as f64 / 1024.0 / 1024.0 / 1024.0;
    let disk_gb_max = disk_max as f64 / 1024.0 / 1024.0 / 1024.0;
    let disk_line = format!("{:.1} GB / {:.1} GB", disk_gb_used, disk_gb_max);

    let embed = serenity::CreateEmbed::new()
        .title("🏠 Jambon Dashboard")
        .field("🌐 Nodes", format!("{online}/{node_count} online"), true)
        .field("🖥️ VMs", format!("{running_vms}/{total_vms} running"), true)
        .field("📦 Containers", format!("{running_cts}/{total_cts} running"), true)
        .field("💾 Storage", format!("{stor_avail}/{stor_total} available"), true)
        .field("📊 CPU", cpu_pct, true)
        .field("🧠 Memory", mem_line, true)
        .field("💽 Disk", disk_line, false)
        .color(crate::colors::COLOR_INFO);

    let buttons = vec![
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("menu:vm")
                .label("🖥️ Virtual Machines")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("menu:container")
                .label("📦 Containers")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("menu:storage")
                .label("💾 Storage")
                .style(serenity::ButtonStyle::Primary),
        ]),
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("menu:cluster")
                .label("🌐 Cluster")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("menu:node")
                .label("🖥️ Nodes")
                .style(serenity::ButtonStyle::Primary),
            serenity::CreateButton::new("nav:close")
                .label("❌ Close")
                .style(serenity::ButtonStyle::Danger),
        ]),
    ];

    ctx.send(CreateReply::default().embed(embed).components(buttons).ephemeral(true))
        .await?;

    Ok(())
}
