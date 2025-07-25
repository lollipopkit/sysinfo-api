#[derive(Debug, Clone)]
pub(crate) struct Config {
    pub(crate) server_host: String,
    pub(crate) server_port: u16,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) rate_limit: usize,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        Self {
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            username: std::env::var("USERNAME").unwrap_or("admin".to_string()),
            password: std::env::var("PASSWORD").unwrap_or("password123".to_string()),
            rate_limit: std::env::var("RATE_LIMIT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        }
    }
}
