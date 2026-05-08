//! Trackball settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{picker_row, toggle_row};
use crate::config::models::TrackballSettings;
use crate::messages::{Message, TrackballMessage};
use crate::theme::{fonts, neon};
use crate::types::{AccelProfile, ScrollMethod};

pub fn view(settings: &TrackballSettings) -> Element<'_, Message> {
    let content = column![
        // -- 2-COLUMN: SCROLLING | ACCELERATION --
        row![
            // Left column: Scrolling + Device
            column![
                modal_section("\u{25ce}", "SCROLLING", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Natural scroll",
                            "Reverse scroll direction",
                            settings.natural_scroll,
                            |v| Message::Trackball(TrackballMessage::SetNaturalScroll(v))
                        ),
                        toggle_row(
                            "Scroll button lock",
                            "Lock scroll state",
                            settings.scroll_button_lock,
                            |v| Message::Trackball(TrackballMessage::SetScrollButtonLock(v))
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(8),
                picker_row(
                    "Scroll method",
                    "How scrolling is performed",
                    ScrollMethod::all(),
                    Some(settings.scroll_method),
                    |v| Message::Trackball(TrackballMessage::SetScrollMethod(v))
                ),
                Space::new().height(8),
                scroll_button_input(settings.scroll_button, |v| Message::Trackball(
                    TrackballMessage::SetScrollButton(v)
                )),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right column: Acceleration + Buttons
            column![
                modal_section("\u{26a1}", "ACCELERATION", neon::PRIMARY),
                Space::new().height(4),
                styled_slider(
                    "ACCEL SPEED",
                    &format!("{:.2}", settings.accel_speed),
                    -1.0..=1.0,
                    settings.accel_speed as f32,
                    0.01,
                    |v| Message::Trackball(TrackballMessage::SetAccelSpeed(v))
                ),
                picker_row(
                    "Accel profile",
                    "Adaptive or flat acceleration",
                    AccelProfile::all(),
                    Some(settings.accel_profile),
                    |v| Message::Trackball(TrackballMessage::SetAccelProfile(v))
                ),
                Space::new().height(12),
                modal_section("\u{25e7}", "BUTTONS", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Left-handed mode",
                            "Swap left and right buttons",
                            settings.left_handed,
                            |v| Message::Trackball(TrackballMessage::SetLeftHanded(v))
                        ),
                        toggle_row(
                            "Middle emulation",
                            "Left+right = middle click",
                            settings.middle_emulation,
                            |v| Message::Trackball(TrackballMessage::SetMiddleEmulation(v))
                        ),
                        toggle_row(
                            "Disable trackball",
                            "Completely disable this device",
                            settings.off,
                            |v| Message::Trackball(TrackballMessage::SetOff(v))
                        ),
                    ]
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

fn styled_slider<'a>(
    label: &'a str,
    display: &str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_slide: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    let d = display.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(d)
                    .size(11)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide)
                .step(step)
                .width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

fn scroll_button_input<'a>(
    value: Option<i32>,
    on_change: impl Fn(Option<i32>) -> Message + 'a,
) -> Element<'a, Message> {
    let display_value = value.map(|v| v.to_string()).unwrap_or_default();
    container(
        column![
            text("SCROLL BUTTON")
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text("Linux button code for on-button-down scrolling (e.g., 274)")
                .size(11)
                .color(neon::OUTLINE_VARIANT),
            text_input("e.g., 274", &display_value)
                .on_input(move |s| {
                    if s.is_empty() {
                        on_change(None)
                    } else if let Ok(v) = s.parse::<i32>() {
                        on_change(Some(v))
                    } else {
                        on_change(value)
                    }
                })
                .padding(10)
                .size(13),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
