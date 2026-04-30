use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json::{Value, json};
use crate::models::AuditLogRow;
use crate::error::StorageResult;

/// Repository for audit log operations (insert-only)
pub struct AuditLogRepository;

impl AuditLogRepository {
    /// Record a session start event
    pub async fn record_session_start(
        pool: &PgPool,
        session_id: Uuid,
        tree_id: Uuid,
        clinician_id: Option<String>,
        patient_id: Option<String>,
    ) -> StorageResult<()> {
        let event_data = json!({
            "event_type": "session_start",
            "session_id": session_id.to_string(),
            "tree_id": tree_id.to_string(),
            "clinician_id": clinician_id,
            "patient_id": patient_id,
            "timestamp": Utc::now().to_rfc3339()
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log (session_id, tree_id, event_type, event_data, occurred_at, clinician_id, patient_id)
            VALUES ($1, $2, 'session_start', $3, $4, $5, $6)
            "#
        )
        .bind(session_id)
        .bind(tree_id)
        .bind(&event_data)
        .bind(&Utc::now())
        .bind(&clinician_id)
        .bind(&patient_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a node answer event
    pub async fn record_node_answer(
        pool: &PgPool,
        session_id: Uuid,
        tree_id: Uuid,
        node_id: Uuid,
        answer: &Value,
        clinician_id: Option<String>,
        patient_id: Option<String>,
    ) -> StorageResult<()> {
        let event_data = json!({
            "event_type": "node_answer",
            "session_id": session_id.to_string(),
            "tree_id": tree_id.to_string(),
            "node_id": node_id.to_string(),
            "answer": answer,
            "timestamp": Utc::now().to_rfc3339()
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log (session_id, tree_id, event_type, event_data, occurred_at, clinician_id, patient_id)
            VALUES ($1, $2, 'node_answer', $3, $4, $5, $6)
            "#
        )
        .bind(session_id)
        .bind(tree_id)
        .bind(&event_data)
        .bind(&Utc::now())
        .bind(&clinician_id)
        .bind(&patient_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a session completion event
    pub async fn record_session_completed(
        pool: &PgPool,
        session_id: Uuid,
        tree_id: Uuid,
        outcome_node_id: Uuid,
        clinician_id: Option<String>,
        patient_id: Option<String>,
    ) -> StorageResult<()> {
        let event_data = json!({
            "event_type": "session_completed",
            "session_id": session_id.to_string(),
            "tree_id": tree_id.to_string(),
            "outcome_node_id": outcome_node_id.to_string(),
            "timestamp": Utc::now().to_rfc3339()
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log (session_id, tree_id, event_type, event_data, occurred_at, clinician_id, patient_id)
            VALUES ($1, $2, 'session_completed', $3, $4, $5, $6)
            "#
        )
        .bind(session_id)
        .bind(tree_id)
        .bind(&event_data)
        .bind(&Utc::now())
        .bind(&clinician_id)
        .bind(&patient_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a session abandoned event
    pub async fn record_session_abandoned(
        pool: &PgPool,
        session_id: Uuid,
        tree_id: Uuid,
        reason: Option<String>,
        clinician_id: Option<String>,
        patient_id: Option<String>,
    ) -> StorageResult<()> {
        let event_data = json!({
            "event_type": "session_abandoned",
            "session_id": session_id.to_string(),
            "tree_id": tree_id.to_string(),
            "reason": reason,
            "timestamp": Utc::now().to_rfc3339()
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log (session_id, tree_id, event_type, event_data, occurred_at, clinician_id, patient_id)
            VALUES ($1, $2, 'session_abandoned', $3, $4, $5, $6)
            "#
        )
        .bind(session_id)
        .bind(tree_id)
        .bind(&event_data)
        .bind(&Utc::now())
        .bind(&clinician_id)
        .bind(&patient_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a CDS Hooks recommendation event
    pub async fn record_cds_recommendation(
        pool: &PgPool,
        session_id: Uuid,
        tree_id: Uuid,
        card_id: String,
        accepted: bool,
        clinician_id: Option<String>,
        patient_id: Option<String>,
    ) -> StorageResult<()> {
        let event_data = json!({
            "event_type": "cds_recommendation",
            "session_id": session_id.to_string(),
            "tree_id": tree_id.to_string(),
            "card_id": card_id,
            "accepted": accepted,
            "timestamp": Utc::now().to_rfc3339()
        });

        sqlx::query(
            r#"
            INSERT INTO audit_log (session_id, tree_id, event_type, event_data, occurred_at, clinician_id, patient_id)
            VALUES ($1, $2, 'cds_recommendation', $3, $4, $5, $6)
            "#
        )
        .bind(session_id)
        .bind(tree_id)
        .bind(&event_data)
        .bind(&Utc::now())
        .bind(&clinician_id)
        .bind(&patient_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get all audit events for a session
    pub async fn get_session_events(
        pool: &PgPool,
        session_id: Uuid,
    ) -> StorageResult<Vec<AuditLogRow>> {
        let events = sqlx::query_as::<_, AuditLogRow>(
            "SELECT * FROM audit_log WHERE session_id = $1 ORDER BY occurred_at ASC"
        )
        .bind(session_id)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    /// Get audit events for a clinician
    pub async fn get_clinician_events(
        pool: &PgPool,
        clinician_id: &str,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<AuditLogRow>> {
        let events = sqlx::query_as::<_, AuditLogRow>(
            "SELECT * FROM audit_log WHERE clinician_id = $1 ORDER BY occurred_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(clinician_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    /// Get audit events for a patient
    pub async fn get_patient_events(
        pool: &PgPool,
        patient_id: &str,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<AuditLogRow>> {
        let events = sqlx::query_as::<_, AuditLogRow>(
            "SELECT * FROM audit_log WHERE patient_id = $1 ORDER BY occurred_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(patient_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    /// Get audit events for a tree
    pub async fn get_tree_events(
        pool: &PgPool,
        tree_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<AuditLogRow>> {
        let events = sqlx::query_as::<_, AuditLogRow>(
            "SELECT * FROM audit_log WHERE tree_id = $1 ORDER BY occurred_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tree_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(events)
    }

    /// Count events of a specific type for a session
    pub async fn count_event_type(
        pool: &PgPool,
        session_id: Uuid,
        event_type: &str,
    ) -> StorageResult<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM audit_log WHERE session_id = $1 AND event_type = $2"
        )
        .bind(session_id)
        .bind(event_type)
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
    fn test_audit_log_repository_methods_compile() {
        assert!(true);
    }
}
