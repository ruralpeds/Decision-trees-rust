use serde_json::{json, Value};
use uuid::Uuid;
use crate::model::node::{EdgeCondition, ComparisonOperator, ConditionVariable, ConditionValue};
use crate::error::EvalError;

/// Evaluates edge conditions against session state
pub struct Evaluator;

impl Evaluator {
    /// Evaluate an edge condition given the current session state
    ///
    /// Returns true if the condition matches (edge should be followed),
    /// false otherwise.
    pub fn evaluate(
        condition: &EdgeCondition,
        session_state: &SessionState,
    ) -> Result<bool, EvalError> {
        match condition {
            EdgeCondition::Always => Ok(true),

            EdgeCondition::Compare {
                variable,
                operator,
                value,
            } => Self::evaluate_compare(variable, *operator, value, session_state),

            EdgeCondition::InRange {
                variable,
                min,
                max,
                inclusive,
            } => Self::evaluate_in_range(variable, *min, *max, *inclusive, session_state),

            EdgeCondition::Contains { variable, value } => {
                Self::evaluate_contains(variable, value, session_state)
            }

            EdgeCondition::And(conditions) => {
                for cond in conditions {
                    if !Self::evaluate(cond, session_state)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            EdgeCondition::Or(conditions) => {
                for cond in conditions {
                    if Self::evaluate(cond, session_state)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            EdgeCondition::Not(condition) => {
                let result = Self::evaluate(condition, session_state)?;
                Ok(!result)
            }

            EdgeCondition::FhirPath {
                resource_type: _,
                fhir_path: _,
                operator: _,
                value: _,
            } => {
                // FHIR path evaluation deferred to FHIR integration layer
                Err(EvalError::FhirPathError(
                    "FHIR path evaluation not implemented in core evaluator".to_string(),
                ))
            }
        }
    }

    fn evaluate_compare(
        variable: &ConditionVariable,
        operator: ComparisonOperator,
        expected: &ConditionValue,
        session_state: &SessionState,
    ) -> Result<bool, EvalError> {
        let actual = Self::get_variable_value(variable, session_state)?;

        match (actual, expected) {
            (ConditionValue::Number(a), ConditionValue::Number(b)) => {
                Ok(match operator {
                    ComparisonOperator::Eq => (a - b).abs() < f64::EPSILON,
                    ComparisonOperator::Ne => (a - b).abs() >= f64::EPSILON,
                    ComparisonOperator::Lt => a < b,
                    ComparisonOperator::Le => a <= b,
                    ComparisonOperator::Gt => a > b,
                    ComparisonOperator::Ge => a >= b,
                })
            }

            (ConditionValue::Bool(a), ConditionValue::Bool(b)) => {
                Ok(match operator {
                    ComparisonOperator::Eq => a == b,
                    ComparisonOperator::Ne => a != b,
                    _ => return Err(EvalError::InvalidAnswerType {
                        expected: "numeric comparison on boolean".to_string(),
                        got: "boolean".to_string(),
                    }),
                })
            }

            (ConditionValue::Text(a), ConditionValue::Text(b)) => {
                Ok(match operator {
                    ComparisonOperator::Eq => a == b,
                    ComparisonOperator::Ne => a != b,
                    _ => return Err(EvalError::InvalidAnswerType {
                        expected: "text comparison does not support numeric operators".to_string(),
                        got: format!("{:?}", operator),
                    }),
                })
            }

            (ConditionValue::Null, _) | (_, ConditionValue::Null) => {
                Ok(match operator {
                    ComparisonOperator::Eq => matches!(actual, ConditionValue::Null),
                    ComparisonOperator::Ne => !matches!(actual, ConditionValue::Null),
                    _ => false,
                })
            }

            _ => Err(EvalError::InvalidAnswerType {
                expected: format!("{:?}", expected),
                got: format!("{:?}", actual),
            }),
        }
    }

    fn evaluate_in_range(
        variable: &ConditionVariable,
        min: Option<f64>,
        max: Option<f64>,
        inclusive: bool,
        session_state: &SessionState,
    ) -> Result<bool, EvalError> {
        let value = Self::get_variable_value(variable, session_state)?;

        match value {
            ConditionValue::Number(n) => {
                let min_ok = match min {
                    Some(m) => {
                        if inclusive {
                            n >= m
                        } else {
                            n > m
                        }
                    }
                    None => true,
                };

                let max_ok = match max {
                    Some(m) => {
                        if inclusive {
                            n <= m
                        } else {
                            n < m
                        }
                    }
                    None => true,
                };

                Ok(min_ok && max_ok)
            }

            _ => Err(EvalError::InvalidAnswerType {
                expected: "numeric".to_string(),
                got: format!("{:?}", value),
            }),
        }
    }

    fn evaluate_contains(
        variable: &ConditionVariable,
        search_value: &str,
        session_state: &SessionState,
    ) -> Result<bool, EvalError> {
        let value = Self::get_variable_value(variable, session_state)?;

        match value {
            ConditionValue::Text(s) => Ok(s == search_value),
            _ => Err(EvalError::InvalidAnswerType {
                expected: "text".to_string(),
                got: format!("{:?}", value),
            }),
        }
    }

    fn get_variable_value(
        variable: &ConditionVariable,
        session_state: &SessionState,
    ) -> Result<ConditionValue, EvalError> {
        match variable {
            ConditionVariable::NodeAnswer(node_id) => {
                session_state
                    .answers
                    .get(node_id)
                    .cloned()
                    .ok_or(EvalError::NodeNotFound(*node_id))
            }

            ConditionVariable::FhirContext(_) => {
                Err(EvalError::FhirPathError(
                    "FHIR context not available in core evaluator".to_string(),
                ))
            }
        }
    }
}

/// The state of a session during traversal
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Map of node_id → ConditionValue (the answer at that node)
    pub answers: std::collections::HashMap<Uuid, ConditionValue>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            answers: std::collections::HashMap::new(),
        }
    }

    pub fn record_answer(&mut self, node_id: Uuid, answer: ConditionValue) {
        self.answers.insert(node_id, answer);
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_always_condition() -> Result<(), EvalError> {
        let state = SessionState::new();
        let condition = EdgeCondition::Always;
        assert!(Evaluator::evaluate(&condition, &state)?);
        Ok(())
    }

    #[test]
    fn test_compare_numbers() -> Result<(), EvalError> {
        let mut state = SessionState::new();
        let node_id = Uuid::new_v4();
        state.record_answer(node_id, ConditionValue::Number(37.5));

        let condition = EdgeCondition::Compare {
            variable: ConditionVariable::NodeAnswer(node_id),
            operator: ComparisonOperator::Gt,
            value: ConditionValue::Number(37.0),
        };

        assert!(Evaluator::evaluate(&condition, &state)?);
        Ok(())
    }

    #[test]
    fn test_in_range() -> Result<(), EvalError> {
        let mut state = SessionState::new();
        let node_id = Uuid::new_v4();
        state.record_answer(node_id, ConditionValue::Number(50.0));

        let condition = EdgeCondition::InRange {
            variable: ConditionVariable::NodeAnswer(node_id),
            min: Some(40.0),
            max: Some(60.0),
            inclusive: true,
        };

        assert!(Evaluator::evaluate(&condition, &state)?);
        Ok(())
    }

    #[test]
    fn test_and_logic() -> Result<(), EvalError> {
        let mut state = SessionState::new();
        let node_id1 = Uuid::new_v4();
        let node_id2 = Uuid::new_v4();
        state.record_answer(node_id1, ConditionValue::Bool(true));
        state.record_answer(node_id2, ConditionValue::Number(100.0));

        let condition = EdgeCondition::And(vec![
            EdgeCondition::Compare {
                variable: ConditionVariable::NodeAnswer(node_id1),
                operator: ComparisonOperator::Eq,
                value: ConditionValue::Bool(true),
            },
            EdgeCondition::Compare {
                variable: ConditionVariable::NodeAnswer(node_id2),
                operator: ComparisonOperator::Gt,
                value: ConditionValue::Number(50.0),
            },
        ]);

        assert!(Evaluator::evaluate(&condition, &state)?);
        Ok(())
    }

    #[test]
    fn test_or_logic() -> Result<(), EvalError> {
        let mut state = SessionState::new();
        let node_id = Uuid::new_v4();
        state.record_answer(node_id, ConditionValue::Number(10.0));

        let condition = EdgeCondition::Or(vec![
            EdgeCondition::Compare {
                variable: ConditionVariable::NodeAnswer(node_id),
                operator: ComparisonOperator::Lt,
                value: ConditionValue::Number(5.0),
            },
            EdgeCondition::Compare {
                variable: ConditionVariable::NodeAnswer(node_id),
                operator: ComparisonOperator::Gt,
                value: ConditionValue::Number(9.0),
            },
        ]);

        assert!(Evaluator::evaluate(&condition, &state)?);
        Ok(())
    }

    #[test]
    fn test_not_logic() -> Result<(), EvalError> {
        let mut state = SessionState::new();
        let node_id = Uuid::new_v4();
        state.record_answer(node_id, ConditionValue::Bool(false));

        let condition = EdgeCondition::Not(Box::new(EdgeCondition::Compare {
            variable: ConditionVariable::NodeAnswer(node_id),
            operator: ComparisonOperator::Eq,
            value: ConditionValue::Bool(true),
        }));

        assert!(Evaluator::evaluate(&condition, &state)?);
        Ok(())
    }
}
