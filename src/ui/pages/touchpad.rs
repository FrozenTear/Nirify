//! Touchpad settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the touchpad settings page
pub fn touchpad_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let touchpad = settings.touchpad;

    let off = RwSignal::new(touchpad.off);
    let tap = RwSignal::new(touchpad.tap);
    let natural_scroll = RwSignal::new(touchpad.natural_scroll);
    let left_handed = RwSignal::new(touchpad.left_handed);
    let dwt = RwSignal::new(touchpad.dwt);
    let drag = RwSignal::new(touchpad.drag);
    let drag_lock = RwSignal::new(touchpad.drag_lock);
    let accel_speed = RwSignal::new(touchpad.accel_speed);
    let scroll_factor = RwSignal::new(touchpad.scroll_factor);
    let middle_emulation = RwSignal::new(touchpad.middle_emulation);
    let disabled_on_external = RwSignal::new(touchpad.disabled_on_external_mouse);

    // Callbacks
    let on_tap = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.tap = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_drag = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.drag = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_drag_lock = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.drag_lock = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_middle_emulation = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.middle_emulation = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_natural_scroll = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.natural_scroll = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_scroll_factor = { let state = state.clone(); Rc::new(move |val: f64| {
        state.update_settings(|s| s.touchpad.scroll_factor = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_accel_speed = { let state = state.clone(); Rc::new(move |val: f64| {
        state.update_settings(|s| s.touchpad.accel_speed = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_left_handed = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.left_handed = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_dwt = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.dwt = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_disabled_on_external = { let state = state.clone(); Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.disabled_on_external_mouse = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    let on_off = { Rc::new(move |val: bool| {
        state.update_settings(|s| s.touchpad.off = val);
        state.mark_dirty_and_save(SettingsCategory::Touchpad);
    })};

    Stack::vertical((
        // Tap & Click
        section(
            "Tap & Click",
            Stack::vertical((
                toggle_row_with_callback("Tap to click", Some("Tap the touchpad to click"), tap, Some(on_tap)),
                toggle_row_with_callback("Drag gestures", Some("Tap and drag to select"), drag, Some(on_drag)),
                toggle_row_with_callback("Drag lock", Some("Continue dragging without holding"), drag_lock, Some(on_drag_lock)),
                toggle_row_with_callback("Middle button emulation", Some("Two-finger tap for middle click"), middle_emulation, Some(on_middle_emulation)),
            )),
        ),
        // Scrolling
        section(
            "Scrolling",
            Stack::vertical((
                toggle_row_with_callback("Natural scrolling", Some("Content follows finger direction"), natural_scroll, Some(on_natural_scroll)),
                slider_row_with_callback("Scroll speed", Some("Scroll sensitivity multiplier"), scroll_factor, 0.1, 3.0, 0.1, "x", Some(on_scroll_factor)),
            )),
        ),
        // Speed
        section(
            "Speed",
            Stack::vertical((
                slider_row_with_callback("Acceleration", Some("Pointer acceleration speed"), accel_speed, -1.0, 1.0, 0.1, "", Some(on_accel_speed)),
            )),
        ),
        // Behavior
        section(
            "Behavior",
            Stack::vertical((
                toggle_row_with_callback("Left-handed mode", Some("Swap left and right buttons"), left_handed, Some(on_left_handed)),
                toggle_row_with_callback("Disable while typing", Some("Prevent accidental touches"), dwt, Some(on_dwt)),
                toggle_row_with_callback("Disable with external mouse", Some("Turn off when mouse connected"), disabled_on_external, Some(on_disabled_on_external)),
                toggle_row_with_callback("Disable touchpad", Some("Turn off touchpad input"), off, Some(on_off)),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
