use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub timestamp: DateTime<Utc>,
    pub system: SystemOverview,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub processes: ProcessSummary,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemOverview {
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    pub uptime: u64,
    pub boot_time: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    pub global_usage: f32,
    pub cores: Vec<CpuCore>,
    pub physical_core_count: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CpuCore {
    pub name: String,
    pub usage: f32,
    pub frequency: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub swap_free: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProcessSummary {
    pub total_count: usize,
    pub top_cpu_processes: Vec<ProcessInfo>,
    pub top_memory_processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}