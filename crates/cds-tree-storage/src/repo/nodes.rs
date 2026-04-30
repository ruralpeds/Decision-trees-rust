use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{NodeRow, CreateNodeRequest, UpdateNodeRequest};
use crate::error::StorageResult;

/// Repository for node operations
pub struct NodeRepository;

impl NodeRepository {
    /// Create a new node
    pub async fn create(
        pool: &PgPool,
        tree_id: Uuid,
        req: CreateNodeRequest,
    ) -> StorageResult<NodeRow> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let node_kind = req.node_kind.unwrap_or_else(|| "question".to_string());
        let required = req.required.unwrap_or(true);

        // Calculate depth
        let depth = if let Some(parent_id) = req.parent_id {
            let parent = sqlx::query("SELECT depth FROM nodes WHERE id = $1")
                .bind(parent_id)
                .fetch_one(pool)
                .await?;
            let parent_depth: i32 = parent.get("depth");
            parent_depth + 1
        } else {
            0
        };

        let node = sqlx::query_as::<_, NodeRow>(
            r#"
            INSERT INTO nodes (
                id, tree_id, parent_id, depth, node_kind, label, description,
                input, aria_label, required, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(tree_id)
        .bind(req.parent_id)
        .bind(depth)
        .bind(&node_kind)
        .bind(&req.label)
        .bind(&req.description)
        .bind(&req.input)
        .bind(&req.aria_label)
        .bind(required)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(node)
    }

    /// Get a node by ID
    pub async fn get_by_id(
        pool: &PgPool,
        node_id: Uuid,
    ) -> StorageResult<Option<NodeRow>> {
        let node = sqlx::query_as::<_, NodeRow>(
            "SELECT * FROM nodes WHERE id = $1"
        )
        .bind(node_id)
        .fetch_optional(pool)
        .await?;

        Ok(node)
    }

    /// Get all nodes for a tree
    pub async fn list_by_tree(
        pool: &PgPool,
        tree_id: Uuid,
    ) -> StorageResult<Vec<NodeRow>> {
        let nodes = sqlx::query_as::<_, NodeRow>(
            "SELECT * FROM nodes WHERE tree_id = $1 ORDER BY depth ASC, created_at ASC"
        )
        .bind(tree_id)
        .fetch_all(pool)
        .await?;

        Ok(nodes)
    }

    /// Get children of a node
    pub async fn get_children(
        pool: &PgPool,
        parent_id: Uuid,
    ) -> StorageResult<Vec<NodeRow>> {
        let nodes = sqlx::query_as::<_, NodeRow>(
            "SELECT * FROM nodes WHERE parent_id = $1 ORDER BY created_at ASC"
        )
        .bind(parent_id)
        .fetch_all(pool)
        .await?;

        Ok(nodes)
    }

    /// Get root node of a tree
    pub async fn get_root(
        pool: &PgPool,
        tree_id: Uuid,
    ) -> StorageResult<Option<NodeRow>> {
        let node = sqlx::query_as::<_, NodeRow>(
            "SELECT * FROM nodes WHERE tree_id = $1 AND parent_id IS NULL"
        )
        .bind(tree_id)
        .fetch_optional(pool)
        .await?;

        Ok(node)
    }

    /// Update a node
    pub async fn update(
        pool: &PgPool,
        node_id: Uuid,
        req: UpdateNodeRequest,
    ) -> StorageResult<NodeRow> {
        let now = Utc::now();

        let node = sqlx::query_as::<_, NodeRow>(
            r#"
            UPDATE nodes
            SET
                label = COALESCE($2, label),
                description = COALESCE($3, description),
                input = COALESCE($4, input),
                node_kind = COALESCE($5, node_kind),
                aria_label = COALESCE($6, aria_label),
                required = COALESCE($7, required),
                updated_at = $8
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(node_id)
        .bind(&req.label)
        .bind(&req.description)
        .bind(&req.input)
        .bind(&req.node_kind)
        .bind(&req.aria_label)
        .bind(req.required)
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(node)
    }

    /// Update children edges for a node
    pub async fn set_children(
        pool: &PgPool,
        node_id: Uuid,
        children: serde_json::Value,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE nodes SET children = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(&children)
        .bind(&now)
        .bind(node_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update outcome for a node
    pub async fn set_outcome(
        pool: &PgPool,
        node_id: Uuid,
        outcome: serde_json::Value,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE nodes SET outcome = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(&outcome)
        .bind(&now)
        .bind(node_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete a node and its descendants
    pub async fn delete_cascade(
        pool: &PgPool,
        node_id: Uuid,
    ) -> StorageResult<()> {
        // SQLx will handle CASCADE delete
        sqlx::query("DELETE FROM nodes WHERE id = $1")
            .bind(node_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get count of nodes in a tree
    pub async fn count_by_tree(
        pool: &PgPool,
        tree_id: Uuid,
    ) -> StorageResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM nodes WHERE tree_id = $1")
            .bind(tree_id)
            .fetch_one(pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_repository_methods_compile() {
        // Just verify the methods are defined and compile
        assert!(true);
    }
}
