use dioxus::prelude::*;

pub const MAX_CACHE_SIZE: usize = 1000;
pub const DEFAULT_PAGE_SIZE: usize = 100;

#[derive(Clone)]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub cursor: u64,
    pub has_more: bool,
    pub loading: bool,
    pub search_pattern: String,
    pub is_searching: bool,
}

impl<T> Default for PaginatedData<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            cursor: 0,
            has_more: false,
            loading: false,
            search_pattern: String::new(),
            is_searching: false,
        }
    }
}

impl<T: Clone> PaginatedData<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, new_items: Vec<T>, new_cursor: u64) {
        if self.items.len() + new_items.len() > MAX_CACHE_SIZE {
            self.items.clear();
        }

        self.items.extend(new_items);
        self.cursor = new_cursor;
        self.has_more = new_cursor != 0;
        self.loading = false;
    }

    pub fn set_initial(&mut self, items: Vec<T>, total: usize, cursor: u64) {
        self.items = items;
        self.total = total;
        self.cursor = cursor;
        self.has_more = cursor != 0;
        self.loading = false;
    }

    pub fn update_item<F>(&mut self, predicate: F, new_item: T) -> bool
    where
        F: Fn(&T) -> bool,
    {
        if let Some(item) = self.items.iter_mut().find(|x| predicate(x)) {
            *item = new_item;
            true
        } else {
            false
        }
    }

    pub fn remove_item<F>(&mut self, predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        let original_len = self.items.len();
        self.items.retain(|item| !predicate(item));
        if self.items.len() != original_len {
            self.total = self.total.saturating_sub(1);
            true
        } else {
            false
        }
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
        self.total += 1;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.total = 0;
        self.cursor = 0;
        self.has_more = false;
        self.loading = false;
        self.search_pattern.clear();
        self.is_searching = false;
    }

    #[allow(dead_code)]
    pub fn loaded_count(&self) -> usize {
        self.items.len()
    }
}

pub fn format_search_pattern(input: &str) -> String {
    if input.is_empty() {
        "*".to_string()
    } else {
        format!("*{}*", input)
    }
}
