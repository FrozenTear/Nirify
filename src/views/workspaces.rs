//! Workspaces settings view
//!
//! Provides an interface for managing named workspaces.

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::config::models::{
    DefaultColumnDisplay, LayoutOverride, NamedWorkspace, WorkspacesSettings,
};
use crate::messages::{Message, WorkspacesMessage};
use crate::theme::muted_text_container;

/// Creates the workspaces settings view
pub fn view(settings: &WorkspacesSettings) -> Element<'static, Message> {
    let mut content = column![
        page_title("Named Workspaces"),
        info_text(
            "Define named workspaces that persist across sessions. \
             Workspaces can be pinned to specific outputs."
        ),
    ]
    .spacing(4);

    // List of workspaces
    if settings.workspaces.is_empty() {
        content = content.push(card(
            column![container(
                column![
                    container(text("No named workspaces defined").size(14))
                        .style(muted_text_container),
                    spacer(8.0),
                    container(text("Click the button below to add your first workspace").size(13))
                        .style(muted_text_container),
                ]
                .align_x(Alignment::Center)
            )
            .padding(24)
            .center(Length::Fill)]
            .width(Length::Fill),
        ));
    } else {
        for (idx, workspace) in settings.workspaces.iter().enumerate() {
            let ws_name = workspace.name.clone();
            let ws_output = workspace.open_on_output.clone().unwrap_or_default();
            let ws_len = settings.workspaces.len();

            content = content.push(card(
                column![
                    row![
                        container(text(format!("Workspace {}", idx + 1)).size(14))
                            .style(muted_text_container),
                        row![
                            // Move up button
                            if idx > 0 {
                                button(text("↑").size(14))
                                    .on_press(Message::Workspaces(
                                        WorkspacesMessage::MoveWorkspaceUp(idx),
                                    ))
                                    .padding([4, 8])
                                    .style(move_button_style)
                            } else {
                                button(text("↑").size(14))
                                    .padding([4, 8])
                                    .style(disabled_button_style)
                            },
                            // Move down button
                            if idx < ws_len - 1 {
                                button(text("↓").size(14))
                                    .on_press(Message::Workspaces(
                                        WorkspacesMessage::MoveWorkspaceDown(idx),
                                    ))
                                    .padding([4, 8])
                                    .style(move_button_style)
                            } else {
                                button(text("↓").size(14))
                                    .padding([4, 8])
                                    .style(disabled_button_style)
                            },
                            // Delete button
                            button(text("×").size(16))
                                .on_press(Message::ShowDialog(
                                    crate::messages::DialogState::Confirm {
                                        title: "Delete workspace?".to_string(),
                                        message: format!(
                                            "Delete the workspace \"{}\"? This cannot be undone.",
                                            if ws_name.is_empty() {
                                                "unnamed"
                                            } else {
                                                ws_name.as_str()
                                            }
                                        ),
                                        confirm_label: "Delete".to_string(),
                                        on_confirm: crate::messages::ConfirmAction::DeleteWorkspace(
                                            idx
                                        ),
                                    },
                                ))
                                .padding([4, 8])
                                .style(delete_button_style),
                        ]
                        .spacing(4),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    row![
                        column![
                            container(text("Name").size(13)).style(muted_text_container),
                            text_input("Workspace name", &ws_name)
                                .on_input(move |name| Message::Workspaces(
                                    WorkspacesMessage::UpdateWorkspaceName(idx, name)
                                ))
                                .padding(8),
                        ]
                        .spacing(4)
                        .width(Length::Fill),
                        column![
                            container(text("Pin to output (optional)").size(13))
                                .style(muted_text_container),
                            text_input("e.g., HDMI-1", &ws_output)
                                .on_input(move |output| Message::Workspaces(
                                    WorkspacesMessage::UpdateWorkspaceOutput(
                                        idx,
                                        if output.is_empty() {
                                            None
                                        } else {
                                            Some(output)
                                        }
                                    )
                                ))
                                .padding(8),
                        ]
                        .spacing(4)
                        .width(Length::Fill),
                    ]
                    .spacing(16),
                    layout_override_editor(workspace, idx),
                ]
                .spacing(8)
                .padding(12)
                .width(Length::Fill),
            ));
        }
    }

    // Add workspace button
    content = content.push(spacer(16.0));
    content = content.push(
        button(
            row![text("+").size(16), text("Add Workspace").size(14),]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .on_press(Message::Workspaces(WorkspacesMessage::AddWorkspace))
        .padding([12, 20])
        .style(add_button_style),
    );

    content = content.push(spacer(16.0));
    content = content.push(card(
        column![info_text(
            "Tip: Pin workspaces to outputs to have them always appear on a specific monitor."
        ),]
        .padding(12)
        .width(Length::Fill),
    ));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

/// Build a `SetLayoutOverride` message from a mutated clone of the current override.
///
/// Cloning preserves every field (including options this editor does not expose),
/// so a round-tripped workspace override never drops data the UI cannot edit.
fn wo_update(lo: &LayoutOverride, idx: usize, mutate: impl FnOnce(&mut LayoutOverride)) -> Message {
    let mut clone = lo.clone();
    mutate(&mut clone);
    Message::Workspaces(WorkspacesMessage::SetLayoutOverride(
        idx,
        Some(Box::new(clone)),
    ))
}

/// Per-workspace layout override editor (mirrors the per-output editor).
fn layout_override_editor(workspace: &NamedWorkspace, idx: usize) -> Element<'static, Message> {
    let Some(lo) = workspace.layout_override.as_ref() else {
        return column![
            container(text("Layout override: inherits global layout").size(13))
                .style(muted_text_container),
            button(text("Add Layout Override").size(13))
                .on_press(Message::Workspaces(WorkspacesMessage::SetLayoutOverride(
                    idx,
                    Some(Box::new(LayoutOverride::default())),
                )))
                .padding([6, 12]),
        ]
        .spacing(6)
        .into();
    };

    // Gaps override
    let gaps_value = lo.gaps.map(|g| format!("{}", g as i32)).unwrap_or_default();
    let lo_gaps = lo.clone();
    let gaps_input = column![
        container(text("Gaps override (blank = inherit)").size(13)).style(muted_text_container),
        text_input("e.g. 16", &gaps_value)
            .on_input(move |v| {
                let parsed = v.trim().parse::<f32>().ok();
                wo_update(&lo_gaps, idx, |l| l.gaps = parsed)
            })
            .padding(8),
    ]
    .spacing(4);

    // Default column display override
    let lo_disp = lo.clone();
    let disp_picker = column![
        container(text("Default column display").size(13)).style(muted_text_container),
        pick_list(
            vec![DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed],
            lo.default_column_display,
            move |v| wo_update(&lo_disp, idx, |l| l.default_column_display = Some(v)),
        )
        .width(Length::Fixed(140.0)),
    ]
    .spacing(4);

    // Always-center-single-column override
    let lo_center = lo.clone();
    let center_toggle = toggle_row(
        "Always center single column",
        "Override for this workspace",
        lo.always_center_single_column.unwrap_or(false),
        move |v| wo_update(&lo_center, idx, |l| l.always_center_single_column = Some(v)),
    );

    card(
        column![
            row![
                container(text("Layout Override").size(14)).style(muted_text_container),
                iced::widget::Space::new().width(Length::Fill),
                button(text("Remove").size(13))
                    .on_press(Message::Workspaces(WorkspacesMessage::SetLayoutOverride(
                        idx, None
                    )))
                    .padding([4, 10])
                    .style(delete_button_style),
            ]
            .align_y(Alignment::Center),
            row![gaps_input, disp_picker].spacing(16),
            center_toggle,
        ]
        .spacing(10)
        .padding(12)
        .width(Length::Fill),
    )
}

/// Style for move buttons - uses theme text color
fn move_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let text_color = theme.palette().text;
    let bg = match status {
        button::Status::Hovered => iced::Color {
            a: 0.2,
            ..text_color
        },
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color {
            a: 0.7,
            ..text_color
        },
        ..Default::default()
    }
}

/// Style for disabled buttons
fn disabled_button_style(theme: &iced::Theme, _status: button::Status) -> button::Style {
    let text_color = theme.palette().text;
    button::Style {
        background: Some(iced::Background::Color(iced::Color::TRANSPARENT)),
        text_color: iced::Color {
            a: 0.3,
            ..text_color
        },
        ..Default::default()
    }
}

/// Style for delete buttons - uses theme danger color
fn delete_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let danger = theme.palette().danger;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.3, ..danger },
        _ => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: danger,
        ..Default::default()
    }
}

/// Style for add buttons - uses theme primary color
fn add_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let primary = theme.palette().primary;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.5, ..primary },
        button::Status::Pressed => iced::Color { a: 0.6, ..primary },
        _ => iced::Color { a: 0.4, ..primary },
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
}
