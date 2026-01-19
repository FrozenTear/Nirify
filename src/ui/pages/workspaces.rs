//! Workspaces settings page

use freya::prelude::*;

use crate::ui::app::ReactiveState;
use crate::ui::components::{section, value_row};
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the workspaces settings page
pub fn workspaces_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let workspaces = &settings.workspaces.workspaces;

    let workspace_count = workspaces.len();
    let status = if workspaces.is_empty() {
        "No named workspaces configured".to_string()
    } else {
        format!("{} named workspaces configured", workspace_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Named Workspaces summary section
        .child(section(
            "Named Workspaces",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current workspace configuration", &status))
                .child(value_row(
                    "Total workspaces",
                    "Number of named workspaces configured",
                    workspace_count,
                )),
        ))
        // Info section
        .child(section(
            "About Workspaces",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Named workspaces can be pinned to specific outputs \
                             and accessed by name in keybindings.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
