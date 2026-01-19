//! Keyboard settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the keyboard settings page
pub fn keyboard_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let keyboard = &settings.keyboard;

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
        // Layout section
        .child(section(
            "Layout",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(text_row(
                    "Keyboard layout",
                    "XKB layout code (e.g., us, de, fr)",
                    &keyboard.xkb_layout,
                    "us",
                    move |val| {
                        state1.update_and_save(SettingsCategory::Keyboard, |s| {
                            s.keyboard.xkb_layout = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(text_row(
                    "Layout variant",
                    "Layout variant (e.g., dvorak, colemak)",
                    &keyboard.xkb_variant,
                    "",
                    move |val| {
                        state2.update_and_save(SettingsCategory::Keyboard, |s| {
                            s.keyboard.xkb_variant = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(text_row(
                    "XKB options",
                    "Additional options (e.g., compose:ralt)",
                    &keyboard.xkb_options,
                    "",
                    move |val| {
                        state3.update_and_save(SettingsCategory::Keyboard, |s| {
                            s.keyboard.xkb_options = val
                        });
                        refresh3.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Key Repeat section
        .child(section(
            "Key Repeat",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Repeat delay",
                    "Delay before key repeat starts",
                    keyboard.repeat_delay as f64,
                    100.0,
                    1000.0,
                    "ms",
                    move |val| {
                        state4.update_and_save(SettingsCategory::Keyboard, |s| {
                            s.keyboard.repeat_delay = val as i32
                        });
                        refresh4.with_mut(|mut v| *v += 1);
                    },
                ))
                .child(slider_row(
                    "Repeat rate",
                    "Characters per second when held",
                    keyboard.repeat_rate as f64,
                    10.0,
                    100.0,
                    "/s",
                    move |val| {
                        state5.update_and_save(SettingsCategory::Keyboard, |s| {
                            s.keyboard.repeat_rate = val as i32
                        });
                        refresh5.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
        // Options section
        .child(section(
            "Options",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable NumLock",
                    "Turn on NumLock at startup",
                    keyboard.numlock,
                    move |val| {
                        state6.update_and_save(SettingsCategory::Keyboard, |s| {
                            s.keyboard.numlock = val
                        });
                        refresh6.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
