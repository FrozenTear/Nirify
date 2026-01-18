//! Keyboard settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the keyboard settings page
pub fn keyboard_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let keyboard = settings.keyboard;

    let state_layout = state.clone();
    let state_variant = state.clone();
    let state_options = state.clone();
    let state_delay = state.clone();
    let state_rate = state.clone();
    let state_numlock = state.clone();

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
                        state_layout.update_settings(|s| s.keyboard.xkb_layout = val);
                        state_layout.mark_dirty_and_save(SettingsCategory::Keyboard);
                    },
                ))
                .child(text_row(
                    "Layout variant",
                    "Layout variant (e.g., dvorak, colemak)",
                    &keyboard.xkb_variant,
                    "",
                    move |val| {
                        state_variant.update_settings(|s| s.keyboard.xkb_variant = val);
                        state_variant.mark_dirty_and_save(SettingsCategory::Keyboard);
                    },
                ))
                .child(text_row(
                    "XKB options",
                    "Additional options (e.g., compose:ralt)",
                    &keyboard.xkb_options,
                    "",
                    move |val| {
                        state_options.update_settings(|s| s.keyboard.xkb_options = val);
                        state_options.mark_dirty_and_save(SettingsCategory::Keyboard);
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
                        state_delay.update_settings(|s| s.keyboard.repeat_delay = val as i32);
                        state_delay.mark_dirty_and_save(SettingsCategory::Keyboard);
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
                        state_rate.update_settings(|s| s.keyboard.repeat_rate = val as i32);
                        state_rate.mark_dirty_and_save(SettingsCategory::Keyboard);
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
                        state_numlock.update_settings(|s| s.keyboard.numlock = val);
                        state_numlock.mark_dirty_and_save(SettingsCategory::Keyboard);
                    },
                )),
        ))
}
