pub mod config;
pub mod constants;
pub mod diff;
pub mod handlers;
pub mod hardware;
pub mod ipc;
pub mod types;
pub mod ui;
pub mod wizard;

pub use config::models::Settings;
pub use config::paths::ConfigPaths;

// Include Slint-generated code in the library so it's accessible everywhere
slint::include_modules!();
