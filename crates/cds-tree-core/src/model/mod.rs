pub mod tree;
pub mod node;
pub mod input;
pub mod outcome;
pub mod evidence;

pub use tree::{ClinicalDecisionTree, TreeStatus, TreeAuthor, EvidenceLevel, MedicalSpecialty, ClinicalSetting};
pub use node::{DecisionNode, NodeKind, ChildEdge, EdgeCondition, ComparisonOperator, ConditionVariable, ConditionValue};
pub use input::{NodeInput, BooleanInput, SingleSelectInput, MultiSelectInput, SliderInput, NumericInput, ComputedInput, Display};
pub use outcome::{OutcomePayload, SeverityLevel, RecommendedAction, ActionType, PathSummaryStep};
pub use evidence::{EvidenceRef, GuidelineRef};
