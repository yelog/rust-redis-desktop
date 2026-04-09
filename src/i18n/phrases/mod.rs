use std::collections::HashMap;

pub mod app;
pub mod common;
pub mod dialogs;
pub mod panels;
pub mod value_viewer;

pub fn insert_all_zh_cn(m: &mut HashMap<String, String>) {
    common::extend_zh_cn(m);
    app::extend_zh_cn(m);
    dialogs::extend_zh_cn(m);
    panels::extend_zh_cn(m);
    value_viewer::extend_zh_cn(m);
}
