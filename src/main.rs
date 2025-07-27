use axum::{Router, extract::State, middleware, response::Json, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;

mod api;
mod cfg;
mod macros;
mod middlewares;
mod models;
mod service;

use api::Resp;
use service::AppState;

#[derive(Clone)]
struct AuthState {
    expected_credentials: String,
}

// Handler functions
async fn get_all_info(State(app_state): State<Arc<AppState>>) -> Json<Resp<models::SystemInfo>> {
    match app_state.get_system_info() {
        Ok(info) => json_resp_ok!(info),
        Err(e) => json_resp_err!(500, format!("{}", e)),
    }
}

async fn get_system_overview(
    State(app_state): State<Arc<AppState>>,
) -> Json<Resp<models::SystemOverview>> {
    match app_state.get_system_info() {
        Ok(info) => Json(Resp::success(info.system)),
        Err(e) => Json(Resp::error(500, format!("{}", e))),
    }
}

async fn get_cpu_info(State(app_state): State<Arc<AppState>>) -> Json<Resp<models::CpuInfo>> {
    match app_state.get_system_info() {
        Ok(info) => Json(Resp::success(info.cpu)),
        Err(e) => Json(Resp::error(500, format!("{}", e))),
    }
}

async fn get_memory_info(State(app_state): State<Arc<AppState>>) -> Json<Resp<models::MemoryInfo>> {
    match app_state.get_system_info() {
        Ok(info) => Json(Resp::success(info.memory)),
        Err(e) => Json(Resp::error(500, format!("{}", e))),
    }
}

async fn get_process_info(
    State(app_state): State<Arc<AppState>>,
) -> Json<Resp<models::ProcessSummary>> {
    match app_state.get_system_info() {
        Ok(info) => Json(Resp::success(info.processes)),
        Err(e) => Json(Resp::error(500, format!("{}", e))),
    }
}

async fn health_check() -> Json<Resp<serde_json::Value>> {
    use chrono::Utc;
    let health_data = serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    });
    Json(Resp::success(health_data))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config = cfg::Config::from_env();
    let app_state = Arc::new(AppState::new());

    // Pre-compute auth credentials for performance
    let auth_state = AuthState {
        expected_credentials: format!("{}:{}", config.username, config.password),
    };

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

    // Create rate limiting governor
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(config.rate_limit as u64 / 60)
            .burst_size(config.rate_limit as u32)
            .finish()
            .unwrap(),
    );

    // Build the application
    let app = Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/system", get(get_all_info))
        .route("/api/v1/system/overview", get(get_system_overview))
        .route("/api/v1/system/cpu", get(get_cpu_info))
        .route("/api/v1/system/memory", get(get_memory_info))
        .route("/api/v1/system/processes", get(get_process_info))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(GovernorLayer {
                    config: governor_conf,
                })
                .layer(middleware::from_fn_with_state(
                    auth_state.clone(),
                    middlewares::basic_auth,
                ))
                .into_inner(),
        )
        .with_state(app_state);

    // Create socket address
    let addr = SocketAddr::from((
        config.server_host.parse::<std::net::IpAddr>()?,
        config.server_port,
    ));

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
