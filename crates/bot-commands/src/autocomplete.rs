use poise::serenity_prelude as serenity;
use serenity::all::AutocompleteChoice;

use crate::Context;

pub async fn autocomplete_node<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let proxmox = &ctx.data().proxmox;
    match proxmox.list_nodes().await {
        Ok(nodes) => {
            let lower = partial.to_lowercase();
            nodes
                .into_iter()
                .map(|n| n.node)
                .filter(|name| name.to_lowercase().contains(&lower))
                .collect()
        }
        Err(e) => {
            tracing::warn!("autocomplete node failed: {e}");
            vec![]
        }
    }
}

pub async fn autocomplete_vm<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<AutocompleteChoice> {
    let node = match &ctx {
        poise::Context::Application(app) => app.args.iter().find(|o| o.name == "node").and_then(|o| match &o.value {
            serenity::ResolvedValue::String(s) => Some(s.to_string()),
            _ => None,
        }),
        _ => None,
    };

    let Some(node) = node else {
        return vec![]; // can't list VMs without a node
    };

    let proxmox = &ctx.data().proxmox;
    match proxmox.list_vms(&node).await {
        Ok(vms) => {
            let lower = partial.to_lowercase();
            vms.into_iter()
                .filter_map(|vm| {
                    let id_str = vm.vmid.to_string();
                    let label = vm.name.unwrap_or_default();
                    if id_str.contains(&lower) || label.to_lowercase().contains(&lower) {
                        Some(AutocompleteChoice::new(format!("{id_str} ({label})"), id_str))
                    } else {
                        None
                    }
                })
                .take(25)
                .collect()
        }
        Err(e) => {
            tracing::warn!("autocomplete vm failed: {e}");
            vec![]
        }
    }
}

pub async fn autocomplete_storage<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let proxmox = &ctx.data().proxmox;
    match proxmox.list_storage().await {
        Ok(storages) => {
            let lower = partial.to_lowercase();
            storages
                .into_iter()
                .map(|s| s.storage)
                .filter(|name| name.to_lowercase().contains(&lower))
                .take(25)
                .collect()
        }
        Err(e) => {
            tracing::warn!("autocomplete storage failed: {e}");
            vec![]
        }
    }
}
