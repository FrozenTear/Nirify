//! Trackball settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the trackball settings page
pub fn trackball_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let trackball = &settings.trackball;

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
                    trackball.natural_scroll,
                    move |val| {
                        state1.update_and_save(SettingsCategory::Trackball, |s| {
                            s.trackball.natural_scroll = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Left-handed mode",
                    "Swap left and right buttons",
                    trackball.left_handed,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Trackball, |s| {
                            s.trackball.left_handed = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Emulate middle click",
                    trackball.middle_emulation,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Trackball, |s| {
                            s.trackball.middle_emulation = val
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
                .child(toggle_row(
                    "Scroll button lock",
                    "Don't need to hold scroll button",
                    trackball.scroll_button_lock,
                    move |val| {
                        state4.update_and_save(SettingsCategory::Trackball, |s| {
                            s.trackball.scroll_button_lock = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
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
                    trackball.accel_speed,
                    -1.0,
                    1.0,
                    "",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Trackball, |s| {
                            s.trackball.accel_speed = val
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
                    "Disable trackball",
                    "Turn off trackball input",
                    trackball.off,
                    move |val| {
                        state6.update_and_save(SettingsCategory::Trackball, |s| {
                            s.trackball.off = val
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
