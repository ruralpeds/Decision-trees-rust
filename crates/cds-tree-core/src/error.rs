use thiserror::Error;

/// Errors that can occur during tree evaluation or traversal.
#[derive(Error, Debug, Clone)]
pub enum EvalError {
    #[error("Node not found: {0}")]
    NodeNotFound(uuid::Uuid),

    #[error("Invalid answer type for node input: expected {expected}, got {got}")]
    InvalidAnswerType { expected: String, got: String },

    #[error("Numeric value out of bounds: {value} not in [{min}, {max}]")]
    NumericOutOfBounds { value: f64, min: f64, max: f64 },

    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),

    #[error("Formula evaluation failed: {0}")]
    FormulaEvaluationFailed(String),

    #[error("FHIR path evaluation failed: {0}")]
    FhirPathError(String),

    #[error("No valid edge condition matched")]
    NoEdgeMatched,

    #[error("Session state error: {0}")]
    SessionStateError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Errors that occur during tree validation or authoring.
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Orphan node: {node_id} has no parent and is not root")]
    OrphanNode { node_id: uuid::Uuid },

    #[error("Unreachable node: {node_id} cannot be reached from root")]
    UnreachableNode { node_id: uuid::Uuid },

    #[error("Dangling edge: node {from_id} references non-existent child {to_id}")]
    DanglingEdge {
        from_id: uuid::Uuid,
        to_id: uuid::Uuid,
    },

    #[error("Cycle detected: {path}")]
    CycleDetected { path: String },

    #[error("Non-terminal leaf: node {0} has no children but is not an Outcome node")]
    NonTerminalLeaf(uuid::Uuid),

    #[error("Invalid formula: {0}")]
    InvalidFormula(String),

    #[error("Input validation failed: {0}")]
    InputValidationFailed(String),

    #[error("Tree metadata error: {0}")]
    MetadataError(String),
}
