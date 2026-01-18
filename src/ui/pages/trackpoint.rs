//! Trackpoint (pointing stick) settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the trackpoint settings page
pub fn trackpoint_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let trackpoint = settings.trackpoint;

    let state_natural = state.clone();
    let state_left = state.clone();
    let state_middle = state.clone();
    let state_scroll_lock = state.clone();
    let state_accel = state.clone();
    let state_off = state.clone();

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
                    "Natural scrolling",
                    "Invert scroll direction",
                    trackpoint.natural_scroll,
                    move |val| {
                        state_natural.update_settings(|s| s.trackpoint.natural_scroll = val);
                        state_natural.mark_dirty_and_save(SettingsCategory::Trackpoint);
                    },
                ))
                .child(toggle_row(
                    "Left-handed mode",
                    "Swap left and right buttons",
                    trackpoint.left_handed,
                    move |val| {
                        state_left.update_settings(|s| s.trackpoint.left_handed = val);
                        state_left.mark_dirty_and_save(SettingsCategory::Trackpoint);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Emulate middle click",
                    trackpoint.middle_emulation,
                    move |val| {
                        state_middle.update_settings(|s| s.trackpoint.middle_emulation = val);
                        state_middle.mark_dirty_and_save(SettingsCategory::Trackpoint);
                    },
                )),
        ))
        // Scrolling section
        .child(section(
            "Scrolling",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Scroll button lock",
                    "Don't need to hold scroll button",
                    trackpoint.scroll_button_lock,
                    move |val| {
                        state_scroll_lock.update_settings(|s| s.trackpoint.scroll_button_lock = val);
                        state_scroll_lock.mark_dirty_and_save(SettingsCategory::Trackpoint);
                    },
                )),
        ))
        // Speed section
        .child(section(
            "Speed",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Acceleration",
                    "Pointer acceleration speed",
                    trackpoint.accel_speed,
                    -1.0,
                    1.0,
                    "",
                    move |val| {
                        state_accel.update_settings(|s| s.trackpoint.accel_speed = val);
                        state_accel.mark_dirty_and_save(SettingsCategory::Trackpoint);
                    },
                )),
        ))
        // Device section
        .child(section(
            "Device",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable trackpoint",
                    "Turn off trackpoint input",
                    trackpoint.off,
                    move |val| {
                        state_off.update_settings(|s| s.trackpoint.off = val);
                        state_off.mark_dirty_and_save(SettingsCategory::Trackpoint);
                    },
                )),
        ))
}
