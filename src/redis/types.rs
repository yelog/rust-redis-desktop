use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyType {
    String,
    Hash,
    List,
    Set,
    ZSet,
    Stream,
    None,
}

impl From<String> for KeyType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "string" => KeyType::String,
            "hash" => KeyType::Hash,
            "list" => KeyType::List,
            "set" => KeyType::Set,
            "zset" => KeyType::ZSet,
            "stream" => KeyType::Stream,
            "none" => KeyType::None,
            _ => KeyType::None,
        }
    }
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::String => write!(f, "String"),
            KeyType::Hash => write!(f, "Hash"),
            KeyType::List => write!(f, "List"),
            KeyType::Set => write!(f, "Set"),
            KeyType::ZSet => write!(f, "ZSet"),
            KeyType::Stream => write!(f, "Stream"),
            KeyType::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyInfo {
    pub name: String,
    pub key_type: KeyType,
    pub ttl: Option<i64>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub node_id: String,
    pub path: String,
    pub is_leaf: bool,
    pub children: Vec<TreeNode>,
    pub key_info: Option<KeyInfo>,
    pub total_keys: usize,
}
