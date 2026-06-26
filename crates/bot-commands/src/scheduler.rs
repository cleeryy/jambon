use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, TimeZone, Utc};
use cron::Schedule;
use jambon_proxmox_api::ProxmoxClient;
use tracing::{error, info, warn};

fn next_id(counter: &Mutex<u64>) -> u64 {
    let mut c = match counter.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *c += 1;
    *c
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Clone, Debug)]
pub enum ScheduleAction {
    VmStart {
        node: String,
        vmid: u64,
    },
    VmStop {
        node: String,
        vmid: u64,
    },
    VmBackup {
        node: String,
        vmid: String,
        storage: String,
    },
}

#[derive(Clone, Debug)]
pub struct ScheduledJob {
    pub id: u64,
    pub name: String,
    pub action: ScheduleAction,
    pub cron_expr: String,
    pub enabled: bool,
    pub last_run: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum AutoscaleMetric {
    Cpu,
    Memory,
}

#[derive(Clone, Debug)]
pub enum AutoscaleDirection {
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct AutoscaleRule {
    pub id: u64,
    pub vmid: u64,
    pub node: String,
    pub metric: AutoscaleMetric,
    pub threshold: f64,
    pub direction: AutoscaleDirection,
    pub adjustment: u64,
    pub cooldown_until: Option<u64>,
    pub cooldown_secs: u64,
}

#[derive(Clone, Debug)]
pub struct DrainOperation {
    pub node: String,
    pub started_at: u64,
    pub total_vms: usize,
    pub completed_vms: usize,
    pub failed_vms: Vec<u64>,
    pub cancelled: bool,
}

#[derive(Clone, Debug)]
pub struct FenceState {
    pub node: String,
    pub fenced_at: u64,
    pub fenced_by: String,
}

pub struct Scheduler {
    id_counter: Mutex<u64>,
    pub jobs: Mutex<Vec<ScheduledJob>>,
    pub autoscale_rules: Mutex<Vec<AutoscaleRule>>,
    pub drain_ops: Mutex<Vec<DrainOperation>>,
    pub fence_state: Mutex<Vec<FenceState>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            id_counter: Mutex::new(0),
            jobs: Mutex::new(Vec::new()),
            autoscale_rules: Mutex::new(Vec::new()),
            drain_ops: Mutex::new(Vec::new()),
            fence_state: Mutex::new(Vec::new()),
        }
    }

    pub fn add_job(&self, name: String, action: ScheduleAction, cron_expr: String) -> Result<u64, String> {
        cron_expr
            .parse::<cron::Schedule>()
            .map_err(|e| format!("invalid cron expression: {e}"))?;
        let id = next_id(&self.id_counter);
        let job = ScheduledJob {
            id,
            name,
            action,
            cron_expr,
            enabled: true,
            last_run: None,
        };
        match self.jobs.lock() {
            Ok(mut guard) => guard.push(job),
            Err(poisoned) => poisoned.into_inner().push(job),
        }
        Ok(id)
    }

    pub fn remove_job(&self, id: u64) -> bool {
        let mut jobs = match self.jobs.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let len = jobs.len();
        jobs.retain(|j| j.id != id);
        jobs.len() < len
    }

    pub fn set_job_enabled(&self, id: u64, enabled: bool) -> bool {
        let mut jobs = match self.jobs.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        for job in jobs.iter_mut() {
            if job.id == id {
                job.enabled = enabled;
                return true;
            }
        }
        false
    }

