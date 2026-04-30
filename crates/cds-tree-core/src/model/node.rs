use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::model::input::NodeInput;
use crate::model::outcome::OutcomePayload;
use crate::model::evidence::EvidenceRef;

/// A single node in the decision tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub tree_id: Uuid,
    pub depth: u32,
    pub label: String,
    pub description: Option<String>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub input: NodeInput,
    pub children: Vec<ChildEdge>,
    pub node_type: NodeKind,
    pub aria_label: String,
    pub required: bool,
    pub skip_condition: Option<SkipCondition>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Outcome payload (only for Outcome nodes)
    pub outcome: Option<OutcomePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Question,
    Information,
    Calculation,
    Outcome,
    Redirect {
        target_tree_id: Uuid,
        target_node_id: Option<Uuid>,
    },
}

/// An edge from this node to a potential child node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildEdge {
    pub id: Uuid,
    pub child_node_id: Uuid,
    /// When does this edge fire?
    pub condition: EdgeCondition,
    /// Optional label shown on edge (e.g., "Yes", "> 37 weeks")
    pub label: Option<String>,
    /// If multiple edges match, higher priority wins (0 = highest)
    pub priority: u8,
}

/// A condition that determines if an edge is followed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeCondition {
    /// Always follow this edge (default/else branch)
    Always,

    /// Simple value comparison
    Compare {
        variable: ConditionVariable,
        operator: ComparisonOperator,
        value: ConditionValue,
    },

    /// Range check
    InRange {
        variable: ConditionVariable,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        inclusive: bool,
    },

    /// Set membership
    Contains {
        variable: ConditionVariable,
        value: String,
    },

    /// Logical AND
    And(Vec<EdgeCondition>),

    /// Logical OR
    Or(Vec<EdgeCondition>),

    /// Logical NOT
    Not(Box<EdgeCondition>),

    /// FHIR path evaluation
    FhirPath {
        resource_type: String,
        fhir_path: String,
        operator: ComparisonOperator,
        value: ConditionValue,
    },
}

/// A variable that can appear in edge conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConditionVariable {
    /// Reference to a node by ID and extract its answer
    NodeAnswer(Uuid),
    /// FHIR context variable (e.g., "patient.birthDate")
    FhirContext(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComparisonOperator {
    #[serde(rename = "eq")]
    Eq,
    #[serde(rename = "ne")]
    Ne,
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "le")]
    Le,
    #[serde(rename = "gt")]
    Gt,
    #[serde(rename = "ge")]
    Ge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConditionValue {
    Bool(bool),
    Number(f64),
    Text(String),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipCondition {
    pub condition: EdgeCondition,
    pub auto_advance_to_child: bool,
}

impl DecisionNode {
    pub fn new(tree_id: Uuid, label: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            tree_id,
            depth: 0,
            label: label.into(),
            description: None,
            evidence_refs: vec![],
            input: NodeInput::Display,
            children: vec![],
            node_type: NodeKind::Question,
            aria_label: String::new(),
            required: true,
            skip_condition: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            outcome: None,
        }
    }

    /// Validate that this node has sensible configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.aria_label.is_empty() {
            return Err("aria_label cannot be empty".to_string());
        }

        if self.label.is_empty() {
            return Err("label cannot be empty".to_string());
        }

        // Outcome nodes must have an outcome payload
        if matches!(self.node_type, NodeKind::Outcome) && self.outcome.is_none() {
            return Err("Outcome nodes must have an outcome payload".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let tree_id = Uuid::new_v4();
        let node = DecisionNode::new(tree_id, "Is the patient febrile?");
        assert_eq!(node.label, "Is the patient febrile?");
        assert_eq!(node.depth, 0);
    }

    #[test]
    fn test_always_condition() {
        let condition = EdgeCondition::Always;
        assert_eq!(
            serde_json::to_string(&condition).unwrap(),
            r#"{"type":"always"}"#
        );
    }

    #[test]
    fn test_compare_condition() {
        let condition = EdgeCondition::Compare {
            variable: ConditionVariable::NodeAnswer(Uuid::nil()),
            operator: ComparisonOperator::Gt,
            value: ConditionValue::Number(37.0),
        };
        let json = serde_json::to_string(&condition).unwrap();
        assert!(json.contains("\"type\":\"compare\""));
    }
}
