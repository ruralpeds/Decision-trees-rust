use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::model::node::DecisionNode;
use crate::model::evidence::GuidelineRef;

/// Top-level clinical decision tree. Immutable once published.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalDecisionTree {
    pub id: Uuid,
    /// URL-safe identifier (e.g., "neonatal-respiratory-distress")
    pub slug: String,
    pub title: String,
    pub description: String,
    pub version: String, // Semver format
    pub status: TreeStatus,
    pub specialty: Vec<MedicalSpecialty>,
    pub setting: Vec<ClinicalSetting>,
    pub evidence_level: EvidenceLevel,
    pub guideline_refs: Vec<GuidelineRef>,
    pub root_node_id: Uuid,
    /// Flat list; tree structure via node.parent_id and node.children
    pub nodes: Vec<DecisionNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub authors: Vec<TreeAuthor>,
    /// Mandatory review cadence in months
    pub review_cycle_months: u8,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    /// CDS Hooks service ID if registered
    pub cds_hooks_id: Option<String>,
    /// FHIR Library resource URL containing the logic
    pub fhir_library_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TreeStatus {
    Draft,
    PeerReview,
    Published,
    Deprecated { replaced_by: Option<Uuid> },
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeAuthor {
    pub name: String,
    pub title: Option<String>,
    pub organization: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum EvidenceLevel {
    GradeA,
    GradeB,
    GradeC,
    GradeD,
    ExpertConsensus,
    Institutional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum MedicalSpecialty {
    Neonatology,
    Pediatrics,
    EmergencyMedicine,
    InternalMedicine,
    CriticalCare,
    Surgery,
    Obstetrics,
    GeneralPractice,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum ClinicalSetting {
    CriticalAccessHospital,
    Level1Nursery,
    Level2Nursery,
    Level2PlusNursery,
    EmergencyDepartment,
    Outpatient,
    Telemedicine,
}

impl ClinicalDecisionTree {
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            slug: slug.into(),
            title: title.into(),
            description: String::new(),
            version: "0.1.0".to_string(),
            status: TreeStatus::Draft,
            specialty: vec![],
            setting: vec![],
            evidence_level: EvidenceLevel::ExpertConsensus,
            guideline_refs: vec![],
            root_node_id: Uuid::nil(),
            nodes: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            published_at: None,
            authors: vec![],
            review_cycle_months: 12,
            last_reviewed_at: None,
            cds_hooks_id: None,
            fhir_library_url: None,
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: Uuid) -> Option<&DecisionNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    /// Get mutable reference to a node by ID
    pub fn get_node_mut(&mut self, node_id: Uuid) -> Option<&mut DecisionNode> {
        self.nodes.iter_mut().find(|n| n.id == node_id)
    }

    /// Get all children of a given node
    pub fn get_children(&self, node_id: Uuid) -> Vec<&DecisionNode> {
        self.nodes
            .iter()
            .filter(|n| n.parent_id == Some(node_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let tree = ClinicalDecisionTree::new("test-tree", "Test Tree");
        assert_eq!(tree.slug, "test-tree");
        assert_eq!(tree.title, "Test Tree");
        assert_eq!(tree.status, TreeStatus::Draft);
    }
}
