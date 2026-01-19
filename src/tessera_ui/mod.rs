//! Tessera UI module for niri-settings
//!
//! This module provides the Tessera-based user interface, replacing the Slint UI.
//! All backend logic (config loading, KDL parsing, etc.) remains unchanged.

pub mod app;
pub mod pages;
pub mod theme;

pub use app::run_app;
