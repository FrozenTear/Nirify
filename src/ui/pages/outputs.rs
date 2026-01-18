//! Outputs (displays) settings page

use freya::prelude::*;

use crate::ui::components::{section, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the outputs settings page
pub fn outputs_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let outputs = &settings.outputs.outputs;

    let output_count = outputs.len();
    let status = if outputs.is_empty() {
        "No displays configured".to_string()
    } else {
        format!("{} display(s) configured", output_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Configured Displays summary section
        .child(section(
            "Configured Displays",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current display configuration", &status))
                .child(value_row(
                    "Total displays",
                    "Number of displays configured",
                    output_count,
                )),
        ))
        // Info section
        .child(section(
            "About Display Configuration",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Display settings control resolution, refresh rate, scaling, \
                             rotation, and position for each monitor.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
