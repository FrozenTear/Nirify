//! Mouse settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the mouse settings page
pub fn mouse_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let mouse = &settings.mouse;

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
                    mouse.natural_scroll,
                    move |val| {
                        state1.update_and_save(SettingsCategory::Mouse, |s| {
                            s.mouse.natural_scroll = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Left-handed mode",
                    "Swap left and right buttons",
                    mouse.left_handed,
                    move |val| {
                        state2.update_and_save(SettingsCategory::Mouse, |s| {
                            s.mouse.left_handed = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(toggle_row(
                    "Middle button emulation",
                    "Emulate middle click with left+right",
                    mouse.middle_emulation,
                    move |val| {
                        state3.update_and_save(SettingsCategory::Mouse, |s| {
                            s.mouse.middle_emulation = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
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
                        state4.update_and_save(SettingsCategory::Mouse, |s| {
                            s.mouse.accel_speed = val
                        });
                        refresh4.with_mut(|mut v| *v += 1);
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
                        state5.update_and_save(SettingsCategory::Mouse, |s| {
                            s.mouse.scroll_factor = val
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
                    "Disable mouse",
                    "Turn off mouse input",
                    mouse.off,
                    move |val| {
                        state6.update_and_save(SettingsCategory::Mouse, |s| {
                            s.mouse.off = val
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
