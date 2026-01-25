//! Expandable section widget - collapsible container with header
//!
//! Provides a consistent expandable/collapsible section UI pattern
//! with rotation animation for the arrow icon.

use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

/// Creates an expandable/collapsible section with a header and content
///
/// The section shows an arrow that points down when expanded and right when collapsed.
///
/// # Example
/// ```rust,ignore
/// expandable_section(
///     "Advanced Options",
///     self.advanced_expanded,
///     Message::ToggleAdvanced,
///     column![
///         text_input_row("Custom setting", "...", &value, on_change),
///         slider_row("Another setting", "...", value, min, max, "", on_change),
///     ],
/// )
/// ```
pub fn expandable_section<'a, Message: Clone + 'a>(
    title: &'a str,
    is_expanded: bool,
    on_toggle: Message,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let arrow = if is_expanded { "▼" } else { "▶" };

    let header = button(
        row![
            text(arrow)
                .size(14)
                .width(Length::Fixed(20.0)),
            // Title uses theme's default text color (no .color() call)
            text(title)
                .size(16),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    )
    .on_press(on_toggle)
    .padding(12)
    .style(|theme: &iced::Theme, status| {
        let bg_base = theme.palette().background;
        let base_color = match status {
            button::Status::Hovered => iced::Color { r: bg_base.r + 0.15, g: bg_base.g + 0.15, b: bg_base.b + 0.15, a: 0.5 },
            button::Status::Pressed => iced::Color { r: bg_base.r + 0.20, g: bg_base.g + 0.20, b: bg_base.b + 0.20, a: 0.5 },
            _ => iced::Color::TRANSPARENT,
        };

        button::Style {
            background: Some(iced::Background::Color(base_color)),
            border: iced::Border::default(),
            text_color: theme.palette().text,
            ..Default::default()
        }
    });

    let mut col = column![header];

    if is_expanded {
        col = col.push(
            container(content)
                .padding(12)
        );
    }

    col.into()
}
