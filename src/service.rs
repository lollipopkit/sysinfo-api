use chrono::Utc;
use std::sync::{Arc, Mutex};
use sysinfo::System;

use crate::models::{
    SystemInfo, SystemOverview, CpuInfo, CpuCore, MemoryInfo, ProcessSummary, ProcessInfo,
};

#[derive(Clone)]
pub struct AppState {
    system: Arc<Mutex<System>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            system: Arc::new(Mutex::new(sys)),
        }
    }

    pub fn get_system_info(&self) -> Result<SystemInfo, Box<dyn std::error::Error>> {
        let mut sys = self.system.lock().unwrap();
        sys.refresh_all();

        let cpus: Vec<CpuCore> = sys
            .cpus()
            .iter()
            .map(|cpu| CpuCore {
                name: cpu.name().to_string(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect();

        let mut processes: Vec<_> = sys.processes().values().collect();
        processes.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_cpu_processes: Vec<ProcessInfo> = processes
            .iter()
            .take(10)
            .map(|process| ProcessInfo {
                pid: process.pid().as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            })
            .collect();

        processes.sort_by(|a, b| b.memory().cmp(&a.memory()));
        let top_memory_processes: Vec<ProcessInfo> = processes
            .iter()
            .take(10)
            .map(|process| ProcessInfo {
                pid: process.pid().as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            })
            .collect();

        Ok(SystemInfo {
            timestamp: Utc::now(),
            system: SystemOverview {
                name: System::name(),
                kernel_version: System::kernel_version(),
                os_version: System::os_version(),
                host_name: System::host_name(),
                uptime: System::uptime(),
                boot_time: System::boot_time(),
            },
            cpu: CpuInfo {
                global_usage: sys.global_cpu_info().cpu_usage(),
                cores: cpus,
                physical_core_count: sys.physical_core_count(),
            },
            memory: MemoryInfo {
                total: sys.total_memory(),
                available: sys.available_memory(),
                used: sys.used_memory(),
                free: sys.free_memory(),
                swap_total: sys.total_swap(),
                swap_used: sys.used_swap(),
                swap_free: sys.free_swap(),
            },
            processes: ProcessSummary {
                total_count: sys.processes().len(),
                top_cpu_processes,
                top_memory_processes,
            },
        })
    }
}