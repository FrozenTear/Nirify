//! Trackball settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, toggle_row};
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

    Stack::vertical((
        section(
            "General",
            Stack::vertical((
                toggle_row(
                    "Natural scrolling",
                    Some("Invert scroll direction"),
                    natural_scroll,
                ),
                toggle_row(
                    "Left-handed mode",
                    Some("Swap left and right buttons"),
                    left_handed,
                ),
                toggle_row(
                    "Middle button emulation",
                    Some("Emulate middle click"),
                    middle_emulation,
                ),
            )),
        ),
        section(
            "Scrolling",
            Stack::vertical((toggle_row(
                "Scroll button lock",
                Some("Don't need to hold scroll button"),
                scroll_button_lock,
            ),)),
        ),
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
        section(
            "Device",
            Stack::vertical((toggle_row(
                "Disable trackball",
                Some("Turn off trackball input"),
                off,
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
