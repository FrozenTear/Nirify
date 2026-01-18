//! Trackball settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the trackball settings page
pub fn trackball_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let trackball = settings.trackball;

    let off = RwSignal::new(trackball.off);
    let natural_scroll = RwSignal::new(trackball.natural_scroll);
    let left_handed = RwSignal::new(trackball.left_handed);
    let accel_speed = RwSignal::new(trackball.accel_speed);
    let middle_emulation = RwSignal::new(trackball.middle_emulation);
    let scroll_button_lock = RwSignal::new(trackball.scroll_button_lock);

    // Callbacks
    let on_natural_scroll = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.trackball.natural_scroll = val);
            state.mark_dirty_and_save(SettingsCategory::Trackball);
        })
    };

    let on_left_handed = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.trackball.left_handed = val);
            state.mark_dirty_and_save(SettingsCategory::Trackball);
        })
    };

    let on_middle_emulation = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.trackball.middle_emulation = val);
            state.mark_dirty_and_save(SettingsCategory::Trackball);
        })
    };

    let on_scroll_button_lock = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.trackball.scroll_button_lock = val);
            state.mark_dirty_and_save(SettingsCategory::Trackball);
        })
    };

    let on_accel_speed = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.trackball.accel_speed = val);
            state.mark_dirty_and_save(SettingsCategory::Trackball);
        })
    };

    let on_off = {
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.trackball.off = val);
            state.mark_dirty_and_save(SettingsCategory::Trackball);
        })
    };

    Stack::vertical((
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
                    Some("Emulate middle click"),
                    middle_emulation,
                    Some(on_middle_emulation),
                ),
            )),
        ),
        section(
            "Scrolling",
            Stack::vertical((toggle_row_with_callback(
                "Scroll button lock",
                Some("Don't need to hold scroll button"),
                scroll_button_lock,
                Some(on_scroll_button_lock),
            ),)),
        ),
        section(
            "Speed",
            Stack::vertical((slider_row_with_callback(
                "Acceleration",
                Some("Pointer acceleration speed"),
                accel_speed,
                -1.0,
                1.0,
                0.1,
                "",
                Some(on_accel_speed),
            ),)),
        ),
        section(
            "Device",
            Stack::vertical((toggle_row_with_callback(
                "Disable trackball",
                Some("Turn off trackball input"),
                off,
                Some(on_off),
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
