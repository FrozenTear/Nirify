//! Debug settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, toggle_row, value_row};
use crate::ui::theme::SPACING_LG;

/// Create the debug settings page
pub fn debug_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let debug = &settings.debug;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();
    let state5 = state.clone();
    let mut refresh5 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Debug, |s| {
                            s.debug.preview_render = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable cursor plane",
                    "Force cursor through compositor",
                    debug.disable_cursor_plane,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Debug, |s| {
                            s.debug.disable_cursor_plane = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable direct scanout",
                    "Always composite fullscreen windows",
                    debug.disable_direct_scanout,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Debug, |s| {
                            s.debug.disable_direct_scanout = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
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
                        state4.update_and_save(SettingsCategory::Debug, |s| {
                            s.debug.disable_resize_throttling = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable transactions",
                    "Don't wait for synchronized resizing",
                    debug.disable_transactions,
                    move |val| {
                        state5.update_and_save(SettingsCategory::Debug, |s| {
                            s.debug.disable_transactions = val
                        });
                        refresh5.with_mut(|mut v| *v += 1);
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
