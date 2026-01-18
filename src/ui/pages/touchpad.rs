//! Touchpad settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the touchpad settings page
pub fn touchpad_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let touchpad = settings.touchpad;

    let state_tap = state.clone();
    let state_drag = state.clone();
    let state_drag_lock = state.clone();
    let state_middle = state.clone();
    let state_natural = state.clone();
    let state_scroll = state.clone();
    let state_accel = state.clone();
    let state_left = state.clone();
    let state_dwt = state.clone();
    let state_external = state.clone();
    let state_off = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Tap & Click section
        .child(section(
            "Tap & Click",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Tap to click",
                    "Tap the touchpad to click",
                    touchpad.tap,
                    move |val| {
                        state_tap.update_settings(|s| s.touchpad.tap = val);
                        state_tap.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(toggle_row(
                    "Drag gestures",
                    "Tap and drag to select",
                    touchpad.drag,
                    move |val| {
                        state_drag.update_settings(|s| s.touchpad.drag = val);
                        state_drag.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(toggle_row(
                    "Drag lock",
                    "Continue dragging without holding",
                    touchpad.drag_lock,
                    move |val| {
                        state_drag_lock.update_settings(|s| s.touchpad.drag_lock = val);
                        state_drag_lock.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Two-finger tap for middle click",
                    touchpad.middle_emulation,
                    move |val| {
                        state_middle.update_settings(|s| s.touchpad.middle_emulation = val);
                        state_middle.mark_dirty_and_save(SettingsCategory::Touchpad);
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
                    "Natural scrolling",
                    "Content follows finger direction",
                    touchpad.natural_scroll,
                    move |val| {
                        state_natural.update_settings(|s| s.touchpad.natural_scroll = val);
                        state_natural.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(slider_row(
                    "Scroll speed",
                    "Scroll sensitivity multiplier",
                    touchpad.scroll_factor,
                    0.1,
                    3.0,
                    "x",
                    move |val| {
                        state_scroll.update_settings(|s| s.touchpad.scroll_factor = val);
                        state_scroll.mark_dirty_and_save(SettingsCategory::Touchpad);
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
                    touchpad.accel_speed,
                    -1.0,
                    1.0,
                    "",
                    move |val| {
                        state_accel.update_settings(|s| s.touchpad.accel_speed = val);
                        state_accel.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                )),
        ))
        // Behavior section
        .child(section(
            "Behavior",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Left-handed mode",
                    "Swap left and right buttons",
                    touchpad.left_handed,
                    move |val| {
                        state_left.update_settings(|s| s.touchpad.left_handed = val);
                        state_left.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(toggle_row(
                    "Disable while typing",
                    "Prevent accidental touches",
                    touchpad.dwt,
                    move |val| {
                        state_dwt.update_settings(|s| s.touchpad.dwt = val);
                        state_dwt.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(toggle_row(
                    "Disable with external mouse",
                    "Turn off when mouse connected",
                    touchpad.disabled_on_external_mouse,
                    move |val| {
                        state_external.update_settings(|s| s.touchpad.disabled_on_external_mouse = val);
                        state_external.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                ))
                .child(toggle_row(
                    "Disable touchpad",
                    "Turn off touchpad input",
                    touchpad.off,
                    move |val| {
                        state_off.update_settings(|s| s.touchpad.off = val);
                        state_off.mark_dirty_and_save(SettingsCategory::Touchpad);
                    },
                )),
        ))
}
