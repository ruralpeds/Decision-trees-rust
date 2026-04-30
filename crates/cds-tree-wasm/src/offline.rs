use wasm_bindgen::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Offline storage for sessions and trees
#[wasm_bindgen]
pub struct OfflineStorage;

#[wasm_bindgen]
impl OfflineStorage {
    /// Save a tree locally for offline access
    pub fn save_tree(tree_id: &str, tree_json: &str) -> Result<(), JsValue> {
        get_local_storage()?
            .set_item(&format!("tree:{}", tree_id), tree_json)
            .map_err(|_| JsValue::from_str("Failed to save tree to localStorage"))?;
        Ok(())
    }

    /// Load a tree from local storage
    pub fn load_tree(tree_id: &str) -> Result<String, JsValue> {
        let storage = get_local_storage()?;
        storage
            .get_item(&format!("tree:{}", tree_id))
            .map_err(|_| JsValue::from_str("Failed to read tree from localStorage"))?
            .ok_or_else(|| JsValue::from_str("Tree not found in offline storage"))
    }

    /// List all locally stored trees
    pub fn list_trees() -> Result<String, JsValue> {
        let storage = get_local_storage()?;
        let length = storage.length().map_err(|_| JsValue::from_str("Failed to get storage length"))?;
        
        let mut trees = Vec::new();
        for i in 0..length {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with("tree:") {
                    trees.push(key.replace("tree:", ""));
                }
            }
        }

        serde_json::to_string(&trees)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Save an offline session
    pub fn save_session(session_id: &str, session_json: &str) -> Result<(), JsValue> {
        get_local_storage()?
            .set_item(&format!("session:{}", session_id), session_json)
            .map_err(|_| JsValue::from_str("Failed to save session"))?;
        Ok(())
    }

    /// Load an offline session
    pub fn load_session(session_id: &str) -> Result<String, JsValue> {
        let storage = get_local_storage()?;
        storage
            .get_item(&format!("session:{}", session_id))
            .map_err(|_| JsValue::from_str("Failed to read session"))?
            .ok_or_else(|| JsValue::from_str("Session not found"))
    }

    /// List all offline sessions
    pub fn list_sessions() -> Result<String, JsValue> {
        let storage = get_local_storage()?;
        let length = storage.length().map_err(|_| JsValue::from_str("Failed to get storage length"))?;
        
        let mut sessions = Vec::new();
        for i in 0..length {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with("session:") {
                    sessions.push(key.replace("session:", ""));
                }
            }
        }

        serde_json::to_string(&sessions)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Add a session to sync queue (when offline)
    pub fn queue_for_sync(session_id: &str, session_json: &str) -> Result<(), JsValue> {
        let storage = get_local_storage()?;
        
        // Get existing queue
        let queue_json = storage
            .get_item("sync_queue")
            .map_err(|_| JsValue::from_str("Failed to read sync queue"))?
            .unwrap_or_else(|| "[]".to_string());

        let mut queue: Vec<Value> = serde_json::from_str(&queue_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse sync queue: {}", e)))?;

        // Add session to queue
        queue.push(json!({
            "session_id": session_id,
            "session_json": session_json,
            "queued_at": chrono::Utc::now().to_rfc3339(),
            "synced": false
        }));

        // Save updated queue
        storage
            .set_item("sync_queue", &serde_json::to_string(&queue).unwrap())
            .map_err(|_| JsValue::from_str("Failed to save sync queue"))?;

        Ok(())
    }

    /// Get pending sync queue
    pub fn get_sync_queue() -> Result<String, JsValue> {
        let storage = get_local_storage()?;
        
        let queue_json = storage
            .get_item("sync_queue")
            .map_err(|_| JsValue::from_str("Failed to read sync queue"))?
            .unwrap_or_else(|| "[]".to_string());

        Ok(queue_json)
    }

    /// Mark session as synced
    pub fn mark_synced(session_id: &str) -> Result<(), JsValue> {
        let storage = get_local_storage()?;
        
        let queue_json = storage
            .get_item("sync_queue")
            .map_err(|_| JsValue::from_str("Failed to read sync queue"))?
            .unwrap_or_else(|| "[]".to_string());

        let mut queue: Vec<Value> = serde_json::from_str(&queue_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse sync queue: {}", e)))?;

        // Mark as synced
        for item in &mut queue {
            if item["session_id"].as_str() == Some(session_id) {
                item["synced"] = Value::Bool(true);
                item["synced_at"] = Value::String(chrono::Utc::now().to_rfc3339());
            }
        }

        storage
            .set_item("sync_queue", &serde_json::to_string(&queue).unwrap())
            .map_err(|_| JsValue::from_str("Failed to save sync queue"))?;

        Ok(())
    }

    /// Get storage statistics
    pub fn get_stats() -> Result<String, JsValue> {
        let storage = get_local_storage()?;
        let length = storage.length().map_err(|_| JsValue::from_str("Failed to get storage length"))?;

        let mut trees_count = 0;
        let mut sessions_count = 0;
        let mut total_size = 0;

        for i in 0..length {
            if let Ok(Some(key)) = storage.key(i) {
                if key.starts_with("tree:") {
                    trees_count += 1;
                    if let Ok(Some(value)) = storage.get_item(&key) {
                        total_size += value.len();
                    }
                } else if key.starts_with("session:") {
                    sessions_count += 1;
                    if let Ok(Some(value)) = storage.get_item(&key) {
                        total_size += value.len();
                    }
                }
            }
        }

        let stats = json!({
            "trees_count": trees_count,
            "sessions_count": sessions_count,
            "total_items": length,
            "total_size_bytes": total_size,
            "total_size_kb": total_size as f64 / 1024.0
        });

        serde_json::to_string(&stats)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Clear all offline data
    pub fn clear_all() -> Result<(), JsValue> {
        get_local_storage()?
            .clear();
        Ok(())
    }
}

/// Get browser localStorage
fn get_local_storage() -> Result<web_sys::Storage, JsValue> {
    web_sys::window()
        .ok_or_else(|| JsValue::from_str("No window object"))?
        .local_storage()
        .map_err(|_| JsValue::from_str("Failed to access localStorage"))?
        .ok_or_else(|| JsValue::from_str("localStorage not available"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_storage_placeholder() {
        // Offline storage tests require browser environment
        assert!(true);
    }
}
