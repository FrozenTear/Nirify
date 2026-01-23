//! Outputs (displays) settings view - list-detail implementation

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{OutputConfig, OutputSettings};
use crate::ipc::FullOutputInfo;
use crate::messages::{Message, OutputsMessage};
use crate::types::{Transform, VrrMode};

/// Represents an available display mode for dropdown selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeOption {
    /// The mode string (e.g., "1920x1080@60.00")
    pub mode_string: String,
    /// Whether this is the preferred/native mode
    pub is_preferred: bool,
}

impl std::fmt::Display for ModeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_preferred {
            write!(f, "{} (preferred)", self.mode_string)
        } else {
            write!(f, "{}", self.mode_string)
        }
    }
}

/// Creates the outputs settings view with list-detail pattern
/// Returns Element<'_> because text_input widgets borrow from settings
pub fn view<'a>(
    settings: &'a OutputSettings,
    selected_output_index: Option<usize>,
    sections_expanded: &'a HashMap<String, bool>,
    available_outputs: &'a [FullOutputInfo],
) -> Element<'a, Message> {
    // Left panel: List of outputs
    let list_panel = output_list(settings, selected_output_index);

    // Right panel: Detail view for selected output
    let detail_panel = if let Some(idx) = selected_output_index {
        if let Some(output) = settings.outputs.get(idx) {
            output_detail_view(output, idx, sections_expanded, available_outputs)
        } else {
            empty_detail_view()
        }
    } else {
        empty_detail_view()
    };

    // Horizontal split layout (responsive 1:2 ratio)
    row![
        container(list_panel)
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(|_theme| {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.5))),
                    ..Default::default()
                }
            }),
        container(detail_panel)
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .padding(20),
    ]
    .spacing(0)
    .into()
}

