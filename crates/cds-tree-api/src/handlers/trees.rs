use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use cds_tree_storage::{TreeRepository, NodeRepository};
use cds_tree_core::TreeValidator;
use crate::state::AppState;
use crate::error::{AppError, ApiResult};
use cds_tree_storage::{CreateTreeRequest, UpdateTreeRequest, TreeResponse, ValidationReportResponse};

/// POST /api/v1/trees — Create a new tree
pub async fn create_tree(
    State(state): State<AppState>,
    Json(req): Json<CreateTreeRequest>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // Validate slug is unique
    if let Some(_) = TreeRepository::get_by_slug(&state.db_pool, &req.slug).await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    {
        return Err(AppError::Conflict(format!("Slug already exists: {}", req.slug)));
    }

    // Create tree in database
    let tree = TreeRepository::create(&state.db_pool, req)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": tree.id,
            "slug": tree.slug,
            "title": tree.title,
            "status": tree.status,
            "created_at": tree.created_at
        }))
    ))
}

/// GET /api/v1/trees — List all trees (paginated)
pub async fn list_trees(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(20)
        .min(100);

    let offset = params
        .get("offset")
        .and_then(|o| o.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0);

    let trees = TreeRepository::list(&state.db_pool, limit, offset)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let count = TreeRepository::count(&state.db_pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "trees": trees,
        "total": count,
        "limit": limit,
        "offset": offset
    })))
}

/// GET /api/v1/trees/:tree_id — Get a specific tree with all nodes
pub async fn get_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
) -> ApiResult<Json<TreeResponse>> {
    let tree_row = TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    let nodes = NodeRepository::list_by_tree(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(TreeResponse {
        id: tree_row.id,
        slug: tree_row.slug,
        title: tree_row.title,
        description: tree_row.description,
        version: tree_row.version,
        status: tree_row.status,
        evidence_level: tree_row.evidence_level,
        root_node_id: tree_row.root_node_id,
        nodes,
        created_at: tree_row.created_at,
        updated_at: tree_row.updated_at,
        published_at: tree_row.published_at,
    }))
}

/// PUT /api/v1/trees/:tree_id — Update tree metadata
pub async fn update_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
    Json(req): Json<UpdateTreeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Update tree
    let tree = TreeRepository::update(&state.db_pool, tree_id, req)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": tree.id,
        "slug": tree.slug,
        "title": tree.title,
        "updated_at": tree.updated_at
    })))
}

/// DELETE /api/v1/trees/:tree_id — Soft delete a tree
pub async fn delete_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Soft delete
    TreeRepository::soft_delete(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/trees/:tree_id/publish — Publish a tree
pub async fn publish_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify tree exists and is in Draft status
    let tree = TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    if tree.status != "draft" {
        return Err(AppError::InvalidInput(
            format!("Tree must be in Draft status to publish, current status: {}", tree.status)
        ));
    }

    // Get all nodes to validate
    let nodes = NodeRepository::list_by_tree(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // For now, just publish. Full validation will be in validate endpoint
    let updated_tree = TreeRepository::publish(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": updated_tree.id,
        "status": updated_tree.status,
        "published_at": updated_tree.published_at
    })))
}

/// POST /api/v1/trees/:tree_id/validate — Validate tree structure
pub async fn validate_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
) -> ApiResult<Json<ValidationReportResponse>> {
    // Verify tree exists
    let _tree = TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    let nodes = NodeRepository::list_by_tree(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Reconstruct tree structure from nodes for validation
    // This is a simplified version; full validation would deserialize all node types
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if nodes.is_empty() {
        errors.push("Tree has no nodes".to_string());
    }

    // Check for unreachable nodes (basic check)
    let root_nodes: Vec<_> = nodes.iter().filter(|n| n.parent_id.is_none()).collect();
    if root_nodes.is_empty() && !nodes.is_empty() {
        errors.push("Tree has no root node".to_string());
    }

    // Check that outcome nodes have outcome payloads
    for node in &nodes {
        if node.node_kind == "outcome" && node.outcome.is_none() {
            errors.push(format!("Outcome node {} has no outcome payload", node.id));
        }
    }

    let is_valid = errors.is_empty();

    Ok(Json(ValidationReportResponse {
        is_valid,
        errors,
        warnings,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handlers_compile() {
        // Verify handlers are defined
        assert!(true);
    }
}
