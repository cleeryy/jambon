//! Interactive component handling for Discord buttons.
//!
//! Architecture
//! ------------
//! Each button carries a synthetic `custom_id` that encodes:
//!
//!   `<namespace>:<action>:<param1>:<param2>:...`
//!
//! Example:  `vm:start:100:pve1`  →  start VM 100 on node pve1
//!           `vm:page:0:pve1`     →  VM list page 0, node pve1
//!           `conf:vm:stop:100:pve1` → confirm stop VM 100 on pve1
//!           `nav:back`           →  go back to the parent embed
//!
//! Namespaces
//! - `vm` … VM list / status / actions
//! - `store` … storage list / detail
//! - `clu` … cluster status / drill-down
//! - `nav` … navigation (close, back, previous-page, next-page)
//! - `conf` … confirmation gate for destructive actions
//!
//! The function [`handle_component`] is the single entry-point called from the
//! Discord gateway event handler (`events.rs`).  It parses `custom_id`,
//! re-fetches data when necessary, and edits the original message.

use poise::serenity_prelude as serenity;
use serenity::all::{
    ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditInteractionResponse,
};

use crate::{colors, Data, Error};
use jambon_proxmox_api::{ClusterResource, NodeSummary, ProxmoxClient, StorageSummary, VmStatus, VmSummary};

// ---------------------------------------------------------------------------
// Public builders — return (CreateEmbed, Vec<CreateActionRow>) for command handlers

pub fn build_vm_list_embed(
    vms: &[VmSummary],
    page: usize,
    node: &str,
    total_pages: usize,
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * 6;
    let end = (start + 6).min(vms.len());
    let page_vms = &vms[start..end];

    let mut desc = String::new();
    for vm in page_vms {
        let status_icon = match vm.status.as_str() {
            "running" => "🟢",
            "stopped" => "🔴",
            _ => "⚪",
        };
        desc.push_str(&format!(
            "{status_icon} **VM {vmid}** — {name} ({status})\n",
            vmid = vm.vmid,
            name = vm.name.as_deref().unwrap_or("unnamed"),
            status = vm.status,
        ));
    }

    let mut nav_buttons = Vec::new();
    if page > 0 {
        let prev_id = format!("vm:page:{}:{node}", page - 1);
        nav_buttons.push(
            CreateButton::new(&prev_id)
                .label("◀️ Prev")
                .style(ButtonStyle::Secondary),
        );
    }
    nav_buttons.push(
        CreateButton::new("nav:close")
            .label("❌ Close")
            .style(ButtonStyle::Danger),
    );
    if page + 1 < total_pages {
        let next_id = format!("vm:page:{}:{node}", page + 1);
        nav_buttons.push(
            CreateButton::new(&next_id)
                .label("Next ▶️")
                .style(ButtonStyle::Secondary),
        );
    }

    let mut action_rows: Vec<CreateActionRow> = Vec::new();
    let mut current_row: Vec<CreateButton> = Vec::new();
    for vm in page_vms {
        let detail_id = format!("vm:action:detail:{}:{}", vm.vmid, node);
        current_row.push(
            CreateButton::new(&detail_id)
                .label(format!("🔍 VM {}", vm.vmid))
                .style(ButtonStyle::Primary),
        );
        if current_row.len() >= 5 {
            action_rows.push(CreateActionRow::Buttons(std::mem::take(&mut current_row)));
        }
    }
    if !current_row.is_empty() {
        action_rows.push(CreateActionRow::Buttons(current_row));
    }

    let embed = CreateEmbed::new()
        .title(format!("VMs on {node} — page {}/{}", page + 1, total_pages))
        .description(if desc.is_empty() { "No VMs found.".into() } else { desc })
        .color(colors::COLOR_INFO);

    let mut components = action_rows;
    components.push(CreateActionRow::Buttons(nav_buttons));
    (embed, components)
}

