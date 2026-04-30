#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

//! # cds-tree-core
//!
//! The foundational clinical decision tree model and evaluation engine.
//! No external dependencies on HTTP frameworks or databases — pure Rust type system
//! and logic that compiles to both native binaries and WebAssembly.

pub mod model;
pub mod engine;
pub mod error;

// Re-exports for convenience
pub use model::{
    ClinicalDecisionTree, DecisionNode, NodeInput, OutcomePayload, EdgeCondition,
    TreeStatus, EvidenceLevel, SeverityLevel,
};
pub use engine::{Evaluator, TreeValidator};
pub use error::{EvalError, ValidationError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
