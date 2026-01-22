//! Cursor settings view

use iced::widget::{column, container, scrollable, text, text_input};
use iced::Element;

use super::widgets::*;
use crate::config::models::CursorSettings;
use crate::messages::{CursorMessage, Message};

/// Helper to create a text input row with owned string
/// Note: This leaks the string to get a 'static lifetime
fn text_input_row_owned<Message: Clone + 'static>(
    label: &'static str,
    description: &'static str,
    value: String,
    on_change: impl Fn(String) -> Message + 'static,
) -> Element<'static, Message> {
    let value_static: &'static str = Box::leak(value.into_boxed_str());
    column![
        text(label).size(16),
        text(description).size(12).color([0.7, 0.7, 0.7]),
        text_input("", value_static).on_input(on_change).padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Creates the cursor settings view
pub fn view(settings: CursorSettings) -> Element<'static, Message> {
    let theme = settings.theme;
    let size = settings.size;

    let content = column![
        section_header("Cursor Theme"),
        info_text(
            "Configure the cursor theme and size. The theme name should match a cursor theme installed on your system."
        ),
        text_input_row_owned(
            "Cursor theme",
            "Name of the cursor theme (e.g., Adwaita, breeze_cursors)",
            theme,
            |value| Message::Cursor(CursorMessage::SetTheme(value)),
        ),
        spacer(16.0),

        section_header("Cursor Size"),
        info_text(
            "Set the cursor size in pixels. Common sizes are 24 (default) or 32 for larger displays."
        ),
        slider_row_int(
            "Size",
            "Cursor size in pixels",
            size,
            16,
            48,
            " px",
            |value| Message::Cursor(CursorMessage::SetSize(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
