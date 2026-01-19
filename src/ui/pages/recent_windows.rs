//! Recent windows (Alt-Tab) settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the recent windows settings page
pub fn recent_windows_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let recent = &settings.recent_windows;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::RecentWindows, |s| {
                            s.recent_windows.off = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
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
                        state2.update_and_save(SettingsCategory::RecentWindows, |s| {
                            s.recent_windows.debounce_ms = val as i32
                        });
                        refresh2.with_mut(|mut v| *v += 1);
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
                        state3.update_and_save(SettingsCategory::RecentWindows, |s| {
                            s.recent_windows.open_delay_ms = val as i32
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
