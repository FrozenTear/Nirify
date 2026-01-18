//! Settings pages - one module per category
//!
//! Each page provides a UI for configuring a specific category of niri settings.

// Input devices
mod keyboard;
mod mouse;
mod tablet;
mod touch;
mod touchpad;
mod trackball;
mod trackpoint;

// Appearance & Visuals
mod animations;
mod appearance;
mod cursor;
mod overview;
mod recent_windows;

// Behavior
mod behavior;
mod layout_extras;
mod workspaces;

// Rules
mod gestures;
mod layer_rules;
mod window_rules;

// System
mod debug;
mod environment;
mod keybindings;
mod miscellaneous;
mod startup;
mod switch_events;

// Outputs
mod outputs;

// Placeholder for unimplemented pages
mod placeholder;

// Re-export all pages
pub use appearance::appearance_page;
pub use placeholder::placeholder_page;

// Input devices
pub use keyboard::keyboard_page;
pub use mouse::mouse_page;
pub use tablet::tablet_page;
pub use touch::touch_page;
pub use touchpad::touchpad_page;
pub use trackball::trackball_page;
pub use trackpoint::trackpoint_page;

// Visuals
pub use animations::animations_page;
pub use cursor::cursor_page;
pub use overview::overview_page;
pub use recent_windows::recent_windows_page;

// Behavior
pub use behavior::behavior_page;
pub use layout_extras::layout_extras_page;
pub use workspaces::workspaces_page;

// Rules
pub use gestures::gestures_page;
pub use layer_rules::layer_rules_page;
pub use window_rules::window_rules_page;

// System
pub use debug::debug_page;
pub use environment::environment_page;
pub use keybindings::keybindings_page;
pub use miscellaneous::miscellaneous_page;
pub use startup::startup_page;
pub use switch_events::switch_events_page;

// Outputs
pub use outputs::outputs_page;
