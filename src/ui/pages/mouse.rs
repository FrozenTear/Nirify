//! Mouse settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, toggle_row};
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

    Stack::vertical((
        // Basic settings
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
                    Some("Emulate middle click with left+right"),
                    middle_emulation,
                ),
            )),
        ),
        // Speed settings
        section(
            "Speed",
            Stack::vertical((
                slider_row(
                    "Acceleration",
                    Some("Pointer acceleration speed"),
                    accel_speed,
                    -1.0,
                    1.0,
                    0.1,
                    "",
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
        // Device control
        section(
            "Device",
            Stack::vertical((toggle_row(
                "Disable mouse",
                Some("Turn off mouse input"),
                off,
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
