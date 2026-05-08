//! Backups view — neon modal style
//!
//! Browse and restore configuration backups.
//! Backups are created when the config is modified through the wizard.

use iced::widget::{button, column, container, row, scrollable, text, Column, Space};
use iced::{Alignment, Element, Length};

use crate::messages::{BackupEntry, BackupsMessage, Message};
use crate::theme::{fonts, neon};

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
    let content = column![
        // ── 2-COLUMN: ACTIONS | BACKUP LIST ──
        row![
            // Left: Create / refresh + info
            column![
                modal_section("\u{1F6E1}", "BACKUP ACTIONS", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![styled_button(
                        if state.loading_list {
                            "Loading..."
                        } else {
                            "Refresh List"
                        },
                        !state.loading_list,
                        Message::Backups(BackupsMessage::RefreshList),
                        neon::SECONDARY,
                    ),]
                    .spacing(6),
                )
                .padding(12)
                .style(crate::theme::card_style),
                // Status message
                if let Some(status) = &state.status_message {
                    let color = if status.contains("Error") || status.contains("Failed") {
                        neon::ERROR
                    } else {
                        neon::SECONDARY
                    };
                    container(text(status.as_str()).size(11).color(color)).padding([8, 12])
                } else {
                    container(Space::new().height(0))
                },
                Space::new().height(12),
                modal_section("\u{2139}", "INFO", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        text("Backups are stored at:")
                            .size(11)
                            .color(neon::ON_SURFACE_VARIANT),
                        text("~/.config/niri/.nirify-backups/")
                            .size(11)
                            .font(fonts::MONO_FONT)
                            .color(neon::SECONDARY),
                        Space::new().height(4),
                        text("They persist even if you delete the nirify settings folder.")
                            .size(11)
                            .color(neon::OUTLINE),
                    ]
                    .spacing(2)
                    .padding(4),
                )
                .padding(8)
                .style(crate::theme::card_style),
                // Selected backup actions
                if let Some(selected_idx) = state.selected_backup {
                    if let Some(backup) = state.backups.get(selected_idx) {
                        container(
                            column![
                                Space::new().height(12),
                                modal_section("\u{1F527}", "SELECTED BACKUP", neon::PRIMARY),
                                Space::new().height(4),
                                container(
                                    column![
                                        text("FILENAME")
                                            .size(10)
                                            .font(fonts::UI_FONT_SEMIBOLD)
                                            .color(neon::OUTLINE_VARIANT),
                                        text(&backup.filename)
                                            .size(12)
                                            .font(fonts::MONO_FONT)
                                            .color(neon::ON_SURFACE),
                                        Space::new().height(4),
                                        text("DATE")
                                            .size(10)
                                            .font(fonts::UI_FONT_SEMIBOLD)
                                            .color(neon::OUTLINE_VARIANT),
                                        text(&backup.date).size(12).color(neon::ON_SURFACE_VARIANT),
                                        Space::new().height(4),
                                        text("SIZE")
                                            .size(10)
                                            .font(fonts::UI_FONT_SEMIBOLD)
                                            .color(neon::OUTLINE_VARIANT),
                                        text(&backup.size).size(12).color(neon::ON_SURFACE_VARIANT),
                                        Space::new().height(8),
                                        styled_button(
                                            if state.loading_preview {
                                                "Loading..."
                                            } else {
                                                "Load Preview"
                                            },
                                            !state.loading_preview,
                                            Message::Backups(BackupsMessage::SelectBackup(
                                                selected_idx
                                            )),
                                            neon::SECONDARY,
                                        ),
                                        styled_button(
                                            if state.restoring {
                                                "Restoring..."
                                            } else {
                                                "Restore Backup"
                                            },
                                            !state.restoring,
                                            Message::Backups(BackupsMessage::ConfirmRestore(
                                                selected_idx
                                            )),
                                            neon::TERTIARY,
                                        ),
                                    ]
                                    .spacing(2),
                                )
                                .padding(12)
                                .style(crate::theme::card_style),
                            ]
                            .spacing(0),
                        )
                    } else {
                        container(Space::new().height(0))
                    }
                } else {
                    container(Space::new().height(0))
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Backup list + preview
            column![
                modal_section("\u{1F4CB}", "AVAILABLE BACKUPS", neon::PRIMARY),
                text(format!("{} backup(s) found", state.backups.len()))
                    .size(11)
                    .color(neon::OUTLINE),
                Space::new().height(4),
                if state.backups.is_empty() {
                    container(
                        text("No backups found. Backups are created when you first set up Nirify.")
                            .size(12)
                            .color(neon::OUTLINE),
                    )
                    .padding(20)
                    .width(Length::Fill)
                    .style(crate::theme::card_style)
                } else {
                    let mut list = Column::new().spacing(4);
                    for (idx, backup) in state.backups.iter().enumerate() {
                        let is_selected = state.selected_backup == Some(idx);
                        list = list.push(backup_list_item(idx, backup, is_selected));
                    }
                    container(
                        scrollable(list)
                            .height(Length::Fixed(220.0))
                            .width(Length::Fill),
                    )
                    .padding(8)
                    .style(crate::theme::card_style)
                },
                Space::new().height(12),
                // Preview area
                modal_section("\u{1F50D}", "PREVIEW", neon::TERTIARY),
                Space::new().height(4),
                {
                    let preview_display = if state.selected_backup.is_some() {
                        match &state.preview_content {
                            Some(Ok(file_text)) => {
                                if file_text.is_empty() {
                                    text("(empty backup)")
                                        .size(12)
                                        .color(neon::OUTLINE)
                                        .font(fonts::MONO_FONT)
                                } else {
                                    text(file_text)
                                        .size(11)
                                        .font(fonts::MONO_FONT)
                                        .color(neon::ON_SURFACE)
                                }
                            }
                            Some(Err(error)) => text(format!("Error: {}", error))
                                .size(12)
                                .color(neon::ERROR),
                            None => {
                                if state.loading_preview {
                                    text("Loading preview...").size(12).color(neon::OUTLINE)
                                } else {
                                    text("Select a backup and click 'Load Preview'")
                                        .size(12)
                                        .color(neon::OUTLINE)
                                }
                            }
                        }
                    } else {
                        text("Select a backup to preview its contents")
                            .size(12)
                            .color(neon::OUTLINE)
                    };
                    container(
                        scrollable(container(preview_display).padding(12).width(Length::Fill))
                            .height(Length::Fixed(300.0)),
                    )
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(neon::SURFACE_LOW)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            width: 1.0,
                            color: iced::Color {
                                a: 0.2,
                                ..neon::OUTLINE
                            },
                        },
                        ..Default::default()
                    })
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(2)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
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

fn styled_button<'a>(
    label: &'a str,
    enabled: bool,
    message: Message,
    accent: iced::Color,
) -> Element<'a, Message> {
    let mut btn = button(text(label).size(12).font(fonts::UI_FONT_SEMIBOLD))
        .padding([6, 16])
        .width(Length::Fill)
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => iced::Color { a: 0.35, ..accent },
                button::Status::Pressed => iced::Color { a: 0.45, ..accent },
                _ => iced::Color { a: 0.2, ..accent },
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: accent,
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

    if enabled {
        btn = btn.on_press(message);
    }

    btn.into()
}

