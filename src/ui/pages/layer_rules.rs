//! Layer rules settings page

use freya::prelude::*;

use crate::ui::components::{section, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the layer rules settings page
pub fn layer_rules_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let rules = &settings.layer_rules.rules;

    let rule_count = rules.len();
    let status = if rules.is_empty() {
        "No layer rules configured".to_string()
    } else {
        format!("{} layer rule(s) configured", rule_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Layer Rules summary section
        .child(section(
            "Layer Rules",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current layer rule configuration", &status))
                .child(value_row(
                    "Total rules",
                    "Number of layer rules configured",
                    rule_count,
                )),
        ))
        // Info section
        .child(section(
            "About Layer Rules",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Layer rules control the behavior of layer-shell surfaces \
                             like panels, notifications, and overlays.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
