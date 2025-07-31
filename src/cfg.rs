#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) server_host: String,
    pub(crate) server_port: u16,
    pub(crate) mcp_port: u16,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) rate_limit: usize,
    pub(crate) mcp_mode: McpMode,
}

#[derive(Debug, Clone)]
pub(crate) enum McpMode {
    Stdio,
    Http,
    Both,
    RestOnly,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        let mcp_mode = match std::env::var("MCP_MODE").unwrap_or_else(|_| "both".to_string()).to_lowercase().as_str() {
            "stdio" => McpMode::Stdio,
            "http" => McpMode::Http,
            "both" => McpMode::Both,
            "rest-only" | "rest_only" => McpMode::RestOnly,
            _ => McpMode::Both,
        };

        Self {
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            mcp_port: std::env::var("MCP_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .expect("MCP_PORT must be a valid port number"),
            username: std::env::var("AUTH_USERNAME").unwrap_or("admin".to_string()),
            password: std::env::var("AUTH_PASSWORD").unwrap_or("password123".to_string()),
            rate_limit: std::env::var("RATE_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            mcp_mode,
        }
    }
}
