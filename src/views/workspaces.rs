//! Workspaces settings view
//!
//! Provides an interface for managing named workspaces.

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::WorkspacesSettings;
use crate::messages::{Message, WorkspacesMessage};

/// Creates the workspaces settings view
pub fn view(settings: &WorkspacesSettings) -> Element<'static, Message> {
    let mut content = column![
        section_header("Named Workspaces"),
        info_text(
            "Define named workspaces that persist across sessions. \
             Workspaces can be pinned to specific outputs."
        ),
        spacer(8.0),
    ]
    .spacing(8);

    // List of workspaces
    if settings.workspaces.is_empty() {
        content = content.push(
            container(
                column![
                    text("No named workspaces defined")
                        .size(14)
                        .color([0.75, 0.75, 0.75]),
                    spacer(8.0),
                    text("Click the button below to add your first workspace")
                        .size(13)
                        .color([0.5, 0.5, 0.5]),
                ]
                .align_x(Alignment::Center)
            )
            .padding(24)
            .center(Length::Fill)
        );
    } else {
        for (idx, workspace) in settings.workspaces.iter().enumerate() {
            let ws_name = workspace.name.clone();
            let ws_output = workspace.open_on_output.clone().unwrap_or_default();

            content = content.push(
                container(
                    column![
                        row![
                            text(format!("Workspace {}", idx + 1))
                                .size(14)
                                .color([0.7, 0.7, 0.7]),
                            row![
                                // Move up button
                                if idx > 0 {
                                    button(text("↑").size(14))
                                        .on_press(Message::Workspaces(WorkspacesMessage::MoveWorkspaceUp(idx)))
                                        .padding([4, 8])
                                        .style(|_theme, status| {
                                            let bg = match status {
                                                button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.4),
                                                _ => iced::Color::TRANSPARENT,
                                            };
                                            button::Style {
                                                background: Some(iced::Background::Color(bg)),
                                                text_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
                                                ..Default::default()
                                            }
                                        })
                                } else {
                                    button(text("↑").size(14).color([0.3, 0.3, 0.3]))
                                        .padding([4, 8])
                                        .style(|_theme, _| button::Style {
                                            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                                            text_color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                                            ..Default::default()
                                        })
                                },
                                // Move down button
                                if idx < settings.workspaces.len() - 1 {
                                    button(text("↓").size(14))
                                        .on_press(Message::Workspaces(WorkspacesMessage::MoveWorkspaceDown(idx)))
                                        .padding([4, 8])
                                        .style(|_theme, status| {
                                            let bg = match status {
                                                button::Status::Hovered => iced::Color::from_rgba(0.3, 0.4, 0.5, 0.4),
                                                _ => iced::Color::TRANSPARENT,
                                            };
                                            button::Style {
                                                background: Some(iced::Background::Color(bg)),
                                                text_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
                                                ..Default::default()
                                            }
                                        })
                                } else {
                                    button(text("↓").size(14).color([0.3, 0.3, 0.3]))
                                        .padding([4, 8])
                                        .style(|_theme, _| button::Style {
                                            background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
                                            text_color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                                            ..Default::default()
                                        })
                                },
                                // Delete button
                                button(text("×").size(16))
                                    .on_press(Message::Workspaces(WorkspacesMessage::RemoveWorkspace(idx)))
                                    .padding([4, 8])
                                    .style(|_theme, status| {
                                        let bg = match status {
                                            button::Status::Hovered => iced::Color::from_rgba(0.6, 0.2, 0.2, 0.5),
                                            _ => iced::Color::TRANSPARENT,
                                        };
                                        button::Style {
                                            background: Some(iced::Background::Color(bg)),
                                            text_color: iced::Color::from_rgb(0.8, 0.4, 0.4),
                                            ..Default::default()
                                        }
                                    }),
                            ]
                            .spacing(4),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        row![
                            column![
                                text("Name").size(13).color([0.75, 0.75, 0.75]),
                                text_input("Workspace name", &ws_name)
                                    .on_input(move |name| Message::Workspaces(WorkspacesMessage::UpdateWorkspaceName(idx, name)))
                                    .padding(8),
                            ]
                            .spacing(4)
                            .width(Length::Fill),
                            column![
                                text("Pin to output (optional)").size(13).color([0.75, 0.75, 0.75]),
                                text_input("e.g., HDMI-1", &ws_output)
                                    .on_input(move |output| Message::Workspaces(WorkspacesMessage::UpdateWorkspaceOutput(idx, if output.is_empty() { None } else { Some(output) })))
                                    .padding(8),
                            ]
                            .spacing(4)
                            .width(Length::Fill),
                        ]
                        .spacing(16),
                    ]
                    .spacing(8)
                )
                .padding(16)
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.15, 0.15, 0.15, 0.4))),
                        border: iced::Border {
                            color: iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                            width: 1.0,
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    }
                })
            );
        }
    }

    // Add workspace button
    content = content.push(spacer(8.0));
    content = content.push(
        button(
            row![
                text("+").size(16),
                text("Add Workspace").size(14),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .on_press(Message::Workspaces(WorkspacesMessage::AddWorkspace))
        .padding([12, 20])
        .style(|_theme, status| {
            let bg = match status {
                button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.5),
                button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5),
                _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.4),
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    );

    content = content.push(spacer(16.0));
    content = content.push(info_text(
        "Tip: Pin workspaces to outputs to have them always appear on a specific monitor."
    ));

    scrollable(container(content).padding(20)).into()
}
