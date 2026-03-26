use crate::redis::{KeyType, TreeNode};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct FlatNode {
    pub id: String,
    pub path: String,
    pub name: String,
    pub depth: usize,
    pub is_folder: bool,
    pub children_count: usize,
    pub key_type: Option<KeyType>,
}

pub struct FlatTreeAdapter {
    visible_nodes: Vec<FlatNode>,
    expanded_paths: HashSet<String>,
    item_height: f32,
}

impl FlatTreeAdapter {
    pub fn new(item_height: f32) -> Self {
        Self {
            visible_nodes: Vec::new(),
            expanded_paths: HashSet::new(),
            item_height,
        }
    }

    pub fn visible_nodes(&self) -> &[FlatNode] {
        &self.visible_nodes
    }

    pub fn total_height(&self) -> f32 {
        self.visible_nodes.len() as f32 * self.item_height
    }

    pub fn item_height(&self) -> f32 {
        self.item_height
    }

    pub fn is_expanded(&self, path: &str) -> bool {
        self.expanded_paths.contains(path)
    }

    pub fn toggle_expanded(&mut self, path: &str) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_string());
        }
    }

    pub fn expanded_paths(&self) -> &HashSet<String> {
        &self.expanded_paths
    }

    pub fn set_expanded_paths(&mut self, paths: HashSet<String>) {
        self.expanded_paths = paths;
    }

    pub fn build_from_tree(&mut self, nodes: &[TreeNode]) {
        self.visible_nodes.clear();
        for node in nodes {
            self.flatten_node(node, 0);
        }
    }

    fn flatten_node(&mut self, node: &TreeNode, depth: usize) {
        let flat_node = FlatNode {
            id: node.node_id.clone(),
            path: node.path.clone(),
            name: node.name.clone(),
            depth,
            is_folder: !node.is_leaf,
            children_count: node.children.len(),
            key_type: node.key_info.as_ref().map(|info| info.key_type.clone()),
        };

        self.visible_nodes.push(flat_node);

        if !node.is_leaf && self.expanded_paths.contains(&node.path) {
            for child in &node.children {
                self.flatten_node(child, depth + 1);
            }
        }
    }

    pub fn get_visible_range(
        &self,
        scroll_top: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> (usize, usize) {
        if self.visible_nodes.is_empty() {
            return (0, 0);
        }

        let first_visible = (scroll_top / self.item_height) as usize;
        let visible_count = (viewport_height / self.item_height).ceil() as usize;

        let start = first_visible.saturating_sub(overscan);
        let end = (first_visible + visible_count + overscan).min(self.visible_nodes.len());

        (start, end)
    }

    pub fn get_node_at_index(&self, index: usize) -> Option<&FlatNode> {
        self.visible_nodes.get(index)
    }

    pub fn len(&self) -> usize {
        self.visible_nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.visible_nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::KeyInfo;

    fn create_test_tree() -> Vec<TreeNode> {
        vec![
            TreeNode {
                node_id: "folder:folder1:".to_string(),
                path: "folder1:".to_string(),
                name: "folder1".to_string(),
                is_leaf: false,
                children: vec![TreeNode {
                    node_id: "leaf:folder1:key1".to_string(),
                    path: "folder1:key1".to_string(),
                    name: "key1".to_string(),
                    is_leaf: true,
                    children: vec![],
                    key_info: Some(KeyInfo {
                        name: "folder1:key1".to_string(),
                        key_type: KeyType::String,
                        ttl: None,
                        size: None,
                    }),
                    total_keys: 1,
                }],
                key_info: None,
                total_keys: 1,
            },
            TreeNode {
                node_id: "leaf:key2".to_string(),
                path: "key2".to_string(),
                name: "key2".to_string(),
                is_leaf: true,
                children: vec![],
                key_info: Some(KeyInfo {
                    name: "key2".to_string(),
                    key_type: KeyType::Hash,
                    ttl: None,
                    size: None,
                }),
                total_keys: 1,
            },
        ]
    }

    #[test]
    fn test_build_from_tree_collapsed() {
        let mut adapter = FlatTreeAdapter::new(28.0);
        let tree = create_test_tree();
        adapter.build_from_tree(&tree);

        assert_eq!(adapter.len(), 2);
    }

    #[test]
    fn test_build_from_tree_expanded() {
        let mut adapter = FlatTreeAdapter::new(28.0);
        let mut expanded = HashSet::new();
        expanded.insert("folder1:".to_string());
        adapter.set_expanded_paths(expanded);

        let tree = create_test_tree();
        adapter.build_from_tree(&tree);

        assert_eq!(adapter.len(), 3);
        assert_eq!(adapter.get_node_at_index(0).unwrap().path, "folder1:");
        assert_eq!(adapter.get_node_at_index(1).unwrap().path, "folder1:key1");
        assert_eq!(adapter.get_node_at_index(2).unwrap().path, "key2");
    }

    #[test]
    fn test_get_visible_range() {
        let mut adapter = FlatTreeAdapter::new(28.0);
        let tree = create_test_tree();
        adapter.build_from_tree(&tree);

        let (start, end) = adapter.get_visible_range(0.0, 100.0, 5);
        assert_eq!(start, 0);
        assert!(end >= 2);

        let (start, end) = adapter.get_visible_range(56.0, 28.0, 0);
        assert_eq!(start, 2);
        assert_eq!(end, 2);
    }

    #[test]
    fn test_toggle_expanded() {
        let mut adapter = FlatTreeAdapter::new(28.0);

        assert!(!adapter.is_expanded("folder1:"));

        adapter.toggle_expanded("folder1:");
        assert!(adapter.is_expanded("folder1:"));

        adapter.toggle_expanded("folder1:");
        assert!(!adapter.is_expanded("folder1:"));
    }
}
