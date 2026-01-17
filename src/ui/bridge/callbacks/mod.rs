//! UI callback modules
//!
//! Each module handles callbacks for a specific settings category.
//! Dynamic modules use model-driven UI with generic callbacks.

// Dynamic callback modules (model-driven)
pub mod animations_dynamic;
pub mod appearance_dynamic;
pub mod behavior_dynamic;
pub mod cursor_dynamic;
pub mod debug_dynamic;
pub mod environment_dynamic;
pub mod gestures_dynamic;
pub mod keyboard_dynamic;
pub mod layout_extras_dynamic;
pub mod layer_rules_dynamic;
pub mod miscellaneous_dynamic;
pub mod mouse_dynamic;
pub mod overview_dynamic;
pub mod recent_windows_dynamic;
pub mod startup_dynamic;
pub mod switch_events_dynamic;
pub mod tablet_dynamic;
pub mod touch_dynamic;
pub mod touchpad_dynamic;
pub mod trackball_dynamic;
pub mod trackpoint_dynamic;
pub mod window_rules_dynamic;
pub mod workspaces_dynamic;

// Static callback modules (complex UIs that don't fit dynamic pattern)
pub mod backups;
pub mod keybindings;
pub mod outputs;
pub mod rules_common;
