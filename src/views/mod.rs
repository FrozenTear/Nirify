//! View layer - UI construction functions
//!
//! Each module contains view() functions that create iced Elements from state.
//! Views are pure functions: same state → same UI.

pub mod dialogs;
pub mod navigation;
pub mod search_results;
pub mod status_bar;

pub mod animations;
pub mod appearance;
pub mod behavior;
pub mod cursor;
pub mod debug;
pub mod environment;
pub mod gestures;
pub mod keyboard;
pub mod keybindings;
pub mod layer_rules;
pub mod layout_extras;
pub mod miscellaneous;
pub mod mouse;
pub mod outputs;
pub mod recent_windows;
pub mod sidebar;
pub mod startup;
pub mod switch_events;
pub mod tablet;
pub mod tools;
pub mod touch;
pub mod touchpad;
pub mod trackball;
pub mod trackpoint;
pub mod widget_demo;
pub mod widgets;
pub mod window_rules;
pub mod workspaces;
