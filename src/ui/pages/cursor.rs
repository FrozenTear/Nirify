//! Cursor settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, text_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the cursor settings page
pub fn cursor_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let cursor = settings.cursor;

    let theme = RwSignal::new(cursor.theme);
    let size = RwSignal::new(cursor.size as f64);
    let hide_when_typing = RwSignal::new(cursor.hide_when_typing);
    let hide_after_inactive = RwSignal::new(cursor.hide_after_inactive_ms.unwrap_or(0) as f64);
    let auto_hide_enabled = RwSignal::new(cursor.hide_after_inactive_ms.is_some());

    Stack::vertical((
        section(
            "Appearance",
            Stack::vertical((
                text_row(
                    "Cursor theme",
                    Some("Theme name (empty = system default)"),
                    theme,
                    "Adwaita",
                ),
                slider_row(
                    "Cursor size",
                    Some("Size in pixels"),
                    size,
                    12.0,
                    100.0,
                    4.0,
                    "px",
                ),
            )),
        ),
        section(
            "Behavior",
            Stack::vertical((
                toggle_row(
                    "Hide while typing",
                    Some("Hide cursor when using keyboard"),
                    hide_when_typing,
                ),
                toggle_row(
                    "Auto-hide when idle",
                    Some("Hide cursor after inactivity"),
                    auto_hide_enabled,
                ),
                slider_row(
                    "Auto-hide delay",
                    Some("Milliseconds before hiding"),
                    hide_after_inactive,
                    100.0,
                    10000.0,
                    100.0,
                    "ms",
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
