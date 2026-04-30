use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::model::{ClinicalDecisionTree, NodeKind};
use crate::error::ValidationError;

/// Validates that a tree structure is well-formed
pub struct TreeValidator;

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl TreeValidator {
    /// Perform a complete validation of the tree structure
    pub fn validate(tree: &ClinicalDecisionTree) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check root node exists
        if tree.root_node_id == Uuid::nil() {
            errors.push("Root node ID is nil".to_string());
            return ValidationReport {
                is_valid: false,
                errors,
                warnings,
            };
        }

        let root_exists = tree.nodes.iter().any(|n| n.id == tree.root_node_id);
        if !root_exists {
            errors.push(format!("Root node {} does not exist", tree.root_node_id));
            return ValidationReport {
                is_valid: false,
                errors,
                warnings,
            };
        }

        // Build node map for fast lookup
        let mut node_map: HashMap<Uuid, _> = HashMap::new();
        for node in &tree.nodes {
            node_map.insert(node.id, node);
        }

        // Check for orphan nodes
        for node in &tree.nodes {
            if node.id == tree.root_node_id {
                // Root node must have parent_id == None
                if node.parent_id.is_some() {
                    errors.push(format!("Root node {} has a parent", node.id));
                }
            } else {
                // All non-root nodes must have parent_id set and parent must exist
                if let Some(parent_id) = node.parent_id {
                    if !node_map.contains_key(&parent_id) {
                        errors.push(format!(
                            "Node {} parent {} does not exist",
                            node.id, parent_id
                        ));
                    }
                } else {
                    errors.push(format!("Node {} has no parent and is not root", node.id));
                }
            }
        }

        // Check for dangling edges
        for node in &tree.nodes {
            for edge in &node.children {
                if !node_map.contains_key(&edge.child_node_id) {
                    errors.push(format!(
                        "Node {} has dangling edge to child {}",
                        node.id, edge.child_node_id
                    ));
                }
            }
        }

        // Check for reachability from root (simple DFS)
        let reachable = Self::find_reachable(&tree.root_node_id, &node_map);
        for node in &tree.nodes {
            if !reachable.contains(&node.id) {
                warnings.push(format!("Node {} is unreachable from root", node.id));
            }
        }

        // Check that leaf nodes are outcome nodes
        for node in &tree.nodes {
            if node.children.is_empty() {
                if !matches!(node.node_type, NodeKind::Outcome) {
                    errors.push(format!(
                        "Leaf node {} is not an Outcome node",
                        node.id
                    ));
                }
            }
        }

        // Validate each node
        for node in &tree.nodes {
            if let Err(e) = node.validate() {
                errors.push(format!("Node {} validation failed: {}", node.id, e));
            }
        }

        // Check for cycles (should never happen with tree structure, but be safe)
        if let Err(e) = Self::check_cycles(&node_map) {
            errors.push(e);
        }

        let is_valid = errors.is_empty();

        ValidationReport {
            is_valid,
            errors,
            warnings,
        }
    }

    fn find_reachable(
        start: &Uuid,
        node_map: &HashMap<Uuid, &crate::model::node::DecisionNode>,
    ) -> HashSet<Uuid> {
        let mut visited = HashSet::new();
        let mut stack = vec![*start];

        while let Some(node_id) = stack.pop() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id);

            if let Some(node) = node_map.get(&node_id) {
                for edge in &node.children {
                    if !visited.contains(&edge.child_node_id) {
                        stack.push(edge.child_node_id);
                    }
                }
            }
        }

        visited
    }

    fn check_cycles(node_map: &HashMap<Uuid, &crate::model::node::DecisionNode>) -> Result<(), String> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &node_id in node_map.keys() {
            if !visited.contains(&node_id) {
                Self::dfs_cycle_check(node_id, node_map, &mut visited, &mut rec_stack)?;
            }
        }

        Ok(())
    }

    fn dfs_cycle_check(
        node_id: Uuid,
        node_map: &HashMap<Uuid, &crate::model::node::DecisionNode>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> Result<(), String> {
        visited.insert(node_id);
        rec_stack.insert(node_id);

        if let Some(node) = node_map.get(&node_id) {
            for edge in &node.children {
                if !visited.contains(&edge.child_node_id) {
                    Self::dfs_cycle_check(edge.child_node_id, node_map, visited, rec_stack)?;
                } else if rec_stack.contains(&edge.child_node_id) {
                    return Err(format!("Cycle detected involving nodes {} and {}", 
                        node_id, edge.child_node_id));
                }
            }
        }

        rec_stack.remove(&node_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ClinicalDecisionTree, DecisionNode, OutcomePayload, SeverityLevel};

    #[test]
    fn test_valid_tree() {
        let mut tree = ClinicalDecisionTree::new("test", "Test Tree");
        let root_id = Uuid::new_v4();
        tree.root_node_id = root_id;

        let mut root = DecisionNode::new(tree.id, "Root question");
        root.id = root_id;
        root.aria_label = "Root".to_string();
        root.node_type = NodeKind::Outcome;
        root.outcome = Some(OutcomePayload::new(
            SeverityLevel::Info,
            "Done",
            "Completed",
        ));

        tree.nodes.push(root);

        let report = TreeValidator::validate(&tree);
        assert!(report.is_valid);
    }

    #[test]
    fn test_orphan_node() {
        let mut tree = ClinicalDecisionTree::new("test", "Test Tree");
        let root_id = Uuid::new_v4();
        tree.root_node_id = root_id;

        let mut root = DecisionNode::new(tree.id, "Root");
        root.id = root_id;
        root.aria_label = "Root".to_string();
        root.node_type = NodeKind::Outcome;
        root.outcome = Some(OutcomePayload::new(SeverityLevel::Info, "Done", "Completed"));
        tree.nodes.push(root);

        let orphan_id = Uuid::new_v4();
        let mut orphan = DecisionNode::new(tree.id, "Orphan");
        orphan.id = orphan_id;
        orphan.parent_id = None; // No parent, not root
        orphan.aria_label = "Orphan".to_string();
        tree.nodes.push(orphan);

        let report = TreeValidator::validate(&tree);
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("orphan")));
    }

    #[test]
    fn test_dangling_edge() {
        let mut tree = ClinicalDecisionTree::new("test", "Test Tree");
        let root_id = Uuid::new_v4();
        tree.root_node_id = root_id;

        let mut root = DecisionNode::new(tree.id, "Root");
        root.id = root_id;
        root.aria_label = "Root".to_string();
        root.children.push(crate::model::node::ChildEdge {
            id: Uuid::new_v4(),
            child_node_id: Uuid::new_v4(), // Non-existent child
            condition: crate::model::node::EdgeCondition::Always,
            label: None,
            priority: 0,
        });
        tree.nodes.push(root);

        let report = TreeValidator::validate(&tree);
        assert!(!report.is_valid);
        assert!(report.errors.iter().any(|e| e.contains("dangling")));
    }
}
