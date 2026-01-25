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
    // Selection indicator with theme-aware color
    let indicator = container(
        text(if is_selected { "●" } else { "○" })
            .size(12)
            .width(Length::Fixed(20.0))
    )
    .style(move |theme: &iced::Theme| {
        let color = if is_selected {
            theme.palette().primary
        } else {
            let txt = theme.palette().text;
            iced::Color { a: 0.5, ..txt }
        };
        container::Style {
            text_color: Some(color),
            ..Default::default()
        }
    });

    // Label with theme-aware color based on selection
    let label_container = container(text(label).size(14))
        .style(move |theme: &iced::Theme| {
            let txt = theme.palette().text;
            let color = if is_selected {
                txt
            } else {
                iced::Color { a: 0.8, ..txt }
            };
            container::Style {
                text_color: Some(color),
                ..Default::default()
            }
        });

    let mut content = row![indicator, label_container]
        .spacing(8)
        .align_y(Alignment::Center);

    // Add badge if provided
    if let Some(badge_text) = badge {
        content = content.push(
            container(text(badge_text).size(11))
                .padding([2, 6])
                .style(|theme: &iced::Theme| {
                    let primary = theme.palette().primary;
                    container::Style {
                        background: Some(iced::Background::Color(
                            iced::Color { a: 0.3, ..primary }
                        )),
                        text_color: Some(theme.palette().text),
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
        .style(move |theme, status| {
            let primary = theme.palette().primary;
            let bg_base = theme.palette().background;

            let background = match (is_selected, status) {
                (true, button::Status::Hovered) => iced::Color { a: 0.5, ..primary },
                (true, button::Status::Pressed) => iced::Color { a: 0.6, ..primary },
                (true, _) => iced::Color { a: 0.4, ..primary },
                (false, button::Status::Hovered) => iced::Color { r: bg_base.r + 0.1, g: bg_base.g + 0.1, b: bg_base.b + 0.1, a: 0.5 },
                (false, button::Status::Pressed) => iced::Color { r: bg_base.r + 0.15, g: bg_base.g + 0.15, b: bg_base.b + 0.15, a: 0.5 },
                (false, _) => iced::Color::TRANSPARENT,
            };

            button::Style {
                background: Some(iced::Background::Color(background)),
                border: iced::Border::default(),
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .into()
}
