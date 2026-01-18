//! Startup commands settings page

use freya::prelude::*;

use crate::ui::components::{section, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the startup settings page
pub fn startup_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let commands = &settings.startup.commands;

    let command_count = commands.len();
    let status = if commands.is_empty() {
        "No startup commands configured".to_string()
    } else {
        format!("{} startup command(s) configured", command_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Startup Commands summary section
        .child(section(
            "Startup Commands",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current startup configuration", &status))
                .child(value_row(
                    "Total commands",
                    "Number of startup commands configured",
                    command_count,
                )),
        ))
        // Info section
        .child(section(
            "About Startup Commands",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Startup commands run when niri starts. \
                             Use them to launch panels, background services, or other apps.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
