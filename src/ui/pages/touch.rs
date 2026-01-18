//! Touch screen settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the touch screen settings page
pub fn touch_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let touch = settings.touch;

    let off = RwSignal::new(touch.off);
    let map_to_output = RwSignal::new(touch.map_to_output);

    Stack::vertical((
        section(
            "Display Mapping",
            Stack::vertical((text_row(
                "Map to output",
                Some("Monitor name to map touch to (e.g., eDP-1)"),
                map_to_output,
                "",
            ),)),
        ),
        section(
            "Device",
            Stack::vertical((toggle_row(
                "Disable touch",
                Some("Turn off touch input"),
                off,
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
