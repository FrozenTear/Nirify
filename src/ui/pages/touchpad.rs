//! Touchpad settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the touchpad settings page
pub fn touchpad_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let touchpad = &settings.touchpad;

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
    let state6 = state.clone();
    let mut refresh6 = state.refresh.clone();
    let state7 = state.clone();
    let mut refresh7 = state.refresh.clone();
    let state8 = state.clone();
    let mut refresh8 = state.refresh.clone();
    let state9 = state.clone();
    let mut refresh9 = state.refresh.clone();
    let state10 = state.clone();
    let mut refresh10 = state.refresh.clone();
    let state11 = state.clone();
    let mut refresh11 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.tap = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Drag gestures",
                    "Tap and drag to select",
                    touchpad.drag,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.drag = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Drag lock",
                    "Continue dragging without holding",
                    touchpad.drag_lock,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.drag_lock = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Two-finger tap for middle click",
                    touchpad.middle_emulation,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.middle_emulation = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
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
                        state5.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.natural_scroll = val
                        });
                        refresh5.with_mut(|mut v| *v += 1);
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
                        state6.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.scroll_factor = val
                        });
                        refresh6.with_mut(|mut v| *v += 1);
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
                        state7.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.accel_speed = val
                        });
                        refresh7.with_mut(|mut v| *v += 1);
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
                        state8.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.left_handed = val
                        });
                        refresh8.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable while typing",
                    "Prevent accidental touches",
                    touchpad.dwt,
                    move |val| {
                        state9.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.dwt = val
                        });
                        refresh9.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable with external mouse",
                    "Turn off when mouse connected",
                    touchpad.disabled_on_external_mouse,
                    move |val| {
                        state10.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.disabled_on_external_mouse = val
                        });
                        refresh10.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Disable touchpad",
                    "Turn off touchpad input",
                    touchpad.off,
                    move |val| {
                        state11.update_and_save(SettingsCategory::Touchpad, |s| {
                            s.touchpad.off = val
                        });
                        refresh11.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
