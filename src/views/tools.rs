//! Tools view
//!
//! IPC tools for interacting with niri - query windows, workspaces,
//! outputs, reload config, and validate config.

use iced::widget::{button, column, container, row, scrollable, text, Column, Space};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::ipc::{FullOutputInfo, WindowInfo, WorkspaceInfo};
use crate::messages::{Message, ToolsMessage};
use crate::theme::{fonts, neon};

/// State for the tools page (cached IPC data)
#[derive(Debug, Clone, Default)]
pub struct ToolsState {
    /// Niri version string
    pub version: Option<String>,
    /// List of open windows
    pub windows: Vec<WindowInfo>,
    /// List of workspaces
    pub workspaces: Vec<WorkspaceInfo>,
    /// List of outputs/displays
    pub outputs: Vec<FullOutputInfo>,
    /// Currently focused window
    pub focused_window: Option<WindowInfo>,
    /// Loading states
    pub loading_windows: bool,
    pub loading_workspaces: bool,
    pub loading_outputs: bool,
    pub loading_version: bool,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Last validation result
    pub validation_result: Option<Result<String, String>>,
    /// Is reload in progress
    pub reloading: bool,
    /// Is validation in progress
    pub validating: bool,
}

/// Creates the tools view
pub fn view(state: &ToolsState, niri_connected: bool) -> Element<'_, Message> {
    let status_color = if niri_connected {
        neon::SECONDARY
    } else {
        neon::ERROR
    };
    let status_text = if niri_connected {
        "Connected"
    } else {
        "Disconnected"
    };

    let reload_label = if state.reloading {
        "Reloading..."
    } else {
        "Reload Config"
    };
    let validate_label = if state.validating {
        "Validating..."
    } else {
        "Validate Config"
    };

    let neon_btn = |_: &iced::Theme, status: iced::widget::button::Status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => iced::Color {
                a: 0.15,
                ..neon::SECONDARY
            },
            _ => iced::Color {
                a: 0.08,
                ..neon::SECONDARY
            },
        };
        iced::widget::button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: neon::SECONDARY,
            border: iced::Border {
                radius: 8.0.into(),
                color: iced::Color {
                    a: 0.25,
                    ..neon::SECONDARY
                },
                width: 1.0,
            },
            ..Default::default()
        }
    };
    let query_btn = |_: &iced::Theme, status: iced::widget::button::Status| {
        let bg = match status {
            iced::widget::button::Status::Hovered => iced::Color {
                a: 0.12,
                ..neon::PRIMARY
            },
            _ => iced::Color {
                a: 0.06,
                ..neon::PRIMARY
            },
        };
        iced::widget::button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: neon::PRIMARY,
            border: iced::Border {
                radius: 8.0.into(),
                color: iced::Color {
                    a: 0.2,
                    ..neon::PRIMARY
                },
                width: 1.0,
            },
            ..Default::default()
        }
    };

    let mut reload_btn = button(text(reload_label).size(12).font(fonts::UI_FONT_MEDIUM))
        .padding([8, 14])
        .style(neon_btn);
    let mut validate_btn = button(text(validate_label).size(12).font(fonts::UI_FONT_MEDIUM))
        .padding([8, 14])
        .style(neon_btn);
    let consolidate_btn = button(
        text("Consolidate Rules")
            .size(12)
            .font(fonts::UI_FONT_MEDIUM),
    )
    .padding([8, 14])
    .style(neon_btn)
    .on_press(Message::AnalyzeConsolidation);

    if niri_connected && !state.reloading {
        reload_btn = reload_btn.on_press(Message::Tools(ToolsMessage::ReloadConfig));
    }
    if !state.validating {
        validate_btn = validate_btn.on_press(Message::Tools(ToolsMessage::ValidateConfig));
    }

    let refresh_all_disabled = !niri_connected;
    let mut refresh_windows_btn = button(
        text(if state.loading_windows {
            "Loading..."
        } else {
            "Windows"
        })
        .size(11)
        .font(fonts::UI_FONT_MEDIUM),
    )
    .padding([6, 12])
    .style(query_btn);
    let mut refresh_workspaces_btn = button(
        text(if state.loading_workspaces {
            "Loading..."
        } else {
            "Workspaces"
        })
        .size(11)
        .font(fonts::UI_FONT_MEDIUM),
    )
    .padding([6, 12])
    .style(query_btn);
    let mut refresh_outputs_btn = button(
        text(if state.loading_outputs {
            "Loading..."
        } else {
            "Outputs"
        })
        .size(11)
        .font(fonts::UI_FONT_MEDIUM),
    )
    .padding([6, 12])
    .style(query_btn);
    let mut refresh_version_btn = button(
        text(if state.loading_version {
            "Loading..."
        } else {
            "Version"
        })
        .size(11)
        .font(fonts::UI_FONT_MEDIUM),
    )
    .padding([6, 12])
    .style(query_btn);

    if !refresh_all_disabled && !state.loading_windows {
        refresh_windows_btn =
            refresh_windows_btn.on_press(Message::Tools(ToolsMessage::RefreshWindows));
    }
    if !refresh_all_disabled && !state.loading_workspaces {
        refresh_workspaces_btn =
            refresh_workspaces_btn.on_press(Message::Tools(ToolsMessage::RefreshWorkspaces));
    }
    if !refresh_all_disabled && !state.loading_outputs {
        refresh_outputs_btn =
            refresh_outputs_btn.on_press(Message::Tools(ToolsMessage::RefreshOutputs));
    }
    if !refresh_all_disabled && !state.loading_version {
        refresh_version_btn =
            refresh_version_btn.on_press(Message::Tools(ToolsMessage::RefreshVersion));
    }

    // Validation result
    let validation_element: Element<'_, Message> = if let Some(result) = &state.validation_result {
        let (result_text, color) = match result {
            Ok(msg) => (msg.clone(), neon::SECONDARY),
            Err(msg) => (msg.clone(), neon::ERROR),
        };
        container(text(result_text).size(11).color(color))
            .padding([8, 12])
            .width(Length::Fill)
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color { a: 0.08, ..color })),
                border: iced::Border {
                    radius: 8.0.into(),
                    color: iced::Color { a: 0.2, ..color },
                    width: 1.0,
                },
                ..Default::default()
            })
            .into()
    } else {
        Space::new().into()
    };

    let error_element: Element<'_, Message> = if let Some(error) = &state.last_error {
        container(
            text(format!("Error: {}", error))
                .size(11)
                .color(neon::ERROR),
        )
        .padding([8, 12])
        .into()
    } else {
        Space::new().into()
    };

    // ── 2-COLUMN LAYOUT ──
    let content = column![
        // Status bar
        container(
            row![
                text("●").size(10).color(status_color),
                text(status_text)
                    .size(12)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(status_color),
                Space::new().width(16),
                text(state.version.as_deref().unwrap_or("Unknown"))
                    .size(12)
                    .color(neon::ON_SURFACE_VARIANT),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([8, 16])
        .width(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
            border: iced::Border {
                radius: 10.0.into(),
                color: iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                },
                width: 1.0
            },
            ..Default::default()
        }),
        Space::new().height(16),
        // 2-column: Actions + Query | Data Lists
        row![
            // Left column: Actions + Query
            column![
                modal_section("⚡", "ACTIONS", neon::SECONDARY),
                container(
                    column![
                        row![reload_btn, validate_btn].spacing(8),
                        Space::new().height(6),
                        row![consolidate_btn].spacing(8),
                    ]
                    .spacing(0)
                )
                .padding(12)
                .style(crate::theme::card_style),
                Space::new().height(4),
                validation_element,
                error_element,
                Space::new().height(16),
                modal_section("◎", "REFRESH DATA", neon::PRIMARY),
                container(
                    column![
                        row![refresh_windows_btn, refresh_workspaces_btn].spacing(8),
                        Space::new().height(6),
                        row![refresh_outputs_btn, refresh_version_btn].spacing(8),
                    ]
                    .spacing(0)
                )
                .padding(12)
                .style(crate::theme::card_style),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
            // Right column: Data Lists
            column![
                modal_section("▤", "LIVE DATA", neon::TERTIARY),
                text(format!("Windows ({})", state.windows.len()))
                    .size(13)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::ON_SURFACE),
                if state.windows.is_empty() {
                    Element::from(
                        text("Click Refresh to load")
                            .size(11)
                            .color(neon::ON_SURFACE_VARIANT),
                    )
                } else {
                    windows_list(&state.windows, &state.focused_window)
                },
                Space::new().height(12),
                text(format!("Workspaces ({})", state.workspaces.len()))
                    .size(13)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::ON_SURFACE),
                if state.workspaces.is_empty() {
                    Element::from(
                        text("Click Refresh to load")
                            .size(11)
                            .color(neon::ON_SURFACE_VARIANT),
                    )
                } else {
                    workspaces_list(&state.workspaces)
                },
                Space::new().height(12),
                text(format!("Outputs ({})", state.outputs.len()))
                    .size(13)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::ON_SURFACE),
                if state.outputs.is_empty() {
                    Element::from(
                        text("Click Refresh to load")
                            .size(11)
                            .color(neon::ON_SURFACE_VARIANT),
                    )
                } else {
                    outputs_list(&state.outputs)
                },
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
    ]
    .spacing(0);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

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

