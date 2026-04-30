use tracing::Level;
use tracing_subscriber::fmt;

/// Initialize tracing with JSON output
pub fn init_tracing(rust_log: &str) {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_env_filter(rust_log)
        .with_writer(std::io::stdout)
        .init();
}

/// Initialize tracing with JSON format (for production)
pub fn init_json_tracing(rust_log: &str) {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(Level::INFO)
        .with_env_filter(rust_log)
        .init();
}
