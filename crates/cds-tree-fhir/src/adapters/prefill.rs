use serde_json::{Value, json};
use uuid::Uuid;
use crate::models::{FhirObservation, FhirCodeableConcept};

/// FHIR prefill adapter
pub struct FhirPrefillAdapter;

impl FhirPrefillAdapter {
    /// Extract value from FHIR observation for a node input
    pub fn extract_observation_value(
        obs: &FhirObservation,
        expected_type: &str, // "numeric", "boolean", "select", "slider"
    ) -> Result<Value, String> {
        match expected_type {
            "numeric" | "slider" => {
                // Extract numeric value from quantity
                obs.value_quantity
                    .as_ref()
                    .map(|q| json!(q.value))
                    .ok_or_else(|| "No numeric value in observation".to_string())
            }
            "select" | "single_select" => {
                // Extract code from codeable concept
                obs.value_code
                    .as_ref()
                    .and_then(|cc| cc.coding.as_ref())
                    .and_then(|codes| codes.first())
                    .map(|coding| json!(coding.code.clone()))
                    .ok_or_else(|| "No code in observation".to_string())
            }
            "display" => {
                // Return the display text
                Ok(json!(obs.value_quantity
                    .as_ref()
                    .map(|q| format!("{} {}", q.value, q.unit.as_deref().unwrap_or("")))
                    .unwrap_or_else(|| "N/A".to_string())))
            }
            _ => Err(format!("Unknown input type: {}", expected_type)),
        }
    }

    /// Extract weight from observation (in kg)
    pub fn extract_weight_kg(obs: &FhirObservation) -> Result<f64, String> {
        obs.value_quantity
            .as_ref()
            .map(|q| {
                // Convert to kg if in pounds
                match q.code.as_deref() {
                    Some("lbs") => Ok(q.value / 2.205),
                    Some("kg") | _ => Ok(q.value),
                }
            })
            .ok_or_else(|| "No weight value".to_string())?
    }

    /// Extract gestational age (in weeks)
    pub fn extract_gestational_age_weeks(obs: &FhirObservation) -> Result<f64, String> {
        obs.value_quantity
            .as_ref()
            .map(|q| {
                // Convert to weeks if in days
                match q.code.as_deref() {
                    Some("days") => Ok(q.value / 7.0),
                    Some("weeks") | _ => Ok(q.value),
                }
            })
            .ok_or_else(|| "No gestational age value".to_string())?
    }

    /// Extract temperature (convert to Celsius)
    pub fn extract_temperature_celsius(obs: &FhirObservation) -> Result<f64, String> {
        obs.value_quantity
            .as_ref()
            .map(|q| {
                // Convert to Celsius if in Fahrenheit
                match q.code.as_deref() {
                    Some("Fahrenheit") | Some("F") => Ok((q.value - 32.0) * 5.0 / 9.0),
                    Some("Celsius") | Some("C") | _ => Ok(q.value),
                }
            })
            .ok_or_else(|| "No temperature value".to_string())?
    }

    /// Extract blood pressure (systolic/diastolic)
    pub fn extract_blood_pressure(obs: &FhirObservation) -> Result<(f64, f64), String> {
        // BP is typically represented as two separate observations or components
        // This is a simplified version
        obs.value_quantity
            .as_ref()
            .map(|q| (q.value, 0.0)) // Simplified
            .ok_or_else(|| "No blood pressure value".to_string())
    }

    /// Extract respiratory rate (breaths per minute)
    pub fn extract_respiratory_rate(obs: &FhirObservation) -> Result<f64, String> {
        obs.value_quantity
            .as_ref()
            .map(|q| q.value)
            .ok_or_else(|| "No respiratory rate value".to_string())
    }

    /// Check if observation matches LOINC code
    pub fn matches_loinc_code(obs: &FhirObservation, loinc_code: &str) -> bool {
        obs.code
            .coding
            .as_ref()
            .map(|codings| {
                codings.iter().any(|c| {
                    c.system == "http://loinc.org" && c.code == loinc_code
                })
            })
            .unwrap_or(false)
    }

