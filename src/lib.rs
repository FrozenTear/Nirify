//! Niri Settings - Native settings application for niri Wayland compositor
//!
//! Built with iced 0.14

pub mod app;
pub mod config;
pub mod constants;
pub mod messages;
pub mod save_manager;
pub mod search;
pub mod system_theme;
pub mod theme;
pub mod types;
pub mod version;
pub mod views;
pub mod ipc;

// Re-export config types
pub use config::{ConfigPaths, DirtyTracker, Settings, SettingsCategory};
