//! Config Editor view
//!
//! Viewer and editor for generated KDL config files.
//! Read-only by default, with optional edit mode for advanced users.

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_editor, toggler};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::messages::{ConfigEditorMessage, Message};
use crate::theme::fonts;

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
    /// Whether edit mode is enabled
    pub edit_mode: bool,
    /// Whether there are unsaved changes
    pub has_unsaved_changes: bool,
}

/// Creates the config editor view
pub fn view<'a>(state: &'a ConfigEditorState, editor_content: &'a text_editor::Content) -> Element<'a, Message> {
    let mut content = column![
        page_title("Config Editor"),
        info_text(
            "View the generated KDL configuration files. \
             These files are automatically generated from your settings."
        ),
    ]
    .spacing(4);

    // Edit mode toggle with warning
    if state.edit_mode {
        // Warning banner when edit mode is enabled
        content = content.push(
            container(
                column![
                    row![
                        text("⚠ Edit Mode Active").size(14).color([0.95, 0.6, 0.2]),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text("Warning: Manual edits will be OVERWRITTEN when you change settings in the app. \
                          Only edit if you know what you're doing!")
                        .size(12)
                        .color([0.9, 0.7, 0.4]),
                ]
                .spacing(4)
            )
            .padding([12, 16])
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.5, 0.25, 0.1, 0.4,
                ))),
                border: iced::Border {
                    radius: 6.0.into(),
                    color: iced::Color::from_rgba(0.8, 0.4, 0.1, 0.6),
                    width: 1.0,
                },
                ..Default::default()
            }),
        );
    } else {
        // Info banner when in read-only mode
        content = content.push(
            container(
                text("Read-only mode. Enable edit mode below to make changes.")
                    .size(12)
                    .color([0.6, 0.7, 0.9]),
            )
            .padding([8, 12])
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.2, 0.3, 0.5, 0.3,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }

    // Edit mode toggle
    content = content.push(
        row![
            text("Enable Edit Mode").size(14),
            toggler(state.edit_mode)
                .on_toggle(|enabled| Message::ConfigEditor(ConfigEditorMessage::ToggleEditMode(enabled))),
            if state.has_unsaved_changes {
                text("(unsaved changes)").size(12).color([0.9, 0.6, 0.3])
            } else {
                text("").size(12)
            },
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding([8, 0]),
    );

    content = content.push(spacer(12.0));

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

        // Header with filename and action buttons
        let mut header_row = row![
            text(format!("Contents of {}", filename))
                .size(15)
                .color([0.8, 0.8, 0.8]),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        // Add save/discard buttons when in edit mode with changes
        if state.edit_mode && state.has_unsaved_changes {
            header_row = header_row.push(
                button(text("Save").size(12))
                    .padding([4, 12])
                    .on_press(Message::ConfigEditor(ConfigEditorMessage::SaveEdits))
                    .style(|_theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.2, 0.5, 0.3, 0.7),
                            button::Status::Pressed => iced::Color::from_rgba(0.3, 0.6, 0.4, 0.8),
                            _ => iced::Color::from_rgba(0.2, 0.4, 0.3, 0.5),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
            );
            header_row = header_row.push(
                button(text("Discard").size(12))
                    .padding([4, 12])
                    .on_press(Message::ConfigEditor(ConfigEditorMessage::DiscardEdits))
                    .style(|_theme, status| {
                        let bg = match status {
                            button::Status::Hovered => iced::Color::from_rgba(0.5, 0.2, 0.2, 0.7),
                            _ => iced::Color::from_rgba(0.4, 0.2, 0.2, 0.4),
                        };
                        button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: iced::Color::from_rgb(0.9, 0.7, 0.7),
                            border: iced::Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
            );
        }

        content = content.push(header_row);
        content = content.push(spacer(8.0));

        // Content area - either editable or read-only
        if state.edit_mode {
            // Multi-line text editor
            content = content.push(
                container(
                    text_editor(editor_content)
                        .on_action(|action| Message::ConfigEditor(ConfigEditorMessage::EditorAction(action)))
                        .font(fonts::MONO_FONT)
                        .size(12)
                        .padding(12)
                        .height(Length::Fixed(500.0))
                )
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.1, 0.1, 0.12,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: iced::Color::from_rgb(0.4, 0.35, 0.2),
                    },
                    ..Default::default()
                }),
            );
        } else {
            // Read-only display
            let content_display = match &state.file_content {
                Some(Ok(file_text)) => {
                    if file_text.is_empty() {
                        text("(empty file)")
                            .size(13)
                            .color([0.5, 0.5, 0.5])
                            .font(fonts::MONO_FONT)
                    } else {
                        text(file_text)
                            .size(12)
                            .font(fonts::MONO_FONT)
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
        }
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
