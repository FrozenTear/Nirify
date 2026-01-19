//! Trackpoint (pointing stick) settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::types::{AccelProfile, ScrollMethod};
use crate::ui::app::{
    ReactiveState, DROPDOWN_TRACKPOINT_ACCEL_PROFILE, DROPDOWN_TRACKPOINT_SCROLL_METHOD,
};
use crate::ui::components::{section, select_row_with_state, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Acceleration profile options
const ACCEL_PROFILE_OPTIONS: &[&str] = &["Adaptive", "Flat"];

/// Scroll method options
const SCROLL_METHOD_OPTIONS: &[&str] = &["Two Finger", "Edge", "On Button Down", "No Scroll"];

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

/// Create the trackpoint settings page
pub fn trackpoint_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let trackpoint = &settings.trackpoint;

    let accel_profile = trackpoint.accel_profile;
    let scroll_method = trackpoint.scroll_method;

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
                        state1.update_and_save(SettingsCategory::Trackpoint, |s| {
                            s.trackpoint.natural_scroll = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Left-handed mode",
                    "Swap left and right buttons",
                    trackpoint.left_handed,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Trackpoint, |s| {
                            s.trackpoint.left_handed = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Emulate middle click",
                    trackpoint.middle_emulation,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Trackpoint, |s| {
                            s.trackpoint.middle_emulation = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Scrolling section
        .child(section(
            "Scrolling",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child({
                    let state_clone = state.clone();
                    let mut refresh = state.refresh.clone();
                    select_row_with_state(
                        "Scroll method",
                        "How scrolling is triggered",
                        SCROLL_METHOD_OPTIONS,
                        scroll_method_to_index(scroll_method),
                        DROPDOWN_TRACKPOINT_SCROLL_METHOD,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Trackpoint, |s| {
                                s.trackpoint.scroll_method = index_to_scroll_method(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(toggle_row(
                    "Scroll button lock",
                    "Don't need to hold scroll button",
                    trackpoint.scroll_button_lock,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Trackpoint, |s| {
                            s.trackpoint.scroll_button_lock = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
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
                        DROPDOWN_TRACKPOINT_ACCEL_PROFILE,
                        &state,
                        move |i| {
                            state_clone.update_and_save(SettingsCategory::Trackpoint, |s| {
                                s.trackpoint.accel_profile = index_to_accel_profile(i);
                            });
                            *refresh.write() += 1;
                        },
                    )
                })
                .child(slider_row(
                    "Acceleration speed",
                    "Pointer acceleration intensity",
                    trackpoint.accel_speed,
                    -1.0,
                    1.0,
                    "",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Trackpoint, |s| {
                            s.trackpoint.accel_speed = val
                        });
                        refresh5.with_mut(|mut v| *v += 1);
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
                        state6.update_and_save(SettingsCategory::Trackpoint, |s| {
                            s.trackpoint.off = val
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
