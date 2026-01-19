//! Keybindings settings page

use freya::prelude::*;

use crate::ui::app::ReactiveState;
use crate::ui::components::{section, value_row};
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the keybindings settings page
pub fn keybindings_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let bindings = &settings.keybindings.bindings;

    let binding_count = bindings.len();
    let status = if bindings.is_empty() {
        "No keybindings configured".to_string()
    } else {
        format!("{} keybindings configured", binding_count)
    };

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Keybindings summary section
        .child(section(
            "Keybindings",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row("Status", "Current keybinding configuration", &status))
                .child(value_row(
                    "Total bindings",
                    "Number of keybindings configured",
                    binding_count,
                )),
        ))
        // Info section
        .child(section(
            "About Keybindings",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Keybindings map key combinations to actions. \
                             Edit your config file to add or modify bindings.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
