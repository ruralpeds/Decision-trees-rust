use axum::{
    extract::{State, Path},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use chrono::Utc;
use cds_tree_storage::{SessionRepository, AuditLogRepository, NodeRepository, TreeRepository};
use cds_tree_fhir::{FhirExportAdapter, FhirAuditEvent};
use crate::state::AppState;
use crate::error::{AppError, ApiResult};

/// GET /api/v1/sessions/:session_id/audit-log — Get audit trail for session
pub async fn get_session_audit_log(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify session exists
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    // Get audit events
    let events = AuditLogRepository::get_session_events(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Format for response
    let audit_entries: Vec<_> = events
        .iter()
        .map(|event| {
            serde_json::json!({
                "id": event.id,
                "event_type": event.event_type,
                "occurred_at": event.occurred_at,
                "details": event.event_data,
                "clinician_id": event.clinician_id,
                "patient_id": event.patient_id
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "tree_id": session.tree_id,
        "status": session.status,
        "audit_log": audit_entries,
        "total_events": audit_entries.len(),
        "started_at": session.started_at,
        "completed_at": session.completed_at
    })))
}

/// GET /api/v1/sessions/:session_id/fhir-audit-event — Export as FHIR AuditEvent
pub async fn export_session_as_fhir_audit_event(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<FhirAuditEvent>> {
    // Get session
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    // Get tree for title
    let tree = TreeRepository::get_by_id(&state.db_pool, session.tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(session.tree_id.to_string()))?;

    // Get outcome node if completed
    let outcome_title = if let Some(outcome_id) = session.outcome_node_id {
        NodeRepository::get_by_id(&state.db_pool, outcome_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .map(|n| n.label)
    } else {
        None
    };

    // Create FHIR AuditEvent
    let audit_event = FhirExportAdapter::session_to_audit_event(
        session_id,
        session.tree_id,
        &tree.title,
        session.started_at,
        session.completed_at,
        session.clinician_id.clone(),
        session.patient_id.clone(),
        outcome_title,
    );

    Ok(Json(audit_event))
}

/// GET /api/v1/sessions/:session_id/fhir-observations — Export answers as FHIR Observations
pub async fn export_session_as_fhir_observations(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Get session
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    let patient_id = session
        .patient_id
        .as_deref()
        .unwrap_or("unknown");

    // Get answers
    let answers = SessionRepository::get_answers(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Convert answers to FHIR Observations
    let observations: Vec<_> = answers
        .iter()
        .filter_map(|(node_id_str, answer)| {
            let node_id = uuid::Uuid::parse_str(node_id_str).ok()?;
            let obs = FhirExportAdapter::node_answer_to_observation(
                node_id,
                "Decision Support Answer",
                answer,
                Utc::now(),
                patient_id,
            );
            Some(serde_json::to_value(obs).ok())
        })
        .collect();

    Ok(Json(serde_json::json!({
        "resource_type": "Bundle",
        "type": "collection",
        "total": observations.len(),
        "entry": observations.iter().map(|obs| serde_json::json!({
            "resource": obs
        })).collect::<Vec<_>>()
    })))
}

/// GET /api/v1/sessions/:session_id/summary — Get session summary with justification
pub async fn get_session_summary(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Get session
    let session = SessionRepository::get_by_id(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

    // Get tree
    let tree = TreeRepository::get_by_id(&state.db_pool, session.tree_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound(session.tree_id.to_string()))?;

    // Get answers
    let answers = SessionRepository::get_answers(&state.db_pool, session_id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Calculate duration
    let duration_minutes = session
        .completed_at
        .map(|completed| {
            (completed - session.started_at).num_minutes() as f64
        })
        .unwrap_or(0.0);

    // Get outcome title
    let outcome_title = if let Some(outcome_id) = session.outcome_node_id {
        NodeRepository::get_by_id(&state.db_pool, outcome_id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .map(|n| n.label)
    } else {
        None
    };

    // Build summary
    let summary = FhirExportAdapter::build_session_summary(
        session_id,
        &tree.title,
        answers.len(),
        duration_minutes,
        outcome_title,
        session.clinician_id.clone(),
        session.patient_id.clone(),
    );

    // Add path details
    let mut result = summary.as_object().unwrap().clone();

    // Build decision path with justification
    let mut decision_path = Vec::new();
    for (node_id_str, answer) in &answers {
        if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
            if let Ok(Some(node)) = NodeRepository::get_by_id(&state.db_pool, node_id).await {
                decision_path.push(serde_json::json!({
                    "node_id": node_id.to_string(),
                    "label": node.label.clone(),
                    "answer": answer,
                    "depth": node.depth
                }));
            }
        }
    }

    result.insert("decision_path".to_string(), serde_json::json!(decision_path));
    result.insert("status".to_string(), serde_json::json!(session.status));
    result.insert("created_at".to_string(), serde_json::json!(session.started_at));

    Ok(Json(serde_json::Value::Object(result)))
}

/// GET /api/v1/clinicians/:clinician_id/audit-log — Get audit trail for clinician
pub async fn get_clinician_audit_log(
    State(state): State<AppState>,
    Path(clinician_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let limit = 100i64;
    let offset = 0i64;

    // Get events
    let events = AuditLogRepository::get_clinician_events(
        &state.db_pool,
        &clinician_id,
        limit,
        offset,
    )
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let entries: Vec<_> = events
        .iter()
        .map(|event| {
            serde_json::json!({
                "id": event.id,
                "event_type": event.event_type,
                "session_id": event.session_id,
                "tree_id": event.tree_id,
                "occurred_at": event.occurred_at,
                "patient_id": event.patient_id
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "clinician_id": clinician_id,
        "audit_log": entries,
        "total_events": entries.len(),
        "limit": limit,
        "offset": offset
    })))
}

/// GET /api/v1/patients/:patient_id/audit-log — Get audit trail for patient
pub async fn get_patient_audit_log(
    State(state): State<AppState>,
    Path(patient_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let limit = 100i64;
    let offset = 0i64;

    // Get events
    let events = AuditLogRepository::get_patient_events(
        &state.db_pool,
        &patient_id,
        limit,
        offset,
    )
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let entries: Vec<_> = events
        .iter()
        .map(|event| {
            serde_json::json!({
                "id": event.id,
                "event_type": event.event_type,
                "session_id": event.session_id,
                "tree_id": event.tree_id,
                "occurred_at": event.occurred_at,
                "clinician_id": event.clinician_id
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "patient_id": patient_id,
        "audit_log": entries,
        "total_events": entries.len(),
        "limit": limit,
        "offset": offset
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handlers_compile() {
        assert!(true);
    }
}