pub fn build_vm_detail_embed(status: &VmStatus, node: &str, vmid: u64) -> (CreateEmbed, Vec<CreateActionRow>) {
    let color = match status.status.as_str() {
        "running" => colors::COLOR_SUCCESS,
        "stopped" => colors::COLOR_ERROR,
        _ => colors::COLOR_WARNING,
    };

    let embed = CreateEmbed::new()
        .title(format!("VM {vmid} — {}", status.name.as_deref().unwrap_or("unnamed"),))
        .field("Status", &status.status, true)
        .field("Node", node, true)
        .field("CPU", format!("{:.1}%", status.cpu.unwrap_or(0.0) * 100.0), true)
        .field(
            "Memory",
            format!(
                "{:.1} GB / {:.1} GB",
                status.mem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
                status.maxmem.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0,
            ),
            true,
        )
        .field("Uptime", format_uptime(status.uptime.unwrap_or(0)), true)
        .color(color);

    let mut buttons = Vec::new();
    match status.status.as_str() {
        "running" => {
            let shutdown_id = format!("vm:action:shutdown:{vmid}:{node}");
            buttons.push(
                CreateButton::new(&shutdown_id)
                    .label("⏻ Shutdown")
                    .style(ButtonStyle::Primary),
            );
            let stop_id = format!("vm:action:stop:{vmid}:{node}");
            buttons.push(
                CreateButton::new(&stop_id)
                    .label("⏹ Force-stop")
                    .style(ButtonStyle::Danger),
            );
        }
        "stopped" => {
            let start_id = format!("vm:action:start:{vmid}:{node}");
            buttons.push(
                CreateButton::new(&start_id)
                    .label("▶️ Start")
                    .style(ButtonStyle::Success),
            );
        }
        _ => {}
    }
    let back_id = format!("vm:page:0:{node}");
    buttons.push(
        CreateButton::new(&back_id)
            .label("◀️ Back")
            .style(ButtonStyle::Secondary),
    );
    buttons.push(
        CreateButton::new("nav:close")
            .label("❌ Close")
            .style(ButtonStyle::Danger),
    );

    let components = vec![CreateActionRow::Buttons(buttons)];
    (embed, components)
}

pub fn build_storage_list_embed(
    storages: &[StorageSummary],
    page: usize,
    total_pages: usize,
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let page = page.min(total_pages.saturating_sub(1));
    let start = page * 5;
    let end = (start + 5).min(storages.len());
    let page_storage = &storages[start..end];

    let mut desc = String::new();
    for s in page_storage {
        let status_icon = match s.status.as_deref() {
            Some("available") => "🟢",
            _ => "🔴",
        };
        let usage = s
            .used_fraction
            .map(|f| format!("{:.1}%", f * 100.0))
            .unwrap_or_default();
        let active = s
            .active
            .map(|a| if a == 1 { "active" } else { "inactive" })
            .unwrap_or("?");
        desc.push_str(&format!(
            "{status_icon} **{name}** — {kind} | {content} | {usage} used | {active}\n",
            name = s.storage,
            kind = s.kind.as_deref().unwrap_or("?"),
            content = s.content.as_deref().unwrap_or("?"),
        ));
    }

    let mut nav_buttons = Vec::new();
    if page > 0 {
        let prev_id = format!("store:page:{}", page - 1);
        nav_buttons.push(
            CreateButton::new(&prev_id)
                .label("◀️ Prev")
                .style(ButtonStyle::Secondary),
        );
    }
    nav_buttons.push(
        CreateButton::new("nav:close")
            .label("❌ Close")
            .style(ButtonStyle::Danger),
    );
    if page + 1 < total_pages {
        let next_id = format!("store:page:{}", page + 1);
        nav_buttons.push(
            CreateButton::new(&next_id)
                .label("Next ▶️")
                .style(ButtonStyle::Secondary),
        );
    }

    let mut detail_buttons = Vec::new();
    for s in page_storage {
        let detail_id = format!("store:detail:{}", s.storage);
        detail_buttons.push(
            CreateButton::new(&detail_id)
                .label(format!("📦 {}", s.storage))
                .style(ButtonStyle::Primary),
        );
        if detail_buttons.len() >= 5 {
            break;
        }
    }

    let embed = CreateEmbed::new()
        .title(format!("Storage Pools — page {}/{}", page + 1, total_pages))
        .description(if desc.is_empty() {
            "No storage pools found.".into()
        } else {
            desc
        })
        .color(colors::COLOR_INFO);

    let mut components: Vec<CreateActionRow> = Vec::new();
    if !detail_buttons.is_empty() {
        components.push(CreateActionRow::Buttons(detail_buttons));
    }
    components.push(CreateActionRow::Buttons(nav_buttons));
    (embed, components)
}

