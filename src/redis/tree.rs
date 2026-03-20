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

        for key in &keys {
            self.insert_key(&mut root, "", key, key);
        }

        let mut result: Vec<TreeNode> = root.into_values().collect();
        self.sort_tree(&mut result);
        self.calculate_total_keys(&mut result);

        result
    }

    fn calculate_total_keys(&self, nodes: &mut [TreeNode]) -> usize {
        let mut total = 0;
        for node in nodes.iter_mut() {
            if node.is_leaf {
                node.total_keys = 1;
                total += 1;
            } else {
                node.total_keys = self.calculate_total_keys(&mut node.children);
                total += node.total_keys;
            }
        }
        total
    }

    fn folder_node_id(path: &str) -> String {
        format!("folder:{path}")
    }

    fn leaf_node_id(path: &str) -> String {
        format!("leaf:{path}")
    }

    fn insert_key(
        &self,
        nodes: &mut HashMap<String, TreeNode>,
        parent_path: &str,
        key: &str,
        full_path: &str,
    ) {
        let parts: Vec<&str> = key.splitn(2, &self.delimiter).collect();

        if parts.len() == 1 {
            let path = full_path.to_string();
            let node_id = Self::leaf_node_id(&path);
            nodes.insert(
                node_id.clone(),
                TreeNode {
                    name: key.to_string(),
                    node_id,
                    path,
                    is_leaf: true,
                    children: Vec::new(),
                    key_info: None,
                    total_keys: 0,
                },
            );
        } else {
            let node_name = parts[0].to_string();
            let remaining = parts[1];
            let node_path = format!("{}{}{}", parent_path, parts[0], self.delimiter);
            let node_id = Self::folder_node_id(&node_path);

            let node = nodes.entry(node_id.clone()).or_insert_with(|| TreeNode {
                name: node_name,
                node_id: node_id.clone(),
                path: node_path.clone(),
                is_leaf: false,
                children: Vec::new(),
                key_info: None,
                total_keys: 0,
            });

            let mut children_map: HashMap<String, TreeNode> = node
                .children
                .drain(..)
                .map(|c| (c.node_id.clone(), c))
                .collect();

            if remaining.is_empty() {
                let path = full_path.to_string();
                let node_id = Self::leaf_node_id(&path);
                children_map.insert(
                    node_id.clone(),
                    TreeNode {
                        name: "[空]".to_string(),
                        node_id,
                        path,
                        is_leaf: true,
                        children: Vec::new(),
                        key_info: None,
                        total_keys: 0,
                    },
                );
            } else {
                self.insert_key(&mut children_map, &node_path, remaining, full_path);
            }

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

#[cfg(test)]
mod tests {
    use super::TreeBuilder;

    #[test]
    fn trailing_delimiter_key_creates_distinct_folder_and_empty_leaf_nodes() {
        let tree = TreeBuilder::new(":").build(vec!["cache:sys_file:".to_string()]);

        assert_eq!(tree.len(), 1);

        let cache = &tree[0];
        assert_eq!(cache.name, "cache");
        assert_eq!(cache.node_id, "folder:cache:");
        assert_eq!(cache.path, "cache:");
        assert_eq!(cache.children.len(), 1);

        let sys_file = &cache.children[0];
        assert_eq!(sys_file.name, "sys_file");
        assert_eq!(sys_file.node_id, "folder:cache:sys_file:");
        assert_eq!(sys_file.path, "cache:sys_file:");
        assert!(!sys_file.is_leaf);
        assert_eq!(sys_file.children.len(), 1);

        let empty_leaf = &sys_file.children[0];
        assert_eq!(empty_leaf.name, "[空]");
        assert_eq!(empty_leaf.node_id, "leaf:cache:sys_file:");
        assert_eq!(empty_leaf.path, "cache:sys_file:");
        assert!(empty_leaf.is_leaf);

        assert_ne!(sys_file.node_id, empty_leaf.node_id);
        assert_eq!(sys_file.path, empty_leaf.path);
    }
}