/// List panel showing all outputs
fn output_list<'a>(settings: &'a OutputSettings, selected_index: Option<usize>) -> Element<'a, Message> {
    let mut list = column![
        row![
            text("Outputs").size(18),
            button(text("+").size(18))
                .on_press(Message::Outputs(OutputsMessage::AddOutput))
                .padding([4, 12])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => iced::Color::from_rgba(0.3, 0.5, 0.7, 0.5),
                        button::Status::Pressed => iced::Color::from_rgba(0.4, 0.6, 0.8, 0.5),
                        _ => iced::Color::from_rgba(0.2, 0.4, 0.6, 0.4),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    }
                }),
        ]
        .spacing(10)
        .padding([12, 20])
        .align_y(Alignment::Center),
    ]
    .spacing(0);

    if settings.outputs.is_empty() {
        list = list.push(
            container(
                text("No outputs configured\nClick + to add one")
                    .size(13)
                    .color([0.75, 0.75, 0.75])
                    .center()
            )
            .padding(20)
            .center(Length::Fill)
        );
    } else {
        for (idx, output) in settings.outputs.iter().enumerate() {
            let badge = if output.enabled {
                Some("enabled")
            } else {
                Some("disabled")
            };

            let display_name = if output.name.is_empty() {
                format!("Output {}", idx + 1)
            } else {
                output.name.clone()
            };

            list = list.push(
                button(
                    row![
                        text(if selected_index == Some(idx) { "●" } else { "○" })
                            .size(12)
                            .width(Length::Fixed(20.0))
                            .color(if selected_index == Some(idx) { [0.5, 0.7, 1.0] } else { [0.5, 0.5, 0.5] }),
                        text(display_name)
                            .size(14)
                            .color(if selected_index == Some(idx) { [1.0, 1.0, 1.0] } else { [0.8, 0.8, 0.8] }),
                        if let Some(badge_text) = badge {
                            container(
                                text(badge_text)
                                    .size(11)
                                    .color([0.9, 0.9, 0.9])
                            )
                            .padding([2, 6])
                            .style(|_theme| {
                                container::Style {
                                    background: Some(iced::Background::Color(
                                        iced::Color::from_rgba(0.3, 0.5, 0.7, 0.3)
                                    )),
                                    border: iced::Border {
                                        radius: 3.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            })
                        } else {
                            container(text(""))
                        },
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                )
                .on_press(Message::Outputs(OutputsMessage::SelectOutput(idx)))
                .padding([8, 12])
                .width(Length::Fill)
                .style(move |_theme, status| {
                    let is_selected = selected_index == Some(idx);
                    let background = match (is_selected, status) {
                        (true, button::Status::Hovered) => iced::Color::from_rgba(0.3, 0.4, 0.6, 0.5),
                        (true, button::Status::Pressed) => iced::Color::from_rgba(0.4, 0.5, 0.7, 0.5),
                        (true, _) => iced::Color::from_rgba(0.2, 0.3, 0.5, 0.4),
                        (false, button::Status::Hovered) => iced::Color::from_rgba(0.25, 0.25, 0.25, 0.5),
                        (false, button::Status::Pressed) => iced::Color::from_rgba(0.3, 0.3, 0.3, 0.5),
                        (false, _) => iced::Color::TRANSPARENT,
                    };

                    button::Style {
                        background: Some(iced::Background::Color(background)),
                        border: iced::Border::default(),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    }
                })
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty detail view shown when no output is selected
fn empty_detail_view() -> Element<'static, Message> {
    container(
        text("Select an output to configure")
            .size(16)
            .color([0.75, 0.75, 0.75])
    )
    .center(Length::Fill)
    .into()
}

/// Get available modes for an output by matching its name with IPC data
fn get_available_modes(output_name: &str, available_outputs: &[FullOutputInfo]) -> Vec<ModeOption> {
    // Find matching output from IPC data
    let ipc_output = available_outputs.iter().find(|o| o.name == output_name);

    if let Some(ipc_out) = ipc_output {
        ipc_out.modes.iter().map(|mode| {
            let refresh_hz = mode.refresh_rate as f64 / 1000.0;
            ModeOption {
                mode_string: format!("{}x{}@{:.2}", mode.width, mode.height, refresh_hz),
                is_preferred: mode.is_preferred,
            }
        }).collect()
    } else {
        Vec::new()
    }
}

/// Create the mode selection row - dropdown if modes available, text input as fallback
fn mode_row<'a>(idx: usize, current_mode: &'a str, available_modes: &[ModeOption]) -> Element<'a, Message> {
    if available_modes.is_empty() {
        // No IPC data - fall back to text input
        text_input_row(
            "Mode",
            "Resolution and refresh rate (e.g., 1920x1080@60)",
            current_mode,
            move |value| Message::Outputs(OutputsMessage::SetMode(idx, value)),
        )
    } else {
        // Have available modes - show dropdown
        let mode_strings: Vec<String> = available_modes.iter().map(|m| m.to_string()).collect();

        // Find the currently selected mode (match by mode_string prefix, ignoring " (preferred)" suffix)
        let selected: Option<String> = mode_strings.iter()
            .find(|m| m.starts_with(current_mode) || current_mode.starts_with(&m.split(" (").next().unwrap_or("")))
            .cloned();

        column![
            row![
                text("Mode").size(14).width(Length::FillPortion(1)),
                pick_list(
                    mode_strings.clone(),
                    selected,
                    move |selected_str: String| {
                        // Extract just the mode string without " (preferred)" suffix
                        let mode = selected_str.split(" (").next().unwrap_or(&selected_str).to_string();
                        Message::Outputs(OutputsMessage::SetMode(idx, mode))
                    },
                )
                .width(Length::FillPortion(2))
                .padding(8),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            text("Resolution and refresh rate").size(12).color([0.75, 0.75, 0.75]),
        ]
        .spacing(4)
        .into()
    }
}

/// Detail view for a selected output
/// Borrows from output to allow text_input widgets, returns Element<'a>
fn output_detail_view<'a>(
    output: &'a OutputConfig,
    idx: usize,
    sections_expanded: &HashMap<String, bool>,
    available_outputs: &[FullOutputInfo],
) -> Element<'a, Message> {
    let basic_expanded = sections_expanded.get("basic").copied().unwrap_or(true);
    let hot_corners_expanded = sections_expanded.get("hot_corners").copied().unwrap_or(false);
    let advanced_expanded = sections_expanded.get("advanced").copied().unwrap_or(false);

    // Extract text values with proper lifetimes
    let mode_str = output.mode.as_str();
    let modeline_str = output.modeline.as_deref().unwrap_or("");

    // Get available modes from IPC data
    let available_modes = get_available_modes(&output.name, available_outputs);

    let mut content = column![
        // Header with output name and delete button
        row![
            column![
                text("Output name").size(16),
                text("Display identifier (e.g., HDMI-1, eDP-1)").size(12).color([0.7, 0.7, 0.7]),
                text_input("", &output.name)
                    .on_input(move |name| Message::Outputs(OutputsMessage::SetOutputName(idx, name)))
                    .padding(8),
            ]
            .spacing(6)
            .padding(12),
            button(text("Delete").size(14))
                .on_press(Message::Outputs(OutputsMessage::RemoveOutput(idx)))
                .padding([8, 16])
                .style(|_theme, status| {
                    let bg = match status {
                        button::Status::Hovered => iced::Color::from_rgba(0.8, 0.2, 0.2, 0.6),
                        button::Status::Pressed => iced::Color::from_rgba(0.9, 0.3, 0.3, 0.7),
                        _ => iced::Color::from_rgba(0.7, 0.2, 0.2, 0.5),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: iced::Color::WHITE,
                        ..Default::default()
                    }
                }),
        ]
        .spacing(20)
        .align_y(Alignment::Center),
        spacer(12.0),
    ];

    // Basic Settings Section
    content = content.push(expandable_section(
        "Basic Settings",
        basic_expanded,
        Message::Outputs(OutputsMessage::ToggleSection("basic".to_string())),
        column![
            toggle_row(
                "Enabled",
                "Whether this output is active",
                output.enabled,
                move |value| Message::Outputs(OutputsMessage::SetEnabled(idx, value)),
            ),
            slider_row(
                "Scale",
                "Display scaling factor for HiDPI",
                output.scale as f32,
                0.5,
                4.0,
                "x",
                move |value| Message::Outputs(OutputsMessage::SetScale(idx, value as f64)),
            ),
            // Mode selection - dropdown if modes available, text input as fallback
            mode_row(idx, mode_str, &available_modes),
            slider_row_int(
                "Position X",
                "Horizontal position in the global coordinate space",
                output.position_x,
                -8192,
                8192,
                "px",
                move |value| Message::Outputs(OutputsMessage::SetPositionX(idx, value)),
            ),
            slider_row_int(
                "Position Y",
                "Vertical position in the global coordinate space",
                output.position_y,
                -8192,
                8192,
                "px",
                move |value| Message::Outputs(OutputsMessage::SetPositionY(idx, value)),
            ),
            picker_row(
                "Transform",
                "Rotation and mirroring",
                Transform::all(),
                Some(output.transform),
                move |value| Message::Outputs(OutputsMessage::SetTransform(idx, value)),
            ),
            picker_row(
                "Variable Refresh Rate",
                "Adaptive sync / FreeSync / G-Sync",
                VrrMode::all(),
                Some(output.vrr),
                move |value| Message::Outputs(OutputsMessage::SetVrr(idx, value)),
            ),
            toggle_row(
                "Focus at startup",
                "Give this output focus when niri starts",
                output.focus_at_startup,
                move |value| Message::Outputs(OutputsMessage::SetFocusAtStartup(idx, value)),
            ),
        ]
        .spacing(8),
    ));

    // Hot Corners Section
    content = content.push(expandable_section(
        "Hot Corners",
        hot_corners_expanded,
        Message::Outputs(OutputsMessage::ToggleSection("hot_corners".to_string())),
        if let Some(hot_corners) = output.hot_corners.as_ref() {
            column![
                info_text("Configure which corners trigger overview mode on this output"),
                toggle_row(
                    "Top Left",
                    "Trigger overview when cursor hits top-left corner",
                    hot_corners.top_left,
                    move |value| Message::Outputs(OutputsMessage::SetHotCornerTopLeft(idx, value)),
                ),
                toggle_row(
                    "Top Right",
                    "Trigger overview when cursor hits top-right corner",
                    hot_corners.top_right,
                    move |value| Message::Outputs(OutputsMessage::SetHotCornerTopRight(idx, value)),
                ),
                toggle_row(
                    "Bottom Left",
                    "Trigger overview when cursor hits bottom-left corner",
                    hot_corners.bottom_left,
                    move |value| Message::Outputs(OutputsMessage::SetHotCornerBottomLeft(idx, value)),
                ),
                toggle_row(
                    "Bottom Right",
                    "Trigger overview when cursor hits bottom-right corner",
                    hot_corners.bottom_right,
                    move |value| Message::Outputs(OutputsMessage::SetHotCornerBottomRight(idx, value)),
                ),
            ]
        } else {
            column![
                info_text("Configure which corners trigger overview mode on this output"),
                text("Hot corners not configured for this output")
                    .size(13)
                    .color([0.75, 0.75, 0.75]),
                button(text("Enable Hot Corners").size(14))
                    .on_press(Message::Outputs(OutputsMessage::SetHotCornersEnabled(idx, Some(true))))
                    .padding([8, 16]),
            ]
        }
        .spacing(8),
    ));

    // Advanced Section
    content = content.push(expandable_section(
        "Advanced",
        advanced_expanded,
        Message::Outputs(OutputsMessage::ToggleSection("advanced".to_string())),
        column![
            toggle_row(
                "Custom modeline",
                "DANGEROUS: Use custom display timing (can damage monitors)",
                output.modeline.is_some(),
                move |value| {
                    if value {
                        Message::Outputs(OutputsMessage::SetModeline(idx, Some(String::new())))
                    } else {
                        Message::Outputs(OutputsMessage::SetModeline(idx, None))
                    }
                },
            ),
            if output.modeline.is_some() {
                text_input_row(
                    "Modeline string",
                    "Custom display timing (use with caution!)",
                    modeline_str,
                    move |value| Message::Outputs(OutputsMessage::SetModeline(idx, Some(value))),
                )
            } else {
                spacer(0.0)
            },
            info_text(
                "Layout overrides and other advanced settings will be added in a future update"
            ),
        ]
        .spacing(8),
    ));

    scrollable(content).height(Length::Fill).into()
}
