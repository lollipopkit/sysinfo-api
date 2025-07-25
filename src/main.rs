use chrono::{DateTime, Utc};
use ntex::web::{self, App, HttpResponse, HttpServer, middleware};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::System;

use crate::api::Resp;

mod api;
mod cfg;

#[derive(Serialize, Deserialize, Clone)]
struct SystemInfo {
    timestamp: DateTime<Utc>,
    system: SystemOverview,
    cpu: CpuInfo,
    memory: MemoryInfo,
    processes: ProcessSummary,
}

#[derive(Serialize, Deserialize, Clone)]
struct SystemOverview {
    name: Option<String>,
    kernel_version: Option<String>,
    os_version: Option<String>,
    host_name: Option<String>,
    uptime: u64,
    boot_time: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct CpuInfo {
    global_usage: f32,
    cores: Vec<CpuCore>,
    physical_core_count: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CpuCore {
    name: String,
    usage: f32,
    frequency: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct MemoryInfo {
    total: u64,
    available: u64,
    used: u64,
    free: u64,
    swap_total: u64,
    swap_used: u64,
    swap_free: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProcessSummary {
    total_count: usize,
    top_cpu_processes: Vec<ProcessInfo>,
    top_memory_processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64,
}

#[derive(Clone)]
struct AppState {
    system: Arc<Mutex<System>>,
}

impl AppState {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            system: Arc::new(Mutex::new(sys)),
        }
    }

    fn get_system_info(&self) -> Result<SystemInfo, Box<dyn std::error::Error>> {
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

async fn get_all_info(data: web::types::State<AppState>) -> HttpResponse {
    match data.get_system_info() {
        Ok(info) => HttpResponse::Ok().json(&Resp::success(info)),
        Err(e) => HttpResponse::Ok().json(&Resp::<()>::error(
            500,
            format!("Internal server error: {}", e),
        )),
    }
}

async fn get_system_overview(data: web::types::State<AppState>) -> HttpResponse {
    match data.get_system_info() {
        Ok(info) => HttpResponse::Ok().json(&Resp::success(info.system)),
        Err(e) => HttpResponse::Ok().json(&Resp::<()>::error(
            500,
            format!("Internal server error: {}", e),
        )),
    }
}

async fn get_cpu_info(data: web::types::State<AppState>) -> HttpResponse {
    match data.get_system_info() {
        Ok(info) => HttpResponse::Ok().json(&Resp::success(info.cpu)),
        Err(e) => HttpResponse::Ok().json(&Resp::<()>::error(
            500,
            format!("Internal server error: {}", e),
        )),
    }
}

async fn get_memory_info(data: web::types::State<AppState>) -> HttpResponse {
    match data.get_system_info() {
        Ok(info) => HttpResponse::Ok().json(&Resp::success(info.memory)),
        Err(e) => HttpResponse::Ok().json(&Resp::<()>::error(
            500,
            format!("Internal server error: {}", e),
        )),
    }
}

async fn get_process_info(data: web::types::State<AppState>) -> HttpResponse {
    match data.get_system_info() {
        Ok(info) => HttpResponse::Ok().json(&Resp::success(info.processes)),
        Err(e) => HttpResponse::Ok().json(&Resp::<()>::error(
            500,
            format!("Internal server error: {}", e),
        )),
    }
}

async fn health_check() -> HttpResponse {
    let health_data = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    });
    HttpResponse::Ok().json(&Resp::success(health_data))
}

#[ntex::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = cfg::Config::from_env();
    let app_state = AppState::new();

    log::info!(
        "Starting server on http://{}:{}",
        config.server_host,
        config.server_port
    );
    log::info!(
        "Using username: {}, password: {}",
        config.username,
        config.password
    );

    HttpServer::new(move || {
        let rate_limiter = ntex_ratelimiter::RateLimiter::new(config.rate_limit, 60);
        let app = App::new()
            .state(app_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(ntex_ratelimiter::RateLimit::new(rate_limiter));

        let auth =
            ntex_basicauth::BasicAuth::with_user(config.username.clone(), config.password.clone())
                .expect("Failed to create basic auth middleware");

        app.service(
            web::scope("/api/v1")
                .route("/health", web::get().to(health_check))
                .wrap(auth)
                .route("/system", web::get().to(get_all_info))
                .route("/system/overview", web::get().to(get_system_overview))
                .route("/system/cpu", web::get().to(get_cpu_info))
                .route("/system/memory", web::get().to(get_memory_info))
                .route("/system/processes", web::get().to(get_process_info)),
        )
    })
    .bind((config.server_host, config.server_port))?
    .run()
    .await?;

    Ok(())
}
