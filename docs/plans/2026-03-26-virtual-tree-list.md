# 虚拟滚动键列表实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为键列表实现真正的虚拟滚动，支持百万级键的流畅渲染

**Architecture:** 使用 FlatTreeAdapter 将树结构扁平化为连续列表，VirtualTreeList 只渲染可见区域 + overscan 的节点，实现 O(visible) 复杂度而非 O(total)

**Tech Stack:** Rust, Dioxus, Freya

---

## Task 1: 创建 FlatTreeAdapter 核心结构

**Files:**
- Create: `src/ui/flat_tree_adapter.rs`
- Modify: `src/ui/mod.rs`

**Step 1: 添加 FlatNode 和 FlatTreeAdapter 结构**

```rust
use crate::redis::{KeyType, TreeNode};
use std::collections::HashSet;

#[derive(Clone, Debug)]
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
}
```

**Step 2: 在 mod.rs 中导出模块**

修改 `src/ui/mod.rs`，添加：
```rust
mod flat_tree_adapter;
pub use flat_tree_adapter::{FlatNode, FlatTreeAdapter};
```

**Step 3: 提交**

```bash
git add src/ui/flat_tree_adapter.rs src/ui/mod.rs
git commit -m "feat(ui): add FlatTreeAdapter core structure for virtual scrolling"
```

---

## Task 2: 实现树结构扁平化算法

**Files:**
- Modify: `src/ui/flat_tree_adapter.rs`

**Step 1: 添加树扁平化方法**

```rust
impl FlatTreeAdapter {
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

    pub fn get_visible_range(&self, scroll_top: f32, viewport_height: f32, overscan: usize) -> (usize, usize) {
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
```

**Step 2: 提交**

```bash
git add src/ui/flat_tree_adapter.rs
git commit -m "feat(ui): implement tree flattening algorithm for virtual scrolling"
```

---

## Task 3: 创建 VirtualTreeList 组件

**Files:**
- Create: `src/ui/virtual_tree_list.rs`
- Modify: `src/ui/mod.rs`

**Step 1: 创建 VirtualTreeList 组件**

