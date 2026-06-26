# Architecture

## Crate Layout

```
crates/
├── proxmox-api/    # Proxmox VE REST API client (standalone library)
├── bot-core/       # Poise/Serenity framework setup, config, error types
├── bot-commands/   # All slash command handlers
└── bot-bin/        # Binary entrypoint (main.rs)
```

## Data Flow

```
Discord Client
    ⬇ (slash command)
bot-bin (main.rs)
    ⬇
bot-core (framework.rs → dispatches command)
    ⬇
bot-commands/*.rs (command handler)
    ⬇ (calls proxmox-api)
proxmox-api/client.rs (HTTP request → Proxmox VE API)
    ⬇
Proxmox VE Server
    ⬆ (JSON response)
proxmox-api/models.rs (deserialized)
    ⬆
bot-commands (builds embed)
    ⬆
bot-core (sends reply to Discord)
    ⬆
Discord User
```

## Key Design Decisions

- **API Token auth** over username/password — tokens are stateless, long-lived, and revocable independently.
- **Poise framework** instead of raw Serenity — Poise provides subcommand groups, argument parsing, error handling, and is the officially recommended approach.
- **Cluster resources** for VM discovery — the Proxmox web UI uses `GET /cluster/resources` to locate VMs across nodes; the bot follows the same pattern.
- **Workspace layout** — each concern in its own crate enables independent testing, reusability, and cleaner dependency graph.
- **Self-signed certs** — Proxmox uses self-signed certificates by default; `danger_accept_invalid_certs(true)` is the pragmatic default.
