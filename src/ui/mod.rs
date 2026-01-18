//! Freya-based UI for niri-settings
//!
//! This module contains all the UI components built with Freya.

pub mod app;
pub mod components;
pub mod nav;
pub mod pages;
pub mod state;
pub mod theme;

pub use app::app_view;
pub use state::AppState;
