//! Cursor settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::config::models::CursorSettings;
use crate::messages::{CursorMessage, Message};
use crate::theme::{fonts, neon};

/// Creates the cursor settings view
pub fn view(settings: &CursorSettings) -> Element<'_, Message> {
    let content = column![row![
        // Left: Theme
        column![
            modal_section("\u{25CE}", "CURSOR THEME", neon::PRIMARY),
            Space::new().height(4),
            styled_text_input(
                "THEME NAME",
                "e.g., Adwaita, breeze_cursors",
                &settings.theme,
                |v| Message::Cursor(CursorMessage::SetTheme(v)),
            ),
        ]
        .spacing(6)
        .width(Length::FillPortion(1)),
        // Right: Size
        column![
            modal_section("\u{25A6}", "CURSOR SIZE", neon::SECONDARY),
            Space::new().height(4),
            styled_slider_int(
                "SIZE",
                &format!("{} px", settings.size),
                16..=48,
                settings.size,
                |v| Message::Cursor(CursorMessage::SetSize(v)),
            ),
        ]
        .spacing(6)
        .width(Length::FillPortion(1)),
    ]
    .spacing(32)
    .align_y(Alignment::Start),]
    .spacing(0)
    .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────

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

fn styled_slider_int<'a>(
    label: &'a str,
    display_value: &str,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    let d = display_value.to_string();
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
            iced::widget::slider(range, value, on_slide).width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
