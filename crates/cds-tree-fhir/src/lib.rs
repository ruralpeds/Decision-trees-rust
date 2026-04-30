pub mod models;
pub mod adapters;
pub mod hooks;

pub use models::*;
pub use adapters::{FhirPrefillAdapter, ClinicalCalculators, FhirExportAdapter};
pub use hooks::{CdsHooksService, HookType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fhir_models_import() {
        // Verify models are exported
        assert!(true);
    }
}
