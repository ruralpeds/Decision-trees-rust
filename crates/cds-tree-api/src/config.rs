use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Server binding address (default: 0.0.0.0:3000)
    pub server_host: String,
    pub server_port: u16,

    /// Database connection string
    pub database_url: String,

    /// JWT configuration
    pub jwt_public_key: String,
    pub jwt_issuer: String,

    /// Logging level (default: info)
    pub rust_log: String,

    /// Allowed CORS origins (comma-separated)
    pub allowed_origins: String,

    /// Rate limiting: requests per second per IP
    pub rate_limit_per_ip: u32,

    /// Whether to run database migrations on startup
    pub auto_migrate: bool,

    /// Application version
    pub app_version: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv::dotenv().ok(); // Load .env file if present

        envy::from_env::<Self>()
    }

    /// Create a default configuration (for testing)
    pub fn default_test() -> Self {
        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            database_url: "postgres://user:password@localhost/cds_tree".to_string(),
            jwt_public_key: "mock-key".to_string(),
            jwt_issuer: "https://auth.example.com".to_string(),
            rust_log: "debug".to_string(),
            allowed_origins: "http://localhost:3000,http://localhost:5173".to_string(),
            rate_limit_per_ip: 100,
            auto_migrate: true,
            app_version: "0.1.0".to_string(),
        }
    }

    /// Get the server socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server_host, self.server_port)
            .parse()
            .expect("Invalid server address")
    }

    /// Get list of allowed CORS origins
    pub fn cors_origins(&self) -> Vec<&str> {
        self.allowed_origins.split(',').map(|s| s.trim()).collect()
    }
}

/// JWT claims extracted from tokens
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JwtClaims {
    pub sub: String,           // User ID
    pub iss: String,           // Issuer
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    pub roles: Vec<String>,    // User roles
    pub org_id: Option<String>, // Organization ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default_test();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.app_version, "0.1.0");
    }

    #[test]
    fn test_socket_addr() {
        let config = Config::default_test();
        let addr = config.socket_addr();
        assert_eq!(addr.port(), 3000);
    }

    #[test]
    fn test_cors_origins() {
        let config = Config::default_test();
        let origins = config.cors_origins();
        assert_eq!(origins.len(), 2);
    }
}
