//! Config Editor view
//!
//! Read-only viewer for generated KDL config files.
//! Shows the raw KDL content for debugging and verification.

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::messages::{ConfigEditorMessage, Message};

/// List of config files that can be viewed
pub const CONFIG_FILES: &[&str] = &[
    "main.kdl",
    "appearance.kdl",
    "behavior.kdl",
    "animations.kdl",
    "cursor.kdl",
    "keyboard.kdl",
    "mouse.kdl",
    "touchpad.kdl",
    "trackpoint.kdl",
    "trackball.kdl",
    "tablet.kdl",
    "touch.kdl",
    "outputs.kdl",
    "workspaces.kdl",
    "keybindings.kdl",
    "layout-extras.kdl",
    "gestures.kdl",
    "misc.kdl",
    "startup.kdl",
    "environment.kdl",
    "debug.kdl",
    "switch-events.kdl",
    "window-rules.kdl",
    "layer-rules.kdl",
    "recent-windows.kdl",
];

/// State for the config editor page
#[derive(Debug, Clone, Default)]
pub struct ConfigEditorState {
    /// Currently selected file index
    pub selected_file: Option<usize>,
    /// Content of the selected file (or error message)
    pub file_content: Option<Result<String, String>>,
    /// Whether content is loading
    pub loading: bool,
}

/// Creates the config editor view
pub fn view(state: &ConfigEditorState) -> Element<'_, Message> {
    let mut content = column![
        page_title("Config Editor"),
        info_text(
            "View the generated KDL configuration files. \
             These files are automatically generated from your settings and should not be edited manually."
        ),
    ]
    .spacing(4);

    // Info banner
    content = content.push(
        container(
            text("These files are read-only. Changes made here will be overwritten when settings are saved.")
                .size(12)
                .color([0.9, 0.7, 0.4]),
        )
        .padding([8, 12])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.4, 0.3, 0.1, 0.3,
            ))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    content = content.push(spacer(16.0));

    // File selector
    let file_names: Vec<&str> = CONFIG_FILES.to_vec();
    let selected_name = state.selected_file.map(|i| CONFIG_FILES[i]);

    let file_picker = pick_list(
        file_names,
        selected_name,
        |name| {
            let idx = CONFIG_FILES.iter().position(|&f| f == name).unwrap_or(0);
            Message::ConfigEditor(ConfigEditorMessage::SelectFile(idx))
        },
    )
    .placeholder("Select a file to view...")
    .width(Length::Fixed(250.0));

    let refresh_btn = button(
        text(if state.loading { "Loading..." } else { "Refresh" }).size(13),
    )
    .padding([6, 12])
    .on_press_maybe(
        if state.selected_file.is_some() && !state.loading {
            Some(Message::ConfigEditor(ConfigEditorMessage::Refresh))
        } else {
            None
        },
    );

    content = content.push(
        row![
            text("File:").size(14),
            file_picker,
            refresh_btn,
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    );

    content = content.push(spacer(16.0));

    // File content display
    if let Some(selected_idx) = state.selected_file {
        let filename = CONFIG_FILES[selected_idx];
        content = content.push(
            text(format!("Contents of {}", filename))
                .size(15)
                .color([0.8, 0.8, 0.8]),
        );

        content = content.push(spacer(8.0));

        let content_display = match &state.file_content {
            Some(Ok(file_text)) => {
                if file_text.is_empty() {
                    text("(empty file)")
                        .size(13)
                        .color([0.5, 0.5, 0.5])
                        .font(crate::theme::fonts::MONO_FONT)
                } else {
                    text(file_text)
                        .size(12)
                        .font(crate::theme::fonts::MONO_FONT)
                        .color([0.85, 0.85, 0.85])
                }
            }
            Some(Err(error)) => text(format!("Error: {}", error))
                .size(13)
                .color([0.9, 0.4, 0.4]),
            None => {
                if state.loading {
                    text("Loading...")
                        .size(13)
                        .color([0.6, 0.6, 0.6])
                } else {
                    text("Click Refresh to load file contents")
                        .size(13)
                        .color([0.5, 0.5, 0.5])
                }
            }
        };

        content = content.push(
            container(
                scrollable(
                    container(content_display)
                        .padding(12)
                        .width(Length::Fill),
                )
                .height(Length::Fixed(500.0)),
            )
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.12, 0.12, 0.14,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: iced::Color::from_rgb(0.25, 0.25, 0.28),
                },
                ..Default::default()
            }),
        );
    } else {
        content = content.push(
            container(
                text("Select a file from the dropdown to view its contents")
                    .size(14)
                    .color([0.5, 0.5, 0.5]),
            )
            .padding(40)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
        );
    }

    content = content.push(spacer(32.0));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}
