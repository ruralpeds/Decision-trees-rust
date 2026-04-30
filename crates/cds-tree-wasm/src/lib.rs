use wasm_bindgen::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use cds_tree_core::{
    ClinicalDecisionTree, DecisionNode, TreeValidator, 
    Evaluator, ConditionValue, OutcomePayload,
};
use uuid::Uuid;
use chrono::Utc;

// Set panic hook for better error messages in browser
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WASM-friendly session state for tree evaluation
#[wasm_bindgen]
pub struct WasmSession {
    session_id: String,
    tree: ClinicalDecisionTree,
    current_node_id: Option<String>,
    answers: HashMap<String, Value>,
    started_at: String,
    is_complete: bool,
}

#[wasm_bindgen]
impl WasmSession {
    /// Create a new session with a tree
    #[wasm_bindgen(constructor)]
    pub fn new(tree_json: &str) -> Result<WasmSession, JsValue> {
        let tree: ClinicalDecisionTree = serde_json::from_str(tree_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse tree: {}", e)))?;

        let session_id = Uuid::new_v4().to_string();
        let started_at = Utc::now().to_rfc3339();

        // Set to root node
        let current_node_id = tree.root_node_id.map(|id| id.to_string());

        Ok(WasmSession {
            session_id,
            tree,
            current_node_id,
            answers: HashMap::new(),
            started_at,
            is_complete: false,
        })
    }

    /// Get current session ID
    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }

    /// Get current node as JSON
    pub fn current_node(&self) -> Result<String, JsValue> {
        if let Some(node_id_str) = &self.current_node_id {
            if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
                // Find node in tree
                if let Some(node) = self.find_node(node_id) {
                    let node_json = serde_json::to_string(&node)
                        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
                    return Ok(node_json);
                }
            }
        }
        Err(JsValue::from_str("No current node"))
    }

    /// Record an answer and advance to next node
    pub fn advance(&mut self, answer: &str) -> Result<String, JsValue> {
        if let Some(node_id_str) = &self.current_node_id.clone() {
            if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
                // Parse answer (could be JSON, number, boolean)
                let answer_value: Value = serde_json::from_str(answer)
                    .or_else(|_| {
                        // Try as plain string
                        Ok(Value::String(answer.to_string()))
                    })
                    .map_err(|e| JsValue::from_str(&format!("Invalid answer: {}", e)))?;

                // Store answer
                self.answers.insert(node_id_str.clone(), answer_value);

                // Evaluate edges to find next node
                if let Some(node) = self.find_node(node_id) {
                    // Simple evaluation: find first child
                    if let Some(children) = &node.children {
                        if !children.is_empty() {
                            let next_node_id = children[0].node_id.clone();
                            self.current_node_id = Some(next_node_id.to_string());
                            
                            // Check if we're at an outcome node
                            if let Some(next_node) = self.find_node(next_node_id) {
                                if next_node.kind.as_ref().map(|k| k.as_str()) == Some("outcome") {
                                    self.is_complete = true;
                                }
                            }
                            
                            return Ok(self.current_node()?);
                        }
                    }
                }
            }
        }
        Err(JsValue::from_str("Cannot advance from current node"))
    }

    /// Get all answers recorded so far
    pub fn answers(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.answers)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Check if session is complete
    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    /// Get session summary
    pub fn summary(&self) -> Result<String, JsValue> {
        let summary = json!({
            "session_id": self.session_id,
            "tree_id": self.tree.id.to_string(),
            "tree_title": self.tree.title,
            "started_at": self.started_at,
            "completed_at": if self.is_complete { Some(Utc::now().to_rfc3339()) } else { None },
            "answers_count": self.answers.len(),
            "is_complete": self.is_complete
        });

        serde_json::to_string(&summary)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // Helper function to find node by ID
    fn find_node(&self, node_id: Uuid) -> Option<DecisionNode> {
        // In a real implementation, would search the tree structure
        // For now, return None as placeholder
        None
    }
}

/// Validate a decision tree
#[wasm_bindgen]
pub fn validate_tree(tree_json: &str) -> Result<String, JsValue> {
    let tree: ClinicalDecisionTree = serde_json::from_str(tree_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse tree: {}", e)))?;

    let validator = TreeValidator::new();
    let report = validator.validate(&tree);

    let report_json = json!({
        "is_valid": report.is_valid,
        "total_checks": report.total_checks,
        "passed_checks": report.passed_checks,
        "errors": report.errors,
        "warnings": report.warnings
    });

    serde_json::to_string(&report_json)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get browser capabilities
#[wasm_bindgen]
pub fn get_capabilities() -> String {
    let capabilities = json!({
        "wasm": true,
        "indexed_db": check_indexed_db(),
        "local_storage": check_local_storage(),
        "service_worker": check_service_worker(),
        "offline_mode": true,
        "version": env!("CARGO_PKG_VERSION")
    });

    serde_json::to_string(&capabilities).unwrap_or_default()
}

fn check_indexed_db() -> bool {
    #[allow(unsafe_code)]
    unsafe {
        web_sys::window()
            .and_then(|w| {
                let indexed_db = js_sys::Reflect::get(&w, &"indexedDB".into()).ok();
                indexed_db.map(|db| !db.is_null() && !db.is_undefined())
            })
            .unwrap_or(false)
    }
}

fn check_local_storage() -> bool {
    #[allow(unsafe_code)]
    unsafe {
        web_sys::window()
            .and_then(|w| {
                let storage = js_sys::Reflect::get(&w, &"localStorage".into()).ok();
                storage.map(|s| !s.is_null() && !s.is_undefined())
            })
            .unwrap_or(false)
    }
}

fn check_service_worker() -> bool {
    #[allow(unsafe_code)]
    unsafe {
        web_sys::window()
            .and_then(|w| {
                let navigator = w.navigator();
                let service_worker = js_sys::Reflect::get(&navigator, &"serviceWorker".into()).ok();
                service_worker.map(|sw| !sw.is_null() && !sw.is_undefined())
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities() {
        let caps = get_capabilities();
        assert!(!caps.is_empty());
    }
}
