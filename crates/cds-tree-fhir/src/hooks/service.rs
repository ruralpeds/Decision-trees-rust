use crate::models::{
    CdsHooksRequest, CdsHooksResponse, CdsCard, CdsSource, CdsSuggestion, CdsAction,
    CdsLink,
};
use serde_json::{json, Value};
use uuid::Uuid;

/// CDS Hooks service builder
pub struct CdsHooksService;

impl CdsHooksService {
    /// Convert tree outcome to CDS card
    pub fn outcome_to_card(
        outcome_payload: &Value,
        session_id: Uuid,
        tree_id: Uuid,
    ) -> Result<CdsCard, String> {
        let severity = outcome_payload
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("info");

        let title = outcome_payload
            .get("title")
            .and_then(|s| s.as_str())
            .unwrap_or("Decision");

        let summary = outcome_payload
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or(title);

        let recommendation = outcome_payload
            .get("recommendation")
            .and_then(|s| s.as_str());

        let indicator = match severity {
            "critical" => "hard-stop",
            "warning" => "warning",
            "info" => "info",
            _ => "info",
        };

        // Build suggestions from actions
        let suggestions = outcome_payload
            .get("recommended_actions")
            .and_then(|a| a.as_array())
            .map(|actions| {
                actions
                    .iter()
                    .enumerate()
                    .map(|(idx, action)| {
                        let action_type = action
                            .get("action_type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("create");

                        CdsSuggestion {
                            uuid: format!("{}-action-{}", session_id, idx),
                            label: action
                                .get("title")
                                .and_then(|t| t.as_str())
                                .unwrap_or("Recommended Action")
                                .to_string(),
                            actions: Some(vec![CdsAction {
                                action_type: action_type.to_string(),
                                description: action
                                    .get("description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                resource: action.get("resource").cloned(),
                            }]),
                        }
                    })
                    .collect()
            });

        Ok(CdsCard {
            uuid: Uuid::new_v4().to_string(),
            summary: summary.to_string(),
            indicator: indicator.to_string(),
            detail: recommendation.map(|r| r.to_string()),
            source: Some(CdsSource {
                label: "CDS Tree".to_string(),
                url: Some(format!("https://cds.example.com/trees/{}", tree_id)),
            }),
            suggestions,
            links: Some(vec![
                CdsLink {
                    label: "View Full Assessment".to_string(),
                    url: format!("https://cds.example.com/sessions/{}", session_id),
                    open_in_new_tab: Some(true),
                },
                CdsLink {
                    label: "Review Evidence".to_string(),
                    url: format!("https://cds.example.com/trees/{}", tree_id),
                    open_in_new_tab: Some(true),
                },
            ]),
        })
    }

    /// Create response from multiple outcomes
    pub fn create_response(cards: Vec<CdsCard>) -> CdsHooksResponse {
        CdsHooksResponse {
            cards,
            errors: None,
        }
    }

    /// Create error response
    pub fn create_error_response(errors: Vec<String>) -> CdsHooksResponse {
        CdsHooksResponse {
            cards: vec![],
            errors: Some(errors),
        }
    }

    /// Build discovery response for a specific hook
    pub fn build_discovery(
        service_id: &str,
        hook_type: &str,
        title: &str,
        description: &str,
        prefetch_keys: Vec<(&str, &str)>,
    ) -> super::super::models::CdsService {
        let prefetch = prefetch_keys
            .into_iter()
            .map(|(key, fhir_query)| (key.to_string(), fhir_query.to_string()))
            .collect();

        super::super::models::CdsService {
            id: service_id.to_string(),
            hook: hook_type.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            prefetch: Some(prefetch),
        }
    }
}

// ============================================================================
// Common Hook Types
// ============================================================================

pub enum HookType {
    PatientView,
    MedicationOrder,
    OrderReview,
    OrderSign,
}

impl HookType {
    pub fn as_str(&self) -> &str {
        match self {
            HookType::PatientView => "patient-view",
            HookType::MedicationOrder => "medication-order",
            HookType::OrderReview => "order-review",
            HookType::OrderSign => "order-sign",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            HookType::PatientView => {
                "Fires when a patient's chart is opened for review"
            }
            HookType::MedicationOrder => {
                "Fires when a medication is being ordered"
            }
            HookType::OrderReview => {
                "Fires when an order is being reviewed"
            }
            HookType::OrderSign => {
                "Fires when an order is about to be signed"
            }
        }
    }

    pub fn prefetch_resources(&self) -> Vec<(&str, &str)> {
        match self {
            HookType::PatientView => vec![
                ("patient", "Patient/{{context.patientId}}"),
                ("observations", "Observation?patient={{context.patientId}}&_sort=-date&_count=10"),
                ("conditions", "Condition?patient={{context.patientId}}"),
            ],
            HookType::MedicationOrder => vec![
                ("patient", "Patient/{{context.patientId}}"),
                ("medications", "MedicationRequest?patient={{context.patientId}}&status=active"),
                ("allergies", "AllergyIntolerance?patient={{context.patientId}}"),
            ],
            HookType::OrderReview => vec![
                ("patient", "Patient/{{context.patientId}}"),
                ("order", "Order/{{context.orderId}}"),
            ],
            HookType::OrderSign => vec![
                ("patient", "Patient/{{context.patientId}}"),
                ("encounter", "Encounter/{{context.encounterId}}"),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_to_card() {
        let outcome = json!({
            "severity": "warning",
            "title": "Mild Fever",
            "summary": "Patient has mild fever",
            "recommendation": "Monitor and provide supportive care"
        });

        let card = CdsHooksService::outcome_to_card(
            &outcome,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .expect("Should convert outcome to card");

        assert_eq!(card.summary, "Patient has mild fever");
        assert_eq!(card.indicator, "warning");
    }

    #[test]
    fn test_hook_types() {
        assert_eq!(HookType::PatientView.as_str(), "patient-view");
        assert_eq!(HookType::MedicationOrder.as_str(), "medication-order");
        let prefetch = HookType::PatientView.prefetch_resources();
        assert!(!prefetch.is_empty());
    }
}