pub fn build_cluster_status_embed(
    nodes: &[NodeSummary],
    resources: &[ClusterResource],
) -> (CreateEmbed, Vec<CreateActionRow>) {
    let mut online = 0u64;
    for n in nodes {
        if n.status.as_deref() == Some("online") {
            online += 1;
        }
    }
    let running_vms = resources
        .iter()
        .filter(|r| r.kind == "qemu" && r.status.as_deref() == Some("running"))
        .count();

    let embed = CreateEmbed::new()
        .title("🌐 Proxmox Cluster Status")
        .field("Nodes Online", format!("{online}/{}", nodes.len()), true)
        .field("Running VMs", running_vms.to_string(), true)
        .field("Total Resources", resources.len().to_string(), true)
        .field("", "Click a node below to see its VMs:", false)
        .color(colors::COLOR_INFO);

    let mut node_buttons = Vec::new();
    for n in nodes {
        if n.status.as_deref() == Some("online") {
            let node_id = format!("clu:node:{}", n.node);
            node_buttons.push(
                CreateButton::new(&node_id)
                    .label(format!("🖥️ {}", n.node))
                    .style(ButtonStyle::Primary),
            );
        }
        if node_buttons.len() >= 5 {
            break;
        }
    }
    node_buttons.push(
        CreateButton::new("nav:close")
            .label("❌ Close")
            .style(ButtonStyle::Danger),
    );

    let components = vec![CreateActionRow::Buttons(node_buttons)];
    (embed, components)
}

// ---------------------------------------------------------------------------
// Public entry-point
// ---------------------------------------------------------------------------

/// Called from the Discord gateway event handler when a component interaction
/// arrives.  The function acknowledges the interaction, executes the encoded
/// action, and edits the original message.
pub async fn handle_component(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
        )
        .await
        .map_err(|e| format!("failed to defer component interaction: {e}"))?;

    if let Some(rest) = custom_id.strip_prefix("vm:") {
        handle_vm(ctx, interaction, data, rest).await?;
    } else if let Some(rest) = custom_id.strip_prefix("store:") {
        handle_storage(ctx, interaction, data, rest).await?;
    } else if let Some(rest) = custom_id.strip_prefix("clu:") {
        handle_cluster(ctx, interaction, data, rest).await?;
    } else if let Some(rest) = custom_id.strip_prefix("nav:") {
        handle_nav(ctx, interaction, data, rest).await?;
    } else if let Some(rest) = custom_id.strip_prefix("conf:") {
        handle_confirm(ctx, interaction, data, rest).await?;
    } else if let Some(action) = custom_id.strip_prefix("menu:") {
        handle_menu(ctx, interaction, data, action).await?;
    } else {
        tracing::warn!("unknown component interaction: {custom_id}");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// VM interactions
// ---------------------------------------------------------------------------

async fn handle_vm(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
    payload: &str,
) -> Result<(), Error> {
    let proxmox = &data.proxmox;
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        "page" if parts.len() >= 3 => {
            let page: usize = parts[1].parse().unwrap_or(0);
            let node = parts[2].to_string();
            show_vm_page(ctx, interaction, proxmox, page, &node).await?;
        }
        "action" if parts.len() >= 4 => {
            let action = parts[1];
            let vmid: u64 = parts[2].parse().unwrap_or(0);
            let node = parts[3].to_string();

            match action {
                "start" => match proxmox.vm_start(&node, vmid).await {
                    Ok(task) => {
                        edit_response_simple(
                            ctx,
                            interaction,
                            "✅ VM Started",
                            &format!("VM **{vmid}** on **{node}** is starting.\nTask: `{}`", task.data),
                            0x00ff00,
                        )
                        .await?;
                    }
                    Err(e) => {
                        edit_response_simple(
                            ctx,
                            interaction,
                            "❌ Error",
                            &format!("Failed to start VM {vmid} on {node}:\n```{e}```"),
                            0xff0000,
                        )
                        .await?;
                    }
                },
                "stop" => {
                    let confirm_id = format!("conf:vm:stop:{vmid}:{node}");
                    let cancel_id = format!("nav:back:vm:action:stop:{vmid}:{node}");
                    let embed = CreateEmbed::new()
                        .title("⚠️ Confirm VM Stop")
                        .description(format!(
                            "Are you sure you want to **force-stop** VM **{vmid}** on **{node}**?\n\
                             This is equivalent to pulling the power cord."
                        ))
                        .color(0xf39c12);
                    let components = vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(&confirm_id)
                            .label("✅ Confirm")
                            .style(ButtonStyle::Danger),
                        CreateButton::new(&cancel_id)
                            .label("❌ Cancel")
                            .style(ButtonStyle::Secondary),
                    ])];
                    edit_response(ctx, interaction, embed, components).await?;
                }
                "shutdown" => {
                    let confirm_id = format!("conf:vm:shutdown:{vmid}:{node}");
                    let cancel_id = format!("nav:back:vm:action:shutdown:{vmid}:{node}");
                    let embed = CreateEmbed::new()
                        .title("⚠️ Confirm VM Shutdown")
                        .description(format!("Send ACPI shutdown signal to VM **{vmid}** on **{node}**?"))
                        .color(0xf39c12);
                    let components = vec![CreateActionRow::Buttons(vec![
                        CreateButton::new(&confirm_id)
                            .label("✅ Confirm")
                            .style(ButtonStyle::Danger),
                        CreateButton::new(&cancel_id)
                            .label("❌ Cancel")
                            .style(ButtonStyle::Secondary),
                    ])];
                    edit_response(ctx, interaction, embed, components).await?;
                }
                "detail" => {
                    show_vm_detail(ctx, interaction, proxmox, &node, vmid).await?;
                }
                _ => {
                    tracing::warn!("unknown vm action: {action}");
                }
            }
        }
        _ => {
            tracing::warn!("malformed vm payload: {payload}");
        }
    }
    Ok(())
}

