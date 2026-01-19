pub mod config;
pub mod constants;
pub mod diff;
pub mod handlers;
pub mod hardware;
pub mod ipc;
pub mod tessera_ui;
pub mod types;
pub mod ui;
pub mod wizard;

pub use config::models::Settings;
pub use config::paths::ConfigPaths;

// Tessera UI doesn't need code generation like Slint
// All UI code is written in pure Rust in the tessera_ui module
