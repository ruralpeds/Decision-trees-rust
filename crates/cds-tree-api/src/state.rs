use crate::config::Config;
use sqlx::PgPool;
use std::sync::Arc;
use prometheus::Registry;

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    /// Configuration
    pub config: Arc<Config>,

    /// PostgreSQL connection pool
    pub db_pool: PgPool,

    /// Prometheus metrics registry
    pub metrics_registry: Arc<Registry>,
}

impl AppState {
    /// Create a new application state
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Create connection pool
        let db_pool = PgPool::connect(&config.database_url).await?;

        // Create metrics registry
        let metrics_registry = Arc::new(Registry::new());

        Ok(Self {
            config: Arc::new(config),
            db_pool,
            metrics_registry,
        })
    }

    /// Health check: verify database connectivity
    pub async fn health_check(&self) -> Result<HealthStatus, String> {
        // Try a simple query
        sqlx::query("SELECT 1")
            .execute(&self.db_pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(HealthStatus {
            status: "healthy".to_string(),
            version: self.config.app_version.clone(),
            database: true,
        })
    }
}

/// Health check response
#[derive(serde::Serialize, Debug)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub database: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        let health = HealthStatus {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            database: true,
        };
        assert_eq!(health.status, "healthy");
    }
}
