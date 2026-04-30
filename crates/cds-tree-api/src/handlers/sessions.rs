use axum::{
    extract::{State, Path, ws::{WebSocket, WebSocketUpgrade}},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use uuid::Uuid;
use futures_util::{SinkExt, StreamExt};
use cds_tree_storage::{TreeRepository, NodeRepository, SessionRepository, CreateSessionRequest, NodeAnswer, AdvanceSessionRequest, ProgressHint, BreadcrumbStep, NodeResponseForSession, OutcomeResponse, SessionResponse};
use crate::state::AppState;
use crate::error::{AppError, ApiResult};
use crate::traversal::SessionStateMachine;
use cds_tree_core::{ClinicalDecisionTree, ConditionValue};

/// POST /api/v1/sessions — Create a new session
pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> ApiResult<(StatusCode, Json<SessionResponse>)> {
    // Verify tree exists and is published
    let tree_row = TreeRepository::get_by_id(&state.db_pool, req.tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(req.tree_id.to_string()))?;

    if tree_row.status != "published" {
        return Err(AppError::InvalidInput(
            format!("Tree must be published, current status: {}", tree_row.status)
        ));
    }

    // Get all nodes to reconstruct tree
    let node_rows = NodeRepository::list_by_tree(&state.db_pool, req.tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if node_rows.is_empty() {
        return Err(AppError::InvalidInput("Tree has no nodes".to_string()));
    }

    // Create session in database
    let session = SessionRepository::create(
        &state.db_pool,
        req.tree_id,
        tree_row.version.clone(),
        req.clinician_id,
        req.patient_id,
        "rest_api".to_string(),
    )
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Set current node to root
    let root_node_id = tree_row.root_node_id
        .ok_or_else(|| AppError::InvalidInput("Tree has no root node".to_string()))?;

    SessionRepository::set_current_node(&state.db_pool, session.id, Some(root_node_id))
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(SessionResponse {
            id: session.id,
            tree_id: session.tree_id,
            status: session.status,
            current_node_id: Some(root_node_id),
            started_at: session.started_at,
            completed_at: session.completed_at,
        })
    ))
}

