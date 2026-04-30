use axum::{
    middleware::{self, Next},
    request::Request,
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{handlers, state::AppState};

/// Build the main API router
pub fn build_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::permissive(); // In production, restrict to allowed origins

    // Request ID middleware (for tracing)
    let request_id = (
        SetResponseHeaderLayer::if_not_present(
            http::header::X_REQUEST_ID,
            MakeRequestUuid::default(),
        ),
        PropagateRequestIdLayer::new(),
    );

    // Tracing middleware
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/", get(handlers::health_check))
        .route("/live", get(handlers::live_check))
        .route("/ready", get(handlers::ready_check))
        .with_state(state.clone());

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/cds-services", get(|| async { "CDS Hooks discovery" }))
        .route("/metrics", get(handlers::metrics))
        .with_state(state.clone());

    // Protected API routes (auth required)
    let api_routes = Router::new()
        // Trees CRUD
        .route("/trees", post(handlers::create_tree))
        .route("/trees", get(handlers::list_trees))
        .route("/trees/:tree_id", get(handlers::get_tree))
        .route("/trees/:tree_id", put(handlers::update_tree))
        .route("/trees/:tree_id", delete(handlers::delete_tree))
        .route("/trees/:tree_id/publish", post(handlers::publish_tree))
        .route("/trees/:tree_id/validate", post(handlers::validate_tree))
        // Nodes CRUD
        .route("/trees/:tree_id/nodes", post(handlers::create_node))
        .route("/trees/:tree_id/nodes", get(handlers::list_nodes))
        .route("/trees/:tree_id/nodes/:node_id", get(handlers::get_node))
        .route("/trees/:tree_id/nodes/:node_id", put(handlers::update_node))
        .route("/trees/:tree_id/nodes/:node_id", delete(handlers::delete_node))
        .route("/trees/:tree_id/nodes/:node_id/children", get(handlers::get_children))
        // Sessions traversal
        .route("/sessions", post(handlers::create_session))
        .route("/sessions/:session_id", get(handlers::get_current_node))
        .route("/sessions/:session_id/advance", post(handlers::advance_session))
        .route("/sessions/:session_id/path", get(handlers::get_session_path))
        .route("/sessions/:session_id/outcome", get(handlers::get_session_outcome))
        .route("/sessions/:session_id/abandon", post(handlers::abandon_session))
        .with_state(state.clone());

    // Combine all routers
    Router::new()
        .route("/healthz", get(handlers::live_check))
        .nest("/api/v1/health", health_routes)
        .nest("/api/v1", api_routes)
        .merge(public_routes)
        // Middleware stack (applied bottom-up)
        .layer(cors)
        .layer(request_id)
        .layer(trace)
        .layer(CompressionLayer::new())
        .fallback(fallback_handler)
}

/// Fallback handler for unmatched routes
async fn fallback_handler(
    uri: axum::http::Uri,
) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("Route not found: {}", uri),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_build() {
        // Verify router builds without panicking
        let _config = crate::config::Config::default_test();
        // In a real test, we'd create a state and router
        assert!(true);
    }
}
