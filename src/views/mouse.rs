//! Mouse settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{info_text, picker_row, toggle_row};
use crate::config::models::MouseSettings;
use crate::messages::{Message, MouseMessage};
use crate::theme::{fonts, neon};
use crate::types::{AccelProfile, ScrollMethod};

pub fn view(settings: &MouseSettings) -> Element<'_, Message> {
    let content = column![
        // ── 2-COLUMN: SCROLLING | ACCELERATION ──
        row![
            column![
                modal_section("◎", "SCROLLING", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Natural scroll",
                            "Reverse scroll direction",
                            settings.natural_scroll,
                            |v| Message::Mouse(MouseMessage::ToggleNaturalScroll(v))
                        ),
                        toggle_row(
                            "Scroll button lock",
                            "Lock scroll state",
                            settings.scroll_button_lock,
                            |v| Message::Mouse(MouseMessage::ToggleScrollButtonLock(v))
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(8),
                styled_slider(
                    "SCROLL FACTOR",
                    &format!("{:.1}x", settings.scroll_factor),
                    0.1..=10.0,
                    settings.scroll_factor as f32,
                    0.1,
                    |v| Message::Mouse(MouseMessage::SetScrollFactor(v))
                ),
                styled_slider(
                    "HORIZ SCROLL",
                    &format!(
                        "{:.1}x",
                        settings
                            .scroll_factor_horizontal
                            .unwrap_or(settings.scroll_factor) as f32
                    ),
                    0.1..=10.0,
                    settings
                        .scroll_factor_horizontal
                        .unwrap_or(settings.scroll_factor) as f32,
                    0.1,
                    |v| Message::Mouse(MouseMessage::SetScrollFactorHorizontal(Some(v)))
                ),
                picker_row(
                    "Scroll method",
                    "How scrolling is performed",
                    ScrollMethod::all(),
                    Some(settings.scroll_method),
                    |v| Message::Mouse(MouseMessage::SetScrollMethod(v))
                ),
                {
                    let sb_display = settings
                        .scroll_button
                        .map(|v| v.to_string())
                        .unwrap_or_default();
                    styled_text_input(
                        "SCROLL BUTTON",
                        "Button code (e.g., 274)",
                        &sb_display,
                        |s| {
                            if s.is_empty() {
                                Message::Mouse(MouseMessage::SetScrollButton(None))
                            } else if let Ok(v) = s.parse::<i32>() {
                                Message::Mouse(MouseMessage::SetScrollButton(Some(v)))
                            } else {
                                Message::NoOp
                            }
                        },
                    )
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                modal_section("⚡", "ACCELERATION", neon::PRIMARY),
                Space::new().height(4),
                styled_slider(
                    "ACCEL SPEED",
                    &format!("{:.2}", settings.accel_speed),
                    -1.0..=1.0,
                    settings.accel_speed as f32,
                    0.01,
                    |v| Message::Mouse(MouseMessage::SetAccelSpeed(v))
                ),
                picker_row(
                    "Accel profile",
                    "Adaptive or flat acceleration",
                    AccelProfile::all(),
                    Some(settings.accel_profile),
                    |v| Message::Mouse(MouseMessage::SetAccelProfile(v))
                ),
                Space::new().height(12),
                modal_section("◧", "BUTTONS", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Left-handed mode",
                            "Swap left and right buttons",
                            settings.left_handed,
                            |v| Message::Mouse(MouseMessage::ToggleLeftHanded(v))
                        ),
                        toggle_row(
                            "Middle emulation",
                            "Left+right = middle click",
                            settings.middle_emulation,
                            |v| Message::Mouse(MouseMessage::ToggleMiddleEmulation(v))
                        ),
                        toggle_row(
                            "Disable on touchpad",
                            "Disable when touchpad active",
                            settings.off,
                            |v| Message::Mouse(MouseMessage::ToggleOffOnTouchpad(v))
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

fn styled_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let v = value.to_string();
    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text_input(placeholder, &v)
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
