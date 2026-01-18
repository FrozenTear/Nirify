//! Mouse settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the mouse settings page
pub fn mouse_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let mouse = settings.mouse;

    let off = RwSignal::new(mouse.off);
    let natural_scroll = RwSignal::new(mouse.natural_scroll);
    let left_handed = RwSignal::new(mouse.left_handed);
    let accel_speed = RwSignal::new(mouse.accel_speed);
    let scroll_factor = RwSignal::new(mouse.scroll_factor);
    let middle_emulation = RwSignal::new(mouse.middle_emulation);

    // Callbacks
    let on_natural_scroll = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.mouse.natural_scroll = val);
            state.mark_dirty_and_save(SettingsCategory::Mouse);
        })
    };

    let on_left_handed = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.mouse.left_handed = val);
            state.mark_dirty_and_save(SettingsCategory::Mouse);
        })
    };

    let on_middle_emulation = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.mouse.middle_emulation = val);
            state.mark_dirty_and_save(SettingsCategory::Mouse);
        })
    };

    let on_accel_speed = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.mouse.accel_speed = val);
            state.mark_dirty_and_save(SettingsCategory::Mouse);
        })
    };

    let on_scroll_factor = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.mouse.scroll_factor = val);
            state.mark_dirty_and_save(SettingsCategory::Mouse);
        })
    };

    let on_off = {
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.mouse.off = val);
            state.mark_dirty_and_save(SettingsCategory::Mouse);
        })
    };

    Stack::vertical((
        // Basic settings
        section(
            "General",
            Stack::vertical((
                toggle_row_with_callback(
                    "Natural scrolling",
                    Some("Invert scroll direction"),
                    natural_scroll,
                    Some(on_natural_scroll),
                ),
                toggle_row_with_callback(
                    "Left-handed mode",
                    Some("Swap left and right buttons"),
                    left_handed,
                    Some(on_left_handed),
                ),
                toggle_row_with_callback(
                    "Middle button emulation",
                    Some("Emulate middle click with left+right"),
                    middle_emulation,
                    Some(on_middle_emulation),
                ),
            )),
        ),
        // Speed settings
        section(
            "Speed",
            Stack::vertical((
                slider_row_with_callback(
                    "Acceleration",
                    Some("Pointer acceleration speed"),
                    accel_speed,
                    -1.0,
                    1.0,
                    0.1,
                    "",
                    Some(on_accel_speed),
                ),
                slider_row_with_callback(
                    "Scroll speed",
                    Some("Scroll sensitivity multiplier"),
                    scroll_factor,
                    0.1,
                    3.0,
                    0.1,
                    "x",
                    Some(on_scroll_factor),
                ),
            )),
        ),
        // Device control
        section(
            "Device",
            Stack::vertical((toggle_row_with_callback(
                "Disable mouse",
                Some("Turn off mouse input"),
                off,
                Some(on_off),
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
