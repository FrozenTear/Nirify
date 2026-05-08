//! Touchpad settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{picker_row, toggle_row};
use crate::config::models::TouchpadSettings;
use crate::messages::{Message, TouchpadMessage};
use crate::theme::{fonts, neon};
use crate::types::{AccelProfile, ClickMethod, ScrollMethod, TapButtonMap};

pub fn view(settings: &TouchpadSettings) -> Element<'_, Message> {
    let content = column![
        // ── ROW 1: TAP & BEHAVIOR | SCROLLING ──
        row![
            column![
                modal_section("▦", "TAP & BEHAVIOR", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Tap to click",
                            "Tap touchpad to register clicks",
                            settings.tap,
                            |v| Message::Touchpad(TouchpadMessage::ToggleTapToClick(v))
                        ),
                        toggle_row(
                            "Disable while typing",
                            "DWT — prevent accidental input",
                            settings.dwt,
                            |v| Message::Touchpad(TouchpadMessage::ToggleDwt(v))
                        ),
                        toggle_row(
                            "Disable while trackpoint",
                            "DWTP — disable on trackpoint use",
                            settings.dwtp,
                            |v| Message::Touchpad(TouchpadMessage::ToggleDwtp(v))
                        ),
                        toggle_row("Drag", "Tap-and-drag gesture", settings.drag, |v| {
                            Message::Touchpad(TouchpadMessage::ToggleDrag(v))
                        }),
                        toggle_row(
                            "Drag lock",
                            "Lock drag until tapped again",
                            settings.drag_lock,
                            |v| Message::Touchpad(TouchpadMessage::ToggleDragLock(v))
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                modal_section("◎", "SCROLLING", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Natural scroll",
                        "Reverse direction (macOS-style)",
                        settings.natural_scroll,
                        |v| Message::Touchpad(TouchpadMessage::ToggleNaturalScroll(v))
                    ),]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(4),
                styled_slider(
                    "SCROLL FACTOR",
                    &format!("{:.1}x", settings.scroll_factor),
                    0.1..=10.0,
                    settings.scroll_factor as f32,
                    0.1,
                    |v| Message::Touchpad(TouchpadMessage::SetScrollFactor(v))
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
                    |v| Message::Touchpad(TouchpadMessage::SetScrollFactorHorizontal(Some(v)))
                ),
                picker_row(
                    "Scroll method",
                    "Two-finger, edge, or button",
                    ScrollMethod::all(),
                    Some(settings.scroll_method),
                    |v| Message::Touchpad(TouchpadMessage::SetScrollMethod(v))
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(20),
        // ── ROW 2: ACCELERATION | BUTTONS ──
        row![
            column![
                modal_section("⚡", "ACCELERATION", neon::PRIMARY),
                Space::new().height(4),
                styled_slider(
                    "ACCEL SPEED",
                    &format!("{:.2}", settings.accel_speed),
                    -1.0..=1.0,
                    settings.accel_speed as f32,
                    0.01,
                    |v| Message::Touchpad(TouchpadMessage::SetAccelSpeed(v))
                ),
                picker_row(
                    "Accel profile",
                    "Adaptive or flat",
                    AccelProfile::all(),
                    Some(settings.accel_profile),
                    |v| Message::Touchpad(TouchpadMessage::SetAccelProfile(v))
                ),
                picker_row(
                    "Click method",
                    "Button areas or clickfinger",
                    ClickMethod::all(),
                    Some(settings.click_method),
                    |v| Message::Touchpad(TouchpadMessage::SetClickMethod(v))
                ),
                picker_row(
                    "Tap button map",
                    "2/3-finger tap mapping",
                    TapButtonMap::all(),
                    Some(settings.tap_button_map),
                    |v| Message::Touchpad(TouchpadMessage::SetTapButtonMap(v))
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                modal_section("◧", "BUTTONS & MODE", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Left-handed mode",
                            "Swap button areas",
                            settings.left_handed,
                            |v| Message::Touchpad(TouchpadMessage::ToggleLeftHanded(v))
                        ),
                        toggle_row(
                            "Middle emulation",
                            "Two-finger tap = middle",
                            settings.middle_emulation,
                            |v| Message::Touchpad(TouchpadMessage::ToggleMiddleEmulation(v))
                        ),
                        toggle_row(
                            "Disable on ext. mouse",
                            "Auto-disable with external mouse",
                            settings.disabled_on_external_mouse,
                            |v| Message::Touchpad(TouchpadMessage::ToggleDisabledOnExternalMouse(
                                v
                            ))
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                {
                    let sb = settings
                        .scroll_button
                        .map(|v| v.to_string())
                        .unwrap_or_default();
                    styled_text_input("SCROLL BUTTON", "Button code (e.g., 274)", &sb, |s| {
                        if s.is_empty() {
                            Message::Touchpad(TouchpadMessage::SetScrollButton(None))
                        } else if let Ok(v) = s.parse::<i32>() {
                            Message::Touchpad(TouchpadMessage::SetScrollButton(Some(v)))
                        } else {
                            Message::NoOp
                        }
                    })
                },
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

// Keep text import for use by toggle_row/picker_row
use iced::widget::text;
