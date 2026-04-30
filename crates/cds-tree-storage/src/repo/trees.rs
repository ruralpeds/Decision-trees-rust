use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{TreeRow, CreateTreeRequest, UpdateTreeRequest};
use crate::error::StorageResult;

/// Repository for tree operations
pub struct TreeRepository;

impl TreeRepository {
    /// Create a new tree in Draft status
    pub async fn create(
        pool: &PgPool,
        req: CreateTreeRequest,
    ) -> StorageResult<TreeRow> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let version = req.version.unwrap_or_else(|| "0.1.0".to_string());
        let evidence_level = req.evidence_level.unwrap_or_else(|| "expert_consensus".to_string());
        let review_cycle_months = req.review_cycle_months.unwrap_or(12);

        let tree = sqlx::query_as::<_, TreeRow>(
            r#"
            INSERT INTO trees (id, slug, title, description, version, status, evidence_level, review_cycle_months, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&req.slug)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&version)
        .bind(&evidence_level)
        .bind(review_cycle_months)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(tree)
    }

    /// Get a tree by ID (including soft-deleted)
    pub async fn get_by_id(
        pool: &PgPool,
        tree_id: Uuid,
    ) -> StorageResult<Option<TreeRow>> {
        let tree = sqlx::query_as::<_, TreeRow>(
            "SELECT * FROM trees WHERE id = $1"
        )
        .bind(tree_id)
        .fetch_optional(pool)
        .await?;

        Ok(tree)
    }

    /// Get a tree by slug (not soft-deleted)
    pub async fn get_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> StorageResult<Option<TreeRow>> {
        let tree = sqlx::query_as::<_, TreeRow>(
            "SELECT * FROM trees WHERE slug = $1 AND deleted_at IS NULL"
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        Ok(tree)
    }

    /// List all non-deleted trees with pagination
    pub async fn list(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<TreeRow>> {
        let trees = sqlx::query_as::<_, TreeRow>(
            "SELECT * FROM trees WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(trees)
    }

    /// List trees by status
    pub async fn list_by_status(
        pool: &PgPool,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<TreeRow>> {
        let trees = sqlx::query_as::<_, TreeRow>(
            "SELECT * FROM trees WHERE status = $1 AND deleted_at IS NULL ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(trees)
    }

    /// Update tree metadata
    pub async fn update(
        pool: &PgPool,
        tree_id: Uuid,
        req: UpdateTreeRequest,
    ) -> StorageResult<TreeRow> {
        let now = Utc::now();

        let tree = sqlx::query_as::<_, TreeRow>(
            r#"
            UPDATE trees
            SET
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                version = COALESCE($4, version),
                evidence_level = COALESCE($5, evidence_level),
                review_cycle_months = COALESCE($6, review_cycle_months),
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(tree_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.version)
        .bind(&req.evidence_level)
        .bind(req.review_cycle_months)
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(tree)
    }

    /// Soft delete a tree
    pub async fn soft_delete(
        pool: &PgPool,
        tree_id: Uuid,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE trees SET deleted_at = $1, updated_at = $1 WHERE id = $2"
        )
        .bind(&now)
        .bind(tree_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Publish a tree (transition from Draft to Published)
    pub async fn publish(
        pool: &PgPool,
        tree_id: Uuid,
    ) -> StorageResult<TreeRow> {
        let now = Utc::now();

        let tree = sqlx::query_as::<_, TreeRow>(
            r#"
            UPDATE trees
            SET status = 'published', published_at = $1, updated_at = $1, last_reviewed_at = $1
            WHERE id = $2 AND status = 'draft'
            RETURNING *
            "#
        )
        .bind(&now)
        .bind(tree_id)
        .fetch_one(pool)
        .await?;

        Ok(tree)
    }

    /// Clone a tree as a new Draft
    pub async fn clone(
        pool: &PgPool,
        source_tree_id: Uuid,
        new_slug: String,
        new_title: String,
    ) -> StorageResult<TreeRow> {
        let new_id = Uuid::new_v4();
        let now = Utc::now();

        let tree = sqlx::query_as::<_, TreeRow>(
            r#"
            INSERT INTO trees (id, slug, title, description, version, status, evidence_level, review_cycle_months, created_at, updated_at)
            SELECT $1, $2, $3, description, version, 'draft', evidence_level, review_cycle_months, $4, $5
            FROM trees
            WHERE id = $6
            RETURNING *
            "#
        )
        .bind(new_id)
        .bind(&new_slug)
        .bind(&new_title)
        .bind(&now)
        .bind(&now)
        .bind(source_tree_id)
        .fetch_one(pool)
        .await?;

        Ok(tree)
    }

    /// Set root node ID for a tree
    pub async fn set_root_node(
        pool: &PgPool,
        tree_id: Uuid,
        root_node_id: Uuid,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE trees SET root_node_id = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(root_node_id)
        .bind(&now)
        .bind(tree_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get count of trees
    pub async fn count(
        pool: &PgPool,
    ) -> StorageResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM trees WHERE deleted_at IS NULL")
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
    fn test_tree_repository_methods_compile() {
        // Just verify the methods are defined and compile
        assert!(true);
    }
}
