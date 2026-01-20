//! Floem-based UI for niri-settings
//!
//! This module contains all the UI components built with Floem.

pub mod app;
pub mod components;
pub mod nav;
pub mod pages;
pub mod state;
pub mod theme;
pub mod wizard;

pub use app::app_view;
pub use state::AppState;
pub use wizard::{wizard_view, WizardState};
