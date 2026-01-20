//! UI module - all UI components

pub mod sidebar;
pub mod keyboard_page;
pub mod mouse_page;
pub mod touchpad_page;

pub use sidebar::build_sidebar;
pub use keyboard_page::build_keyboard_page;
pub use mouse_page::build_mouse_page;
pub use touchpad_page::build_touchpad_page;
