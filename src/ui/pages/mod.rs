//! Settings pages for Freya
//!
//! Each page corresponds to a Category and displays the relevant settings.

mod appearance;
mod keyboard;
mod mouse;
mod touchpad;
mod trackpoint;
mod trackball;
mod tablet;
mod touch;
mod outputs;
mod animations;
mod cursor;
mod overview;
mod recent_windows;
mod behavior;
mod layout_extras;
mod workspaces;
mod window_rules;
mod layer_rules;
mod gestures;
mod keybindings;
mod startup;
mod environment;
mod switch_events;
mod miscellaneous;
mod debug;

pub use appearance::appearance_page;
pub use keyboard::keyboard_page;
pub use mouse::mouse_page;
pub use touchpad::touchpad_page;
pub use trackpoint::trackpoint_page;
pub use trackball::trackball_page;
pub use tablet::tablet_page;
pub use touch::touch_page;
pub use outputs::outputs_page;
pub use animations::animations_page;
pub use cursor::cursor_page;
pub use overview::overview_page;
pub use recent_windows::recent_windows_page;
pub use behavior::behavior_page;
pub use layout_extras::layout_extras_page;
pub use workspaces::workspaces_page;
pub use window_rules::window_rules_page;
pub use layer_rules::layer_rules_page;
pub use gestures::gestures_page;
pub use keybindings::keybindings_page;
pub use startup::startup_page;
pub use environment::environment_page;
pub use switch_events::switch_events_page;
pub use miscellaneous::miscellaneous_page;
pub use debug::debug_page;