/// Render a paginated VM list page.
async fn show_vm_page(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
    page: usize,
    node: &str,
) -> Result<(), Error> {
    const PAGE_SIZE: usize = 6;
    let vms = proxmox.list_vms(node).await?;
    let total_pages = (vms.len().max(1) - 1) / PAGE_SIZE + 1;
    let (embed, components) = build_vm_list_embed(&vms, page, node, total_pages);
    edit_response(ctx, interaction, embed, components).await
}

/// Show VM detail with action buttons (start / stop / shutdown).
async fn show_vm_detail(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
    node: &str,
    vmid: u64,
) -> Result<(), Error> {
    let status = proxmox.vm_status(node, vmid).await?;
    let (embed, components) = build_vm_detail_embed(&status, node, vmid);
    edit_response(ctx, interaction, embed, components).await
}

// ---------------------------------------------------------------------------
// Storage interactions
// ---------------------------------------------------------------------------

async fn handle_storage(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
    payload: &str,
) -> Result<(), Error> {
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        "page" if parts.len() >= 2 => {
            let page: usize = parts[1].parse().unwrap_or(0);
            show_storage_page(ctx, interaction, &data.proxmox, page).await?;
        }
        "detail" if parts.len() >= 2 => {
            let pool = parts[1..].join(":");
            show_storage_detail(ctx, interaction, &data.proxmox, &pool).await?;
        }
        _ => {
            tracing::warn!("malformed store payload: {payload}");
        }
    }
    Ok(())
}

async fn show_storage_page(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
    page: usize,
) -> Result<(), Error> {
    const PAGE_SIZE: usize = 5;
    let storages = proxmox.list_storage().await?;
    let total_pages = (storages.len().max(1) - 1) / PAGE_SIZE + 1;
    let (embed, components) = build_storage_list_embed(&storages, page, total_pages);
    edit_response(ctx, interaction, embed, components).await
}

async fn show_storage_detail(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
    pool: &str,
) -> Result<(), Error> {
    let nodes = proxmox.list_nodes().await?;
    let mut desc = String::new();
    let mut found = false;

    for node_summary in &nodes {
        let node = &node_summary.node;
        if let Ok(storages) = proxmox.node_storage(node).await {
            if let Some(s) = storages.iter().find(|s| s.storage == pool) {
                found = true;
                let used_gb = s.used.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
                let avail_gb = s.avail.unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
                let usage = s
                    .used_fraction
                    .map(|f| format!("{:.1}%", f * 100.0))
                    .unwrap_or_default();
                let status_icon = match s.status.as_deref() {
                    Some("available") => "🟢",
                    _ => "🔴",
                };
                desc.push_str(&format!(
                    "{status_icon} **{node}**: {used_gb:.1} GB / {avail_gb:.1} GB ({usage})\n",
                ));
            }
        }
    }

    if !found {
        desc = format!("Pool `{pool}` not found on any reachable node.");
    }

    let embed = CreateEmbed::new()
        .title(format!("Storage Pool: {pool}"))
        .description(desc)
        .color(0x3498db);

    let back_id = "store:page:0";
    let back_btn = CreateButton::new(back_id)
        .label("◀️ Back")
        .style(ButtonStyle::Secondary);
    let close_btn = CreateButton::new("nav:close")
        .label("❌ Close")
        .style(ButtonStyle::Danger);

    let components = vec![CreateActionRow::Buttons(vec![back_btn, close_btn])];
    edit_response(ctx, interaction, embed, components).await
}

