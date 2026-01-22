//! Tablet settings view
//!
//! Configure graphics tablet and pen input behavior.

use iced::widget::{column, container, scrollable, text, text_input};
use iced::Element;

use super::widgets::*;
use crate::config::models::TabletSettings;
use crate::messages::{Message, TabletMessage};

/// Creates the tablet settings view
pub fn view(settings: TabletSettings) -> Element<'static, Message> {
    let off = settings.off;
    let left_handed = settings.left_handed;
    let map_to_output = settings.map_to_output.clone();

    let content = column![
        section_header("Tablet Settings"),
        info_text(
            "Configure graphics tablet and pen input behavior."
        ),
        toggle_row(
            "Disable tablet",
            "Completely disable this tablet device",
            off,
            |value| Message::Tablet(TabletMessage::SetOff(value)),
        ),
        spacer(16.0),

        section_header("Output Mapping"),
        info_text(
            "Map the tablet to a specific display. Leave empty to use the default (full desktop)."
        ),
        column![
            text("Map to output").size(14),
            text("Output name (e.g., eDP-1, HDMI-A-1)").size(12).color([0.6, 0.6, 0.6]),
            text_input("Leave empty for default", &map_to_output)
                .on_input(|value| Message::Tablet(TabletMessage::SetMapToOutput(value)))
                .padding(8),
        ]
        .spacing(6)
        .padding(12),
        spacer(16.0),

        section_header("Configuration"),
        toggle_row(
            "Left-handed mode",
            "Rotate tablet 180 degrees for left-handed use",
            left_handed,
            |value| Message::Tablet(TabletMessage::SetLeftHanded(value)),
        ),
        spacer(16.0),

        subsection_header("Calibration Matrix"),
        info_text(
            "Advanced: Calibration matrix for libinput (6 values). \
             Leave empty to use default calibration."
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
