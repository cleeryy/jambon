# Commands

All commands are Discord slash commands. Destructive commands (creating, modifying,
or deleting resources) require the user to have one of these roles:

- `Proxmox Admin`
- `Admin`

## `/vm` — VM Lifecycle

The main command group for virtual machine management.

### `/vm list`

List all VMs on a specific node or across the entire cluster.

```
/vm list node:pve1
/vm list
```

### `/vm status`

Get detailed status of a specific VM (CPU, memory, uptime).

```
/vm status node:pve1 vmid:100
```

### `/vm start`

Start a VM. **Destructive.**

```
/vm start node:pve1 vmid:100
```

### `/vm stop`

Force-stop a VM (equivalent to pulling the power cord). **Destructive.**

```
/vm stop node:pve1 vmid:100
```

### `/vm shutdown`

Gracefully shutdown a VM with an optional timeout before force-stop.

```
/vm shutdown node:pve1 vmid:100 timeout:30
```

### `/vm migrate`

Live-migrate a running VM to another node with zero downtime.

```
/vm migrate node:pve1 vmid:100 target:pve2
```

### `/vm create`

Provision a new VM by cloning from an existing template. **Destructive.**

```
/vm create node:pve1 name:web-server template:9000
/vm create node:pve1 name:db-server template:9000 cores:4 memory:8192 storage:fast-ssd
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `node` | ✅ | Target node |
| `name` | ✅ | Name for the new VM |
| `template` | ✅ | Template VM ID to clone from |
| `vmid` | — | Specific VM ID (auto-assigned if omitted) |
| `cores` | — | Number of CPU cores |
| `memory` | — | Memory in MiB |
| `storage` | — | Target storage pool |

### `/vm delete`

Delete a VM. Requires explicit `confirm:true` to proceed. **Destructive, audit logged.**

```
/vm delete node:pve1 vmid:100 confirm:true
```

### `/vm resize`

Resize a VM's CPU, memory, and/or disk. **Destructive, audit logged.**

```
/vm resize node:pve1 vmid:100 cores:8 memory:16384 disk:scsi0 size:+20G
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `node` | ✅ | Node name |
| `vmid` | ✅ | VM ID |
| `cores` | — | New CPU core count |
| `memory` | — | New memory in MiB |
| `disk` | — | Disk to resize (e.g. `scsi0`, `virtio0`) |
| `size` | — | New disk size (e.g. `+10G`, `32G`) |

### `/vm snapshot list`

List all snapshots for a VM with timestamps and descriptions.

```
/vm snapshot list node:pve1 vmid:100
```

### `/vm snapshot create`

Create a new snapshot. **Destructive, audit logged.**

```
/vm snapshot create node:pve1 vmid:100 name:before-upgrade description:"Pre-upgrade snapshot"
```

### `/vm snapshot rollback`

Roll back to a previous snapshot. **Destructive, audit logged.**

```
/vm snapshot rollback node:pve1 vmid:100 name:before-upgrade
```

### `/vm clone`

Clone an existing VM. **Destructive, audit logged.**

