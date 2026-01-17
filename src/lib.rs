pub mod config;
pub mod constants;
pub mod diff;
pub mod hardware;
pub mod ipc;
pub mod types;
pub mod ui;

// Stub modules (to be reimplemented for Dioxus)
pub mod handlers;
pub mod wizard;

pub use config::models::Settings;
pub use config::paths::ConfigPaths;