```rust
use crate::redis::TreeNode;
use crate::ui::{FlatNode, FlatTreeAdapter};
use dioxus::prelude::*;
use std::collections::HashSet;

const DEFAULT_ITEM_HEIGHT: f32 = 28.0;
const DEFAULT_OVERSCAN: usize = 5;

#[component]
pub fn VirtualTreeList(
    nodes: Vec<TreeNode>,
    selected_key: String,
    expanded_paths: Signal<HashSet<String>>,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let mut scroll_top = use_signal(|| 0.0f32);
    let mut viewport_height = use_signal(|| 600.0f32);
    let mut adapter = use_signal(|| FlatTreeAdapter::new(DEFAULT_ITEM_HEIGHT));

    use_effect(move || {
        let expanded = expanded_paths.read().clone();
        adapter.write().set_expanded_paths(expanded);
        adapter.write().build_from_tree(&nodes);
    });

    let adapter_read = adapter.read();
    let total_height = adapter_read.total_height();
    let (start, end) = adapter_read.get_visible_range(scroll_top(), viewport_height(), DEFAULT_OVERSCAN);
    let item_height = adapter_read.item_height();

    rsx! {
        div {
            height: "100%",
            overflow_y: "auto",
            onscroll: move |e| {
                let data = e.data();
                scroll_top.set(data.scroll_top() as f32);
            },
            onresize: move |e| {
                let data = e.data();
                viewport_height.set(data.height() as f32);
            },

            div {
                height: "{total_height}px",
                position: "relative",

                for idx in start..end {
                    if let Some(node) = adapter_read.get_node_at_index(idx) {
                        {
                            let top = idx as f32 * item_height;
                            let is_selected = !node.is_folder && selected_key == node.path;
                            let indent = node.depth * 16;

                            rsx! {
                                VirtualTreeItem {
                                    key: "{node.id}",
                                    node: node.clone(),
                                    top: top,
                                    indent: indent,
                                    is_selected: is_selected,
                                    on_select: {
                                        let on_select = on_select.clone();
                                        let path = node.path.clone();
                                        move |_| {
                                            on_select.call(path.clone());
                                        }
                                    },
                                    on_toggle: {
                                        let on_toggle = on_toggle.clone();
                                        let path = node.path.clone();
                                        move |_| {
                                            on_toggle.call(path.clone());
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VirtualTreeItem(
    node: FlatNode,
    top: f32,
    indent: usize,
    is_selected: bool,
    on_select: EventHandler<()>,
    on_toggle: EventHandler<()>,
) -> Element {
    let bg_color = if is_selected { "#094771" } else { "transparent" };
    let text_color = if is_selected { "white" } else { "#cccccc" };
    let icon = if node.is_folder {
        "📁"
    } else {
        match node.key_type {
            Some(crate::redis::KeyType::String) => "📄",
            Some(crate::redis::KeyType::Hash) => "📑",
            Some(crate::redis::KeyType::List) => "📋",
            Some(crate::redis::KeyType::Set) => "📦",
            Some(crate::redis::KeyType::ZSet) => "📊",
            Some(crate::redis::KeyType::Stream) => "📜",
            _ => "📄",
        }
    };

    rsx! {
        div {
            position: "absolute",
            top: "{top}px",
            left: "0",
            right: "0",
            height: "28px",
            padding: "6px 8px",
            padding_left: "{indent + 8}px",
            display: "flex",
            align_items: "center",
            gap: "6px",
            background: bg_color,
            cursor: "pointer",
            overflow: "hidden",

            onclick: {
                let on_select = on_select.clone();
                let on_toggle = on_toggle.clone();
                let is_folder = node.is_folder;
                move |_| {
                    if is_folder {
                        on_toggle.call(());
                    } else {
                        on_select.call(());
                    }
                }
            },

            span {
                color: "#888",
                font_size: "12px",

                if node.is_folder {
                    if node.children_count > 0 {
                        "▶"
                    } else {
                        ""
                    }
                } else {
                    ""
                }
            }

            span {
                font_size: "14px",
                "{icon}"
            }

            span {
                color: text_color,
                font_size: "13px",
                overflow: "hidden",
                text_overflow: "ellipsis",
                white_space: "nowrap",

                "{node.name}"
            }

            if node.is_folder && node.children_count > 0 {
                span {
                    color: "#888",
                    font_size: "11px",
                    "({node.children_count})"
                }
            }
        }
    }
}
```

**Step 2: 在 mod.rs 中导出**

```rust
mod virtual_tree_list;
pub use virtual_tree_list::VirtualTreeList;
```

**Step 3: 提交**

```bash
git add src/ui/virtual_tree_list.rs src/ui/mod.rs
git commit -m "feat(ui): add VirtualTreeList component with virtual scrolling"
```

---

## Task 4: 集成到 KeyBrowser

**Files:**
- Modify: `src/ui/key_browser.rs`

**Step 1: 替换 LazyTreeNode 为 VirtualTreeList**

找到渲染树的部分（约 683-704 行），替换为：

```rust
// 在 imports 中添加
use crate::ui::VirtualTreeList;

// 在 KeyBrowser 组件中添加展开状态信号
let mut expanded_paths = use_signal(HashSet::<String>::new);

// 替换树渲染部分
if tree_nodes.read().is_empty() {
    div {
        padding: "40px 20px",
        text_align: "center",
        color: COLOR_TEXT_SECONDARY,
        font_size: "13px",

        if loading() {
            "正在加载..."
        } else {
            "没有找到 key"
        }
    }
} else {
    VirtualTreeList {
        nodes: tree_nodes.read().clone(),
        selected_key: selected_key(),
        expanded_paths: expanded_paths,
        on_select: {
            let on_key_select = on_key_select.clone();
            move |key: String| {
                on_key_select.call(key);
            }
        },
        on_toggle: {
            let tree_nodes = tree_nodes.clone();
            move |path: String| {
                let mut expanded = expanded_paths.write();
                if expanded.contains(&path) {
                    expanded.remove(&path);
                } else {
                    expanded.insert(path);
                }
            }
        },
    }
}
```

**Step 2: 移除不再需要的 LazyTreeNode 递归渲染**

删除或注释掉 `LazyTreeNode` 的使用。

**Step 3: 提交**

