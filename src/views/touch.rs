//! Touch settings view
//!
//! Configure touchscreen behavior and mapping.

use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::*;
use crate::config::models::TouchSettings;
use crate::messages::{Message, TouchMessage};
use crate::theme::muted_text_container;

/// Creates the touch settings view
pub fn view<'a>(settings: &'a TouchSettings, calibration_cache: &'a [String; 6]) -> Element<'a, Message> {
    let off = settings.off;
    let map_to_output = settings.map_to_output.clone();
    let matrix_values = settings.calibration_matrix;

    let content = column![
        page_title("Touch Settings"),
        info_text(
            "Configure touchscreen behavior and mapping."
        ),
        card(column![
            toggle_row(
                "Disable touch",
                "Completely disable this touch device",
                off,
                |value| Message::Touch(TouchMessage::SetOff(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Output Mapping"),
        info_text(
            "Map the touchscreen to a specific display. This is important for multi-monitor setups \
             to ensure touch input is properly aligned with the screen."
        ),
        card(column![
            column![
                text("Map to output").size(16),
                container(text("Output name (e.g., eDP-1, HDMI-A-1)").size(12)).style(muted_text_container),
                text_input("Leave empty for default", &map_to_output)
                    .on_input(|value| Message::Touch(TouchMessage::SetMapToOutput(value)))
                    .padding(8),
            ]
            .spacing(6)
            .padding(12),
        ].spacing(0).width(Length::Fill)),
        subsection_header("Advanced"),
        card(column![
            calibration_matrix(
                "Calibration Matrix",
                "Advanced: Calibration matrix for libinput (6 values). \
                 This transforms touch coordinates and can correct misaligned touch input.",
                matrix_values,
                calibration_cache,
                |msg| match msg {
                    CalibrationMatrixMessage::SetValue(idx, val) =>
                        Message::Touch(TouchMessage::SetCalibrationValue(idx, val)),
                    CalibrationMatrixMessage::Clear =>
                        Message::Touch(TouchMessage::ClearCalibration),
                    CalibrationMatrixMessage::Reset =>
                        Message::Touch(TouchMessage::ResetCalibration),
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