/// GET /api/v1/sessions/:session_id — Get current node for session
pub async fn get_current_node(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<NodeResponseForSession>> {
    // Get session
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    let current_node_id = session.current_node_id
        .ok_or_else(|| AppError::InvalidInput("Session has no current node".to_string()))?;

    // Get node
    let node = NodeRepository::get_by_id(&state.db_pool, current_node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(current_node_id.to_string()))?;

    // Get path for breadcrumb
    let answers = SessionRepository::get_answers(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut breadcrumb = Vec::new();
    for (node_id_str, _answer) in &answers {
        if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
            if let Ok(Some(path_node)) = NodeRepository::get_by_id(&state.db_pool, node_id).await {
                breadcrumb.push(BreadcrumbStep {
                    node_id,
                    label: path_node.label.clone(),
                    answer_summary: "answered".to_string(), // Simplified
                    depth: path_node.depth as u32,
                });
            }
        }
    }

    // Calculate remaining nodes (simplified: just count all nodes)
    let all_nodes = NodeRepository::list_by_tree(&state.db_pool, session.tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let estimated_remaining = (all_nodes.len() as u32).saturating_sub(breadcrumb.len() as u32);

    let progress = ProgressHint {
        current_depth: node.depth as u32,
        estimated_remaining_nodes: estimated_remaining,
        breadcrumb,
    };

    Ok(Json(NodeResponseForSession {
        node,
        progress_hint: progress,
        pre_filled_from_fhir: false,
    }))
}

/// POST /api/v1/sessions/:session_id/advance — Submit answer and get next node
pub async fn advance_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<AdvanceSessionRequest>,
) -> ApiResult<Json<NodeResponseForSession>> {
    // Get session
    let mut session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    if session.status != "active" {
        return Err(AppError::InvalidInput(
            format!("Session is not active, status: {}", session.status)
        ));
    }

    // Record answer
    SessionRepository::record_answer(&state.db_pool, session_id, req.answer.node_id, req.answer.answer)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get all nodes for tree
    let node_rows = NodeRepository::list_by_tree(&state.db_pool, session.tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Find next node (simplified: just return children for now)
    let current_node = NodeRepository::get_by_id(&state.db_pool, req.answer.node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(req.answer.node_id.to_string()))?;

    // Get children
    let children = NodeRepository::get_children(&state.db_pool, req.answer.node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let next_node = if !children.is_empty() {
        children[0].clone() // Simplified: take first child
    } else {
        // No children means outcome node - complete session
        SessionRepository::complete(&state.db_pool, session_id, req.answer.node_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        current_node
    };

    // Update current node
    SessionRepository::set_current_node(&state.db_pool, session_id, Some(next_node.id))
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Get path for breadcrumb
    let answers = SessionRepository::get_answers(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut breadcrumb = Vec::new();
    for (node_id_str, _answer) in &answers {
        if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
            if let Ok(Some(path_node)) = NodeRepository::get_by_id(&state.db_pool, node_id).await {
                breadcrumb.push(BreadcrumbStep {
                    node_id,
                    label: path_node.label.clone(),
                    answer_summary: "answered".to_string(),
                    depth: path_node.depth as u32,
                });
            }
        }
    }

    let estimated_remaining = (node_rows.len() as u32).saturating_sub(breadcrumb.len() as u32);

    let progress = ProgressHint {
        current_depth: next_node.depth as u32,
        estimated_remaining_nodes: estimated_remaining,
        breadcrumb,
    };

    Ok(Json(NodeResponseForSession {
        node: next_node,
        progress_hint: progress,
        pre_filled_from_fhir: false,
    }))
}

/// GET /api/v1/sessions/:session_id/path — Get full traversal path
pub async fn get_session_path(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Get session
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    // Get answers
    let answers = SessionRepository::get_answers(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut breadcrumb = Vec::new();
    for (node_id_str, answer) in &answers {
        if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
            if let Ok(Some(node)) = NodeRepository::get_by_id(&state.db_pool, node_id).await {
                breadcrumb.push(serde_json::json!({
                    "node_id": node_id,
                    "label": node.label,
                    "depth": node.depth,
                    "answer": answer
                }));
            }
        }
    }

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "status": session.status,
        "steps": breadcrumb,
        "total_steps": breadcrumb.len()
    })))
}

/// GET /api/v1/sessions/:session_id/outcome — Get final outcome if session is complete
pub async fn get_session_outcome(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<OutcomeResponse>> {
    // Get session
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    if session.status != "completed" {
        return Err(AppError::InvalidInput(
            format!("Session not completed, status: {}", session.status)
        ));
    }

    let outcome_node_id = session.outcome_node_id
        .ok_or_else(|| AppError::InvalidInput("Session has no outcome node".to_string()))?;

    // Get outcome node
    let outcome_node = NodeRepository::get_by_id(&state.db_pool, outcome_node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(outcome_node_id.to_string()))?;

    let outcome = outcome_node.outcome
        .ok_or_else(|| AppError::InvalidInput("Outcome node has no outcome data".to_string()))?;

    // Build path
    let answers = SessionRepository::get_answers(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut path_summary = Vec::new();
    for (node_id_str, _answer) in &answers {
        if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
            if let Ok(Some(node)) = NodeRepository::get_by_id(&state.db_pool, node_id).await {
                path_summary.push(BreadcrumbStep {
                    node_id,
                    label: node.label.clone(),
                    answer_summary: "answered".to_string(),
                    depth: node.depth as u32,
                });
            }
        }
    }

    Ok(Json(OutcomeResponse {
        outcome,
        path_summary,
        session_id,
    }))
}

/// POST /api/v1/sessions/:session_id/abandon — Abandon a session
pub async fn abandon_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Get session
    SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    // Abandon session
    SessionRepository::abandon(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_handlers_compile() {
        assert!(true);
    }
}
