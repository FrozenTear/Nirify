//! Tablet settings view
//!
//! Configure graphics tablet and pen input behavior.

use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::TabletSettings;
use crate::messages::{Message, TabletMessage};
use crate::theme::muted_text_container;

/// Creates the tablet settings view
pub fn view<'a>(settings: &'a TabletSettings, calibration_cache: &'a [String; 6]) -> Element<'a, Message> {
    let off = settings.off;
    let left_handed = settings.left_handed;
    let map_to_output = settings.map_to_output.clone();
    let matrix_values = settings.calibration_matrix;

    let content = column![
        page_title("Tablet Settings"),
        info_text(
            "Configure graphics tablet and pen input behavior."
        ),
        card(column![
            toggle_row(
                "Disable tablet",
                "Completely disable this tablet device",
                off,
                |value| Message::Tablet(TabletMessage::SetOff(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Output Mapping"),
        info_text(
            "Map the tablet to a specific display. Leave empty to use the default (full desktop)."
        ),
        card(column![
            column![
                text("Map to output").size(14),
                container(text("Output name (e.g., eDP-1, HDMI-A-1)").size(12)).style(muted_text_container),
                text_input("Leave empty for default", &map_to_output)
                    .on_input(|value| Message::Tablet(TabletMessage::SetMapToOutput(value)))
                    .padding(8),
            ]
            .spacing(6)
            .padding(12),
        ].spacing(0).width(Length::Fill)),
        section_header("Configuration"),
        card(column![
            toggle_row(
                "Left-handed mode",
                "Rotate tablet 180 degrees for left-handed use",
                left_handed,
                |value| Message::Tablet(TabletMessage::SetLeftHanded(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        subsection_header("Advanced"),
        card(column![
            calibration_matrix(
                "Calibration Matrix",
                "Advanced: Calibration matrix for libinput (6 values). \
                 This transforms touch coordinates: [x', y'] = [a b c; d e f] * [x, y, 1]",
                matrix_values,
                calibration_cache,
                |msg| match msg {
                    CalibrationMatrixMessage::SetValue(idx, val) =>
                        Message::Tablet(TabletMessage::SetCalibrationValue(idx, val)),
                    CalibrationMatrixMessage::Clear =>
                        Message::Tablet(TabletMessage::ClearCalibration),
                    CalibrationMatrixMessage::Reset =>
                        Message::Tablet(TabletMessage::ResetCalibration),
                },
            ),
        ].spacing(0).width(Length::Fill)),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(Length::Fill))
        .height(Length::Fill)
        .into()
}
