mod connection;
mod config;
mod ui;

use ui::App;

fn main() {
    tracing_subscriber::fmt::init();
    
    dioxus::launch(App);
}