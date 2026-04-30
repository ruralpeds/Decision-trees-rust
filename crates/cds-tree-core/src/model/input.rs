use serde::{Deserialize, Serialize};

/// The input required from the clinician at this node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeInput {
    Boolean(BooleanInput),
    SingleSelect(SingleSelectInput),
    MultiSelect(MultiSelectInput),
    Slider(SliderInput),
    Numeric(NumericInput),
    Computed(ComputedInput),
    /// Display-only, no input required
    Display,
}

/// Binary yes/no input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanInput {
    #[serde(default = "default_true_label")]
    pub true_label: String,
    #[serde(default = "default_false_label")]
    pub false_label: String,
    pub true_sublabel: Option<String>,
    pub false_sublabel: Option<String>,
    pub true_clinical_note: Option<String>,
    pub false_clinical_note: Option<String>,
    pub fhir_prefill: Option<FhirPrefillMapping>,
}

fn default_true_label() -> String {
    "Yes".to_string()
}

fn default_false_label() -> String {
    "No".to_string()
}

/// Select exactly one option from a list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleSelectInput {
    pub options: Vec<SelectOption>,
    #[serde(default = "default_render_hint")]
    pub render_hint: SelectRenderHint,
    pub fhir_prefill: Option<FhirPrefillMapping>,
}

fn default_render_hint() -> SelectRenderHint {
    SelectRenderHint::Buttons
}

/// Select one or more options from a list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSelectInput {
    pub options: Vec<SelectOption>,
    #[serde(default)]
    pub min_selections: u8,
    pub max_selections: Option<u8>,
    pub none_option: Option<SelectOption>,
    pub fhir_prefill: Option<FhirPrefillMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub id: String,
    pub label: String,
    pub sublabel: Option<String>,
    pub clinical_note: Option<String>,
    pub severity_hint: Option<String>,
    pub snomed_code: Option<String>,
    pub icd10_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum SelectRenderHint {
    #[serde(rename = "buttons")]
    Buttons,
    #[serde(rename = "radio_list")]
    RadioList,
    #[serde(rename = "dropdown")]
    Dropdown,
    #[serde(rename = "button_grid")]
    ButtonGrid,
}

/// Continuous range slider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderInput {
    pub min: f64,
    pub max: f64,
    #[serde(default = "default_step")]
    pub step: f64,
    pub unit_ucum: String,
    pub unit_display: String,
    #[serde(default)]
    pub precision: u8,
    pub reference_range: Option<ReferenceRange>,
    pub low_label: Option<String>,
    pub high_label: Option<String>,
    pub default_value: Option<f64>,
    pub fhir_prefill: Option<FhirPrefillMapping>,
}

fn default_step() -> f64 {
    1.0
}

/// Precise numeric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericInput {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub unit_ucum: String,
    pub unit_display: String,
    #[serde(default)]
    pub precision: u8,
    pub reference_range: Option<ReferenceRange>,
    pub critical_low: Option<f64>,
    pub critical_high: Option<f64>,
    pub critical_low_message: Option<String>,
    pub critical_high_message: Option<String>,
    pub default_value: Option<f64>,
    pub fhir_prefill: Option<FhirPrefillMapping>,
    pub show_conversion: Option<UnitConversion>,
}

/// Auto-calculated from previous answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedInput {
    pub formula: ComputedFormula,
    pub unit_ucum: Option<String>,
    pub unit_display: Option<String>,
    #[serde(default)]
    pub precision: u8,
    pub reference_range: Option<ReferenceRange>,
    #[serde(default)]
    pub show_formula: bool,
    pub score_name: Option<String>,
    pub score_interpretation: Vec<ScoreInterpretation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComputedFormula {
    /// Simple arithmetic expression with {node_id} variable references
    Expression(String),
    /// Sum of weighted option IDs
    WeightedSum { variable_ids: Vec<uuid::Uuid>, weights: Vec<f64> },
    /// Built-in clinical formula
    BuiltIn(BuiltInFormula),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BuiltInFormula {
    BmiKgM2 {
        weight_node: uuid::Uuid,
        height_node: uuid::Uuid,
    },
    CorrectedGestationalAge {
        ga_node: uuid::Uuid,
        birth_date_node: uuid::Uuid,
    },
    CrclCockcroftGault {
        scr_node: uuid::Uuid,
        age_node: uuid::Uuid,
        weight_node: uuid::Uuid,
        sex_node: uuid::Uuid,
    },
    MapMmhg {
        sbp_node: uuid::Uuid,
        dbp_node: uuid::Uuid,
    },
    EgfrCkdEpi {
        scr_node: uuid::Uuid,
        age_node: uuid::Uuid,
        sex_node: uuid::Uuid,
        race_node: Option<uuid::Uuid>,
    },
    PediatricDose {
        dose_per_kg_node: uuid::Uuid,
        weight_node: uuid::Uuid,
        max_dose_mg: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRange {
    pub low: f64,
    pub high: f64,
    pub adjustment_formula: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitConversion {
    pub target_unit_ucum: String,
    pub target_unit_display: String,
    pub multiplier: f64,
    pub offset: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreInterpretation {
    pub min_score: f64,
    pub max_score: f64,
    pub label: String,
    pub severity: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirPrefillMapping {
    pub resource_type: String,
    pub fhir_path: String,
    pub unit_ucum: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_serialization() {
        let input = BooleanInput {
            true_label: "Yes".to_string(),
            false_label: "No".to_string(),
            true_sublabel: Some("Confirmed fever".to_string()),
            false_sublabel: None,
            true_clinical_note: None,
            false_clinical_note: None,
            fhir_prefill: None,
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("\"true_label\":\"Yes\""));
    }

    #[test]
    fn test_numeric_input_with_bounds() {
        let input = NumericInput {
            min: Some(0.0),
            max: Some(100.0),
            step: Some(0.1),
            unit_ucum: "kg".to_string(),
            unit_display: "kg".to_string(),
            precision: 1,
            reference_range: None,
            critical_low: Some(2.0),
            critical_high: Some(90.0),
            critical_low_message: Some("Critically low weight".to_string()),
            critical_high_message: None,
            default_value: None,
            fhir_prefill: None,
            show_conversion: None,
        };

        assert_eq!(input.unit_ucum, "kg");
        assert_eq!(input.precision, 1);
    }
}
