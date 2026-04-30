use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json::{json, Value};
use crate::models::SessionRow;
use crate::error::StorageResult;

/// Repository for session operations
pub struct SessionRepository;

impl SessionRepository {
    /// Create a new traversal session
    pub async fn create(
        pool: &PgPool,
        tree_id: Uuid,
        tree_version: String,
        clinician_id: Option<String>,
        patient_id: Option<String>,
        source_context: String,
    ) -> StorageResult<SessionRow> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        let session = sqlx::query_as::<_, SessionRow>(
            r#"
            INSERT INTO sessions (
                id, tree_id, tree_version, clinician_id, patient_id, 
                status, source_context, started_at
            )
            VALUES ($1, $2, $3, $4, $5, 'active', $6, $7)
            RETURNING *
            "#
        )
        .bind(session_id)
        .bind(tree_id)
        .bind(tree_version)
        .bind(&clinician_id)
        .bind(&patient_id)
        .bind(&source_context)
        .bind(&now)
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    /// Get a session by ID
    pub async fn get_by_id(
        pool: &PgPool,
        session_id: Uuid,
    ) -> StorageResult<Option<SessionRow>> {
        let session = sqlx::query_as::<_, SessionRow>(
            "SELECT * FROM sessions WHERE id = $1"
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    /// Record an answer to a node
    pub async fn record_answer(
        pool: &PgPool,
        session_id: Uuid,
        node_id: Uuid,
        answer: Value,
    ) -> StorageResult<()> {
        let now = Utc::now();

        // Get current answers
        let row = sqlx::query("SELECT answers FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(pool)
            .await?;

        let mut answers: serde_json::Map<String, Value> = 
            row.get::<serde_json::Value, _>("answers")
                .as_object()
                .map(|m| m.clone())
                .unwrap_or_default();

        // Add new answer
        answers.insert(node_id.to_string(), answer);

        // Update session
        sqlx::query(
            "UPDATE sessions SET answers = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(serde_json::to_value(answers)?)
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get answers for a session
    pub async fn get_answers(
        pool: &PgPool,
        session_id: Uuid,
    ) -> StorageResult<serde_json::Map<String, Value>> {
        let row = sqlx::query("SELECT answers FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(pool)
            .await?;

        let answers: serde_json::Value = row.get("answers");
        let map = answers
            .as_object()
            .map(|m| m.clone())
            .unwrap_or_default();

        Ok(map)
    }

    /// Update current node
    pub async fn set_current_node(
        pool: &PgPool,
        session_id: Uuid,
        node_id: Option<Uuid>,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE sessions SET current_node_id = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(node_id)
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark session as completed
    pub async fn complete(
        pool: &PgPool,
        session_id: Uuid,
        outcome_node_id: Uuid,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE sessions SET status = 'completed', completed_at = $1, outcome_node_id = $2 WHERE id = $3"
        )
        .bind(&now)
        .bind(outcome_node_id)
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark session as abandoned
    pub async fn abandon(
        pool: &PgPool,
        session_id: Uuid,
    ) -> StorageResult<()> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE sessions SET status = 'abandoned', abandoned_at = $1 WHERE id = $2"
        )
        .bind(&now)
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get sessions for a clinician
    pub async fn list_by_clinician(
        pool: &PgPool,
        clinician_id: &str,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<SessionRow>> {
        let sessions = sqlx::query_as::<_, SessionRow>(
            "SELECT * FROM sessions WHERE clinician_id = $1 ORDER BY started_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(clinician_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(sessions)
    }

    /// Get active sessions count
    pub async fn count_active(
        pool: &PgPool,
    ) -> StorageResult<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM sessions WHERE status = 'active'")
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
    fn test_session_repository_methods_compile() {
        assert!(true);
    }
}
