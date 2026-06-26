# Getting Started

## Prerequisites

- Rust 1.81 or newer
- A Discord bot token
- A Proxmox VE instance with API access

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/cleeryy/jambon.git
cd jambon
```

### 2. Configure environment

Copy the example environment file and edit it:

```bash
cp .env.example .env
```

Fill in your credentials:

```env
DISCORD_TOKEN=your_discord_bot_token
PROXMOX_URL=https://pve1:8006
PROXMOX_TOKEN_ID=root@pam!discord-bot
PROXMOX_TOKEN_SECRET=your-uuid-secret
```

### 3. Create a Proxmox API token

On your Proxmox host:

```bash
pveum user token add root@pam discord-bot --privsep=0
```

This will output a token secret (UUID) — copy it to `PROXMOX_TOKEN_SECRET`.

### 4. Build and run

```bash
cargo run --release
```

## Discord Setup

1. Create an application at https://discord.com/developers/applications
2. Create a bot and copy the token
3. Enable Gateway Intents: Server Members, Message Content
4. Invite the bot with `applications.commands` scope
5. Run the bot — it will register its slash commands globally (up to 1 hour) or instantly in a dev guild if `DEV_GUILD_ID` is set
