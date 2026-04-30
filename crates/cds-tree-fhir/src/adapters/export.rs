use crate::models::{FhirAuditEvent, FhirCodeableConcept, FhirCoding, FhirObservation, FhirQuantity, FhirPeriod};
use serde_json::json;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// FHIR export adapter
pub struct FhirExportAdapter;

impl FhirExportAdapter {
    /// Convert session to FHIR AuditEvent (for compliance/audit logging)
    pub fn session_to_audit_event(
        session_id: Uuid,
        tree_id: Uuid,
        tree_title: &str,
        started_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
        clinician_id: Option<String>,
        patient_id: Option<String>,
        outcome_title: Option<String>,
    ) -> FhirAuditEvent {
        FhirAuditEvent {
            id: session_id.to_string(),
            event_type: FhirCodeableConcept {
                coding: Some(vec![FhirCoding {
                    system: "http://dicom.nema.org/resources/ontology/DCM".to_string(),
                    code: "ITI-8".to_string(),
                    display: Some("Retrieve FHIR Resource".to_string()),
                }]),
                text: Some("Clinical Decision Support Assessment".to_string()),
            },
            action: "E".to_string(), // Execute
            period: Some(FhirPeriod {
                start: Some(started_at.to_rfc3339()),
                end: completed_at.map(|d| d.to_rfc3339()),
            }),
            recorded: Utc::now().to_rfc3339(),
            outcome: completed_at.map(|_| "0".to_string()), // Success
            detail: Some(json!({
                "session_id": session_id.to_string(),
                "tree_id": tree_id.to_string(),
                "tree_title": tree_title,
                "outcome": outcome_title,
                "clinician_id": clinician_id,
                "patient_id": patient_id,
                "decision_support_type": "Clinical Decision Tree"
            })),
        }
    }

    /// Convert node answer to FHIR Observation
    pub fn node_answer_to_observation(
        node_id: Uuid,
        node_label: &str,
        answer_value: &serde_json::Value,
        effective_date_time: DateTime<Utc>,
        patient_id: &str,
    ) -> FhirObservation {
        FhirObservation {
            id: format!("obs-{}", Uuid::new_v4()),
            code: FhirCodeableConcept {
                coding: Some(vec![FhirCoding {
                    system: "http://loinc.org".to_string(),
                    code: "11008-6".to_string(), // "Natality status" or generic assessment
                    display: Some(node_label.to_string()),
                }]),
                text: Some(node_label.to_string()),
            },
            value_quantity: extract_quantity_from_answer(answer_value),
            value_code: extract_code_from_answer(answer_value),
            effective_date_time: Some(effective_date_time.to_rfc3339()),
        }
    }

    /// Build session summary for display
    pub fn build_session_summary(
        session_id: Uuid,
        tree_title: &str,
        total_steps: usize,
        duration_minutes: f64,
        outcome_title: Option<String>,
        clinician_id: Option<String>,
        patient_id: Option<String>,
    ) -> serde_json::Value {
        json!({
            "session_id": session_id.to_string(),
            "tree_title": tree_title,
            "summary": {
                "total_steps": total_steps,
                "duration_minutes": duration_minutes,
                "outcome": outcome_title,
                "clinician": clinician_id,
                "patient": patient_id,
                "timestamp": Utc::now().to_rfc3339()
            }
        })
    }

    /// Generate justification text for a decision
    pub fn generate_decision_justification(
        node_label: &str,
        answer: &str,
        next_node_label: &str,
        decision_logic: &str,
    ) -> String {
        format!(
            "Decision Point: {}\n\
             Patient answered: {}\n\
             \n\
             Clinical Logic: {}\n\
             \n\
             Result: Proceeding to: {}\n\
             \n\
             This decision was based on clinical evidence and guidelines \
             incorporated into the decision tree logic.",
            node_label, answer, decision_logic, next_node_label
        )
    }
}

// Helper functions

fn extract_quantity_from_answer(value: &serde_json::Value) -> Option<FhirQuantity> {
    if let Some(num) = value.as_f64() {
        Some(FhirQuantity {
            value: num,
            unit: None,
            code: None,
        })
    } else if let Some(num) = value.as_i64() {
        Some(FhirQuantity {
            value: num as f64,
            unit: None,
            code: None,
        })
    } else {
        None
    }
}

fn extract_code_from_answer(value: &serde_json::Value) -> Option<FhirCodeableConcept> {
    if let Some(str_val) = value.as_str() {
        Some(FhirCodeableConcept {
            coding: Some(vec![FhirCoding {
                system: "http://snomed.info/sct".to_string(),
                code: str_val.to_string(),
                display: None,
            }]),
            text: Some(str_val.to_string()),
        })
    } else if value.is_boolean() {
        Some(FhirCodeableConcept {
            coding: Some(vec![FhirCoding {
                system: "http://loinc.org".to_string(),
                code: if value.as_bool().unwrap_or(false) {
                    "LA33-6"
                } else {
                    "LA32-8"
                }
                .to_string(),
                display: Some(
                    if value.as_bool().unwrap_or(false) {
                        "Yes"
                    } else {
                        "No"
                    }
                    .to_string(),
                ),
            }]),
            text: None,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_to_audit_event() {
        let event = FhirExportAdapter::session_to_audit_event(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "Fever Assessment",
            Utc::now(),
            Some(Utc::now()),
            Some("dr-smith".to_string()),
            Some("pt-123".to_string()),
            Some("Mild Fever".to_string()),
        );

        assert_eq!(event.action, "E");
        assert_eq!(event.outcome, Some("0".to_string()));
    }

    #[test]
    fn test_decision_justification() {
        let justification = FhirExportAdapter::generate_decision_justification(
            "Febrile?",
            "Yes",
            "Fever Duration",
            "If febrile, assess duration",
        );

        assert!(justification.contains("Febrile?"));
        assert!(justification.contains("Yes"));
        assert!(justification.contains("Fever Duration"));
    }
}
