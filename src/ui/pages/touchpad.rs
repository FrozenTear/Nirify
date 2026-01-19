//! Touchpad settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::types::{AccelProfile, ClickMethod, ScrollMethod, TapButtonMap};
use crate::ui::app::{
    ReactiveState, DROPDOWN_TOUCHPAD_ACCEL_PROFILE, DROPDOWN_TOUCHPAD_CLICK_METHOD,
    DROPDOWN_TOUCHPAD_SCROLL_METHOD, DROPDOWN_TOUCHPAD_TAP_BUTTON_MAP,
};
use crate::ui::components::{section, select_row_with_state, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Acceleration profile options
const ACCEL_PROFILE_OPTIONS: &[&str] = &["Adaptive", "Flat"];

/// Scroll method options
const SCROLL_METHOD_OPTIONS: &[&str] = &["Two Finger", "Edge", "On Button Down", "No Scroll"];

/// Click method options
const CLICK_METHOD_OPTIONS: &[&str] = &["Button Areas", "Clickfinger"];

/// Tap button map options
const TAP_BUTTON_MAP_OPTIONS: &[&str] = &["Left-Right-Middle", "Left-Middle-Right"];

fn accel_profile_to_index(p: AccelProfile) -> usize {
    match p {
        AccelProfile::Adaptive => 0,
        AccelProfile::Flat => 1,
    }
}

fn index_to_accel_profile(i: usize) -> AccelProfile {
    match i {
        0 => AccelProfile::Adaptive,
        1 => AccelProfile::Flat,
        _ => AccelProfile::Adaptive,
    }
}

fn scroll_method_to_index(m: ScrollMethod) -> usize {
    match m {
        ScrollMethod::TwoFinger => 0,
        ScrollMethod::Edge => 1,
        ScrollMethod::OnButtonDown => 2,
        ScrollMethod::NoScroll => 3,
    }
}

fn index_to_scroll_method(i: usize) -> ScrollMethod {
    match i {
        0 => ScrollMethod::TwoFinger,
        1 => ScrollMethod::Edge,
        2 => ScrollMethod::OnButtonDown,
        3 => ScrollMethod::NoScroll,
        _ => ScrollMethod::TwoFinger,
    }
}

fn click_method_to_index(m: ClickMethod) -> usize {
    match m {
        ClickMethod::ButtonAreas => 0,
        ClickMethod::Clickfinger => 1,
    }
}

fn index_to_click_method(i: usize) -> ClickMethod {
    match i {
        0 => ClickMethod::ButtonAreas,
        1 => ClickMethod::Clickfinger,
        _ => ClickMethod::ButtonAreas,
    }
}

fn tap_button_map_to_index(m: TapButtonMap) -> usize {
    match m {
        TapButtonMap::LeftRightMiddle => 0,
        TapButtonMap::LeftMiddleRight => 1,
    }
}

fn index_to_tap_button_map(i: usize) -> TapButtonMap {
    match i {
        0 => TapButtonMap::LeftRightMiddle,
        1 => TapButtonMap::LeftMiddleRight,
        _ => TapButtonMap::LeftRightMiddle,
    }
}

/// Create the touchpad settings page
pub fn touchpad_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let touchpad = &settings.touchpad;

    let accel_profile = touchpad.accel_profile;
    let scroll_method = touchpad.scroll_method;
    let click_method = touchpad.click_method;
    let tap_button_map = touchpad.tap_button_map;

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
                ))
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Click method",
                        "How multi-finger taps are interpreted",
                        CLICK_METHOD_OPTIONS,
                        click_method_to_index(click_method),
                        DROPDOWN_TOUCHPAD_CLICK_METHOD,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Touchpad, |s| {
                                s.touchpad.click_method = index_to_click_method(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Tap button map",
                        "Which buttons multi-finger taps trigger",
                        TAP_BUTTON_MAP_OPTIONS,
                        tap_button_map_to_index(tap_button_map),
                        DROPDOWN_TOUCHPAD_TAP_BUTTON_MAP,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Touchpad, |s| {
                                s.touchpad.tap_button_map = index_to_tap_button_map(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                }),
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
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Scroll method",
                        "How scrolling is triggered",
                        SCROLL_METHOD_OPTIONS,
                        scroll_method_to_index(scroll_method),
                        DROPDOWN_TOUCHPAD_SCROLL_METHOD,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Touchpad, |s| {
                                s.touchpad.scroll_method = index_to_scroll_method(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
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
        // Speed & Acceleration section
        .child(section(
            "Speed & Acceleration",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Acceleration profile",
                        "Pointer acceleration curve",
                        ACCEL_PROFILE_OPTIONS,
                        accel_profile_to_index(accel_profile),
                        DROPDOWN_TOUCHPAD_ACCEL_PROFILE,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Touchpad, |s| {
                                s.touchpad.accel_profile = index_to_accel_profile(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(slider_row(
                    "Acceleration speed",
                    "Pointer acceleration intensity",
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