fn backup_list_item<'a>(
    idx: usize,
    backup: &'a BackupEntry,
    is_selected: bool,
) -> Element<'a, Message> {
    let accent = if is_selected {
        neon::PRIMARY
    } else {
        neon::SURFACE_CONTAINER_HIGHEST
    };

    button(
        container(
            row![
                column![
                    text(&backup.filename)
                        .size(12)
                        .font(fonts::UI_FONT_SEMIBOLD)
                        .color(if is_selected {
                            neon::PRIMARY
                        } else {
                            neon::ON_SURFACE
                        }),
                    text(&backup.date).size(10).color(neon::OUTLINE),
                ]
                .spacing(2)
                .width(Length::Fill),
                text(&backup.size)
                    .size(11)
                    .font(fonts::MONO_FONT)
                    .color(neon::OUTLINE),
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
    .style(move |_theme, status| {
        let bg = match status {
            button::Status::Hovered => iced::Color {
                a: 0.15,
                ..neon::PRIMARY
            },
            _ => {
                if is_selected {
                    iced::Color { a: 0.1, ..accent }
                } else {
                    neon::SURFACE_CONTAINER
                }
            }
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                radius: 6.0.into(),
                width: if is_selected { 1.0 } else { 0.0 },
                color: iced::Color {
                    a: 0.3,
                    ..neon::PRIMARY
                },
            },
            text_color: neon::ON_SURFACE,
            ..Default::default()
        }
    })
    .into()
}