/// Render the windows list
fn windows_list<'a>(
    windows: &'a [WindowInfo],
    focused: &'a Option<WindowInfo>,
) -> Element<'a, Message> {
    let focused_id = focused.as_ref().map(|w| w.id);

    let mut list = Column::new().spacing(4);

    for window in windows {
        let is_focused = Some(window.id) == focused_id;
        let focus_indicator = if is_focused { " (focused)" } else { "" };
        let floating_indicator = if window.is_floating {
            " [floating]"
        } else {
            ""
        };

        let app_id = if window.app_id.is_empty() {
            "<no app_id>"
        } else {
            &window.app_id
        };
        let title = if window.title.is_empty() {
            "<no title>"
        } else {
            &window.title
        };

        let row_color = if is_focused {
            [0.3, 0.5, 0.3]
        } else {
            [0.25, 0.25, 0.25]
        };

        list = list.push(
            container(
                column![
                    row![
                        text(format!("#{}", window.id))
                            .size(12)
                            .color([0.6, 0.6, 0.6]),
                        text(format!(
                            "{}{}{}",
                            app_id, focus_indicator, floating_indicator
                        ))
                        .size(13),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    text(title).size(11).color([0.6, 0.6, 0.6]),
                ]
                .spacing(2),
            )
            .padding([6, 10])
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    row_color[0],
                    row_color[1],
                    row_color[2],
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }

    list.into()
}

/// Render the workspaces list
fn workspaces_list(workspaces: &[WorkspaceInfo]) -> Element<'_, Message> {
    let mut list = Column::new().spacing(4);

    for ws in workspaces {
        let name = ws.name.as_deref().unwrap_or("<unnamed>");
        let output = ws.output.as_deref().unwrap_or("<no output>");

        let indicators = format!(
            "{}{}",
            if ws.is_active { " (active)" } else { "" },
            if ws.is_focused { " (focused)" } else { "" }
        );

        let row_color = if ws.is_focused {
            [0.3, 0.4, 0.5]
        } else if ws.is_active {
            [0.3, 0.35, 0.4]
        } else {
            [0.25, 0.25, 0.25]
        };

        list = list.push(
            container(
                row![
                    text(format!("#{}", ws.idx))
                        .size(12)
                        .color([0.6, 0.6, 0.6])
                        .width(Length::Fixed(40.0)),
                    text(format!("{}{}", name, indicators)).size(13),
                    text(format!("on {}", output))
                        .size(11)
                        .color([0.5, 0.5, 0.5]),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding([6, 10])
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    row_color[0],
                    row_color[1],
                    row_color[2],
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }

    list.into()
}

/// Render the outputs list
fn outputs_list(outputs: &[FullOutputInfo]) -> Element<'_, Message> {
    let mut list = Column::new().spacing(4);

    for output in outputs {
        let mode = output.current_mode_string();
        let scale = output.scale();
        let pos = format!("{}x{}", output.position_x(), output.position_y());
        let transform = output.transform_string();
        let vrr = if output.vrr_enabled {
            "VRR on"
        } else {
            "VRR off"
        };

        let make_model = if output.make.is_empty() && output.model.is_empty() {
            String::new()
        } else {
            format!(" ({} {})", output.make, output.model)
        };

        list = list.push(
            container(
                column![
                    row![
                        text(&output.name).size(14),
                        text(make_model).size(12).color([0.5, 0.5, 0.5]),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    row![
                        text(format!("Mode: {}", mode))
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                        text(format!("Scale: {:.2}x", scale))
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                        text(format!("Pos: {}", pos))
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                        text(format!("Transform: {}", transform))
                            .size(11)
                            .color([0.6, 0.6, 0.6]),
                        text(vrr).size(11).color([0.6, 0.6, 0.6]),
                    ]
                    .spacing(16),
                ]
                .spacing(4),
            )
            .padding([8, 12])
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.25, 0.25, 0.25,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }

    list.into()
}
