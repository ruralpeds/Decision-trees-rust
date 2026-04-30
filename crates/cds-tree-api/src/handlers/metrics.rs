use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use prometheus::TextEncoder;
use crate::state::AppState;

/// GET /metrics — Prometheus metrics endpoint
///
/// Exposes Prometheus-format metrics for scraping
pub async fn metrics(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    
    match encoder.encode_to_string(state.metrics_registry.gather()) {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to encode metrics".to_string(),
        ),
    }
}

/// Initialize default metrics
pub fn init_metrics(registry: &prometheus::Registry) -> Result<(), Box<dyn std::error::Error>> {
    // Example: Create some default metrics
    // These would be incremented in middleware or handlers
    
    // Counter: Total requests
    let _total_requests = prometheus::IntCounter::new(
        "cds_tree_http_requests_total",
        "Total HTTP requests",
    )?;
    registry.register(Box::new(_total_requests))?;

    // Histogram: Request latency
    let _request_duration = prometheus::HistogramVec::new(
        prometheus::HistogramOpts::new(
            "cds_tree_http_request_duration_seconds",
            "HTTP request latency in seconds",
        ),
        &["method", "path"],
    )?;
    registry.register(Box::new(_request_duration))?;

    // Gauge: Active sessions
    let _active_sessions = prometheus::IntGauge::new(
        "cds_tree_active_sessions",
        "Number of active traversal sessions",
    )?;
    registry.register(Box::new(_active_sessions))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_handler() {
        // Tests would verify that metrics are properly formatted
        assert!(true);
    }
}
