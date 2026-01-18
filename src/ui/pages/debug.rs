//! Debug settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, toggle_row, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the debug settings page
pub fn debug_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let debug = settings.debug;

    let state_preview = state.clone();
    let state_cursor = state.clone();
    let state_scanout = state.clone();
    let state_throttle = state.clone();
    let state_trans = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Warning section
        .child(section(
            "Warning",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row(
                    "Debug settings",
                    "These may cause visual glitches or performance issues",
                    "Use with caution",
                )),
        ))
        // Rendering section
        .child(section(
            "Rendering",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Preview render",
                    "Render monitors like screencast",
                    debug.preview_render,
                    move |val| {
                        state_preview.update_settings(|s| s.debug.preview_render = val);
                        state_preview.mark_dirty_and_save(SettingsCategory::Debug);
                    },
                ))
                .child(toggle_row(
                    "Disable cursor plane",
                    "Force cursor through compositor",
                    debug.disable_cursor_plane,
                    move |val| {
                        state_cursor.update_settings(|s| s.debug.disable_cursor_plane = val);
                        state_cursor.mark_dirty_and_save(SettingsCategory::Debug);
                    },
                ))
                .child(toggle_row(
                    "Disable direct scanout",
                    "Always composite fullscreen windows",
                    debug.disable_direct_scanout,
                    move |val| {
                        state_scanout.update_settings(|s| s.debug.disable_direct_scanout = val);
                        state_scanout.mark_dirty_and_save(SettingsCategory::Debug);
                    },
                )),
        ))
        // Window Management section
        .child(section(
            "Window Management",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable resize throttling",
                    "Send resize events immediately",
                    debug.disable_resize_throttling,
                    move |val| {
                        state_throttle
                            .update_settings(|s| s.debug.disable_resize_throttling = val);
                        state_throttle.mark_dirty_and_save(SettingsCategory::Debug);
                    },
                ))
                .child(toggle_row(
                    "Disable transactions",
                    "Don't wait for synchronized resizing",
                    debug.disable_transactions,
                    move |val| {
                        state_trans.update_settings(|s| s.debug.disable_transactions = val);
                        state_trans.mark_dirty_and_save(SettingsCategory::Debug);
                    },
                )),
        ))
        // Info section
        .child(section(
            "Info",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row(
                    "Documentation",
                    "Additional debug settings available in config file",
                    "See niri docs",
                )),
        ))
}