    pub fn get_jobs(&self) -> Vec<ScheduledJob> {
        let jobs = match self.jobs.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        jobs.clone()
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    pub fn add_autoscale_rule(
        &self,
        vmid: u64,
        node: String,
        metric: AutoscaleMetric,
        threshold: f64,
        direction: AutoscaleDirection,
        adjustment: u64,
        cooldown_secs: u64,
    ) -> u64 {
        let id = next_id(&self.id_counter);
        let rule = AutoscaleRule {
            id,
            vmid,
            node,
            metric,
            threshold,
            direction,
            adjustment,
            cooldown_until: None,
            cooldown_secs,
        };
        match self.autoscale_rules.lock() {
            Ok(mut guard) => guard.push(rule),
            Err(poisoned) => poisoned.into_inner().push(rule),
        }
        id
    }

    pub fn remove_autoscale_rule(&self, id: u64) -> bool {
        let mut rules = match self.autoscale_rules.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let len = rules.len();
        rules.retain(|r| r.id != id);
        rules.len() < len
    }

    pub fn get_autoscale_rules(&self) -> Vec<AutoscaleRule> {
        let rules = match self.autoscale_rules.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        rules.clone()
    }

    pub fn get_autoscale_rules_for_vm(&self, vmid: u64) -> Vec<AutoscaleRule> {
        let rules = match self.autoscale_rules.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        rules.iter().filter(|r| r.vmid == vmid).cloned().collect()
    }

    pub fn add_drain_op(&self, node: String, total_vms: usize) {
        let op = DrainOperation {
            node,
            started_at: now_secs(),
            total_vms,
            completed_vms: 0,
            failed_vms: Vec::new(),
            cancelled: false,
        };
        match self.drain_ops.lock() {
            Ok(mut guard) => guard.push(op),
            Err(poisoned) => poisoned.into_inner().push(op),
        }
    }

    pub fn get_drain_ops(&self) -> Vec<DrainOperation> {
        let ops = match self.drain_ops.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        ops.clone()
    }

    pub fn cancel_drain(&self, node: &str) -> bool {
        let mut ops = match self.drain_ops.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        for op in ops.iter_mut() {
            if op.node == node && !op.cancelled {
                op.cancelled = true;
                return true;
            }
        }
        false
    }

    pub fn add_fence(&self, node: String, fenced_by: String) {
        match self.fence_state.lock() {
            Ok(mut guard) => {
                if !guard.iter().any(|f| f.node == node) {
                    guard.push(FenceState {
                        node,
                        fenced_at: now_secs(),
                        fenced_by,
                    });
                }
            }
            Err(poisoned) => {
                let mut guard = poisoned.into_inner();
                if !guard.iter().any(|f| f.node == node) {
                    guard.push(FenceState {
                        node,
                        fenced_at: now_secs(),
                        fenced_by,
                    });
                }
            }
        }
    }

    pub fn remove_fence(&self, node: &str) -> bool {
        let mut fences = match self.fence_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let len = fences.len();
        fences.retain(|f| f.node != node);
        fences.len() < len
    }

    pub fn get_fenced_nodes(&self) -> Vec<FenceState> {
        let fences = match self.fence_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        fences.clone()
    }

    pub fn is_fenced(&self, node: &str) -> bool {
        let fences = match self.fence_state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        fences.iter().any(|f| f.node == node)
    }

    pub fn start(self: Arc<Self>, proxmox: ProxmoxClient) {
        tokio::spawn(async move {
            info!("Scheduler background loop started (interval: 30s)");
            tokio::time::sleep(Duration::from_secs(10)).await;
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let s = &*self;
                check_cron_jobs(s, &proxmox).await;
                check_autoscale(s, &proxmox).await;
                process_drain_ops(s, &proxmox).await;
                check_fencing(s, &proxmox).await;
            }
        });
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

async fn check_cron_jobs(scheduler: &Scheduler, proxmox: &ProxmoxClient) {
    let jobs = scheduler.get_jobs();
    let now = Utc::now();
    for job in &jobs {
        if !job.enabled {
            continue;
        }
        let schedule: Schedule = match job.cron_expr.parse() {
            Ok(s) => s,
            Err(e) => {
                error!("Invalid cron expression for job {}: {e}", job.id);
                continue;
            }
        };
        let last: DateTime<Utc> = job
            .last_run
            .and_then(|ts| Utc.timestamp_opt(ts as i64, 0).single())
            .unwrap_or(Utc.timestamp_opt(0, 0).single().unwrap_or(DateTime::UNIX_EPOCH));
        let should_run = schedule.after(&last).next().is_some_and(|next| next <= now);
        if !should_run {
            continue;
        }
        info!("Executing scheduled job {} ({})", job.id, job.name);
        execute_job(job, proxmox).await;
        let mut jobs = match scheduler.jobs.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(j) = jobs.iter_mut().find(|j| j.id == job.id) {
            j.last_run = Some(now_secs());
        }
    }
}

async fn execute_job(job: &ScheduledJob, proxmox: &ProxmoxClient) {
    let result = match &job.action {
        ScheduleAction::VmStart { node, vmid } => proxmox.vm_start(node, *vmid).await.map(|_| ()),
        ScheduleAction::VmStop { node, vmid } => proxmox.vm_stop(node, *vmid).await.map(|_| ()),
        ScheduleAction::VmBackup { node, vmid, storage } => proxmox
            .create_backup(node, vmid, storage, Some("snapshot"), Some("zstd"))
            .await
            .map(|_| ()),
    };
    match result {
        Ok(()) => info!("Scheduled job {} completed successfully", job.id),
        Err(e) => error!("Scheduled job {} failed: {e}", job.id),
    }
}

async fn check_autoscale(scheduler: &Scheduler, proxmox: &ProxmoxClient) {
    let rules = scheduler.get_autoscale_rules();
    let now = now_secs();
    for rule in &rules {
        if let Some(cooldown) = rule.cooldown_until {
            if now < cooldown {
                continue;
            }
        }
        let status = match proxmox.vm_status(&rule.node, rule.vmid).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Auto-scale: failed to get status for VM {}: {e}", rule.vmid);
                continue;
            }
        };
        let current = match rule.metric {
            AutoscaleMetric::Cpu => status.cpu.unwrap_or(0.0),
            AutoscaleMetric::Memory => {
                let used = status.mem.unwrap_or(0) as f64;
                let total = status.maxmem.unwrap_or(1) as f64;
                if total > 0.0 {
                    used / total
                } else {
                    0.0
                }
            }
        };
        let should_scale = match rule.direction {
            AutoscaleDirection::Up => current > rule.threshold,
            AutoscaleDirection::Down => current < rule.threshold,
        };
        if !should_scale {
            continue;
        }
        info!(
            "Auto-scale: VM {} metric={:.2} threshold={:.2}",
            rule.vmid, current, rule.threshold
        );
        let result = match rule.direction {
            AutoscaleDirection::Up => {
                let new_cores = status.maxcpu.unwrap_or(1).saturating_add(rule.adjustment);
                let mut config = serde_json::Map::new();
                config.insert(
                    "cores".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(new_cores)),
                );
                proxmox
                    .vm_config_set(&rule.node, rule.vmid, &serde_json::Value::Object(config))
                    .await
            }
            AutoscaleDirection::Down => {
                let current_cores = status.maxcpu.unwrap_or(1);
                let new_cores = if current_cores > rule.adjustment {
                    current_cores - rule.adjustment
                } else {
                    1
                };
                let mut config = serde_json::Map::new();
                config.insert(
                    "cores".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(new_cores)),
                );
                proxmox
                    .vm_config_set(&rule.node, rule.vmid, &serde_json::Value::Object(config))
                    .await
            }
        };
        match result {
            Ok(_) => {
                info!("Auto-scale: VM {} scaled successfully", rule.vmid);
                let mut rules = match scheduler.autoscale_rules.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                if let Some(r) = rules.iter_mut().find(|r| r.id == rule.id) {
                    r.cooldown_until = Some(now + rule.cooldown_secs);
                }
            }
            Err(e) => error!("Auto-scale: failed to scale VM {}: {e}", rule.vmid),
        }
    }
}

async fn process_drain_ops(scheduler: &Scheduler, _proxmox: &ProxmoxClient) {
    for op in scheduler.get_drain_ops() {
        if op.cancelled || op.completed_vms >= op.total_vms {
            continue;
        }
        info!(
            "Drain in progress: node={} {}/{} VMs migrated",
            op.node, op.completed_vms, op.total_vms
        );
    }
}

async fn check_fencing(scheduler: &Scheduler, proxmox: &ProxmoxClient) {
    let nodes = match proxmox.list_nodes().await {
        Ok(n) => n,
        Err(e) => {
            warn!("Fencing check: failed to list nodes: {e}");
            return;
        }
    };
    for node_summary in &nodes {
        let node_name = &node_summary.node;
        if scheduler.is_fenced(node_name) {
            continue;
        }
        if node_summary.status.as_deref() == Some("offline") {
            info!("Auto-fencing offline node: {node_name}");
            scheduler.add_fence(node_name.clone(), "auto".to_string());
        }
    }
}
