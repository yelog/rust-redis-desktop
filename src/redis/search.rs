use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SearchMode {
    #[default]
    Pattern,
    Regex,
    Prefix,
    Suffix,
    Exact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KeyFilter {
    pub mode: SearchMode,
    pub pattern: String,
    pub key_types: Vec<KeyTypeFilter>,
    pub ttl_min: Option<i64>,
    pub ttl_max: Option<i64>,
    pub exclude_expired: bool,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyTypeFilter {
    String,
    Hash,
    List,
    Set,
    ZSet,
    Stream,
}

impl KeyTypeFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyTypeFilter::String => "string",
            KeyTypeFilter::Hash => "hash",
            KeyTypeFilter::List => "list",
            KeyTypeFilter::Set => "set",
            KeyTypeFilter::ZSet => "zset",
            KeyTypeFilter::Stream => "stream",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "string" => Some(KeyTypeFilter::String),
            "hash" => Some(KeyTypeFilter::Hash),
            "list" => Some(KeyTypeFilter::List),
            "set" => Some(KeyTypeFilter::Set),
            "zset" => Some(KeyTypeFilter::ZSet),
            "stream" => Some(KeyTypeFilter::Stream),
            _ => None,
        }
    }

    pub fn all() -> Vec<KeyTypeFilter> {
        vec![
            KeyTypeFilter::String,
            KeyTypeFilter::Hash,
            KeyTypeFilter::List,
            KeyTypeFilter::Set,
            KeyTypeFilter::ZSet,
            KeyTypeFilter::Stream,
        ]
    }
}

impl Default for KeyTypeFilter {
    fn default() -> Self {
        KeyTypeFilter::String
    }
}

impl KeyFilter {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            mode: SearchMode::Pattern,
            pattern: pattern.into(),
            key_types: Vec::new(),
            ttl_min: None,
            ttl_max: None,
            exclude_expired: false,
            size_min: None,
            size_max: None,
        }
    }

    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_key_types(mut self, types: Vec<KeyTypeFilter>) -> Self {
        self.key_types = types;
        self
    }

    pub fn with_ttl_range(mut self, min: Option<i64>, max: Option<i64>) -> Self {
        self.ttl_min = min;
        self.ttl_max = max;
        self
    }

    pub fn with_size_range(mut self, min: Option<u64>, max: Option<u64>) -> Self {
        self.size_min = min;
        self.size_max = max;
        self
    }

    pub fn exclude_expired(mut self, exclude: bool) -> Self {
        self.exclude_expired = exclude;
        self
    }

    pub fn to_redis_pattern(&self) -> String {
        if self.pattern.is_empty() {
            return "*".to_string();
        }

        match self.mode {
            SearchMode::Pattern => {
                if self.pattern.contains('*') || self.pattern.contains('?') {
                    self.pattern.clone()
                } else {
                    format!("*{}*", self.pattern)
                }
            }
            SearchMode::Regex | SearchMode::Exact => "*".to_string(),
            SearchMode::Prefix => format!("{}*", self.pattern),
            SearchMode::Suffix => format!("*{}", self.pattern),
        }
    }

    pub fn matches_key(&self, key: &str) -> bool {
        if self.pattern.is_empty() {
            return true;
        }

        match self.mode {
            SearchMode::Pattern => {
                let pattern = self.to_redis_pattern();
                matches_glob_pattern(key, &pattern)
            }
            SearchMode::Regex => {
                if let Ok(re) = Regex::new(&self.pattern) {
                    re.is_match(key)
                } else {
                    key.contains(&self.pattern)
                }
            }
            SearchMode::Prefix => key.starts_with(&self.pattern),
            SearchMode::Suffix => key.ends_with(&self.pattern),
            SearchMode::Exact => key == self.pattern,
        }
    }

    pub fn matches_ttl(&self, ttl: Option<i64>) -> bool {
        if self.ttl_min.is_none() && self.ttl_max.is_none() {
            return true;
        }

        let ttl_value = match ttl {
            Some(t) => t,
            None => return self.ttl_min.is_none() && self.ttl_max.is_none(),
        };

        if ttl_value < 0 {
            return false;
        }

        if let Some(min) = self.ttl_min {
            if ttl_value < min {
                return false;
            }
        }

        if let Some(max) = self.ttl_max {
            if ttl_value > max {
                return false;
            }
        }

        true
    }

    pub fn matches_size(&self, size: Option<u64>) -> bool {
        if self.size_min.is_none() && self.size_max.is_none() {
            return true;
        }

        let size_value = match size {
            Some(s) => s,
            None => return false,
        };

        if let Some(min) = self.size_min {
            if size_value < min {
                return false;
            }
        }

        if let Some(max) = self.size_max {
            if size_value > max {
                return false;
            }
        }

        true
    }

    pub fn has_type_filter(&self) -> bool {
        !self.key_types.is_empty()
    }

    pub fn has_ttl_filter(&self) -> bool {
        self.ttl_min.is_some() || self.ttl_max.is_some()
    }

    pub fn has_size_filter(&self) -> bool {
        self.size_min.is_some() || self.size_max.is_some()
    }

    pub fn is_simple_pattern(&self) -> bool {
        !self.has_type_filter()
            && !self.has_ttl_filter()
            && !self.has_size_filter()
            && self.mode == SearchMode::Pattern
    }
}

fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
    fn match_helper(text: &[char], pattern: &[char]) -> bool {
        let mut text_idx = 0;
        let mut pattern_idx = 0;
        let mut star_idx: Option<usize> = None;
        let mut match_idx = 0;

        while text_idx < text.len() {
            if pattern_idx < pattern.len() && pattern[pattern_idx] == '?' {
                text_idx += 1;
                pattern_idx += 1;
            } else if pattern_idx < pattern.len() && pattern[pattern_idx] == '*' {
                star_idx = Some(pattern_idx);
                match_idx = text_idx;
                pattern_idx += 1;
            } else if pattern_idx < pattern.len() && text[text_idx] == pattern[pattern_idx] {
                text_idx += 1;
                pattern_idx += 1;
            } else if let Some(si) = star_idx {
                pattern_idx = si + 1;
                match_idx += 1;
                text_idx = match_idx;
            } else {
                return false;
            }
        }

        while pattern_idx < pattern.len() && pattern[pattern_idx] == '*' {
            pattern_idx += 1;
        }

        pattern_idx == pattern.len()
    }

    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    match_helper(&text_chars, &pattern_chars)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistory {
    pub queries: Vec<String>,
    pub max_size: usize,
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self {
            queries: Vec::new(),
            max_size: 50,
        }
    }
}

impl SearchHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }

        self.queries.retain(|q| q != &query);
        self.queries.insert(0, query);

        if self.queries.len() > self.max_size {
            self.queries.truncate(self.max_size);
        }
    }

    pub fn clear(&mut self) {
        self.queries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_filter_pattern() {
        let filter = KeyFilter::new("user");
        assert!(filter.matches_key("user:1"));
        assert!(filter.matches_key("my_user_key"));
        assert!(!filter.matches_key("admin"));
    }

    #[test]
    fn test_key_filter_prefix() {
        let filter = KeyFilter::new("user:").with_mode(SearchMode::Prefix);
        assert!(filter.matches_key("user:1"));
        assert!(filter.matches_key("user:admin"));
        assert!(!filter.matches_key("my_user"));
    }

    #[test]
    fn test_key_filter_suffix() {
        let filter = KeyFilter::new(":cache").with_mode(SearchMode::Suffix);
        assert!(filter.matches_key("user:cache"));
        assert!(filter.matches_key("data:cache"));
        assert!(!filter.matches_key("cache:user"));
    }

    #[test]
    fn test_key_filter_exact() {
        let filter = KeyFilter::new("user:1").with_mode(SearchMode::Exact);
        assert!(filter.matches_key("user:1"));
        assert!(!filter.matches_key("user:2"));
        assert!(!filter.matches_key("my_user:1"));
    }

    #[test]
    fn test_key_filter_regex() {
        let filter = KeyFilter::new(r"user:\d+").with_mode(SearchMode::Regex);
        assert!(filter.matches_key("user:1"));
        assert!(filter.matches_key("user:123"));
        assert!(!filter.matches_key("user:abc"));
    }

    #[test]
    fn test_key_filter_ttl() {
        let filter = KeyFilter::new("").with_ttl_range(Some(100), Some(500));
        assert!(filter.matches_ttl(Some(100)));
        assert!(filter.matches_ttl(Some(300)));
        assert!(filter.matches_ttl(Some(500)));
        assert!(!filter.matches_ttl(Some(50)));
        assert!(!filter.matches_ttl(Some(600)));
    }

    #[test]
    fn test_to_redis_pattern() {
        let filter = KeyFilter::new("user");
        assert_eq!(filter.to_redis_pattern(), "*user*");

        let filter = KeyFilter::new("user*").with_mode(SearchMode::Pattern);
        assert_eq!(filter.to_redis_pattern(), "user*");

        let filter = KeyFilter::new("user:").with_mode(SearchMode::Prefix);
        assert_eq!(filter.to_redis_pattern(), "user:*");

        let filter = KeyFilter::new(":cache").with_mode(SearchMode::Suffix);
        assert_eq!(filter.to_redis_pattern(), "*:cache");
    }

    #[test]
    fn test_matches_glob_pattern() {
        assert!(matches_glob_pattern("user:1", "user:*"));
        assert!(matches_glob_pattern("user:123", "user:*"));
        assert!(!matches_glob_pattern("admin:1", "user:*"));
        assert!(matches_glob_pattern("key", "k?y"));
        assert!(!matches_glob_pattern("keyy", "k?y"));
    }
}
