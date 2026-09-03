//! Outputs (displays) settings view - list-detail implementation

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{OutputConfig, OutputSettings};
use crate::ipc::FullOutputInfo;
use crate::messages::{Message, OutputsMessage};
use crate::theme::muted_text_container;
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

/// Connector name option for the identity picker
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorOption {
    pub name: String,
    pub label: String,
}

impl std::fmt::Display for ConnectorOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

fn connector_options(current: &str, available: &[FullOutputInfo]) -> Vec<ConnectorOption> {
    let mut opts: Vec<ConnectorOption> = Vec::new();
    for info in available {
        opts.push(ConnectorOption {
            name: info.name.clone(),
            label: info.display_label(),
        });
        if info.has_monitor_identity() {
            let mms = info.make_model_serial();
            if !mms.eq_ignore_ascii_case(&info.name)
                && !opts.iter().any(|opt| opt.name.eq_ignore_ascii_case(&mms))
            {
                opts.push(ConnectorOption {
                    name: mms.clone(),
                    label: format!("{mms} (make/model/serial)"),
                });
            }
        }
    }
    if !current.is_empty() && !opts.iter().any(|opt| opt.name == current) {
        opts.insert(
            0,
            ConnectorOption {
                name: current.to_string(),
                label: current.to_string(),
            },
        );
    }
    opts
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
            .style(|theme: &iced::Theme| {
                let bg = theme.palette().background;
                container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.5, ..bg })),
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
fn output_list<'a>(
    settings: &'a OutputSettings,
    selected_index: Option<usize>,
) -> Element<'a, Message> {
    let mut list = column![row![
        text("Outputs").size(18),
        button(text("+").size(18))
            .on_press(Message::Outputs(OutputsMessage::AddOutput))
            .padding([4, 12])
            .style(add_button_style),
    ]
    .spacing(10)
    .padding([12, 20])
    .align_y(Alignment::Center),]
    .spacing(0);

    if settings.outputs.is_empty() {
        list = list.push(
            container(
                container(
                    text("No outputs configured\nClick + to add one")
                        .size(13)
                        .center(),
                )
                .style(muted_text_container),
            )
            .padding(20)
            .center(Length::Fill),
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
                        text(if selected_index == Some(idx) {
                            "●"
                        } else {
                            "○"
                        })
                        .size(12)
                        .width(Length::Fixed(20.0)),
                        text(display_name).size(14),
                        if let Some(badge_text) = badge {
                            container(text(badge_text).size(11)).padding([2, 6]).style(
                                |theme: &iced::Theme| {
                                    let primary = theme.palette().primary;
                                    container::Style {
                                        text_color: Some(theme.palette().text),
                                        background: Some(iced::Background::Color(iced::Color {
                                            a: 0.3,
                                            ..primary
                                        })),
                                        border: iced::Border {
                                            radius: 3.0.into(),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }
                                },
                            )
                        } else {
                            container(text(""))
                        },
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::Outputs(OutputsMessage::SelectOutput(idx)))
                .padding([8, 12])
                .width(Length::Fill)
                .style(move |theme: &iced::Theme, status| {
                    let is_selected = selected_index == Some(idx);
                    let primary = theme.palette().primary;
                    let text_color = theme.palette().text;
                    let background = match (is_selected, status) {
                        (true, button::Status::Hovered) => iced::Color { a: 0.5, ..primary },
                        (true, button::Status::Pressed) => iced::Color { a: 0.6, ..primary },
                        (true, _) => iced::Color { a: 0.4, ..primary },
                        (false, button::Status::Hovered) => iced::Color {
                            a: 0.15,
                            ..text_color
                        },
                        (false, button::Status::Pressed) => iced::Color {
                            a: 0.2,
                            ..text_color
                        },
                        (false, _) => iced::Color::TRANSPARENT,
                    };

                    button::Style {
                        background: Some(iced::Background::Color(background)),
                        border: iced::Border::default(),
                        text_color,
                        ..Default::default()
                    }
                }),
            );
        }
    }

    scrollable(list).height(Length::Fill).into()
}

