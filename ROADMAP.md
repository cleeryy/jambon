# Jambon Roadmap

> Semantic versioning via [release-plz](https://release-plz.dev) — every push to `master`
> with conventional commits can trigger a release.
>
> Current: `v0.1.0` (scaffold, bot connects, commands registered)

---

## Tier 1 — Functional Core (v0.2.0)

The skeleton becomes a real tool. Every command actually calls the Proxmox API.

| Feature | Priority |
|---|---|
| **VM commands** — `/vm list`, `/vm status`, `/vm start`, `/vm stop`, `/vm shutdown`, `/vm migrate` with real API calls | 🏆 |
| **Node commands** — `/node list`, `/node status` with real-time metrics | 🏆 |
| **Cluster commands** — `/cluster status`, `/cluster resources` | 🏆 |
| **Admin commands** — `/admin ping` with Proxmox latency, `/admin register` | 🏆 |
| **Error feedback** — User-friendly Discord messages on failure (no raw error dumps) | ✅ |
| **Permission gating** — Role-based access for destructive commands | ✅ |

## Tier 2 — Operations & Monitoring (v0.3.0)

| Feature | Description |
|---|---|
| **Health monitor** — Periodic Proxmox ping, posts to alert channel on failure | 🏆 |
| **Storage commands** — `/storage list`, `/storage status [pool]` with usage graphs | |
| **Backup commands** — `/backup list`, `/backup create [vmid]`, `/backup status [job-id]` | |
| **Audit log** — Track all destructive operations (who stopped what, when) | |
| **Embeds everywhere** — Colour-coded embeds (green=ok, yellow=warning, red=error) | |

## Tier 3 — VM Lifecycle (v0.4.0)

| Feature | Description |
|---|---|
| **VM provision** — `/vm create --template <id> --name <name> --cores <n> --memory <mb> --disk <gb>` | 🏆 |
| **VM destroy** — `/vm delete <node> <vmid>` with confirmation | |
| **VM resize** — `/vm resize <node> <vmid> --cpu <n> --memory <mb> --disk <gb>` | |
| **Snapshot management** — `/vm snapshot list/create/rollback` | |
| **Clone VM** — `/vm clone <node> <vmid> --name <name>` | |

## Tier 4 — Scheduling & Automation (v0.5.0)

| Feature | Description |
|---|---|
| **Scheduled actions** — `/schedule start --vmid 100 --cron "0 9 * * 1-5"` (cron-based) | |
| **Auto-scaling** — Rule-based: "if CPU > 80% for 5 min, add 2 cores" | |
| **Scheduled backups** — `/schedule backup --vmid 100 --cron "0 2 * * 6"` | |
| **Maintenance mode** — `/node drain <node>` — migrates all VMs off a node | |
| **Node fencing** — Automatically fence unresponsive nodes | |

## Tier 5 — Advanced Resources (v0.6.0)

| Feature | Description |
|---|---|
| **Container (LXC) commands** — `/container create/list/status/start/stop` | |
| **Pool management** — `/pool create/list/assign` | |
| **User & ACL management** — `/user create/list`, `/acl set/list` | |
| **Firewall** — `/fw rules list/set`, `/fw log` | |
| **QEMU agent integration** — Guest IP, hostname, info via agent | |

## Tier 6 — Production Ready (v1.0.0)

| Feature | Description |
|---|---|
| **Test coverage** — Unit + integration tests for all crates | |
| **Docker image** — Multi-stage Dockerfile, GitHub Container Registry | |
| **Helm chart** — Kubernetes deployment with configmap | |
| **Prometheus metrics** — `/metrics` endpoint (expose via sidecar) | |
| **i18n** — Multi-language command descriptions & responses (EN, FR) | |
| **Comprehensive docs** — mdBook sections for admin, user, developer | |

---

## 🧪 The Crazy Zone

These are intentionally ambitious. Pick one for the long-term vision.

### ⚡ 1. Proxmox Chaos Monkey

> *"If it hurts, do it more often."*

An intentionally destructive mode that tests cluster resilience.

**Commands:**
- `/chaos kill-vm --probability 0.1` — randomly kills running VMs on a schedule
- `/chaos network-partition --node pve2 --duration 30s` — drops traffic to a node
- `/chaos fill-disk --pool rbd --level 90` — fills storage to test monitoring alerts
- `/chaos start --mode gentle|normal|psycho` / `/chaos stop`
- `/chaos report` — summary of incidents and how HA handled them

**Use case**: Validate that your Proxmox HA cluster actually works before production goes down at 3 AM.

---

### 🤖 2. AI Capacity Forecaster

> *"You'll run out of disk in 47 days."*

Uses historical metrics from Proxmox RRD data to predict resource exhaustion.

**Commands:**
- `/forecast cpu --days 30` — "CPU trend: 90% utilization predicted in 14 days"
- `/forecast disk --pool rbd` — "Pool will be full in 3 months at current growth rate"
- `/forecast memory --node pve1` — "Memory pressure critical by next month"
- `/forecast recommend` — "Add 2 nodes by August or reduce over-provisioning ratio from 4:1 to 3:1"

**Implementation**: Store periodic snapshots of cluster metrics, run linear regression or exponential smoothing.

---

### 💬 3. VM Personality Profile

> *"The database VM sighs: 'Another SELECT * without WHERE...' "*

Assigns personalities to VMs. The bot roleplays as your infrastructure.

- A grumpy mail server that complains about spam volumes
- A database that speaks in SQL JOINs
- A Kubernetes node speaking only in YAML errors
- Memory usage affects mood: high pressure = snappy, idle = playful

**Commands:**
- `/vm talk <node> <vmid>` — one-liner from the VM's personality
- `/vm personality set <node> <vmid> --mood grumpy|chill|tsundere|sql`  
- `/vm mood <node> <vmid>` — "I'm at 92% RAM, feeling sassy today."

**Use case**: None. But it's hilarious for homelab demos.

---

### 🧯 4. Automated Disaster Recovery Drills

> *"Monthly fire drill for your infrastructure."*

The bot runs non-destructive disaster recovery tests on a schedule.

**How it works:**
1. Creates an isolated VLAN/subnet for DR testing
2. Spawns a fresh VM from your latest backup
3. Verifies service health on the restored VM
4. Generates a detailed report with timings
5. Cleans up and posts results to Discord

**Commands:**
- `/drill run --type backup-restore --vmid 100`
- `/drill schedule --cron "0 9 1 * *"` — first of every month
- `/drill report` — "Last 5 drills: ✅✅❌✅✅ (93% success rate)"
- `/drill config` — set thresholds, excluded VMs, network isolation method

---

### 💰 5. Proxmox CRM with Billing

> *"Your homelab, but make it SaaS."*

Track resource allocation per team/project with cost allocation.

**Commands:**
- `/project create --team backend --budget-cpu 16 --budget-ram 32 --budget-disk 500`
- `/project list` — overview of all projects and usage
- `/project usage <name>` — "Backend: 12/16 vCPU (75%), 28/32 GB RAM (87%)"
- `/project overage` — "3 projects exceeding budget this month"
- `/project alert set --threshold 80` — DM when project hits threshold
- `/invoice generate --month 06-2026` — generates CSV cost report by project

**Implementation**: Track per-VM metadata (project tag), query Proxmox for actual usage, compute hours * allocated resources.

---

### 🧠 6. THE BIG ONE: Self-Aware Proxmox Cluster

> *Your homelab runs itself. You're just here to watch.*

A full autonomous orchestration layer on top of Proxmox. The bot graduates from a control-panel to a self-driving infrastructure.

#### Pillars

**A. Predictive Auto-Scaling** (Machine Learning)
- Learns per-VM load patterns (peak hours, weekend dips, batch jobs)
- Pre-scales CPU/memory before load arrives
- Suggests right-sizing for over-provisioned VMs
- `/insights vm <vmid>` — "You over-provision this VM by 4 vCPUs. Save 2x by right-sizing."

**B. Autonomous Maintenance**
- Monitors hardware health signals via Proxmox:
  - ECC error counters (memory degradation)
  - Disk reallocated sectors (impending SSD failure)
  - Temperature trends (cooling failure)
- When a failure is predicted:
  1. Live-migrates all VMs to healthy nodes
  2. Posts detailed alert with evidence
  3. Opens a GitHub issue in your infra repo
  4. (optional) Creates a Cloudflare ticket for RMA
- `/health report` — overall cluster health score
- `/health timeline` — "PVE2 disk predicted failure: migrated VMs on June 24"

**C. Homelab Defence System** (Security)
- Detects crypto-mining: unusual sustained CPU on normally-idle VMs
- Detects port scans from unknown IPs in your Proxmox network
- On alert:
  1. Suspends the suspicious VM
  2. Takes a forensic memory/disk snapshot
  3. Blocks the offending IP in Proxmox firewall
  4. Alerts Discord with "🚨 Suspicious activity detected on VM 101"
- `/defence status` — active alerts, blocked IPs
- `/defence whitelist add --vmid 100` — VM is intentionally compute-heavy
- `/defence report --days 7` — security summary

**D. Cross-Cluster Federation** (Distributed)
- Link multiple Proxmox clusters together (homelab friends, second site, datacenter)
- `/federation link --remote https://friend.lab --token pve-token-abc`
- `/federation status` — shows linked clusters, latency, available resources
- **Emergency colocation**: If your cluster goes dark, bot auto-migrates critical VMs to the friend's cluster
- **Global resource pool**: Treat all federated clusters as one big pool

**E. GitOps for Proxmox** (Declarative)
- Store desired infrastructure state in a git repo (vms.yaml, pools.yaml)
- When a PR is merged, the bot reconciles Proxmox state with git state
- Drift detection runs hourly: alerts when manual changes diverge from git
- `/gitops diff` — "3 VMs differ from declared state. Your CI pipeline was modified 2 hours ago."
- `/gitops reconcile` — apply the git state now (force override)
- `/gitops history` — changelog of infrastructure changes with author

```
# infrastructure/vms.yaml
vms:
  - name: web-prod-01
    template: debian-12
    node: pve1
    cores: 4
    memory: 8192
    disk: 100
    tags: [production, web, critical]
    backup: daily
    auto_scale:
      min_cores: 2
      max_cores: 8
      cpu_threshold: 75
```

**Use case**: A complete GitOps-driven, self-healing, AI-predictive infrastructure that you manage entirely through Discord and GitHub. You become the observer, jambon becomes the operator.

---

## How we'll implement

Each feature follows the same workflow:

1. **Branch**: `feat/<feature-name>` from `dev`
2. **Implement** — crate-by-crate, test each layer
3. **Verify** — `cargo check`, `cargo clippy`, `cargo test`
4. **PR** — squash-merge to `dev`, then to `master` with conventional commit
5. **Release** — release-plz catches the merge and bumps semver

Want to start anywhere on this roadmap. Pick a tier or a crazy idea and let's go.
