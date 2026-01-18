//! Touchpad settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, toggle_row};
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

    Stack::vertical((
        // Tap & Click
        section(
            "Tap & Click",
            Stack::vertical((
                toggle_row("Tap to click", Some("Tap the touchpad to click"), tap),
                toggle_row(
                    "Drag gestures",
                    Some("Tap and drag to select"),
                    drag,
                ),
                toggle_row(
                    "Drag lock",
                    Some("Continue dragging without holding"),
                    drag_lock,
                ),
                toggle_row(
                    "Middle button emulation",
                    Some("Two-finger tap for middle click"),
                    middle_emulation,
                ),
            )),
        ),
        // Scrolling
        section(
            "Scrolling",
            Stack::vertical((
                toggle_row(
                    "Natural scrolling",
                    Some("Content follows finger direction"),
                    natural_scroll,
                ),
                slider_row(
                    "Scroll speed",
                    Some("Scroll sensitivity multiplier"),
                    scroll_factor,
                    0.1,
                    2.0,
                    0.1,
                    "x",
                ),
            )),
        ),
        // Speed
        section(
            "Speed",
            Stack::vertical((slider_row(
                "Acceleration",
                Some("Pointer acceleration speed"),
                accel_speed,
                -1.0,
                1.0,
                0.1,
                "",
            ),)),
        ),
        // Behavior
        section(
            "Behavior",
            Stack::vertical((
                toggle_row(
                    "Left-handed mode",
                    Some("Swap left and right buttons"),
                    left_handed,
                ),
                toggle_row(
                    "Disable while typing",
                    Some("Prevent accidental touches"),
                    dwt,
                ),
                toggle_row(
                    "Disable with external mouse",
                    Some("Turn off when mouse connected"),
                    disabled_on_external,
                ),
                toggle_row(
                    "Disable touchpad",
                    Some("Turn off touchpad input"),
                    off,
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
