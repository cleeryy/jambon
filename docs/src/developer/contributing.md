# Developer Guide

## Project Architecture

Jambon is organised as a Rust workspace with four crates:

```
crates/
├── proxmox-api/    # Proxmox VE REST API client (no Discord dependency)
├── bot-core/       # Framework integration, config, error handling, i18n
├── bot-commands/   # Slash command handlers
└── bot-bin/        # Binary entrypoint
```

### proxmox-api

A standalone HTTP client for the Proxmox VE API v2. It handles:
- Authentication (API token and ticket-based)
- Request signing and URL construction
- Response deserialization with the `{ "data": ... }` envelope
- Error mapping to typed `Error` variants

This crate has **no Discord dependency** and could be published separately.

### bot-core

Ties the Discord gateway (poise / serenity) to the Proxmox API client.
Responsible for:
- Configuration loading from environment variables
- Framework setup (`build_framework`)
- Error handling and user-friendly Discord error embeds
- Health monitoring background task
- Internationalization (i18n) support

### bot-commands

Contains all slash command handlers, organised by domain:
- `vm` — VM lifecycle (list, status, start, stop, shutdown, migrate, create, delete, resize, snapshot, clone)
- `node` — Node operations (list, status)
- `cluster` — Cluster operations (status, resources)
- `storage` — Storage management (list, status)
- `backup` — Backup management (list, create, status)
- `audit` — Audit log
- `admin` — Administrative commands (register, ping)
- `mod` — Module listing

### bot-bin

The binary entrypoint. Conditionally compiles:
- Prometheus metrics server (behind `prometheus` feature)
- Health check flag for Kubernetes probes

## Development Setup

### Prerequisites

- Rust 1.81+
- `just` command runner (optional but recommended)
- `pre-commit` (for commit hooks)

### Quick Start

```bash
# Clone and enter the project
git clone https://github.com/cleeryy/jambon.git
cd jambon

# Set up environment
cp .env.example .env
# Edit .env with your credentials

# Run tests
cargo test --workspace --all-features

# Run the bot
cargo run --release
```

### Verification Commands

Before committing, always run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-features --tests -- -D warnings
cargo test --workspace --all-features
```

## Feature Flags

| Feature | Crate | Description |
|---|---|---|
| `prometheus` | bot-bin | Enable Prometheus metrics endpoint on port 9090 |
| `i18n` | bot-core | Enable internationalization support |

Enable features at build time:

```bash
cargo build --features prometheus
cargo run --features prometheus
```

## Docker Build

```bash
# Build the image
docker build -t jambon .

# Run with compose
docker compose up -d
```

The Dockerfile uses `cargo-chef` for layer caching.

## Helm Deployment

See the Helm chart at `deploy/helm/jambon/`:

```bash
helm install jambon ./deploy/helm/jambon \
  --set env.DISCORD_TOKEN=your-token \
  --set env.PROXMOX_URL=https://pve1:8006 \
  --set env.PROXMOX_TOKEN_ID=root@pam!discord-bot \
  --set env.PROXMOX_TOKEN_SECRET=your-secret
```

## Prometheus Metrics

When built with `--features prometheus`, the bot exposes a metrics endpoint:

```bash
# Enable the feature
cargo run --features prometheus

# Or set the port (default: 9090)
METRICS_PORT=9090 cargo run --features prometheus
```

Available metrics:

| Metric | Type | Labels | Description |
|---|---|---|---|
| `jambon_requests_total` | Counter | — | Total metrics HTTP requests |
| `jambon_commands_total` | Counter | `command` | Commands invoked |
| `jambon_vm_operations_total` | Counter | `operation` | VM operations |
| `jambon_api_latency_seconds` | Histogram | `endpoint` | API call latency |

## i18n

Add new translations in `crates/bot-core/src/i18n.rs`:

```rust
s.insert(("en", "my.key"), "Hello {name}");
s.insert(("fr", "my.key"), "Bonjour {name}");
```

Use the `tr!` macro:

```rust
use jambon_bot_core::i18n::I18n;
use jambon_bot_core::tr;

let i18n = I18n::new();
let msg = tr!(i18n, "en", "my.key");
```

The system falls back to English when a key is not found for the requested
language.

## Pull Request Workflow

1. Create a feature branch from `dev`: `git checkout -b feat/my-feature`
2. Implement your changes.
3. Run verification checks (see above).
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat: add support for VM snapshots`
   - `fix: handle timeout during VM shutdown`
   - `docs: update command examples`
5. Push and open a Pull Request against `dev`.
6. After review, squash-merge to `dev`.

## Releases

Releases are automated via [release-plz](https://release-plz.dev).
Every push to `master` with conventional commits can trigger a release.
