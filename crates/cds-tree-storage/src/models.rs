use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use cds_tree_core::{ClinicalDecisionTree, DecisionNode};

/// SQLx row type for trees table
/// Note: This is separate from the domain ClinicalDecisionTree type
/// to keep database concerns separate from business logic
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TreeRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub version: String,
    pub status: String,
    pub evidence_level: String,
    pub root_node_id: Option<Uuid>,
    pub review_cycle_months: i16,
    pub cds_hooks_id: Option<String>,
    pub fhir_library_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// SQLx row type for nodes table
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct NodeRow {
    pub id: Uuid,
    pub tree_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub depth: i32,
    pub node_kind: String,
    pub label: String,
    pub description: Option<String>,
    pub input: serde_json::Value,
    pub outcome: Option<serde_json::Value>,
    pub children: serde_json::Value,
    pub evidence_refs: serde_json::Value,
    pub aria_label: String,
    pub required: bool,
    pub skip_condition: Option<serde_json::Value>,
    pub fhir_prefill: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SQLx row type for sessions table
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: Uuid,
    pub tree_id: Uuid,
    pub tree_version: String,
    pub clinician_id: Option<String>,
    pub patient_id: Option<String>,
    pub fhir_encounter_id: Option<String>,
    pub status: String,
    pub current_node_id: Option<Uuid>,
    pub fhir_context: Option<serde_json::Value>,
    pub answers: serde_json::Value,
    pub source_context: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub abandoned_at: Option<DateTime<Utc>>,
    pub outcome_node_id: Option<Uuid>,
}

/// SQLx row type for audit_log table
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct AuditLogRow {
    pub id: i64,
    pub session_id: Option<Uuid>,
    pub tree_id: Option<Uuid>,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub clinician_id: Option<String>,
    pub patient_id: Option<String>,
    pub request_id: Option<Uuid>,
}

/// Create request for a new tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTreeRequest {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub evidence_level: Option<String>,
    pub review_cycle_months: Option<i16>,
}

/// Update request for tree metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTreeRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub evidence_level: Option<String>,
    pub review_cycle_months: Option<i16>,
}

/// Create request for a new node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNodeRequest {
    pub parent_id: Option<Uuid>,
    pub label: String,
    pub description: Option<String>,
    pub input: serde_json::Value,
    pub node_kind: Option<String>,
    pub aria_label: String,
    pub required: Option<bool>,
}

/// Update request for node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodeRequest {
    pub label: Option<String>,
    pub description: Option<String>,
    pub input: Option<serde_json::Value>,
    pub node_kind: Option<String>,
    pub aria_label: Option<String>,
    pub required: Option<bool>,
}

/// Response for tree with all nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeResponse {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub version: String,
    pub status: String,
    pub evidence_level: String,
    pub root_node_id: Option<Uuid>,
    pub nodes: Vec<NodeRow>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Validation report response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReportResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Create request for a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub tree_id: Uuid,
    pub clinician_id: Option<String>,
    pub patient_id: Option<String>,
    pub fhir_context: Option<serde_json::Value>,
}

/// Node answer submitted in session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnswer {
    pub node_id: Uuid,
    pub answer: serde_json::Value,
}

/// Request to advance session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvanceSessionRequest {
    pub answer: NodeAnswer,
}

/// Progress hint for current node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressHint {
    pub current_depth: u32,
    pub estimated_remaining_nodes: u32,
    pub breadcrumb: Vec<BreadcrumbStep>,
}

/// Single step in traversal breadcrumb
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbStep {
    pub node_id: Uuid,
    pub label: String,
    pub answer_summary: String,
    pub depth: u32,
}

/// Node response for session (includes progress hint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResponseForSession {
    pub node: NodeRow,
    pub progress_hint: ProgressHint,
    pub pre_filled_from_fhir: bool,
}

/// Outcome response (terminal node)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeResponse {
    pub outcome: serde_json::Value,
    pub path_summary: Vec<BreadcrumbStep>,
    pub session_id: Uuid,
}

/// Session response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub tree_id: Uuid,
    pub status: String,
    pub current_node_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree_request() {
        let req = CreateTreeRequest {
            slug: "test-tree".to_string(),
            title: "Test Tree".to_string(),
            description: None,
            version: None,
            evidence_level: None,
            review_cycle_months: None,
        };

        assert_eq!(req.slug, "test-tree");
        assert_eq!(req.title, "Test Tree");
    }

    #[test]
    fn test_validation_report() {
        let report = ValidationReportResponse {
            is_valid: true,
            errors: vec![],
            warnings: vec!["Consider adding more nodes".to_string()],
        };

        assert!(report.is_valid);
        assert_eq!(report.warnings.len(), 1);
    }
}