```bash
git add src/ui/key_browser.rs
git commit -m "feat(key-browser): integrate VirtualTreeList for virtual scrolling"
```

---

## Task 5: 添加性能监控和优化

**Files:**
- Modify: `src/ui/virtual_tree_list.rs`

**Step 1: 添加性能日志和缓存优化**

```rust
use tracing::debug;

// 在 VirtualTreeList 中添加缓存
#[component]
pub fn VirtualTreeList(
    nodes: Vec<TreeNode>,
    selected_key: String,
    expanded_paths: Signal<HashSet<String>>,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let mut scroll_top = use_signal(|| 0.0f32);
    let mut viewport_height = use_signal(|| 600.0f32);
    let mut adapter = use_signal(|| FlatTreeAdapter::new(DEFAULT_ITEM_HEIGHT));
    let mut last_nodes_len = use_signal(|| 0usize);

    use_effect(move || {
        let expanded = expanded_paths.read().clone();
        let nodes_len = nodes.len();
        
        // 只在节点数量变化或展开状态变化时重建
        if nodes_len != last_nodes_len() || adapter.read().expanded_paths() != &expanded {
            debug!("Rebuilding tree: {} nodes", nodes_len);
            adapter.write().set_expanded_paths(expanded);
            adapter.write().build_from_tree(&nodes);
            last_nodes_len.set(nodes_len);
        }
    });

    // ... 其余代码保持不变
}
```

**Step 2: 提交**

```bash
git add src/ui/virtual_tree_list.rs
git commit -m "perf(ui): add caching and performance logging for virtual tree list"
```

---

## Task 6: 测试和验证

**Files:**
- Create: `src/ui/flat_tree_adapter_test.rs` (for unit tests)

**Step 1: 创建单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::{KeyInfo, TreeNode};

    fn create_test_tree() -> Vec<TreeNode> {
        vec![
            TreeNode {
                node_id: "1".to_string(),
                path: "folder1".to_string(),
                name: "folder1".to_string(),
                is_leaf: false,
                children: vec![
                    TreeNode {
                        node_id: "1-1".to_string(),
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
                    },
                ],
                key_info: None,
            },
            TreeNode {
                node_id: "2".to_string(),
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
            },
        ]
    }

    #[test]
    fn test_build_from_tree_collapsed() {
        let mut adapter = FlatTreeAdapter::new(28.0);
        let tree = create_test_tree();
        adapter.build_from_tree(&tree);

        // 未展开时，应该只有顶层节点
        assert_eq!(adapter.len(), 2);
    }

    #[test]
    fn test_build_from_tree_expanded() {
        let mut adapter = FlatTreeAdapter::new(28.0);
        let mut expanded = HashSet::new();
        expanded.insert("folder1".to_string());
        adapter.set_expanded_paths(expanded);
        
        let tree = create_test_tree();
        adapter.build_from_tree(&tree);

        // 展开后，应该有3个节点
        assert_eq!(adapter.len(), 3);
        assert_eq!(adapter.get_node_at_index(0).unwrap().path, "folder1");
        assert_eq!(adapter.get_node_at_index(1).unwrap().path, "folder1:key1");
        assert_eq!(adapter.get_node_at_index(2).unwrap().path, "key2");
    }

    #[test]
    fn test_get_visible_range() {
        let mut adapter = FlatTreeAdapter::new(28.0);
        let tree = create_test_tree();
        adapter.build_from_tree(&tree);

        // 测试可见范围计算
        let (start, end) = adapter.get_visible_range(0.0, 100.0, 5);
        assert_eq!(start, 0);
        assert!(end >= 2);

        let (start, end) = adapter.get_visible_range(56.0, 28.0, 0);
        assert_eq!(start, 2);
        assert_eq!(end, 2);
    }
}
```

**Step 2: 运行测试**

```bash
cargo test flat_tree_adapter_test
```

**Step 3: 提交**

```bash
git add src/ui/flat_tree_adapter_test.rs
git commit -m "test(ui): add unit tests for FlatTreeAdapter"
```

---

## 最终验证

运行完整的构建和测试：

```bash
cargo build --release
cargo test
```

提交最终更改：

```bash
git add -A
git commit -m "feat(ui): complete virtual scrolling implementation for key list"
```