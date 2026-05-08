//! Touch settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{calibration_matrix, toggle_row, CalibrationMatrixMessage};
use crate::config::models::TouchSettings;
use crate::messages::{Message, TouchMessage};
use crate::theme::{fonts, neon};

pub fn view<'a>(
    settings: &'a TouchSettings,
    calibration_cache: &'a [String; 6],
) -> Element<'a, Message> {
    let map_to_output = settings.map_to_output.clone();

    let content = column![
        // -- 2-COLUMN: MAPPING | CONFIGURATION --
        row![
            // Left column: Output Mapping
            column![
                modal_section("\u{25f0}", "OUTPUT MAPPING", neon::SECONDARY),
                Space::new().height(4),
                styled_text_input(
                    "MAP TO OUTPUT",
                    "e.g., eDP-1, HDMI-A-1",
                    &map_to_output,
                    |v| Message::Touch(TouchMessage::SetMapToOutput(v))
                ),
                Space::new().height(12),
                modal_section("\u{2699}", "CALIBRATION", neon::TERTIARY),
                Space::new().height(4),
                container(calibration_matrix(
                    "Calibration Matrix",
                    "Transform touch coordinates to correct misaligned input",
                    settings.calibration_matrix,
                    calibration_cache,
                    |msg| match msg {
                        CalibrationMatrixMessage::SetValue(idx, val) =>
                            Message::Touch(TouchMessage::SetCalibrationValue(idx, val)),
                        CalibrationMatrixMessage::Clear =>
                            Message::Touch(TouchMessage::ClearCalibration),
                        CalibrationMatrixMessage::Reset =>
                            Message::Touch(TouchMessage::ResetCalibration),
                    },
                ),)
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right column: Device toggle
            column![
                modal_section("\u{25e7}", "DEVICE", neon::PRIMARY),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Disable touch",
                        "Completely disable this device",
                        settings.off,
                        |v| Message::Touch(TouchMessage::SetOff(v))
                    ),]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
    .spacing(0)
    .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

// -- Helpers --

fn modal_section<'a>(icon: &'a str, label: &'a str, accent: iced::Color) -> Element<'a, Message> {
    row![
        text(icon).size(14).color(accent),
        Space::new().width(6),
        text(label)
            .size(11)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(accent),
        Space::new().width(12),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.25, ..accent })),
                ..Default::default()
            }),
    ]
    .spacing(0)
    .align_y(Alignment::Center)
    .padding([14, 0])
    .into()
}

fn styled_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let value_owned = value.to_string();
    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text_input(placeholder, &value_owned)
                .on_input(on_change)
                .padding(10)
                .size(13),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
