# Commands

All commands are Discord slash commands. They require the `ADMINISTRATOR` permission by default.

## `/vm list`

List all VMs on a specific node or across the cluster.

```
/vm list node:pve1
/vm list
```

## `/vm status`

Get detailed status of a specific VM.

```
/vm status node:pve1 vmid:100
```

## `/vm start`

Start a VM.

```
/vm start node:pve1 vmid:100
```

## `/vm stop`

Force-stop a VM.

```
/vm stop node:pve1 vmid:100
```

## `/vm shutdown`

Gracefully shutdown a VM with optional timeout.

```
/vm shutdown node:pve1 vmid:100 timeout:30
```

## `/vm migrate`

Live-migrate a VM to another node.

```
/vm migrate node:pve1 vmid:100 target:pve2
```

## `/node list`

List all nodes in the cluster with CPU, memory, and uptime.

## `/node status`

Get detailed status of a specific node.

```
/node status node_name:pve1
```

## `/cluster status`

Overview of the cluster: online nodes, running VMs, resource counts.

## `/cluster resources`

Detailed listing of all cluster resources (VMs, storage, nodes).

## `/ping`

Check if the bot is responsive. Returns latency in milliseconds.

## `/register`

Opens an interactive button menu to register or unregister slash commands in the current guild.
