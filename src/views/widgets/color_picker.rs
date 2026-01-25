//! Color picker widget for hex color input
//!
//! Provides a text input field with color preview and optional preset swatches.

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Border, Color as IcedColor, Element, Length};

use crate::types::Color;
use crate::theme::{fonts, muted_text_container};

/// Creates a color picker row with hex input and color preview
///
/// # Example
/// ```rust,ignore
/// color_picker_row(
///     "Focus ring color",
///     "Color of the focus ring around focused windows",
///     &settings.focus_ring_color,
///     |hex| Message::Appearance(AppearanceMessage::SetFocusRingColor(hex)),
/// )
/// ```
pub fn color_picker_row<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    color: &Color,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let hex_value = color.to_hex();

    // Convert Color to iced::Color for preview
    let preview_color = IcedColor::from_rgb8(color.r, color.g, color.b);

    // Color preview box
    let preview = container(text(""))
        .width(Length::Fixed(40.0))
        .height(Length::Fixed(40.0))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(preview_color)),
            border: Border {
                color: IcedColor::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    // Hex input field (text_input accepts &str directly, no need to leak)
    let hex_input = text_input("", &hex_value)
        .on_input(on_change)
        .padding(8)
        .width(Length::Fixed(100.0))
        .font(fonts::MONO_FONT);

    row![
        // Left side: Label and description
        column![
            text(label).size(16),
            container(text(description).size(12)).style(muted_text_container),
        ]
        .spacing(4)
        .width(Length::Fill),
        // Right side: Color preview and hex input
        row![preview, hex_input]
            .spacing(8)
            .align_y(Alignment::Center),
    ]
    .spacing(20)
    .padding(12)
    .align_y(Alignment::Center)
    .into()
}

/// Common color swatches for quick selection
pub const COMMON_COLORS: &[(&str, &str)] = &[
    ("Blue", "#7fc8ff"),
    ("Red", "#ff5555"),
    ("Green", "#50fa7b"),
    ("Yellow", "#f1fa8c"),
    ("Purple", "#bd93f9"),
    ("Cyan", "#8be9fd"),
    ("Orange", "#ffb86c"),
    ("Pink", "#ff79c6"),
    ("White", "#ffffff"),
    ("Gray", "#6272a4"),
];

/// Creates a color picker with preset swatches
pub fn color_picker_with_swatches<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    color: &Color,
    on_change: impl Fn(String) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    let hex_value = color.to_hex();

    // Convert Color to iced::Color for preview
    let preview_color = IcedColor::from_rgb8(color.r, color.g, color.b);

    // Color preview box
    let preview = container(text(""))
        .width(Length::Fixed(40.0))
        .height(Length::Fixed(40.0))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(preview_color)),
            border: Border {
                color: IcedColor::from_rgb(0.4, 0.4, 0.4),
                width: 2.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    // Hex input field (text_input accepts &str directly, no need to leak)
    let hex_input = text_input("", &hex_value)
        .on_input(on_change)
        .padding(8)
        .width(Length::Fixed(100.0))
        .font(fonts::MONO_FONT);

    // Color swatches
    let mut swatches = row![].spacing(4);
    for (_name, hex) in COMMON_COLORS.iter().take(8) {
        let swatch_color = Color::from_hex(hex).unwrap_or_default();
        let swatch_iced = IcedColor::from_rgb8(swatch_color.r, swatch_color.g, swatch_color.b);

        let swatch_button = button(text(""))
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(swatch_iced)),
                border: Border {
                    color: IcedColor::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            })
            .on_press(on_change(hex.to_string()));

        swatches = swatches.push(swatch_button);
    }

    column![
        row![
            // Left side: Label and description
            column![
                text(label).size(16),
                container(text(description).size(12)).style(muted_text_container),
            ]
            .spacing(4)
            .width(Length::Fill),
            // Right side: Color preview and hex input
            row![preview, hex_input]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .spacing(20)
        .align_y(Alignment::Center),
        // Swatches row
        container(swatches).padding([8.0, 12.0]),
    ]
    .spacing(8)
    .padding(12)
    .into()
}