// ---------------------------------------------------------------------------
// Cluster interactions
// ---------------------------------------------------------------------------

async fn handle_cluster(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
    payload: &str,
) -> Result<(), Error> {
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        "status" => {
            show_cluster_status_page(ctx, interaction, &data.proxmox).await?;
        }
        "node" if parts.len() >= 2 => {
            let node = parts[1].to_string();
            show_cluster_node_vms(ctx, interaction, &data.proxmox, &node).await?;
        }
        _ => {
            tracing::warn!("malformed clu payload: {payload}");
        }
    }
    Ok(())
}

async fn show_cluster_status_page(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
) -> Result<(), Error> {
    let nodes = proxmox.cluster_status().await?;
    let resources = proxmox.cluster_resources().await?;
    let (embed, components) = build_cluster_status_embed(&nodes, &resources);
    edit_response(ctx, interaction, embed, components).await
}

async fn show_cluster_node_vms(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
    node: &str,
) -> Result<(), Error> {
    let vms = proxmox.list_vms(node).await?;

    let mut desc = String::new();
    for vm in &vms {
        let status_icon = match vm.status.as_str() {
            "running" => "🟢",
            "stopped" => "🔴",
            _ => "⚪",
        };
        desc.push_str(&format!(
            "{status_icon} **VM {vmid}** — {name} ({status})\n",
            vmid = vm.vmid,
            name = vm.name.as_deref().unwrap_or("unnamed"),
            status = vm.status,
        ));
    }

    if desc.is_empty() {
        desc = "No VMs on this node.".into();
    }

    let embed = CreateEmbed::new()
        .title(format!("VMs on {node}"))
        .description(desc)
        .color(0x3498db);

    let back_btn = CreateButton::new("clu:status")
        .label("◀️ Back")
        .style(ButtonStyle::Secondary);
    let close_btn = CreateButton::new("nav:close")
        .label("❌ Close")
        .style(ButtonStyle::Danger);

    let components = vec![CreateActionRow::Buttons(vec![back_btn, close_btn])];
    edit_response(ctx, interaction, embed, components).await
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

async fn handle_nav(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    _data: &Data,
    payload: &str,
) -> Result<(), Error> {
    match payload {
        "close" => {
            interaction
                .message
                .delete(&ctx.http)
                .await
                .map_err(|e| format!("failed to delete message: {e}"))?;
        }
        back_payload if back_payload.starts_with("back:") => {
            interaction
                .message
                .delete(&ctx.http)
                .await
                .map_err(|e| format!("failed to delete message: {e}"))?;
        }
        _ => {
            tracing::warn!("unknown nav action: {payload}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Confirmation
// ---------------------------------------------------------------------------

async fn handle_confirm(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
    payload: &str,
) -> Result<(), Error> {
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.len() < 3 {
        return Ok(());
    }

    let ns = parts[0];
    let action = parts[1];
    let params = &parts[2..];
    let proxmox = &data.proxmox;

    match (ns, action) {
        ("vm", "stop") if params.len() >= 2 => {
            let vmid: u64 = params[0].parse().unwrap_or(0);
            let node = params[1];
            match proxmox.vm_stop(node, vmid).await {
                Ok(task) => {
                    edit_response_simple(
                        ctx,
                        interaction,
                        "⏹ VM Stopped",
                        &format!(
                            "VM **{vmid}** on **{node}** has been force-stopped.\nTask: `{}`",
                            task.data
                        ),
                        0xff0000,
                    )
                    .await?;
                }
                Err(e) => {
                    edit_response_simple(
                        ctx,
                        interaction,
                        "❌ Error",
                        &format!("Failed to stop VM {vmid} on {node}:\n```{e}```"),
                        0xff0000,
                    )
                    .await?;
                }
            }
        }
        ("vm", "shutdown") if params.len() >= 2 => {
            let vmid: u64 = params[0].parse().unwrap_or(0);
            let node = params[1];
            match proxmox.vm_shutdown(node, vmid, None).await {
                Ok(task) => {
                    edit_response_simple(
                        ctx,
                        interaction,
                        "⏻ VM Shutdown",
                        &format!("VM **{vmid}** on **{node}** is shutting down.\nTask: `{}`", task.data),
                        0xf39c12,
                    )
                    .await?;
                }
                Err(e) => {
                    edit_response_simple(
                        ctx,
                        interaction,
                        "❌ Error",
                        &format!("Failed to shutdown VM {vmid} on {node}:\n```{e}```"),
                        0xff0000,
                    )
                    .await?;
                }
            }
        }
        _ => {
            tracing::warn!("unknown confirmation action: {ns}:{action}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

async fn handle_menu(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    data: &Data,
    action: &str,
) -> Result<(), Error> {
    let proxmox = &data.proxmox;
    match action {
        "vm" => {
            // Pick the first online node and show its VMs.
            let nodes = proxmox.cluster_status().await?;
            let first_online = nodes.iter().find(|n| n.status.as_deref() == Some("online"));
            if let Some(n) = first_online {
                show_vm_page(ctx, interaction, proxmox, 0, &n.node).await?;
            } else {
                edit_response_simple(ctx, interaction, "No Nodes", "No online nodes found.", 0xff0000).await?;
            }
        }
        "container" => {
            let nodes = proxmox.cluster_status().await?;
            let first_online = nodes.iter().find(|n| n.status.as_deref() == Some("online"));
            if let Some(n) = first_online {
                show_container_page(ctx, interaction, proxmox, 0, &n.node).await?;
            } else {
                edit_response_simple(ctx, interaction, "No Nodes", "No online nodes found.", 0xff0000).await?;
            }
        }
        "storage" => {
            show_storage_page(ctx, interaction, proxmox, 0).await?;
        }
        "cluster" => {
            show_cluster_status_page(ctx, interaction, proxmox).await?;
        }
        "node" => {
            show_cluster_status_page(ctx, interaction, proxmox).await?;
        }
        _ => {
            tracing::warn!("unknown menu action: {action}");
        }
    }
    Ok(())
}

async fn show_container_page(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    proxmox: &ProxmoxClient,
    _page: usize,
    node: &str,
) -> Result<(), Error> {
    let containers = proxmox.list_containers(node).await?;
    let total = containers.len();
    let mut desc = String::new();
    for ct in &containers {
        let status_icon = match ct.status.as_str() {
            "running" => "🟢",
            "stopped" => "🔴",
            _ => "⚪",
        };
        desc.push_str(&format!(
            "{status_icon} **CT {vmid}** — {name} ({})\n",
            ct.status,
            vmid = ct.vmid,
            name = ct.name.as_deref().unwrap_or("unnamed"),
        ));
    }
    if desc.is_empty() {
        desc = "No containers on this node.".into();
    }
    let embed = CreateEmbed::new()
        .title(format!("Containers on {node} ({total})"))
        .description(desc)
        .color(colors::COLOR_INFO);
    let close = CreateButton::new("nav:close")
        .label("❌ Close")
        .style(ButtonStyle::Danger);
    edit_response(ctx, interaction, embed, vec![CreateActionRow::Buttons(vec![close])]).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Edit the original interaction response with a new embed and component rows.
async fn edit_response(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    embed: CreateEmbed,
    components: Vec<CreateActionRow>,
) -> Result<(), Error> {
    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().embed(embed).components(components),
        )
        .await
        .map_err(|e| format!("failed to edit interaction response: {e}"))?;
    Ok(())
}

/// Edit the original interaction response with a simple text embed and empty
/// components (clears all buttons).
async fn edit_response_simple(
    ctx: &serenity::Context,
    interaction: &ComponentInteraction,
    title: &str,
    description: &str,
    color: u32,
) -> Result<(), Error> {
    let embed = CreateEmbed::new().title(title).description(description).color(color);
    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new().embed(embed).components(vec![]),
        )
        .await
        .map_err(|e| format!("failed to edit interaction response: {e}"))?;
    Ok(())
}

/// Format uptime seconds into a human-readable string.
fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn test_format_uptime_hours_only() {
        assert_eq!(format_uptime(3660), "1h 1m");
    }

    #[test]
    fn test_format_uptime_minutes_only() {
        assert_eq!(format_uptime(60), "1m");
    }

    #[test]
    fn test_format_uptime_seconds_rounded_down() {
        assert_eq!(format_uptime(59), "0m");
        assert_eq!(format_uptime(0), "0m");
    }

    #[test]
    fn test_format_uptime_exact_day() {
        assert_eq!(format_uptime(86400), "1d 0h 0m");
    }

    #[test]
    fn test_format_uptime_mixed_values() {
        assert_eq!(format_uptime(100000), "1d 3h 46m");
    }
}
