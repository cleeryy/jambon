# Configuration

Jambon is configured through environment variables. Copy `.env.example` to `.env` and fill in your values.

## Required

| Variable | Description |
|---|---|
| `DISCORD_TOKEN` | Your Discord bot token from the Developer Portal |
| `PROXMOX_URL` | Proxmox VE API URL (e.g. `https://pve1:8006`) |
| `PROXMOX_TOKEN_ID` | API token ID (e.g. `root@pam!discord-bot`) |
| `PROXMOX_TOKEN_SECRET` | API token secret (UUID from `pveum`) |

## Optional

| Variable | Default | Description |
|---|---|---|
| `DEV_GUILD_ID` | — | Guild ID for instant command registration during development |
| `ALERT_CHANNEL_ID` | — | Channel to send health alert messages |
| `MONITOR_INTERVAL_SECS` | 60 | How often to check Proxmox health (seconds) |
| `ACCEPT_INVALID_CERTS` | true | Accept self-signed TLS certificates |