    /// Check if observation matches SNOMED code
    pub fn matches_snomed_code(obs: &FhirObservation, snomed_code: &str) -> bool {
        obs.code
            .coding
            .as_ref()
            .map(|codings| {
                codings.iter().any(|c| {
                    c.system == "http://snomed.info/sct" && c.code == snomed_code
                })
            })
            .unwrap_or(false)
    }

    /// Get LOINC code from observation
    pub fn get_loinc_code(obs: &FhirObservation) -> Option<String> {
        obs.code
            .coding
            .as_ref()
            .and_then(|codings| {
                codings
                    .iter()
                    .find(|c| c.system == "http://loinc.org")
                    .map(|c| c.code.clone())
            })
    }

    /// Format observation for display in breadcrumb
    pub fn format_for_display(obs: &FhirObservation) -> String {
        match (&obs.value_quantity, &obs.value_code) {
            (Some(qty), _) => {
                format!(
                    "{} {}",
                    qty.value,
                    qty.unit.as_deref().unwrap_or("")
                )
            }
            (_, Some(code)) => {
                code.text
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string())
            }
            _ => "N/A".to_string(),
        }
    }
}

// ============================================================================
// Built-in Clinical Calculators
// ============================================================================

/// Clinical calculator helper
pub struct ClinicalCalculators;

impl ClinicalCalculators {
    /// Calculate BMI from weight (kg) and height (cm)
    pub fn calculate_bmi(weight_kg: f64, height_cm: f64) -> f64 {
        let height_m = height_cm / 100.0;
        weight_kg / (height_m * height_m)
    }

    /// Calculate corrected gestational age (for preemie, in weeks)
    pub fn calculate_corrected_ga(
        chronological_age_weeks: f64,
        birth_ga_weeks: f64,
    ) -> f64 {
        chronological_age_weeks - (40.0 - birth_ga_weeks)
    }

    /// Calculate creatinine clearance (Cockcroft-Gault)
    pub fn calculate_crcl(
        age_years: f64,
        weight_kg: f64,
        creatinine_mg_dl: f64,
        is_male: bool,
    ) -> f64 {
        let factor = if is_male { 1.0 } else { 0.85 };
        ((140.0 - age_years) * weight_kg * factor) / (72.0 * creatinine_mg_dl)
    }

    /// Calculate mean arterial pressure
    pub fn calculate_map(systolic: f64, diastolic: f64) -> f64 {
        diastolic + (1.0 / 3.0) * (systolic - diastolic)
    }

    /// Calculate pediatric dose (mg/kg)
    pub fn calculate_pediatric_dose(
        weight_kg: f64,
        dose_per_kg: f64,
    ) -> f64 {
        weight_kg * dose_per_kg
    }

    /// Classify fever severity
    pub fn classify_fever_severity(temp_celsius: f64) -> String {
        match temp_celsius {
            t if t < 38.0 => "normal".to_string(),
            t if t < 38.5 => "low".to_string(),
            t if t < 39.5 => "moderate".to_string(),
            _ => "high".to_string(),
        }
    }

    /// Classify respiratory distress severity (RR in breaths/min)
    pub fn classify_respiratory_distress(
        resp_rate: f64,
        age_months: f64,
    ) -> String {
        // Simplified classification
        let upper_limit = match age_months {
            m if m < 2.0 => 60.0,
            m if m < 12.0 => 50.0,
            m if m < 60.0 => 40.0,
            _ => 30.0,
        };

        if resp_rate > upper_limit * 1.5 {
            "severe".to_string()
        } else if resp_rate > upper_limit {
            "moderate".to_string()
        } else {
            "normal".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmi_calculation() {
        let bmi = ClinicalCalculators::calculate_bmi(70.0, 175.0);
        assert!((bmi - 22.86).abs() < 0.1);
    }

    #[test]
    fn test_corrected_ga_calculation() {
        let corrected = ClinicalCalculators::calculate_corrected_ga(12.0, 28.0);
        assert_eq!(corrected, 0.0); // 12 - (40 - 28) = 0
    }

    #[test]
    fn test_fever_classification() {
        assert_eq!(
            ClinicalCalculators::classify_fever_severity(37.5),
            "normal"
        );
        assert_eq!(
            ClinicalCalculators::classify_fever_severity(38.2),
            "low"
        );
        assert_eq!(
            ClinicalCalculators::classify_fever_severity(39.0),
            "moderate"
        );
    }
}
