//! Mouse settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the mouse settings page
pub fn mouse_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let mouse = settings.mouse;

    let state_natural = state.clone();
    let state_left = state.clone();
    let state_middle = state.clone();
    let state_accel = state.clone();
    let state_scroll = state.clone();
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
                    mouse.natural_scroll,
                    move |val| {
                        log::info!("Natural scroll toggled to: {}", val);
                        state_natural.update_settings(|s| s.mouse.natural_scroll = val);
                        state_natural.mark_dirty_and_save(SettingsCategory::Mouse);
                    },
                ))
                .child(toggle_row(
                    "Left-handed mode",
                    "Swap left and right buttons",
                    mouse.left_handed,
                    move |val| {
                        state_left.update_settings(|s| s.mouse.left_handed = val);
                        state_left.mark_dirty_and_save(SettingsCategory::Mouse);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Emulate middle click with left+right",
                    mouse.middle_emulation,
                    move |val| {
                        state_middle.update_settings(|s| s.mouse.middle_emulation = val);
                        state_middle.mark_dirty_and_save(SettingsCategory::Mouse);
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
                    mouse.accel_speed,
                    -1.0,
                    1.0,
                    "",
                    move |val| {
                        state_accel.update_settings(|s| s.mouse.accel_speed = val);
                        state_accel.mark_dirty_and_save(SettingsCategory::Mouse);
                    },
                ))
                .child(slider_row(
                    "Scroll speed",
                    "Scroll sensitivity multiplier",
                    mouse.scroll_factor,
                    0.1,
                    3.0,
                    "x",
                    move |val| {
                        state_scroll.update_settings(|s| s.mouse.scroll_factor = val);
                        state_scroll.mark_dirty_and_save(SettingsCategory::Mouse);
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
                    "Disable mouse",
                    "Turn off mouse input",
                    mouse.off,
                    move |val| {
                        state_off.update_settings(|s| s.mouse.off = val);
                        state_off.mark_dirty_and_save(SettingsCategory::Mouse);
                    },
                )),
        ))
}
