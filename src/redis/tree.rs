use super::TreeNode;
use std::collections::HashMap;

pub struct TreeBuilder {
    delimiter: String,
}

impl TreeBuilder {
    pub fn new(delimiter: impl Into<String>) -> Self {
        Self {
            delimiter: delimiter.into(),
        }
    }

    pub fn build(&self, keys: Vec<String>) -> Vec<TreeNode> {
        let mut root: HashMap<String, TreeNode> = HashMap::new();

        for key in keys {
            self.insert_key(&mut root, &key, &key);
        }

        let mut result: Vec<TreeNode> = root.into_values().collect();
        self.sort_tree(&mut result);

        result
    }

    fn insert_key(&self, nodes: &mut HashMap<String, TreeNode>, key: &str, full_path: &str) {
        let parts: Vec<&str> = key.splitn(2, &self.delimiter).collect();

        if parts.len() == 1 {
            nodes.insert(
                key.to_string(),
                TreeNode {
                    name: key.to_string(),
                    full_path: full_path.to_string(),
                    is_leaf: true,
                    children: Vec::new(),
                    key_info: None,
                },
            );
        } else {
            let node_name = parts[0].to_string();
            let remaining = parts[1];

            let node = nodes.entry(node_name.clone()).or_insert_with(|| TreeNode {
                name: node_name,
                full_path: format!("{}{}", parts[0], self.delimiter),
                is_leaf: false,
                children: Vec::new(),
                key_info: None,
            });

            let mut children_map: HashMap<String, TreeNode> = node
                .children
                .drain(..)
                .map(|c| (c.name.clone(), c))
                .collect();

            self.insert_key(&mut children_map, remaining, full_path);

            node.children = children_map.into_values().collect();
            self.sort_tree(&mut node.children);
        }
    }

    fn sort_tree(&self, nodes: &mut Vec<TreeNode>) {
        nodes.sort_by(|a, b| match (a.is_leaf, b.is_leaf) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new(":")
    }
}
