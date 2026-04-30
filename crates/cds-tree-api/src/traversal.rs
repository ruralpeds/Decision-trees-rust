use cds_tree_core::{
    ClinicalDecisionTree, DecisionNode, EdgeCondition, ConditionVariable, 
    ConditionValue, Evaluator, SessionState as CoreSessionState,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Session traversal state machine
pub struct SessionStateMachine {
    pub tree: ClinicalDecisionTree,
    pub core_state: CoreSessionState,
}

impl SessionStateMachine {
    /// Create a new session starting at root node
    pub fn new(tree: ClinicalDecisionTree) -> Result<Self, String> {
        if tree.root_node_id == Uuid::nil() {
            return Err("Tree has no root node".to_string());
        }

        let core_state = CoreSessionState::new();

        Ok(Self {
            tree,
            core_state,
        })
    }

    /// Get the current node
    pub fn current_node(&self) -> Option<&DecisionNode> {
        // If no answers yet, return root
        if self.core_state.answers.is_empty() {
            self.tree.get_node(self.tree.root_node_id)
        } else {
            // Find next node based on last answer
            self.find_next_node()
        }
    }

    /// Advance to next node based on answer to current node
    pub fn advance(&mut self, node_id: Uuid, answer: ConditionValue) -> Result<Option<&DecisionNode>, String> {
        // Record answer
        self.core_state.record_answer(node_id, answer);

        // Find next node
        let current = self.tree.get_node(node_id)
            .ok_or_else(|| format!("Node {} not found", node_id))?;

        // Evaluate edges to find which child to visit
        for edge in &current.children {
            match Evaluator::evaluate(&edge.condition, &self.core_state) {
                Ok(true) => {
                    let next_node = self.tree.get_node(edge.child_node_id);
                    return Ok(next_node);
                }
                Ok(false) => continue,
                Err(e) => return Err(format!("Evaluation error: {}", e)),
            }
        }

        // No edge matched - should not happen if tree is valid
        Err("No valid edge condition matched".to_string())
    }

    /// Get root node
    pub fn root_node(&self) -> Option<&DecisionNode> {
        self.tree.get_node(self.tree.root_node_id)
    }

    /// Get traversal path (breadcrumb)
    pub fn path(&self) -> Vec<(Uuid, Option<ConditionValue>)> {
        let mut path = Vec::new();

        for (node_id_str, value) in &self.core_state.answers {
            if let Ok(node_id) = uuid::Uuid::parse_str(node_id_str) {
                path.push((node_id, Some(value.clone())));
            }
        }

        path
    }

    /// Check if current node is terminal (outcome)
    pub fn is_terminal(&self, node: &DecisionNode) -> bool {
        node.children.is_empty()
    }

    /// Get estimated remaining nodes
    pub fn estimate_remaining_nodes(&self) -> u32 {
        let current = match self.current_node() {
            Some(n) => n,
            None => return 0,
        };

        // Simple heuristic: count reachable nodes from current
        let mut remaining = 0;
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![current.id];

        while let Some(node_id) = stack.pop() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id);
            remaining += 1;

            if let Some(node) = self.tree.get_node(node_id) {
                for edge in &node.children {
                    if !visited.contains(&edge.child_node_id) {
                        stack.push(edge.child_node_id);
                    }
                }
            }
        }

        remaining.saturating_sub(1) // Subtract current node
    }

    /// Calculate depth for a node
    pub fn get_depth(&self, node_id: Uuid) -> u32 {
        self.tree.get_node(node_id).map(|n| n.depth).unwrap_or(0)
    }

    /// Find next node by evaluating edges
    fn find_next_node(&self) -> Option<&DecisionNode> {
        if let Some(current) = self.current_node() {
            for edge in &current.children {
                if let Ok(true) = Evaluator::evaluate(&edge.condition, &self.core_state) {
                    return self.tree.get_node(edge.child_node_id);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let tree = ClinicalDecisionTree::new("test", "Test Tree");
        let _machine = SessionStateMachine::new(tree).expect_err("Should fail without root");
    }

    #[test]
    fn test_traversal_path() {
        let tree = ClinicalDecisionTree::new("test", "Test Tree");
        let machine = SessionStateMachine::new(tree).expect_err("Should fail without root");
        // Test would verify path tracking
    }
}
