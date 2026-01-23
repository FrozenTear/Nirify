//! Cursor settings view

use iced::widget::{column, container, scrollable, text, text_input};
use iced::Element;

use super::widgets::*;
use crate::config::models::CursorSettings;
use crate::messages::{CursorMessage, Message};

/// Creates the cursor settings view
/// Takes reference to settings to allow text_input to borrow the theme string
pub fn view(settings: &CursorSettings) -> Element<'_, Message> {
    let content = column![
        page_title("Cursor Theme"),
        info_text(
            "Configure the cursor theme and size. The theme name should match a cursor theme installed on your system."
        ),
        column![
            text("Cursor theme").size(16),
            text("Name of the cursor theme (e.g., Adwaita, breeze_cursors)").size(12).color([0.7, 0.7, 0.7]),
            text_input("", &settings.theme)
                .on_input(|value| Message::Cursor(CursorMessage::SetTheme(value)))
                .padding(8),
        ]
        .spacing(6)
        .padding(12),
        section_header("Cursor Size"),
        info_text(
            "Set the cursor size in pixels. Common sizes are 24 (default) or 32 for larger displays."
        ),
        slider_row_int(
            "Size",
            "Cursor size in pixels",
            settings.size,
            16,
            48,
            " px",
            |value| Message::Cursor(CursorMessage::SetSize(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}
