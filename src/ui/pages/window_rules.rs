//! Window rules settings page

use freya::prelude::*;

use crate::ui::components::{section, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the window rules settings page
pub fn window_rules_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let rules = &settings.window_rules.rules;

    let rule_count = rules.len();
    let status = if rules.is_empty() {
        "No window rules configured".to_string()
    } else {
        format!("{} window rule(s) configured", rule_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Window Rules summary section
        .child(section(
            "Window Rules",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current window rule configuration", &status))
                .child(value_row(
                    "Total rules",
                    "Number of window rules configured",
                    rule_count,
                )),
        ))
        // Info section
        .child(section(
            "About Window Rules",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Window rules let you customize behavior for specific windows \
                             based on their title, app-id, or other properties.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
