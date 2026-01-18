//! Environment variables settings page

use freya::prelude::*;

use crate::ui::components::{section, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the environment settings page
pub fn environment_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let variables = &settings.environment.variables;

    let var_count = variables.len();
    let status = if variables.is_empty() {
        "No environment variables configured".to_string()
    } else {
        format!("{} environment variable(s) configured", var_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Environment Variables summary section
        .child(section(
            "Environment Variables",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current environment configuration", &status))
                .child(value_row(
                    "Total variables",
                    "Number of environment variables configured",
                    var_count,
                )),
        ))
        // Info section
        .child(section(
            "About Environment Variables",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Environment variables are set for all processes spawned by niri. \
                             Common uses include setting GTK_THEME, QT_QPA_PLATFORM, etc.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
