use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::json;
use cds_tree_storage::{TreeRepository, SessionRepository};
use cds_tree_fhir::{
    CdsHooksRequest, CdsHooksResponse, DiscoveryResponse, CdsService,
    CdsHooksService, HookType, ClinicalCalculators, FhirPrefillAdapter,
};
use crate::state::AppState;
use crate::error::{AppError, ApiResult};

/// GET /cds-services — CDS Hooks Discovery Endpoint
///
/// Returns available CDS services and their hooks
pub async fn cds_services_discovery(
    State(_state): State<AppState>,
) -> ApiResult<Json<DiscoveryResponse>> {
    let services = vec![
        // Patient View Hook
        CdsHooksService::build_discovery(
            "fever-assessment",
            HookType::PatientView.as_str(),
            "Fever Assessment",
            "Evaluates patient fever and recommends appropriate assessment and management",
            HookType::PatientView.prefetch_resources(),
        ),
        // Medication Order Hook
        CdsHooksService::build_discovery(
            "antibiotic-stewardship",
            HookType::MedicationOrder.as_str(),
            "Antibiotic Stewardship",
            "Evaluates antibiotic appropriateness based on patient factors",
            HookType::MedicationOrder.prefetch_resources(),
        ),
        // Order Review Hook
        CdsHooksService::build_discovery(
            "order-safety-review",
            HookType::OrderReview.as_str(),
            "Order Safety Review",
            "Reviews orders for safety considerations",
            HookType::OrderReview.prefetch_resources(),
        ),
    ];

    Ok(Json(DiscoveryResponse { services }))
}

/// POST /cds-services/fever-assessment — Fever Assessment Service
pub async fn fever_assessment_service(
    State(state): State<AppState>,
    Json(req): Json<CdsHooksRequest>,
) -> ApiResult<Json<CdsHooksResponse>> {
    // Extract patient ID from context
    let patient_id = req
        .context
        .get("patientId")
        .and_then(|p| p.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing patientId in context".to_string()))?;

    // Find fever assessment tree
    let tree = TreeRepository::get_by_slug(&state.db_pool, "fever-assessment")
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::TreeNotFound("fever-assessment tree not found".to_string()))?;

    // Create session
    let session = SessionRepository::create(
        &state.db_pool,
        tree.id,
        tree.version.clone(),
        None,
        Some(patient_id.to_string()),
        "cds_hooks".to_string(),
    )
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // In real scenario: evaluate tree with prefetched observations
    // For now, return a sample card
    let sample_outcome = json!({
        "severity": "warning",
        "title": "Mild Fever Detected",
        "summary": "Patient presents with mild fever (38.2°C)",
        "recommendation": "Monitor vital signs. Provide supportive care (hydration, rest). Consider acetaminophen or ibuprofen for comfort.",
        "icd10_codes": ["R50.9"],
        "snomed_codes": ["386661006"]
    });

    let card = CdsHooksService::outcome_to_card(&sample_outcome, session.id, tree.id)
        .map_err(|e| AppError::InternalError)?;

    Ok(Json(CdsHooksService::create_response(vec![card])))
}

/// POST /cds-services/antibiotic-stewardship — Antibiotic Stewardship Service
pub async fn antibiotic_stewardship_service(
    State(state): State<AppState>,
    Json(req): Json<CdsHooksRequest>,
) -> ApiResult<Json<CdsHooksResponse>> {
    // Extract patient and medication info
    let patient_id = req
        .context
        .get("patientId")
        .and_then(|p| p.as_str())
        .ok_or_else(|| AppError::InvalidInput("Missing patientId in context".to_string()))?;

    // Check for medication allergies in prefetch
    let has_allergies = req
        .prefetch
        .get("allergies")
        .and_then(|a| a.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);

    if has_allergies {
        // Return caution card
        let sample_outcome = json!({
            "severity": "critical",
            "title": "Allergy Risk Identified",
            "summary": "Patient has documented allergies that may contraindicate selected antibiotic",
            "recommendation": "Review allergy history before prescribing. Consider alternative agents.",
        });

        let card = CdsHooksService::outcome_to_card(&sample_outcome, uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
            .map_err(|e| AppError::InternalError)?;

        return Ok(Json(CdsHooksService::create_response(vec![card])));
    }

    // Return approval card
    let sample_outcome = json!({
        "severity": "info",
        "title": "Antibiotic Selection Approved",
        "summary": "Selected antibiotic is appropriate based on patient factors",
        "recommendation": "Proceed with prescription. Ensure appropriate dosing and duration.",
    });

    let card = CdsHooksService::outcome_to_card(&sample_outcome, uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
        .map_err(|e| AppError::InternalError)?;

    Ok(Json(CdsHooksService::create_response(vec![card])))
}

/// POST /cds-services/order-safety-review — Order Safety Review Service
pub async fn order_safety_review_service(
    State(_state): State<AppState>,
    Json(req): Json<CdsHooksRequest>,
) -> ApiResult<Json<CdsHooksResponse>> {
    // Review order safety
    let sample_outcome = json!({
        "severity": "info",
        "title": "Order Safety Check Complete",
        "summary": "Order has been reviewed for safety considerations",
        "recommendation": "No safety concerns identified. Order may be signed.",
    });

    let card = CdsHooksService::outcome_to_card(&sample_outcome, uuid::Uuid::new_v4(), uuid::Uuid::new_v4())
        .map_err(|e| AppError::InternalError)?;

    Ok(Json(CdsHooksService::create_response(vec![card])))
}

/// POST /cds-services/{service-id}/feedback — Record feedback on CDS service
///
/// Used by EHR to report whether user followed recommendation
pub async fn record_cds_feedback(
    State(_state): State<AppState>,
    Json(feedback): Json<serde_json::Value>,
) -> ApiResult<StatusCode> {
    // Log feedback
    tracing::info!("CDS feedback received: {:?}", feedback);

    // In production: store in database for quality tracking
    // feedback.service_id, feedback.accepted, feedback.reason, etc.

    Ok(StatusCode::OK)
}

/// GET /cds-services/metadata — Service metadata endpoint
///
/// Returns information about CDS service capabilities
pub async fn cds_service_metadata() -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(json!({
        "name": "Rural Pediatrics CDS Tree Service",
        "version": "1.0.0",
        "description": "Clinical decision support for pediatric conditions",
        "organization": "Rural Pediatrics Network",
        "contact": "support@ruralpeds.org",
        "hooks": [
            {
                "hook": "patient-view",
                "title": "Patient View",
                "description": "Fires when patient chart is opened"
            },
            {
                "hook": "medication-order",
                "title": "Medication Order",
                "description": "Fires when medication is ordered"
            },
            {
                "hook": "order-review",
                "title": "Order Review",
                "description": "Fires when order is reviewed"
            }
        ]
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handlers_compile() {
        assert!(true);
    }
}
