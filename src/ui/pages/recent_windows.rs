//! Recent windows (Alt-Tab) settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the recent windows settings page
pub fn recent_windows_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let recent = settings.recent_windows;

    let state_off = state.clone();
    let state_debounce = state.clone();
    let state_delay = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // General section
        .child(section(
            "General",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable recent windows",
                    "Turn off the Alt-Tab switcher",
                    recent.off,
                    move |val| {
                        state_off.update_settings(|s| s.recent_windows.off = val);
                        state_off.mark_dirty_and_save(SettingsCategory::RecentWindows);
                    },
                )),
        ))
        // Timing section
        .child(section(
            "Timing",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Debounce delay",
                    "Delay before adding to recent list",
                    recent.debounce_ms as f64,
                    0.0,
                    1000.0,
                    "ms",
                    move |val| {
                        state_debounce.update_settings(|s| s.recent_windows.debounce_ms = val as i32);
                        state_debounce.mark_dirty_and_save(SettingsCategory::RecentWindows);
                    },
                ))
                .child(slider_row(
                    "Open delay",
                    "Delay before UI appears",
                    recent.open_delay_ms as f64,
                    0.0,
                    500.0,
                    "ms",
                    move |val| {
                        state_delay.update_settings(|s| s.recent_windows.open_delay_ms = val as i32);
                        state_delay.mark_dirty_and_save(SettingsCategory::RecentWindows);
                    },
                )),
        ))
}
