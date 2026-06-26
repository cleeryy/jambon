<div align="center">

# 🥖 Jambon

**A Discord bot to control Proxmox VE instances**

[![CI](https://img.shields.io/github/actions/workflow/status/cleeryy/jambon/ci.yml?branch=main&style=flat-square)](https://github.com/cleeryy/jambon/actions)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)](https://github.com/cleeryy/jambon#license)
[![Rust](https://img.shields.io/badge/rust-1.81%2B-orange?style=flat-square)](https://rust-lang.org)
[![Discord](https://img.shields.io/badge/discord-serenity-5865F2?style=flat-square)](https://github.com/serenity-rs/serenity)

</div>

## Features

- **VM Management** — List, start, stop, shutdown, and live-migrate virtual machines via Discord slash commands.
- **Container Management** — Manage LXC containers.
- **Node Monitoring** — Monitor cluster node status, CPU, memory, and uptime.
- **Cluster Overview** — See all resources across your Proxmox cluster.
- **Rich Embeds** — Color-coded status displays with real-time data.
- **Role-Based Access** — Discord permission system controls who can run destructive commands.
- **Secure Auth** — API-token-based authentication (no passwords in transit).

## Quick Start

```bash
# Clone and enter the project
git clone https://github.com/cleeryy/jambon.git
cd jambon

# Configure
cp .env.example .env
# Edit .env with your Discord bot token and Proxmox credentials

# Run
cargo run --release
```

### Prerequisites

- Rust 1.81+
- A [Discord application](https://discord.com/developers/applications) with a bot token
- A Proxmox VE instance with API access

### Creating a Proxmox API Token

On your Proxmox host:

```bash
pveum user token add root@pam discord-bot --privsep=0
```

## Commands

| Command | Description |
|---|---|
| `/vm list` | List VMs on a node or cluster-wide |
| `/vm status` | Detailed VM status |
| `/vm start` | Start a VM |
| `/vm stop` | Force-stop a VM |
| `/vm shutdown` | Gracefully shutdown a VM |
| `/vm migrate` | Live-migrate a VM |
| `/node list` | List all cluster nodes |
| `/node status` | Detailed node status |
| `/cluster status` | Cluster health overview |
| `/cluster resources` | All cluster resources |
| `/ping` | Check bot latency |
| `/register` | Manage command registration |

## Project Structure

```
crates/
├── proxmox-api/    # Proxmox VE REST API client
├── bot-core/       # Framework integration, config, errors
├── bot-commands/   # Slash command handlers
└── bot-bin/        # Binary entrypoint
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All contributions are welcome!

## License

Licensed under either of:

- [MIT License](LICENSE-MIT) or
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
