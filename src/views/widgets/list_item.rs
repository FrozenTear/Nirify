//! List item widget - selectable item for list views
//!
//! Provides a consistent list item UI with selection indicator

use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};

/// Creates a selectable list item with optional badge
///
/// # Example
/// ```rust,ignore
/// list_item(
///     "HDMI-1",
///     self.selected_output == Some(0),
///     Message::SelectOutput(0),
///     Some("enabled"),
/// )
/// ```
pub fn list_item<Message: Clone + 'static>(
    label: String,
    is_selected: bool,
    on_select: Message,
    badge: Option<String>,
) -> Element<'static, Message> {
    let mut content = row![
        // Selection indicator (bullet point)
        text(if is_selected { "●" } else { "○" })
            .size(12)
            .width(Length::Fixed(20.0))
            .color(if is_selected { [0.5, 0.7, 1.0] } else { [0.5, 0.5, 0.5] }),
        // Label
        text(label)
            .size(14)
            .color(if is_selected { [1.0, 1.0, 1.0] } else { [0.8, 0.8, 0.8] }),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    // Add badge if provided
    if let Some(badge_text) = badge {
        content = content.push(
            container(
                text(badge_text)
                    .size(11)
                    .color([0.9, 0.9, 0.9])
            )
            .padding([2, 6])
            .style(|_theme| {
                container::Style {
                    background: Some(iced::Background::Color(
                        iced::Color::from_rgba(0.3, 0.5, 0.7, 0.3)
                    )),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        );
    }

    button(content)
        .on_press(on_select)
        .padding([8, 12])
        .width(Length::Fill)
        .style(move |_theme, status| {
            let background = match (is_selected, status) {
                (true, button::Status::Hovered) => iced::Color::from_rgba(0.3, 0.4, 0.6, 0.5),
                (true, button::Status::Pressed) => iced::Color::from_rgba(0.4, 0.5, 0.7, 0.5),
                (true, _) => iced::Color::from_rgba(0.2, 0.3, 0.5, 0.4),
                (false, button::Status::Hovered) => iced::Color::from_rgba(0.25, 0.25, 0.25, 0.5),
                (false, button::Status::Pressed) => iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                (false, _) => iced::Color::TRANSPARENT,
            };

            button::Style {
                background: Some(iced::Background::Color(background)),
                border: iced::Border::default(),
                text_color: iced::Color::WHITE,
                ..Default::default()
            }
        })
        .into()
}
