//! Keyboard settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{
    section, slider_row_with_callback, text_row_with_callback, toggle_row_with_callback,
};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the keyboard settings page
pub fn keyboard_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let keyboard = settings.keyboard;

    let off = RwSignal::new(keyboard.off);
    let xkb_layout = RwSignal::new(keyboard.xkb_layout);
    let xkb_variant = RwSignal::new(keyboard.xkb_variant);
    let xkb_options = RwSignal::new(keyboard.xkb_options);
    let repeat_delay = RwSignal::new(keyboard.repeat_delay as f64);
    let repeat_rate = RwSignal::new(keyboard.repeat_rate as f64);
    let numlock = RwSignal::new(keyboard.numlock);

    // XKB Callbacks
    let on_xkb_layout = {
        let state = state.clone();
        Rc::new(move |val: String| {
            state.update_settings(|s| s.keyboard.xkb_layout = val);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    let on_xkb_variant = {
        let state = state.clone();
        Rc::new(move |val: String| {
            state.update_settings(|s| s.keyboard.xkb_variant = val);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    let on_xkb_options = {
        let state = state.clone();
        Rc::new(move |val: String| {
            state.update_settings(|s| s.keyboard.xkb_options = val);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    // Repeat Callbacks
    let on_repeat_delay = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.keyboard.repeat_delay = val as i32);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    let on_repeat_rate = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.keyboard.repeat_rate = val as i32);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    let on_numlock = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.keyboard.numlock = val);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    let on_off = {
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.keyboard.off = val);
            state.mark_dirty_and_save(SettingsCategory::Keyboard);
        })
    };

    Stack::vertical((
        // XKB Settings section
        section(
            "Layout",
            Stack::vertical((
                text_row_with_callback(
                    "Keyboard layout",
                    Some("XKB layout code (e.g., us, de, fr)"),
                    xkb_layout,
                    "us",
                    Some(on_xkb_layout),
                ),
                text_row_with_callback(
                    "Layout variant",
                    Some("Layout variant (e.g., dvorak, colemak)"),
                    xkb_variant,
                    "",
                    Some(on_xkb_variant),
                ),
                text_row_with_callback(
                    "XKB options",
                    Some("Additional options (e.g., compose:ralt)"),
                    xkb_options,
                    "",
                    Some(on_xkb_options),
                ),
            )),
        ),
        // Repeat settings
        section(
            "Key Repeat",
            Stack::vertical((
                slider_row_with_callback(
                    "Repeat delay",
                    Some("Delay before key repeat starts"),
                    repeat_delay,
                    100.0,
                    1000.0,
                    25.0,
                    "ms",
                    Some(on_repeat_delay),
                ),
                slider_row_with_callback(
                    "Repeat rate",
                    Some("Characters per second when held"),
                    repeat_rate,
                    10.0,
                    100.0,
                    5.0,
                    "/s",
                    Some(on_repeat_rate),
                ),
            )),
        ),
        // Other settings
        section(
            "Options",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable NumLock",
                    Some("Turn on NumLock at startup"),
                    numlock,
                    Some(on_numlock),
                ),
                toggle_row_with_callback(
                    "Disable keyboard",
                    Some("WARNING: This can lock you out!"),
                    off,
                    Some(on_off),
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
