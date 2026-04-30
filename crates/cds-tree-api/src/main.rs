use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;

mod config;
mod error;
mod handlers;
mod middleware;
mod router;
mod state;
mod traversal;

use config::Config;
use middleware::tracing_middleware;
use state::AppState;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let mut config = Config::from_env().unwrap_or_else(|_| {
        eprintln!("Warning: Using default configuration. Set environment variables to override.");
        Config::default_test()
    });

    // Set version
    config.app_version = APP_VERSION.to_string();

    // Initialize tracing
    tracing_middleware::init_json_tracing(&config.rust_log);

    tracing::info!(
        version = %config.app_version,
        host = %config.server_host,
        port = config.server_port,
        "Starting cds-tree-api"
    );

    // Create application state
    let state = AppState::new(config.clone())
        .await
        .expect("Failed to initialize application state");

    // Run migrations if configured
    if config.auto_migrate {
        tracing::info!("Running database migrations...");
        cds_tree_storage::run_migrations(&state.db_pool)
            .await
            .expect("Failed to run migrations");
        tracing::info!("Migrations completed successfully");
    }

    // Initialize metrics
    handlers::metrics::init_metrics(state.metrics_registry.as_ref()).ok();

    // Build router
    let app = router::build_router(state);

    // Get socket address
    let addr = config.socket_addr();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind socket");

    tracing::info!(
        address = %addr,
        "Server listening"
    );

    // Run server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
