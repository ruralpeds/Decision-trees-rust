use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::evidence::EvidenceRef;

/// The clinical recommendation at a terminal (Outcome) node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomePayload {
    pub severity: SeverityLevel,
    pub title: String,
    pub summary: String,
    pub full_recommendation: String,
    pub rationale: Option<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub actions: Vec<RecommendedAction>,
    pub path_summary: Vec<PathSummaryStep>,
    pub icd10_suggestions: Vec<String>,
    pub snomed_suggestions: Vec<String>,
    pub order_set_ids: Vec<String>,
    pub documentation_template: Option<String>,
    pub follow_up_tree_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SeverityLevel {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "suggestion")]
    Suggestion,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "critical")]
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: ActionType,
    pub label: String,
    pub description: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    OrderMedication,
    OrderLab,
    OrderImaging,
    Consult,
    Admit,
    Discharge,
    TransferLevel,
    EhrNote,
    PatientEducation,
    FollowUp,
    EmergencyAction,
    Custom(String),
}

/// Represents a single step in the decision path taken to reach this outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSummaryStep {
    pub node_id: Uuid,
    pub node_label: String,
    pub answer_summary: String,
    pub depth: u32,
}

impl OutcomePayload {
    pub fn new(severity: SeverityLevel, title: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            severity,
            title: title.into(),
            summary: summary.into(),
            full_recommendation: String::new(),
            rationale: None,
            evidence_refs: vec![],
            actions: vec![],
            path_summary: vec![],
            icd10_suggestions: vec![],
            snomed_suggestions: vec![],
            order_set_ids: vec![],
            documentation_template: None,
            follow_up_tree_id: None,
        }
    }

    pub fn with_recommendation(mut self, rec: impl Into<String>) -> Self {
        self.full_recommendation = rec.into();
        self
    }

    pub fn with_evidence(mut self, refs: Vec<EvidenceRef>) -> Self {
        self.evidence_refs = refs;
        self
    }

    pub fn with_action(mut self, action: RecommendedAction) -> Self {
        self.actions.push(action);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_creation() {
        let outcome = OutcomePayload::new(
            SeverityLevel::Warning,
            "Respiratory Distress Confirmed",
            "Patient exhibits signs of respiratory distress",
        );
        assert_eq!(outcome.severity, SeverityLevel::Warning);
        assert_eq!(outcome.title, "Respiratory Distress Confirmed");
    }

    #[test]
    fn test_outcome_builder() {
        let outcome = OutcomePayload::new(SeverityLevel::Critical, "Critical Action Required", "Immediate intervention needed")
            .with_recommendation("Transfer to Level III NICU immediately");
        assert!(!outcome.full_recommendation.is_empty());
    }

    #[test]
    fn test_severity_serialization() {
        let severity = SeverityLevel::Critical;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"critical\"");
    }
}