```
/vm clone node:pve1 vmid:100 name:web-v2
/vm clone node:pve1 vmid:100 name:web-v2 newid:200 storage:fast-ssd
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `node` | ✅ | Node name |
| `vmid` | ✅ | Source VM ID |
| `name` | ✅ | Name for the cloned VM |
| `newid` | — | Specific VM ID for clone (auto-assigned if omitted) |
| `storage` | — | Target storage pool |

### `/vm agent info`

Show QEMU guest agent information (hostname, OS, version).

```
/vm agent info node:pve1 vmid:100
```

### `/vm agent network`

Show guest network interfaces as reported by the QEMU agent.

```
/vm agent network node:pve1 vmid:100
```

### `/vm agent exec`

Execute a command inside the guest via the QEMU agent. **Destructive.**

```
/vm agent exec node:pve1 vmid:100 command:"uptime"
```

---

## `/node` — Node Management

### `/node list`

List all nodes in the cluster with CPU usage, memory, and uptime.

```
/node list
```

### `/node status`

Get detailed status of a specific node (CPU, memory, swap, kernel version).

```
/node status node_name:pve1
```

### `/node drain`

Live-migrate all VMs off a node for maintenance. **Destructive, audit logged.**

```
/node drain node_name:pve1
```

### `/node drain status`

Show the status of ongoing drain operations.

```
/node drain status
```

### `/node drain cancel`

Cancel an active drain operation.

```
/node drain cancel node_name:pve1
```

### `/node fence`

Force-fence an unresponsive node. **Destructive, audit logged.**

```
/node fence node_name:pve1
```

### `/node fence status`

Show currently fenced nodes.

```
/node fence status
```

---

## `/cluster` — Cluster Overview

### `/cluster status`

Overview of the cluster: online nodes, running VMs, resource counts.

```
/cluster status
```

### `/cluster resources`

Detailed listing of all cluster resources (VMs, storage, nodes, pools).

```
/cluster resources
```

---

## `/container` — LXC Container Management

### `/container list`

List all containers on a node.

```
/container list node:pve1
```

### `/container status`

Get detailed status of a container.

```
/container status node:pve1 vmid:200
```

### `/container create`

Provision a new container from a template. **Destructive, audit logged.**

```
/container create node:pve1 vmid:200 ostemplate:debian-12 hostname:web
```

### `/container start`

Start a container. **Destructive.**

```
/container start node:pve1 vmid:200
```

### `/container stop`

Force-stop a container. **Destructive.**

```
/container stop node:pve1 vmid:200
```

### `/container shutdown`

Gracefully shutdown a container. **Destructive.**

```
/container shutdown node:pve1 vmid:200 timeout:30
```

### `/container clone`

Clone an existing container. **Destructive, audit logged.**

```
/container clone node:pve1 vmid:200 newid:201
```

---

## `/storage` — Storage Management

### `/storage list`

List all storage pools configured on the cluster.

```
/storage list
```

### `/storage status`

Get detailed usage status of a specific storage pool.

```
/storage status pool:local-zfs
```

---

## `/backup` — Backup Management

### `/backup list`

List all configured backup jobs.

```
/backup list
```

### `/backup create`

Create a one-time backup for a VM. **Destructive, audit logged.**

```
/backup create node:pve1 vmid:100 storage:backup-pool
/backup create node:pve1 vmid:100 storage:backup-pool mode:snapshot compress:zstd
```

| Parameter | Required | Description |
|-----------|----------|-------------|
| `node` | ✅ | Node name |
| `vmid` | ✅ | VM ID (or comma-separated list, e.g. `100,101,102`) |
| `storage` | ✅ | Target storage pool |
| `mode` | — | Backup mode: `snapshot` (default), `suspend`, or `stop` |
| `compress` | — | Compression: `lzo`, `gzip`, or `zstd` |

### `/backup status`

Check the status of a running backup task.

```
/backup status node:pve1 upid:UPID:pve1:...
```

---

## `/schedule` — Scheduling & Automation

### `/schedule vm start`

Schedule a VM to start on a cron schedule.

```
/schedule vm start node:pve1 vmid:100 cron:"0 9 * * 1-5"
```

### `/schedule vm stop`

Schedule a VM to stop on a cron schedule.

```
/schedule vm stop node:pve1 vmid:100 cron:"0 18 * * 1-5"
```

### `/schedule backup`

Schedule recurring backups for a VM.

```
/schedule backup node:pve1 vmid:100 cron:"0 2 * * 6"
```

### `/schedule list`

List all scheduled jobs with their status (active/paused).

```
/schedule list
```

### `/schedule remove`

Remove a scheduled job by ID.

```
/schedule remove id:1
```

### `/schedule pause`

Pause a scheduled job without removing it.

```
/schedule pause id:1
```

### `/schedule resume`

Resume a paused scheduled job.

```
/schedule resume id:1
```

### `/autoscale rule add`

Add an auto-scaling rule for a VM. Example: "if CPU > 80% for 5 minutes, add 2 cores".

```
/autoscale rule add vmid:100 threshold:80 adjustment:2
```

### `/autoscale rule list`

List all auto-scaling rules.

```
/autoscale rule list
```

### `/autoscale rule remove`

Remove an auto-scaling rule.

```
/autoscale rule remove id:1
```

### `/autoscale status`

Show which VMs have auto-scaling active.

```
/autoscale status
```

---

## `/pool` — Pool Management

### `/pool list`

List all Proxmox pools with member counts.

```
/pool list
```

### `/pool status`

Get details about a specific pool (members, storage, permissions).

```
/pool status poolid:production
```

### `/pool create`

Create a new resource pool. **Destructive.**

```
/pool create poolid:staging
```

---

## `/acl` — Access Control

### `/acl list`

List all ACL entries in the cluster.

```
/acl list
```

### `/acl set`

Add a permission for a user/group on a path. **Destructive.**

```
/acl set path:/vms roles:PVEVMAdmin users:user@example.com
```

### `/acl remove`

Remove a permission. **Destructive.**

```
/acl remove path:/vms roles:PVEVMAdmin users:user@example.com
```

---

## `/fw` — Firewall

### `/fw list`

List firewall rules for a VM.

```
/fw list node:pve1 vmid:100
```

### `/fw add`

Add a firewall rule. **Destructive.**

```
/fw add node:pve1 vmid:100 action:ACCEPT source:10.0.0.0/24
```

### `/fw log`

Get recent firewall log entries for a VM.

```
/fw log node:pve1 vmid:100
```

---

## `/audit` — Audit Log

### `/audit recent`

Show recent destructive operations (who did what, when).

```
/audit recent
/audit recent count:20
```

---

## `/ping` — Health Check

Check if the bot is responsive. Returns latency in milliseconds and Proxmox API latency.

```
/ping
```

## `/register` — Command Registration

Opens an interactive button menu to register or unregister slash commands in the current guild. Useful after adding new commands.

```
/register
```

## `/mod list` — Module Info

List all active integration modules.

```
/mod list
```