/// Empty detail view shown when no output is selected
fn empty_detail_view() -> Element<'static, Message> {
    container(container(text("Select an output to configure").size(16)).style(muted_text_container))
        .center(Length::Fill)
        .into()
}

/// Get available modes for an output by matching its name with IPC data
fn get_available_modes(output_name: &str, available_outputs: &[FullOutputInfo]) -> Vec<ModeOption> {
    let ipc_output = crate::config::find_live_output(output_name, available_outputs);

    if let Some(ipc_out) = ipc_output {
        ipc_out
            .modes
            .iter()
            .map(|mode| {
                let refresh_hz = mode.refresh_rate as f64 / 1000.0;
                ModeOption {
                    mode_string: format!("{}x{}@{:.2}", mode.width, mode.height, refresh_hz),
                    is_preferred: mode.is_preferred,
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

/// Create the mode selection row - dropdown if modes available, text input as fallback
fn mode_row<'a>(
    idx: usize,
    current_mode: &'a str,
    available_modes: &[ModeOption],
) -> Element<'a, Message> {
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
        let selected: Option<String> = mode_strings
            .iter()
            .find(|m| {
                m.starts_with(current_mode)
                    || current_mode.starts_with(m.split(" (").next().unwrap_or(""))
            })
            .cloned();

        column![
            row![
                text("Mode").size(14).width(Length::FillPortion(1)),
                pick_list(
                    mode_strings.clone(),
                    selected,
                    move |selected_str: String| {
                        // Extract just the mode string without " (preferred)" suffix
                        let mode = selected_str
                            .split(" (")
                            .next()
                            .unwrap_or(&selected_str)
                            .to_string();
                        Message::Outputs(OutputsMessage::SetMode(idx, mode))
                    },
                )
                .width(Length::FillPortion(2))
                .padding(8),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            container(text("Resolution and refresh rate").size(12)).style(muted_text_container),
        ]
        .spacing(4)
        .into()
    }
}

/// Detail view for a selected output
/// Borrows from output to allow text_input widgets, returns Element<'a>
pub fn output_detail_view<'a>(
    output: &'a OutputConfig,
    idx: usize,
    _sections_expanded: &HashMap<String, bool>,
    available_outputs: &[FullOutputInfo],
) -> Element<'a, Message> {
    use crate::theme::{fonts, neon};
    use iced::widget::Space;

    let mode_str = output.mode.as_str();
    let modeline_str = output.modeline.as_deref().unwrap_or("");
    let available_modes = get_available_modes(&output.name, available_outputs);

    let modal_section = |icon: &'a str,
                         label: &'a str,
                         accent: iced::Color|
     -> Element<'a, Message> {
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
    };

    let connector_choices = connector_options(&output.name, available_outputs);
    let selected_connector = connector_choices
        .iter()
        .find(|opt| opt.name == output.name)
        .cloned();

    let mut content = column![
        // ── IDENTITY ──
        modal_section("◎", "IDENTITY", neon::PRIMARY),
        container(
            column![
                row![
                    column![
                        text("Output name").size(14).width(Length::FillPortion(1)),
                        container(
                            text("Connector, or Make Model Serial (required to save)").size(12),
                        )
                        .style(muted_text_container),
                    ]
                    .spacing(2)
                    .width(Length::FillPortion(1)),
                    pick_list(
                        connector_choices,
                        selected_connector,
                        move |opt: ConnectorOption| {
                            Message::Outputs(OutputsMessage::SetOutputName(idx, opt.name))
                        },
                    )
                    .placeholder("Select connector or identity…")
                    .width(Length::FillPortion(2))
                    .padding(8),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
                text_input_row(
                    "Custom name",
                    "Connector (DP-1) or Make Model Serial if the display is disconnected",
                    output.name.as_str(),
                    move |value| Message::Outputs(OutputsMessage::SetOutputName(idx, value)),
                ),
            ]
            .spacing(4),
        )
        .padding(8)
        .style(crate::theme::card_style),
        if output.name.trim().is_empty() {
            Element::from(
                container(
                    text("Set a connector name — empty output blocks are not saved.")
                        .size(12)
                        .color(neon::ERROR),
                )
                .padding([4, 8]),
            )
        } else {
            spacer(0.0)
        },
        Space::new().height(12),
        // ── ROW 1: DISPLAY MODE | OPTIONS ──
        row![
            column![
                modal_section("◉", "DISPLAY MODE", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Enabled",
                        "Whether this output is active",
                        output.enabled,
                        move |v| Message::Outputs(OutputsMessage::SetEnabled(idx, v))
                    ),]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(4),
                mode_row(idx, mode_str, &available_modes),
                slider_row(
                    "Scale",
                    "HiDPI scaling factor",
                    output.display_scale() as f32,
                    0.5,
                    4.0,
                    "x",
                    move |v| Message::Outputs(OutputsMessage::SetScale(idx, v as f64))
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                modal_section("⚙", "DISPLAY OPTIONS", neon::PRIMARY),
                Space::new().height(4),
                container(
                    column![
                        picker_row(
                            "Transform",
                            "Rotation and mirroring",
                            Transform::all(),
                            Some(output.transform),
                            move |v| Message::Outputs(OutputsMessage::SetTransform(idx, v))
                        ),
                        picker_row(
                            "VRR",
                            "Adaptive sync / FreeSync",
                            VrrMode::all(),
                            Some(output.vrr),
                            move |v| Message::Outputs(OutputsMessage::SetVrr(idx, v))
                        ),
                        toggle_row(
                            "Focus at startup",
                            "Focus this output on niri start",
                            output.focus_at_startup,
                            move |v| Message::Outputs(OutputsMessage::SetFocusAtStartup(idx, v))
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(20),
        // ── ROW 2: POSITION | HOT CORNERS ──
        row![
            column![
                modal_section("⊞", "POSITION", neon::TERTIARY),
                Space::new().height(4),
                toggle_row(
                    "Automatic position",
                    "Let niri place this output automatically",
                    output.position.is_none(),
                    move |v| Message::Outputs(OutputsMessage::SetPositionAuto(idx, v))
                ),
                if let Some((px, py)) = output.position {
                    Element::from(
                        column![
                            slider_row_int(
                                "Position X",
                                "Horizontal position",
                                px,
                                -8192,
                                8192,
                                "px",
                                move |v| Message::Outputs(OutputsMessage::SetPositionX(idx, v))
                            ),
                            slider_row_int(
                                "Position Y",
                                "Vertical position",
                                py,
                                -8192,
                                8192,
                                "px",
                                move |v| Message::Outputs(OutputsMessage::SetPositionY(idx, v))
                            ),
                        ]
                        .spacing(6),
                    )
                } else {
                    spacer(0.0)
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                modal_section("▦", "HOT CORNERS", neon::SECONDARY),
                Space::new().height(4),
                if let Some(hc) = output.hot_corners.as_ref() {
                    Element::from(
                        container(
                            column![
                                toggle_row("Top Left", "Trigger overview", hc.top_left, move |v| {
                                    Message::Outputs(OutputsMessage::SetHotCornerTopLeft(idx, v))
                                }),
                                toggle_row(
                                    "Top Right",
                                    "Trigger overview",
                                    hc.top_right,
                                    move |v| Message::Outputs(
                                        OutputsMessage::SetHotCornerTopRight(idx, v)
                                    )
                                ),
                                toggle_row(
                                    "Bottom Left",
                                    "Trigger overview",
                                    hc.bottom_left,
                                    move |v| Message::Outputs(
                                        OutputsMessage::SetHotCornerBottomLeft(idx, v)
                                    )
                                ),
                                toggle_row(
                                    "Bottom Right",
                                    "Trigger overview",
                                    hc.bottom_right,
                                    move |v| Message::Outputs(
                                        OutputsMessage::SetHotCornerBottomRight(idx, v)
                                    )
                                ),
                            ]
                            .spacing(0),
                        )
                        .padding(8)
                        .style(crate::theme::card_style),
                    )
                } else {
                    Element::from(
                        column![
                            text("Not configured")
                                .size(12)
                                .color(neon::ON_SURFACE_VARIANT),
                            button(text("Enable Hot Corners").size(13))
                                .on_press(Message::Outputs(OutputsMessage::SetHotCornersEnabled(
                                    idx,
                                    Some(true)
                                )))
                                .padding([8, 16]),
                        ]
                        .spacing(8),
                    )
                },
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(20),
        // ── ADVANCED ──
        modal_section("⬡", "ADVANCED", neon::OUTLINE),
        container(
            column![
                toggle_row(
                    "Custom mode",
                    "Use a resolution not advertised by the display",
                    output.mode_custom,
                    move |v| Message::Outputs(OutputsMessage::SetModeCustom(idx, v))
                ),
                if output.mode_custom {
                    text_input_row(
                        "Custom resolution",
                        "Format: WxH@R (e.g. 1920x1080@60)",
                        mode_str,
                        move |v| Message::Outputs(OutputsMessage::SetMode(idx, v)),
                    )
                } else {
                    spacer(0.0)
                },
                toggle_row(
                    "Custom modeline",
                    "DANGEROUS: Custom display timing",
                    output.modeline.is_some(),
                    move |v| {
                        if v {
                            Message::Outputs(OutputsMessage::SetModeline(idx, Some(String::new())))
                        } else {
                            Message::Outputs(OutputsMessage::SetModeline(idx, None))
                        }
                    }
                ),
                if output.modeline.is_some() {
                    text_input_row(
                        "Modeline",
                        "Custom timing (use with caution!)",
                        modeline_str,
                        move |v| Message::Outputs(OutputsMessage::SetModeline(idx, Some(v))),
                    )
                } else {
                    spacer(0.0)
                },
            ]
            .spacing(4)
        )
        .padding(8)
        .style(crate::theme::card_style),
    ];

    // ── COLORS ──
    content = content.push(Space::new().height(20));
    content = content.push(modal_section("◑", "COLORS", neon::PRIMARY));
    content = content.push(
        container(
            column![
                optional_color_row(
                    "Background color",
                    "Solid color shown behind windows on this output",
                    output.background_color.as_ref(),
                    move |c| Message::Outputs(OutputsMessage::SetBackgroundColor(idx, c)),
                ),
                optional_color_row(
                    "Backdrop color",
                    "Color shown in the overview / between workspaces",
                    output.backdrop_color.as_ref(),
                    move |c| Message::Outputs(OutputsMessage::SetBackdropColor(idx, c)),
                ),
            ]
            .spacing(4),
        )
        .padding(8)
        .style(crate::theme::card_style),
    );

    // ── LAYOUT OVERRIDE ──
    content = content.push(Space::new().height(20));
    content = content.push(modal_section("⊡", "LAYOUT OVERRIDE", neon::TERTIARY));
    content = content.push(layout_override_content(
        output.layout_override.as_ref(),
        move |v| Message::Outputs(OutputsMessage::SetLayoutOverride(idx, v)),
        "Override global layout settings (gaps, borders, focus ring, shadow, presets, tab indicator, …) for this output.",
    ));

    scrollable(content.spacing(0)).height(Length::Fill).into()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Button Styles
// ═══════════════════════════════════════════════════════════════════════════════

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
        ..Default::default()
    }
}
