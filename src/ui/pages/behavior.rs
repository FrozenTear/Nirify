//! Behavior settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the behavior settings page
pub fn behavior_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let behavior = settings.behavior;

    let focus_follows_mouse = RwSignal::new(behavior.focus_follows_mouse);
    let always_center_single = RwSignal::new(behavior.always_center_single_column);
    let empty_workspace_above = RwSignal::new(behavior.empty_workspace_above_first);
    let workspace_back_and_forth = RwSignal::new(behavior.workspace_auto_back_and_forth);
    let disable_power_key = RwSignal::new(behavior.disable_power_key_handling);

    let strut_left = RwSignal::new(behavior.strut_left as f64);
    let strut_right = RwSignal::new(behavior.strut_right as f64);
    let strut_top = RwSignal::new(behavior.strut_top as f64);
    let strut_bottom = RwSignal::new(behavior.strut_bottom as f64);

    let default_column_width = RwSignal::new(behavior.default_column_width_proportion as f64);

    Stack::vertical((
        section(
            "Focus",
            Stack::vertical((
                toggle_row(
                    "Focus follows mouse",
                    Some("Automatically focus window under cursor"),
                    focus_follows_mouse,
                ),
                toggle_row(
                    "Workspace back and forth",
                    Some("Switch to previous workspace with same key"),
                    workspace_back_and_forth,
                ),
            )),
        ),
        section(
            "Layout",
            Stack::vertical((
                toggle_row(
                    "Center single column",
                    Some("Always center when only one column exists"),
                    always_center_single,
                ),
                toggle_row(
                    "Empty workspace above first",
                    Some("Add empty workspace above the first one"),
                    empty_workspace_above,
                ),
                slider_row(
                    "Default column width",
                    Some("Width proportion for new columns (0.5 = half)"),
                    default_column_width,
                    0.1,
                    2.0,
                    0.1,
                    "",
                ),
            )),
        ),
        section(
            "Screen Margins (Struts)",
            Stack::vertical((
                slider_row(
                    "Left margin",
                    Some("Reserved space on left edge"),
                    strut_left,
                    0.0,
                    500.0,
                    8.0,
                    "px",
                ),
                slider_row(
                    "Right margin",
                    Some("Reserved space on right edge"),
                    strut_right,
                    0.0,
                    500.0,
                    8.0,
                    "px",
                ),
                slider_row(
                    "Top margin",
                    Some("Reserved space on top edge"),
                    strut_top,
                    0.0,
                    500.0,
                    8.0,
                    "px",
                ),
                slider_row(
                    "Bottom margin",
                    Some("Reserved space on bottom edge"),
                    strut_bottom,
                    0.0,
                    500.0,
                    8.0,
                    "px",
                ),
            )),
        ),
        section(
            "System",
            Stack::vertical((toggle_row(
                "Disable power key handling",
                Some("Let the system handle the power button"),
                disable_power_key,
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
