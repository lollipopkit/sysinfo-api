use std::sync::Arc;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;

use crate::service::AppState;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProcessListRequest {
    /// Number of top processes to return (default: 10, max: 50)
    pub limit: Option<usize>,
    /// Sort by 'cpu' or 'memory' (default: cpu)
    pub sort_by: Option<String>,
}

#[derive(Clone)]
pub struct SysInfoMcp {
    app_state: Arc<AppState>,
    tool_router: ToolRouter<SysInfoMcp>,
}

#[tool_router]
impl SysInfoMcp {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            app_state,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get complete system information including CPU, memory, and processes")]
    async fn get_system_info(&self) -> Result<CallToolResult, McpError> {
        match self.app_state.get_system_info() {
            Ok(info) => {
                let json_str = serde_json::to_string_pretty(&info)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(json_str)]))
            }
            Err(e) => Err(McpError::internal_error(format!("Failed to get system info: {}", e), None)),
        }
    }

    #[tool(description = "Get system overview information (OS, kernel, uptime, etc.)")]
    async fn get_system_overview(&self) -> Result<CallToolResult, McpError> {
        match self.app_state.get_system_info() {
            Ok(info) => {
                let json_str = serde_json::to_string_pretty(&info.system)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(json_str)]))
            }
            Err(e) => Err(McpError::internal_error(format!("Failed to get system overview: {}", e), None)),
        }
    }

    #[tool(description = "Get CPU information including usage and core details")]
    async fn get_cpu_info(&self) -> Result<CallToolResult, McpError> {
        match self.app_state.get_system_info() {
            Ok(info) => {
                let json_str = serde_json::to_string_pretty(&info.cpu)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(json_str)]))
            }
            Err(e) => Err(McpError::internal_error(format!("Failed to get CPU info: {}", e), None)),
        }
    }

    #[tool(description = "Get memory information including RAM and swap usage")]
    async fn get_memory_info(&self) -> Result<CallToolResult, McpError> {
        match self.app_state.get_system_info() {
            Ok(info) => {
                let json_str = serde_json::to_string_pretty(&info.memory)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(json_str)]))
            }
            Err(e) => Err(McpError::internal_error(format!("Failed to get memory info: {}", e), None)),
        }
    }

    #[tool(description = "Get process information with top CPU and memory consumers")]
    async fn get_processes(&self, Parameters(req): Parameters<ProcessListRequest>) -> Result<CallToolResult, McpError> {
        match self.app_state.get_system_info() {
            Ok(mut info) => {
                let limit = req.limit.unwrap_or(10).min(50);
                let sort_by = req.sort_by.unwrap_or_else(|| "cpu".to_string());

                // Truncate process lists based on requested limit
                if sort_by == "memory" {
                    info.processes.top_cpu_processes.truncate(limit);
                    info.processes.top_memory_processes.truncate(limit);
                } else {
                    info.processes.top_cpu_processes.truncate(limit);
                    info.processes.top_memory_processes.truncate(limit);
                }

                let json_str = serde_json::to_string_pretty(&info.processes)
                    .map_err(|e| McpError::internal_error(format!("Serialization error: {}", e), None))?;
                Ok(CallToolResult::success(vec![Content::text(json_str)]))
            }
            Err(e) => Err(McpError::internal_error(format!("Failed to get process info: {}", e), None)),
        }
    }

    #[tool(description = "Get current system timestamp")]
    async fn get_timestamp(&self) -> Result<CallToolResult, McpError> {
        use chrono::Utc;
        let timestamp = Utc::now();
        Ok(CallToolResult::success(vec![Content::text(timestamp.to_rfc3339())]))
    }
}

#[tool_handler]
impl ServerHandler for SysInfoMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides system information monitoring tools. You can get comprehensive system data including CPU usage, memory usage, running processes, and system overview information. Use the tools to monitor system performance and resource utilization.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::resource_not_found(
            "resource_not_found",
            Some(json!({
                "uri": uri
            })),
        ))
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, .. }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params(format!("Prompt '{}' not found", name), None))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "MCP initialize from http server");
        }
        Ok(self.get_info())
    }
}