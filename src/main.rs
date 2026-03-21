mod config;
mod connection;
mod redis;
mod serialization;
mod theme;
mod ui;

use ui::App;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .pretty()
        .init();

    tracing::info!("Starting Redis Desktop Manager");

    dioxus::launch(App);
}
