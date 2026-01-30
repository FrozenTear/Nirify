//! Backups view
//!
//! Browse and restore configuration backups.
//! Backups are created when the config is modified through the wizard.

use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::messages::{BackupEntry, BackupsMessage, Message};

/// State for the backups page
#[derive(Debug, Clone, Default)]
pub struct BackupsState {
    /// List of available backups
    pub backups: Vec<BackupEntry>,
    /// Currently selected backup index
    pub selected_backup: Option<usize>,
    /// Preview content for selected backup
    pub preview_content: Option<Result<String, String>>,
    /// Status message (success/error feedback)
    pub status_message: Option<String>,
    /// Whether the list is loading
    pub loading_list: bool,
    /// Whether preview is loading
    pub loading_preview: bool,
    /// Whether restore is in progress
    pub restoring: bool,
}

/// Creates the backups view
pub fn view(state: &BackupsState) -> Element<'_, Message> {
    let mut content = column![
        page_title("Backups"),
        info_text(
            "Manage configuration backups. Backups are automatically created \
             when you set up Nirify or make significant changes."
        ),
    ]
    .spacing(4);

    // Info banner
    content = content.push(
        container(
            text("Backups are stored at ~/.config/niri/.nirify-backups/ and persist even if you delete the nirify settings folder.")
                .size(12)
                .color([0.7, 0.85, 0.7]),
        )
        .padding([8, 12])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.2, 0.35, 0.2, 0.3,
            ))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    content = content.push(spacer(16.0));

    // Refresh button and status
    let refresh_label = if state.loading_list {
        "Loading..."
    } else {
        "Refresh List"
    };

    let mut refresh_btn = button(text(refresh_label).size(13)).padding([6, 12]);
    if !state.loading_list {
        refresh_btn = refresh_btn.on_press(Message::Backups(BackupsMessage::RefreshList));
    }

    let mut action_row = row![refresh_btn].spacing(12);

    if let Some(status) = &state.status_message {
        action_row = action_row.push(
            text(status)
                .size(13)
                .color(if status.contains("Error") || status.contains("Failed") {
                    [0.9, 0.4, 0.4]
                } else {
                    [0.5, 0.8, 0.5]
                }),
        );
    }

    content = content.push(action_row);

    content = content.push(spacer(16.0));

    // Backup list
    content = content.push(
        text(format!("Available Backups ({})", state.backups.len()))
            .size(15)
            .color([0.8, 0.8, 0.8]),
    );

    content = content.push(spacer(8.0));

    if state.backups.is_empty() {
        content = content.push(
            container(
                text("No backups found. Backups are created when you first set up Nirify.")
                    .size(13)
                    .color([0.5, 0.5, 0.5]),
            )
            .padding(20)
            .width(Length::Fill),
        );
    } else {
        let mut list = Column::new().spacing(4);

        for (idx, backup) in state.backups.iter().enumerate() {
            let is_selected = state.selected_backup == Some(idx);

            let row_color = if is_selected {
                [0.25, 0.35, 0.45]
            } else {
                [0.18, 0.18, 0.2]
            };

            let backup_row = button(
                container(
                    row![
                        column![
                            text(&backup.filename).size(13),
                            text(&backup.date).size(11).color([0.6, 0.6, 0.6]),
                        ]
                        .spacing(2)
                        .width(Length::Fill),
                        text(&backup.size).size(12).color([0.5, 0.5, 0.5]),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
                )
                .padding([8, 12])
                .width(Length::Fill),
            )
            .on_press(Message::Backups(BackupsMessage::SelectBackup(idx)))
            .padding(0)
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    row_color[0],
                    row_color[1],
                    row_color[2],
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                text_color: iced::Color::WHITE,
                ..Default::default()
            });

            list = list.push(backup_row);
        }

        content = content.push(
            scrollable(list)
                .height(Length::Fixed(200.0))
                .width(Length::Fill),
        );
    }

    content = content.push(spacer(16.0));

    // Preview and restore section (only if a backup is selected)
    if let Some(selected_idx) = state.selected_backup {
        if let Some(backup) = state.backups.get(selected_idx) {
            content = content.push(
                text(format!("Preview: {}", backup.filename))
                    .size(15)
                    .color([0.8, 0.8, 0.8]),
            );

            content = content.push(spacer(8.0));

            // Preview/Restore buttons
            let preview_label = if state.loading_preview {
                "Loading..."
            } else {
                "Load Preview"
            };

            let restore_label = if state.restoring {
                "Restoring..."
            } else {
                "Restore Backup"
            };

            let mut preview_btn = button(text(preview_label).size(13)).padding([6, 12]);
            if !state.loading_preview {
                preview_btn = preview_btn.on_press(Message::Backups(BackupsMessage::SelectBackup(selected_idx)));
            }

            let mut restore_btn = button(
                text(restore_label).size(13).color([0.9, 0.6, 0.4]),
            )
            .padding([6, 12]);
            if !state.restoring {
                restore_btn = restore_btn.on_press(Message::Backups(BackupsMessage::ConfirmRestore(selected_idx)));
            }

            content = content.push(
                row![preview_btn, restore_btn]
                    .spacing(12),
            );

            content = content.push(spacer(8.0));

            // Preview content
            let preview_display = match &state.preview_content {
                Some(Ok(file_text)) => {
                    if file_text.is_empty() {
                        text("(empty backup)")
                            .size(13)
                            .color([0.5, 0.5, 0.5])
                            .font(crate::theme::fonts::MONO_FONT)
                    } else {
                        text(file_text)
                            .size(11)
                            .font(crate::theme::fonts::MONO_FONT)
                            .color([0.8, 0.8, 0.8])
                    }
                }
                Some(Err(error)) => text(format!("Error: {}", error))
                    .size(13)
                    .color([0.9, 0.4, 0.4]),
                None => {
                    if state.loading_preview {
                        text("Loading preview...")
                            .size(13)
                            .color([0.6, 0.6, 0.6])
                    } else {
                        text("Click 'Load Preview' to view backup contents")
                            .size(13)
                            .color([0.5, 0.5, 0.5])
                    }
                }
            };

            content = content.push(
                container(
                    scrollable(
                        container(preview_display)
                            .padding(12)
                            .width(Length::Fill),
                    )
                    .height(Length::Fixed(300.0)),
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
    }

    content = content.push(spacer(32.0));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}
