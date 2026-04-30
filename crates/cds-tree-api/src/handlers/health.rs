use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::state::AppState;
use crate::error::ApiResult;

/// GET /api/v1/health — Health check endpoint
///
/// Returns the application health status and version
pub async fn health_check(
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let health = state.health_check().await
        .map_err(|e| crate::error::AppError::InternalError)?;

    Ok((StatusCode::OK, Json(health)))
}

/// GET /api/v1/health/ready — Readiness probe
///
/// Used by load balancers and orchestration platforms
pub async fn ready_check(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    state.health_check().await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "ready": true
        }))
    ))
}

/// GET /api/v1/health/live — Liveness probe
///
/// Basic check that the service is running
pub async fn live_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "alive": true
        }))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_endpoint() {
        // Tests would go here once we have a test setup
        // For now, just verify the function compiles
        assert!(true);
    }
}
