use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use cds_tree_storage::{TreeRepository, NodeRepository, CreateNodeRequest, UpdateNodeRequest};
use crate::state::AppState;
use crate::error::{AppError, ApiResult};

/// POST /api/v1/trees/:tree_id/nodes — Create a new node
pub async fn create_node(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
    Json(req): Json<CreateNodeRequest>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Create node
    let node = NodeRepository::create(&state.db_pool, tree_id, req)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": node.id,
            "tree_id": node.tree_id,
            "label": node.label,
            "depth": node.depth,
            "node_kind": node.node_kind,
            "created_at": node.created_at
        }))
    ))
}

/// GET /api/v1/trees/:tree_id/nodes — List all nodes in a tree
pub async fn list_nodes(
    State(state): State<AppState>,
    Path(tree_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    let nodes = NodeRepository::list_by_tree(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "tree_id": tree_id,
        "count": nodes.len(),
        "nodes": nodes
    })))
}

/// GET /api/v1/trees/:tree_id/nodes/:node_id — Get a specific node
pub async fn get_node(
    State(state): State<AppState>,
    Path((tree_id, node_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Get node and verify it belongs to the tree
    let node = NodeRepository::get_by_id(&state.db_pool, node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(node_id.to_string()))?;

    if node.tree_id != tree_id {
        return Err(AppError::NodeNotFound(node_id.to_string()));
    }

    Ok(Json(serde_json::to_value(&node).unwrap_or_default()))
}

/// PUT /api/v1/trees/:tree_id/nodes/:node_id — Update a node
pub async fn update_node(
    State(state): State<AppState>,
    Path((tree_id, node_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateNodeRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Verify node exists and belongs to tree
    let node = NodeRepository::get_by_id(&state.db_pool, node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(node_id.to_string()))?;

    if node.tree_id != tree_id {
        return Err(AppError::NodeNotFound(node_id.to_string()));
    }

    // Update node
    let updated_node = NodeRepository::update(&state.db_pool, node_id, req)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": updated_node.id,
        "label": updated_node.label,
        "updated_at": updated_node.updated_at
    })))
}

/// DELETE /api/v1/trees/:tree_id/nodes/:node_id — Delete a node (cascade)
pub async fn delete_node(
    State(state): State<AppState>,
    Path((tree_id, node_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Verify node exists and belongs to tree
    let node = NodeRepository::get_by_id(&state.db_pool, node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(node_id.to_string()))?;

    if node.tree_id != tree_id {
        return Err(AppError::NodeNotFound(node_id.to_string()));
    }

    // Delete node (cascade deletes children)
    NodeRepository::delete_cascade(&state.db_pool, node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/trees/:tree_id/nodes/:node_id/children — Get children of a node
pub async fn get_children(
    State(state): State<AppState>,
    Path((tree_id, node_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify tree exists
    TreeRepository::get_by_id(&state.db_pool, tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(tree_id.to_string()))?;

    // Verify node exists
    let node = NodeRepository::get_by_id(&state.db_pool, node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NodeNotFound(node_id.to_string()))?;

    if node.tree_id != tree_id {
        return Err(AppError::NodeNotFound(node_id.to_string()));
    }

    // Get children
    let children = NodeRepository::get_children(&state.db_pool, node_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "parent_id": node_id,
        "count": children.len(),
        "children": children
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_handlers_compile() {
        // Verify handlers are defined
        assert!(true);
    }
}
