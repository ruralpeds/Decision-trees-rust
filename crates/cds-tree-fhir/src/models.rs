use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// CDS Hooks v2.0 Models
// ============================================================================

/// CDS Hooks Discovery Response
/// Describes available CDS services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub services: Vec<CdsService>,
}

/// CDS Service Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsService {
    pub id: String,
    pub hook: String,
    pub title: String,
    pub description: String,
    pub prefetch: Option<std::collections::HashMap<String, String>>,
}

/// CDS Hooks Request (v2.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsHooksRequest {
    /// UUID of the request
    pub request_id: String,
    /// Hook type: patient-view, medication-order, order-review, etc.
    pub hook: String,
    /// Hook instance: specific patient or medication
    pub hook_instance: String,
    /// FHIR context (patient, encounter, medications, etc.)
    #[serde(default)]
    pub context: std::collections::HashMap<String, serde_json::Value>,
    /// Prefetch data (usually requested resources)
    #[serde(default)]
    pub prefetch: std::collections::HashMap<String, serde_json::Value>,
}

/// CDS Hooks Response (v2.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsHooksResponse {
    pub cards: Vec<CdsCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

/// CDS Card (recommendation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsCard {
    pub uuid: String,
    pub summary: String,
    pub indicator: String, // "info", "warning", "critical", "hard-stop"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<CdsSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<CdsSuggestion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<CdsLink>>,
}

/// Card source (evidence/guideline)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsSource {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Action suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsSuggestion {
    pub uuid: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<CdsAction>>,
}

/// Suggested action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsAction {
    pub action_type: String, // "create", "update", "delete", "append"
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<serde_json::Value>,
}

/// Hyperlink on card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdsLink {
    pub label: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_in_new_tab: Option<bool>,
}

// ============================================================================
// FHIR R4 Context Models
// ============================================================================

/// FHIR Patient (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPatient {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<FhirHumanName>>,
}

/// FHIR Human Name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirHumanName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
}

/// FHIR Observation (vital signs, labs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirObservation {
    pub id: String,
    pub code: FhirCodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<FhirQuantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_code: Option<FhirCodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_date_time: Option<String>,
}

/// FHIR Quantity (with value and unit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirQuantity {
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// FHIR Codeable Concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirCodeableConcept {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coding: Option<Vec<FhirCoding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// FHIR Coding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirCoding {
    pub system: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// FHIR Encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirEncounter {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<FhirPeriod>,
}

/// FHIR Period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPeriod {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

/// FHIR AuditEvent (for recording decisions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirAuditEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: FhirCodeableConcept,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<FhirPeriod>,
    pub recorded: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

// ============================================================================
// SMART Launch Models
// ============================================================================

/// SMART Launch Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartLaunchRequest {
    pub launch: String,
    pub iss: String, // FHIR server issuer
}

/// SMART Token Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTokenResponse {
    pub access_token: String,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub patient: String, // Patient FHIR ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// FHIR Server Metadata (CapabilityStatement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub introspection_endpoint: Option<String>,
}

// ============================================================================
// Prefill Mapping Models
// ============================================================================

/// Map FHIR observation/resource to node input value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPrefillMapping {
    pub node_id: Uuid,
    pub fhir_path: String, // FHIRPath expression (e.g., "Observation.value.value")
    pub loinc_code: Option<String>, // Filter observations by LOINC code
    pub snomed_code: Option<String>,
    pub transformation: Option<String>, // JS function or built-in (e.g., "celsius_to_fahrenheit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cds_hooks_request() {
        let req = CdsHooksRequest {
            request_id: "req-123".to_string(),
            hook: "patient-view".to_string(),
            hook_instance: "patient-456".to_string(),
            context: Default::default(),
            prefetch: Default::default(),
        };
        assert_eq!(req.hook, "patient-view");
    }

    #[test]
    fn test_cds_card() {
        let card = CdsCard {
            uuid: uuid::Uuid::new_v4().to_string(),
            summary: "Test recommendation".to_string(),
            indicator: "warning".to_string(),
            detail: None,
            source: None,
            suggestions: None,
            links: None,
        };
        assert_eq!(card.indicator, "warning");
    }
}
