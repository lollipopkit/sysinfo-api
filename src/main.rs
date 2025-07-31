use axum::{Router, extract::State, middleware, response::Json, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rmcp::{ServiceExt, transport::{stdio, streamable_http_server::{StreamableHttpService, session::local::LocalSessionManager}}};
use tracing_subscriber::{self, EnvFilter};

mod api;
mod cfg;
mod macros;
mod mcp;
mod middlewares;
mod models;
mod service;

use api::Resp;
use service::AppState;
use mcp::SysInfoMcp;
use cfg::McpMode;

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
    
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .init();

    let config = cfg::Config::from_env();
    let app_state = Arc::new(AppState::new());

    match config.mcp_mode {
        McpMode::Stdio => {
            run_mcp_stdio(app_state).await
        }
        McpMode::Http => {
            run_mcp_http(app_state, config).await
        }
        McpMode::Both => {
            let config_clone = config.clone();
            let (rest_result, mcp_result) = tokio::try_join!(
                tokio::spawn(run_rest_server(app_state.clone(), config.clone())),
                tokio::spawn(run_mcp_http(app_state, config_clone)),
            )?;
            rest_result?;
            mcp_result?;
            Ok(())
        }
        McpMode::RestOnly => {
            run_rest_server(app_state, config).await
        }
    }
}

async fn run_mcp_stdio(app_state: Arc<AppState>) -> anyhow::Result<()> {
    tracing::info!("Starting MCP server in stdio mode");
    
    let service = SysInfoMcp::new(app_state)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("MCP stdio serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}

async fn run_mcp_http(app_state: Arc<AppState>, config: cfg::Config) -> anyhow::Result<()> {
    tracing::info!("Starting MCP server in HTTP mode on port {}", config.mcp_port);
    
    let service = TowerToHyperService::new(StreamableHttpService::new(
        {
            let app_state = app_state.clone();
            move || Ok(SysInfoMcp::new(app_state.clone()))
        },
        LocalSessionManager::default().into(),
        Default::default(),
    ));

    let mcp_addr = format!("{}:{}", config.server_host, config.mcp_port);
    let listener = tokio::net::TcpListener::bind(&mcp_addr).await?;
    
    loop {
        let io = tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            accept = listener.accept() => {
                TokioIo::new(accept?.0)
            }
        };
        let service = service.clone();
        tokio::spawn(async move {
            let _result = Builder::new(TokioExecutor::default())
                .serve_connection(io, service)
                .await;
        });
    }
    Ok(())
}

async fn run_rest_server(app_state: Arc<AppState>, config: cfg::Config) -> anyhow::Result<()> {
    // Pre-compute auth credentials for performance
    let auth_state = AuthState {
        expected_credentials: format!("{}:{}", config.username, config.password),
    };

    tracing::info!(
        "Starting REST API server on http://{}:{}",
        config.server_host,
        config.server_port
    );
    tracing::info!(
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
