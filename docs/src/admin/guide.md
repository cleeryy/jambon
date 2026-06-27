# Admin Setup Guide

This guide walks you through setting up Jambon for your Discord server and
connecting it to your Proxmox VE cluster.

## Prerequisites

- A Discord server where you have the **Manage Server** permission
- A Proxmox VE cluster with API access
- A machine to host the bot (or Kubernetes cluster for Helm deployment)

## Step 1: Create a Discord Application

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications).
2. Click **New Application** and give it a name (e.g., "Jambon").
3. Navigate to the **Bot** section.
4. Click **Reset Token** and copy the new token. This is your `DISCORD_TOKEN`.
5. Under **Privileged Gateway Intents**, enable:
   - **Server Members Intent**
   - **Message Content Intent**
6. Save changes.

### Invite the Bot to Your Server

In the **OAuth2 → URL Generator** section:

1. Select the **bot** scope.
2. Select the **Administrator** permission (or be more restrictive).
3. Use the generated URL to invite the bot to your server.

## Step 2: Create a Proxmox API Token

On your Proxmox host, run:

```bash
pveum user token add root@pam discord-bot --privsep=0
```

This outputs a token ID and secret. Save these securely — they are your
`PROXMOX_TOKEN_ID` and `PROXMOX_TOKEN_SECRET`.

> **Note**: The `--privsep=0` flag grants full API access. For production,
> consider creating a dedicated API user with restricted permissions.

## Step 3: Configure Environment

Copy the example environment file and fill in your credentials:

```bash
cp .env.example .env
```

Required variables:

| Variable | Description |
|---|---|
| `DISCORD_TOKEN` | Your Discord bot token |
| `PROXMOX_URL` | Proxmox API URL (e.g., `https://pve1:8006`) |
| `PROXMOX_TOKEN_ID` | The API token ID (e.g., `root@pam!discord-bot`) |
| `PROXMOX_TOKEN_SECRET` | The API token secret |

Optional variables:

| Variable | Default | Description |
|---|---|---|
| `DEV_GUILD_ID` | — | Guild ID for instant command registration |
| `ALERT_CHANNEL_ID` | — | Channel for health alerts |
| `MONITOR_INTERVAL_SECS` | `60` | Health check interval in seconds |
| `ACCEPT_INVALID_CERTS` | `true` | Accept self-signed TLS certificates |

## Step 4: Run the Bot

### Docker (recommended)

```bash
docker compose up -d
```

### Native

```bash
cargo run --release
```

### Kubernetes with Helm

See the [Helm deployment](../deploy/helm/README.md) documentation.

## Setting Up Roles

Jambon uses Discord roles for permission control. The destructive commands
(VM start, stop, shutdown, migrate, delete, etc.) require the user to have
one of the following roles:

- **Proxmox Admin**
- **Admin**

Create these roles in your Discord server and assign them to trusted users.

## Health Monitoring

When configured with `ALERT_CHANNEL_ID`, Jambon periodically checks Proxmox
connectivity and posts alerts to the specified channel when the API becomes
unreachable or recovers.
