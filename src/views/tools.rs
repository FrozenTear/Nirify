//! Tools view
//!
//! IPC tools for interacting with niri - query windows, workspaces,
//! outputs, reload config, and validate config.

use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::ipc::{FullOutputInfo, WindowInfo, WorkspaceInfo};
use crate::messages::{Message, ToolsMessage};

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
    let mut content = column![
        page_title("Niri Tools"),
        info_text(
            "Query niri state and perform actions via IPC. \
             These tools help debug and inspect the compositor."
        ),
    ]
    .spacing(4);

    content = content.push(spacer(16.0));

    // Connection status
    let status_color = if niri_connected {
        [0.3, 0.8, 0.3]
    } else {
        [0.8, 0.3, 0.3]
    };
    let status_text = if niri_connected {
        "Connected to niri"
    } else {
        "Not connected to niri"
    };

    content = content.push(
        row![
            text("Status: ").size(14),
            text(status_text).size(14).color(status_color),
        ]
        .spacing(4)
        .padding([8, 0]),
    );

    // Version display
    if let Some(version) = &state.version {
        content = content.push(
            row![
                text("Niri Version: ").size(14),
                text(version).size(14).color([0.7, 0.9, 0.7]),
            ]
            .spacing(4),
        );
    }

    content = content.push(spacer(16.0));

    // Actions section
    content = content.push(subsection_header("Actions"));
    content = content.push(info_text("Perform IPC actions on niri."));

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

    let mut reload_btn = button(text(reload_label).size(14)).padding([8, 16]);
    let mut validate_btn = button(text(validate_label).size(14)).padding([8, 16]);
    let consolidate_btn = button(text("Consolidate Rules").size(14))
        .padding([8, 16])
        .on_press(Message::AnalyzeConsolidation);

    if niri_connected && !state.reloading {
        reload_btn = reload_btn.on_press(Message::Tools(ToolsMessage::ReloadConfig));
    }
    if !state.validating {
        validate_btn = validate_btn.on_press(Message::Tools(ToolsMessage::ValidateConfig));
    }

    content = content.push(
        row![reload_btn, validate_btn, consolidate_btn]
            .spacing(12)
            .padding([8, 0]),
    );

    content = content.push(
        info_text("Consolidate Rules analyzes window and layer rules to find merge opportunities.")
    );

    // Show validation result if available
    if let Some(result) = &state.validation_result {
        let (result_text, color) = match result {
            Ok(msg) => (msg.clone(), [0.3, 0.8, 0.3]),
            Err(msg) => (msg.clone(), [0.8, 0.3, 0.3]),
        };
        content = content.push(
            container(text(result_text).size(12).color(color))
                .padding([8, 12])
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.2, 0.2, 0.2, 0.5,
                    ))),
                    ..Default::default()
                }),
        );
    }

    // Show last error if any
    if let Some(error) = &state.last_error {
        content = content.push(
            container(text(format!("Error: {}", error)).size(12).color([0.9, 0.4, 0.4]))
                .padding([8, 12]),
        );
    }

    content = content.push(spacer(16.0));

    // Query sections
    content = content.push(subsection_header("Query Data"));
    content = content.push(info_text("Fetch information from niri."));

    // Refresh buttons row
    let refresh_all_disabled = !niri_connected;
    let mut refresh_windows_btn = button(
        text(if state.loading_windows {
            "Loading..."
        } else {
            "Refresh Windows"
        })
        .size(13),
    )
    .padding([6, 12]);

    let mut refresh_workspaces_btn = button(
        text(if state.loading_workspaces {
            "Loading..."
        } else {
            "Refresh Workspaces"
        })
        .size(13),
    )
    .padding([6, 12]);

    let mut refresh_outputs_btn = button(
        text(if state.loading_outputs {
            "Loading..."
        } else {
            "Refresh Outputs"
        })
        .size(13),
    )
    .padding([6, 12]);

    let mut refresh_version_btn = button(
        text(if state.loading_version {
            "Loading..."
        } else {
            "Refresh Version"
        })
        .size(13),
    )
    .padding([6, 12]);

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

    content = content.push(
        row![
            refresh_windows_btn,
            refresh_workspaces_btn,
            refresh_outputs_btn,
            refresh_version_btn,
        ]
        .spacing(8)
        .padding([8, 0]),
    );

    content = content.push(spacer(16.0));

    // Windows list
    content = content.push(
        text(format!("Windows ({})", state.windows.len()))
            .size(15)
            .color([0.8, 0.8, 0.8]),
    );
    if state.windows.is_empty() {
        content = content.push(text("No windows loaded. Click 'Refresh Windows' to fetch.").size(12).color([0.5, 0.5, 0.5]));
    } else {
        content = content.push(windows_list(&state.windows, &state.focused_window));
    }

    content = content.push(spacer(16.0));

    // Workspaces list
    content = content.push(
        text(format!("Workspaces ({})", state.workspaces.len()))
            .size(15)
            .color([0.8, 0.8, 0.8]),
    );
    if state.workspaces.is_empty() {
        content = content.push(text("No workspaces loaded. Click 'Refresh Workspaces' to fetch.").size(12).color([0.5, 0.5, 0.5]));
    } else {
        content = content.push(workspaces_list(&state.workspaces));
    }

    content = content.push(spacer(16.0));

    // Outputs list
    content = content.push(
        text(format!("Outputs ({})", state.outputs.len()))
            .size(15)
            .color([0.8, 0.8, 0.8]),
    );
    if state.outputs.is_empty() {
        content = content.push(text("No outputs loaded. Click 'Refresh Outputs' to fetch.").size(12).color([0.5, 0.5, 0.5]));
    } else {
        content = content.push(outputs_list(&state.outputs));
    }

    content = content.push(spacer(32.0));

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
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
        let floating_indicator = if window.is_floating { " [floating]" } else { "" };

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
                        text(format!("{}{}{}", app_id, focus_indicator, floating_indicator))
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
        let vrr = if output.vrr_enabled { "VRR on" } else { "VRR off" };

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
                        text(format!("Mode: {}", mode)).size(11).color([0.6, 0.6, 0.6]),
                        text(format!("Scale: {:.2}x", scale)).size(11).color([0.6, 0.6, 0.6]),
                        text(format!("Pos: {}", pos)).size(11).color([0.6, 0.6, 0.6]),
                        text(format!("Transform: {}", transform)).size(11).color([0.6, 0.6, 0.6]),
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
