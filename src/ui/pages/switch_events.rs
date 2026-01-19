//! Switch events (lid, tablet mode) settings page

use freya::prelude::*;

use crate::ui::app::ReactiveState;
use crate::ui::components::{section, value_row};
use crate::ui::theme::{SPACING_LG, TEXT_SECONDARY};

/// Create the switch events settings page
pub fn switch_events_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let events = &settings.switch_events;

    let lid_close_cmd = events.lid_close.spawn.join(" ");
    let lid_open_cmd = events.lid_open.spawn.join(" ");
    let tablet_on_cmd = events.tablet_mode_on.spawn.join(" ");
    let tablet_off_cmd = events.tablet_mode_off.spawn.join(" ");

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Lid Events section
        .child(section(
            "Lid Events",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row(
                    "Lid close",
                    "Command when lid closes",
                    if lid_close_cmd.is_empty() {
                        "(none)"
                    } else {
                        &lid_close_cmd
                    },
                ))
                .child(value_row(
                    "Lid open",
                    "Command when lid opens",
                    if lid_open_cmd.is_empty() {
                        "(none)"
                    } else {
                        &lid_open_cmd
                    },
                )),
        ))
        // Tablet Mode Events section
        .child(section(
            "Tablet Mode Events",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(value_row(
                    "Tablet mode on",
                    "Command when entering tablet mode",
                    if tablet_on_cmd.is_empty() {
                        "(none)"
                    } else {
                        &tablet_on_cmd
                    },
                ))
                .child(value_row(
                    "Tablet mode off",
                    "Command when exiting tablet mode",
                    if tablet_off_cmd.is_empty() {
                        "(none)"
                    } else {
                        &tablet_off_cmd
                    },
                )),
        ))
        // Info section
        .child(section(
            "About Switch Events",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(
                    label()
                        .text(
                            "Switch events trigger actions when hardware switches change state, \
                             such as closing the laptop lid or entering tablet mode.",
                        )
                        .color(TEXT_SECONDARY)
                        .max_lines(2),
                ),
        ))
}
