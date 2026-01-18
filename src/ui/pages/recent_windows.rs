//! Recent windows (Alt-Tab) settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the recent windows settings page
pub fn recent_windows_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let recent = settings.recent_windows;

    let off = RwSignal::new(recent.off);
    let debounce_ms = RwSignal::new(recent.debounce_ms as f64);
    let open_delay_ms = RwSignal::new(recent.open_delay_ms as f64);

    Stack::vertical((
        section(
            "General",
            Stack::vertical((toggle_row(
                "Disable recent windows",
                Some("Turn off the Alt-Tab switcher"),
                off,
            ),)),
        ),
        section(
            "Timing",
            Stack::vertical((
                slider_row(
                    "Debounce delay",
                    Some("Delay before adding to recent list"),
                    debounce_ms,
                    0.0,
                    1000.0,
                    50.0,
                    "ms",
                ),
                slider_row(
                    "Open delay",
                    Some("Delay before UI appears"),
                    open_delay_ms,
                    0.0,
                    500.0,
                    25.0,
                    "ms",
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
