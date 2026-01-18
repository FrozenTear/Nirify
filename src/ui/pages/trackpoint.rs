//! Trackpoint (pointing stick) settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the trackpoint settings page
pub fn trackpoint_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let trackpoint = settings.trackpoint;

    let off = RwSignal::new(trackpoint.off);
    let natural_scroll = RwSignal::new(trackpoint.natural_scroll);
    let left_handed = RwSignal::new(trackpoint.left_handed);
    let accel_speed = RwSignal::new(trackpoint.accel_speed);
    let middle_emulation = RwSignal::new(trackpoint.middle_emulation);
    let scroll_button_lock = RwSignal::new(trackpoint.scroll_button_lock);

    // Callbacks
    let on_natural_scroll = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.trackpoint.natural_scroll = val);
        state.mark_dirty_and_save(SettingsCategory::Trackpoint);
    })};

    let on_left_handed = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.trackpoint.left_handed = val);
        state.mark_dirty_and_save(SettingsCategory::Trackpoint);
    })};

    let on_middle_emulation = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.trackpoint.middle_emulation = val);
        state.mark_dirty_and_save(SettingsCategory::Trackpoint);
    })};

    let on_scroll_button_lock = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.trackpoint.scroll_button_lock = val);
        state.mark_dirty_and_save(SettingsCategory::Trackpoint);
    })};

    let on_accel_speed = { let state = state.clone(); Rc::new(move |val: f64| {
        state.update_settings(|s| s.trackpoint.accel_speed = val);
        state.mark_dirty_and_save(SettingsCategory::Trackpoint);
    })};

    let on_off = { Rc::new(move |val: bool| {
        state.update_settings(|s| s.trackpoint.off = val);
        state.mark_dirty_and_save(SettingsCategory::Trackpoint);
    })};

    Stack::vertical((
        section(
            "General",
            Stack::vertical((
                toggle_row_with_callback("Natural scrolling", Some("Invert scroll direction"), natural_scroll, Some(on_natural_scroll)),
                toggle_row_with_callback("Left-handed mode", Some("Swap left and right buttons"), left_handed, Some(on_left_handed)),
                toggle_row_with_callback("Middle button emulation", Some("Emulate middle click"), middle_emulation, Some(on_middle_emulation)),
            )),
        ),
        section(
            "Scrolling",
            Stack::vertical((
                toggle_row_with_callback("Scroll button lock", Some("Don't need to hold scroll button"), scroll_button_lock, Some(on_scroll_button_lock)),
            )),
        ),
        section(
            "Speed",
            Stack::vertical((
                slider_row_with_callback("Acceleration", Some("Pointer acceleration speed"), accel_speed, -1.0, 1.0, 0.1, "", Some(on_accel_speed)),
            )),
        ),
        section(
            "Device",
            Stack::vertical((
                toggle_row_with_callback("Disable trackpoint", Some("Turn off trackpoint input"), off, Some(on_off)),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
