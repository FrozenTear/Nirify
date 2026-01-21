//! UI callback modules
//!
//! Each module handles callbacks for a specific settings category.

// Shared utilities
#[macro_use]
pub mod setting_builders;

// Settings page callback modules
pub mod animations;
pub mod appearance;
pub mod behavior;
pub mod cursor;
pub mod debug;
pub mod environment;
pub mod gestures;
pub mod keyboard;
pub mod layer_rules;
pub mod layout_extras;
pub mod miscellaneous;
pub mod mouse;
pub mod overview;
pub mod recent_windows;
pub mod startup;
pub mod switch_events;
pub mod tablet;
pub mod touch;
pub mod touchpad;
pub mod trackball;
pub mod trackpoint;
pub mod window_rules;
pub mod workspaces;

// Utility and special-purpose modules
pub mod backups;
pub mod keybindings;
pub mod outputs;
pub mod rules_common;
