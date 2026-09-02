//! Displays screen — monitor arrangement preview + per-output cards
//!
//! Two-section layout: visual monitor preview at top, per-output config cards below.
//! Full output editing is done through a modal overlay.

use iced::widget::{button, column, container, mouse_area, row, scrollable, stack, text, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use crate::config::models::OutputSettings;
use crate::ipc::FullOutputInfo;
use crate::messages::{DialogState, Message, OutputsMessage};
use crate::theme::{fonts, neon};
use crate::views;
use crate::views::display_layout::{
    calculate_canvas_size, collect_monitors, compute_preview_layout, unconfigured_outputs,
    PREVIEW_MAX_HEIGHT, PREVIEW_MAX_WIDTH,
};

/// Displays screen with monitor preview + output cards
pub fn view<'a>(
    outputs: &'a OutputSettings,
    selected_output_index: Option<usize>,
    _sections_expanded: &'a HashMap<String, bool>,
    available_outputs: &'a [FullOutputInfo],
    dragging_index: Option<usize>,
) -> Element<'a, Message> {
    let output_count = outputs.outputs.len();
    let connected_count = available_outputs.len();
    let unused_count = unconfigured_outputs(&outputs.outputs, available_outputs).len();

    // Calculate canvas size from configured outputs (logical pixels)
    let (canvas_w, canvas_h) = calculate_canvas_size(outputs, available_outputs);

    let content = column![
        // ── Hero Header ────────────────────────────────────────────────
        row![
            column![
                super::hero_header(
                    "HARDWARE INTERFACE",
                    "Display Matrix",
                    "Monitor configuration, resolution, scale, variable refresh rate, and per-output layout overrides.",
                    neon::SECONDARY,
                ),
            ].width(Length::Fill),
            column![
                row![
                    stat_label("CONFIGURED", &format!("{}", output_count)),
                    Space::new().width(20),
                    stat_label("CONNECTED", &format!("{}", connected_count)),
                    Space::new().width(20),
                    stat_label("CANVAS", &format!("{}×{}", canvas_w, canvas_h)),
                ].align_y(Alignment::End),
            ],
        ].align_y(Alignment::End),

        Space::new().height(20),

        // ── Monitor Arrangement Preview ────────────────────────────────
        monitor_preview(outputs, available_outputs, selected_output_index, dragging_index),
        Space::new().height(8),
        text("Drag monitors to rearrange · Click a display to configure")
            .size(12)
            .color(neon::ON_SURFACE_VARIANT),

        Space::new().height(32),

        // ── Monitor Specifics Header ───────────────────────────────────
        row![
            column![
                text("Monitor Specifics")
                    .size(28)
                    .font(fonts::UI_FONT_SEMIBOLD),
                container(Space::new().width(Length::Fill).height(1))
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color { a: 0.12, ..neon::OUTLINE_VARIANT })),
                        ..Default::default()
                    }),
            ].spacing(8).width(Length::Fill),
            row![
                button(text("Import connected layout").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::Outputs(OutputsMessage::ImportConnectedLayout))
                    .padding([10, 16])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => iced::Color { a: 0.16, ..neon::SECONDARY },
                            _ => iced::Color { a: 0.08, ..neon::SECONDARY },
                        };
                        iced::widget::button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: neon::SECONDARY,
                            border: iced::Border { radius: 12.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                    }),
                if unused_count > 0 {
                    Element::from(
                        button(
                            row![
                                text("↓").size(16),
                                text(format!("Adopt Connected ({unused_count})"))
                                    .size(14)
                                    .font(fonts::UI_FONT_MEDIUM),
                            ]
                            .spacing(6)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::Outputs(OutputsMessage::AdoptConnected))
                        .padding([10, 20])
                        .style(|_: &iced::Theme, status| {
                            let bg = match status {
                                iced::widget::button::Status::Hovered => neon::PRIMARY,
                                _ => iced::Color {
                                    a: 0.85,
                                    ..neon::PRIMARY
                                },
                            };
                            iced::widget::button::Style {
                                background: Some(iced::Background::Color(bg)),
                                text_color: neon::SURFACE_LOW,
                                border: iced::Border {
                                    radius: 12.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }),
                    )
                } else {
                    Space::new().into()
                },
                button(
                    row![
                        text("+").size(16),
                        text("Add Output").size(14).font(fonts::UI_FONT_MEDIUM),
                    ].spacing(6).align_y(Alignment::Center),
                )
                .on_press(Message::Outputs(OutputsMessage::AddOutput))
                .padding([10, 20])
                .style(|_: &iced::Theme, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => neon::SECONDARY,
                        _ => iced::Color { a: 0.8, ..neon::SECONDARY },
                    };
                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: neon::SURFACE_LOW,
                        border: iced::Border { radius: 12.0.into(), ..Default::default() },
                        ..Default::default()
                    }
                }),
            ].spacing(8).align_y(Alignment::Center),
        ].align_y(Alignment::End),

        Space::new().height(16),

        // ── Output Cards Grid ──────────────────────────────────────────
        output_cards_grid(outputs, available_outputs),
    ]
    .spacing(0)
    .padding(32)
    .width(Length::Fill);

    scrollable(content).height(Length::Fill).into()
}

// ── Monitor Arrangement Preview ────────────────────────────────────────────

fn monitor_preview<'a>(
    outputs: &'a OutputSettings,
    available: &'a [FullOutputInfo],
    selected_index: Option<usize>,
    dragging_index: Option<usize>,
) -> Element<'a, Message> {
    if outputs.outputs.is_empty() && available.is_empty() {
        return container(
            text("No monitors configured")
                .size(14)
                .color(neon::ON_SURFACE_VARIANT),
        )
        .width(Length::Fill)
        .padding(40)
        .center(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_LOW)),
            border: iced::Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into();
    }

    let monitors = collect_monitors(outputs, available);

    if monitors.is_empty() {
        return Space::new().into();
    }

    let Some(layout) = compute_preview_layout(&monitors, PREVIEW_MAX_HEIGHT, PREVIEW_MAX_WIDTH)
    else {
        return Space::new().into();
    };

    let canvas_width = layout.width;
    let canvas_height = layout.height;
    let monitor_layers: Vec<Element<'a, Message>> = layout
        .monitors
        .into_iter()
        .map(|preview| {
            let mon = preview.rect;
            let is_selected = mon.config_index == selected_index;
            let is_dragging = mon.config_index == dragging_index;
            let is_startup = mon.focus_at_startup;
            let enabled = mon.enabled;
            let accent = if is_startup || is_selected {
                neon::PRIMARY
            } else if mon.is_auto {
                neon::TERTIARY
            } else {
                neon::SECONDARY
            };
            let label = if is_startup {
                format!("STARTUP:{}", mon.name)
            } else if mon.is_auto {
                format!("AUTO:{}", mon.name)
            } else {
                mon.name.clone()
            };

            let monitor_box: Element<'a, Message> = container(
                column![
                    Space::new().height(Length::Fill),
                    text(label)
                        .size(if is_startup || mon.is_auto { 10 } else { 16 })
                        .font(if is_startup || is_selected {
                            fonts::UI_FONT_SEMIBOLD
                        } else {
                            fonts::UI_FONT
                        })
                        .color(if enabled {
                            iced::Color {
                                a: 0.6,
                                ..neon::ON_SURFACE
                            }
                        } else {
                            iced::Color {
                                a: 0.3,
                                ..neon::ON_SURFACE
                            }
                        }),
                    Space::new().height(Length::Fill),
                ]
                .align_x(Alignment::Center),
            )
            .width(Length::Fixed(preview.width))
            .height(Length::Fixed(preview.height))
            .center(Length::Shrink)
            .style(move |_: &iced::Theme| {
                let (bg, border_color, bw) = if enabled {
                    (
                        neon::SURFACE_CONTAINER_HIGH,
                        iced::Color { a: 0.4, ..accent },
                        2.0,
                    )
                } else {
                    (
                        neon::SURFACE_CONTAINER,
                        iced::Color {
                            a: 0.15,
                            ..neon::OUTLINE_VARIANT
                        },
                        1.0,
                    )
                };
                container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        color: border_color,
                        width: bw,
                        radius: 12.0.into(),
                    },
                    shadow: if is_startup || is_dragging {
                        iced::Shadow {
                            color: iced::Color { a: 0.15, ..accent },
                            offset: iced::Vector::new(0.0, 0.0),
                            blur_radius: 30.0,
                        }
                    } else {
                        iced::Shadow::default()
                    },
                    ..Default::default()
                }
            })
            .into();

            container(column![
                Space::new().height(Length::Fixed(preview.top)),
                row![Space::new().width(Length::Fixed(preview.left)), monitor_box,],
            ])
            .width(Length::Fixed(canvas_width))
            .height(Length::Fixed(canvas_height))
            .into()
        })
        .collect();

    let grabbing = dragging_index.is_some();
    let preview_canvas = mouse_area(
        stack(monitor_layers)
            .width(Length::Fixed(canvas_width))
            .height(Length::Fixed(canvas_height)),
    )
    .on_move(|point| Message::Outputs(OutputsMessage::CanvasMove(point.x, point.y)))
    .on_press(Message::Outputs(OutputsMessage::CanvasPress))
    .on_release(Message::Outputs(OutputsMessage::CanvasRelease))
    .on_exit(Message::Outputs(OutputsMessage::CanvasRelease))
    .interaction(if grabbing {
        iced::mouse::Interaction::Grabbing
    } else {
        iced::mouse::Interaction::Grab
    });

    container(preview_canvas)
        .width(Length::Fill)
        .padding(24)
        .center_x(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_LOW)),
            border: iced::Border {
                radius: 20.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

// ── Output Cards Grid ──────────────────────────────────────────────────────

fn output_cards_grid<'a>(
    outputs: &'a OutputSettings,
    available: &'a [FullOutputInfo],
) -> Element<'a, Message> {
    if outputs.outputs.is_empty() {
        return container(
            column![
                container(text("▭").size(32).color(neon::SECONDARY))
                    .width(72)
                    .height(72)
                    .center(Length::Shrink)
                    .style(|_: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(iced::Color {
                            a: 0.12,
                            ..neon::SECONDARY
                        })),
                        border: iced::Border {
                            radius: 999.0.into(),
                            color: iced::Color {
                                a: 0.2,
                                ..neon::SECONDARY
                            },
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                Space::new().height(16),
                text("No Outputs Configured")
                    .size(22)
                    .font(fonts::UI_FONT_SEMIBOLD),
                text("Adopt a connected display, or add one after niri reports a connector.")
                    .size(13)
                    .color(neon::ON_SURFACE_VARIANT),
                Space::new().height(12),
                if !available.is_empty() {
                    Element::from(
                        button(text("Adopt Connected").size(14).font(fonts::UI_FONT_MEDIUM))
                            .on_press(Message::Outputs(OutputsMessage::AdoptConnected))
                            .padding([10, 20])
                            .style(|_: &iced::Theme, status| {
                                let bg = match status {
                                    iced::widget::button::Status::Hovered => neon::PRIMARY,
                                    _ => iced::Color {
                                        a: 0.85,
                                        ..neon::PRIMARY
                                    },
                                };
                                iced::widget::button::Style {
                                    background: Some(iced::Background::Color(bg)),
                                    text_color: neon::SURFACE_LOW,
                                    border: iced::Border {
                                        radius: 12.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }),
                    )
                } else {
                    Space::new().into()
                },
            ]
            .spacing(4)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(60)
        .center(Length::Fill)
        .into();
    }

    let mut col1: Vec<Element<'a, Message>> = Vec::new();
    let mut col2: Vec<Element<'a, Message>> = Vec::new();

    for (idx, output) in outputs.outputs.iter().enumerate() {
        let ipc = available.iter().find(|a| a.name == output.name);
        let card = output_card(idx, output, ipc);
        if idx % 2 == 0 {
            col1.push(card);
        } else {
            col2.push(card);
        }
    }

    row![
        column(col1).spacing(12).width(Length::FillPortion(1)),
        column(col2).spacing(12).width(Length::FillPortion(1)),
    ]
    .spacing(12)
    .align_y(Alignment::Start)
    .into()
}

fn output_card<'a>(
    idx: usize,
    output: &'a crate::config::models::OutputConfig,
    ipc: Option<&'a FullOutputInfo>,
) -> Element<'a, Message> {
    let accent = match idx % 3 {
        0 => neon::PRIMARY,
        1 => neon::SECONDARY,
        _ => neon::TERTIARY,
    };

    let model_name = ipc
        .map(|i| {
            if i.model.is_empty() {
                output.name.clone()
            } else {
                format!("{} {}", i.make, i.model)
            }
        })
        .unwrap_or_else(|| output.name.clone());

    let resolution = ipc
        .map(|i| i.current_mode_string())
        .unwrap_or_else(|| output.mode.clone());

    let unnamed = output.name.trim().is_empty();
    let connector_label = if unnamed {
        "unnamed — set a connector to save".to_string()
    } else {
        format!("{} • {}", output.name, resolution)
    };

    let card = column![
        // Header
        row![
            container(text("▭").size(18).color(accent))
                .width(40)
                .height(40)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.12, ..accent })),
                    border: iced::Border {
                        radius: 10.0.into(),
                        color: iced::Color { a: 0.2, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(12),
            column![
                row![
                    text(if unnamed {
                        "Unnamed output".to_string()
                    } else {
                        model_name.clone()
                    })
                    .size(15)
                    .font(fonts::UI_FONT_SEMIBOLD),
                    if output.focus_at_startup {
                        badge_chip("Startup", neon::PRIMARY)
                    } else {
                        Space::new().into()
                    },
                    if output.position.is_none() {
                        badge_chip("Auto", neon::TERTIARY)
                    } else {
                        Space::new().into()
                    },
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(connector_label).size(11).color(if unnamed {
                    neon::ERROR
                } else {
                    neon::ON_SURFACE_VARIANT
                }),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        // Summary rows
        row![
            summary_field(
                "SCALE",
                &output
                    .scale
                    .map(|s| format!("{:.0}%", s * 100.0))
                    .unwrap_or_else(|| "auto".to_string()),
            ),
            summary_field("VRR", &format!("{}", output.vrr)),
            summary_field(
                "POSITION",
                &output
                    .position
                    .map(|(x, y)| format!("{x},{y}"))
                    .unwrap_or_else(|| "auto".to_string()),
            ),
        ]
        .spacing(12),
        Space::new().height(12),
        // Divider
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
        Space::new().height(8),
        // Configure + remove
        row![
            button(
                text("CONFIGURE")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(accent),
            )
            .on_press(Message::Outputs(OutputsMessage::OpenEditor(idx)))
            .padding([8, 16])
            .width(Length::Fill)
            .style(move |_: &iced::Theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => iced::Color { a: 0.15, ..accent },
                    _ => iced::Color { a: 0.08, ..accent },
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: accent,
                    border: iced::Border {
                        radius: 8.0.into(),
                        color: iced::Color { a: 0.2, ..accent },
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }),
            button(
                text("Remove")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::ERROR),
            )
            .on_press(Message::ShowDialog(DialogState::Confirm {
                title: "Remove output?".to_string(),
                message: format!(
                    "Remove the configuration for \"{}\"? This cannot be undone.",
                    if unnamed {
                        "unnamed output"
                    } else {
                        output.name.as_str()
                    }
                ),
                confirm_label: "Remove".to_string(),
                on_confirm: crate::messages::ConfirmAction::DeleteOutput(idx),
            }))
            .padding([8, 16])
            .style(|_: &iced::Theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered => iced::Color {
                        a: 0.18,
                        ..neon::ERROR
                    },
                    _ => iced::Color {
                        a: 0.08,
                        ..neon::ERROR
                    },
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: neon::ERROR,
                    border: iced::Border {
                        radius: 8.0.into(),
                        color: iced::Color {
                            a: 0.25,
                            ..neon::ERROR
                        },
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }),
        ]
        .spacing(8),
    ];

    container(card)
        .padding(20)
        .width(Length::Fill)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER)),
            border: iced::Border {
                color: iced::Color { a: 0.15, ..accent },
                width: 1.0,
                radius: 16.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color { a: 0.10, ..accent },
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 20.0,
            },
            ..Default::default()
        })
        .into()
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn badge_chip<'a>(label: &'a str, color: iced::Color) -> Element<'a, Message> {
    container(
        text(label)
            .size(9)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(color),
    )
    .padding([2, 8])
    .style(move |_: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color { a: 0.15, ..color })),
        border: iced::Border {
            radius: 4.0.into(),
            color: iced::Color { a: 0.3, ..color },
            width: 1.0,
        },
        ..Default::default()
    })
    .into()
}

fn summary_field<'a>(label: &'a str, value: &str) -> Element<'a, Message> {
    let v = value.to_string();
    container(
        column![
            text(label)
                .size(9)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text(v)
                .size(12)
                .font(fonts::MONO_FONT)
                .color(neon::ON_SURFACE),
        ]
        .spacing(2),
    )
    .width(Length::FillPortion(1))
    .into()
}

fn stat_label<'a>(label: &'a str, value: &str) -> Element<'a, Message> {
    let v = value.to_string();
    column![
        text(label)
            .size(9)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(neon::OUTLINE_VARIANT),
        text(v)
            .size(14)
            .font(fonts::UI_FONT_SEMIBOLD)
            .color(neon::SECONDARY),
    ]
    .spacing(2)
    .into()
}

// ── Output Editor Modal ────────────────────────────────────────────────────

/// Creates a modal overlay for editing an output
pub fn output_editor_modal<'a>(
    idx: usize,
    outputs: &'a OutputSettings,
    sections_expanded: &'a HashMap<String, bool>,
    available_outputs: &'a [FullOutputInfo],
) -> Element<'a, Message> {
    let output = &outputs.outputs[idx];
    let accent = neon::SECONDARY;

    // Wrap the existing outputs detail view
    let detail_content =
        views::outputs::output_detail_view(output, idx, sections_expanded, available_outputs);

    let editor = column![
        // Header
        row![
            container(text("▭").size(24).color(accent))
                .width(48)
                .height(48)
                .center(Length::Shrink)
                .style(move |_: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color { a: 0.15, ..accent })),
                    border: iced::Border {
                        radius: 14.0.into(),
                        color: iced::Color { a: 0.25, ..accent },
                        width: 1.0
                    },
                    ..Default::default()
                }),
            Space::new().width(16),
            column![
                text("OUTPUT CONFIGURATION")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(accent),
                text(if output.name.is_empty() {
                    "Unnamed output"
                } else {
                    output.name.as_str()
                })
                .size(22)
                .font(fonts::UI_FONT_SEMIBOLD),
            ]
            .spacing(4)
            .width(Length::Fill),
            button(text("Remove").size(12).color(neon::ERROR))
                .on_press(Message::ShowDialog(DialogState::Confirm {
                    title: "Remove output?".to_string(),
                    message: format!(
                        "Remove the configuration for \"{}\"? This cannot be undone.",
                        if output.name.is_empty() {
                            "unnamed output"
                        } else {
                            output.name.as_str()
                        }
                    ),
                    confirm_label: "Remove".to_string(),
                    on_confirm: crate::messages::ConfirmAction::DeleteOutput(idx),
                }))
                .padding([8, 12])
                .style(|_: &iced::Theme, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => iced::Color {
                            a: 0.18,
                            ..neon::ERROR
                        },
                        _ => iced::Color {
                            a: 0.08,
                            ..neon::ERROR
                        },
                    };
                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: neon::ERROR,
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
            Space::new().width(8),
            button(text("✕").size(16).color(neon::ON_SURFACE_VARIANT))
                .on_press(Message::Outputs(OutputsMessage::CloseEditor))
                .padding([8, 12])
                .style(|_: &iced::Theme, status| {
                    let bg = match status {
                        iced::widget::button::Status::Hovered => iced::Color {
                            a: 0.15,
                            ..neon::ON_SURFACE
                        },
                        _ => iced::Color {
                            a: 0.08,
                            ..neon::ON_SURFACE
                        },
                    };
                    iced::widget::button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: neon::ON_SURFACE,
                        border: iced::Border {
                            radius: 999.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                }),
        ]
        .spacing(0)
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Content from existing view
        detail_content,
        // Footer
        Space::new().height(16),
        container(Space::new().width(Length::Fill).height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(iced::Color {
                    a: 0.15,
                    ..neon::OUTLINE_VARIANT
                })),
                ..Default::default()
            }),
        container(
            row![
                row![
                    text("●").size(10).color(neon::SECONDARY),
                    text("Live Configuration Sync Active")
                        .size(12)
                        .color(neon::ON_SURFACE_VARIANT),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                button(text("Close").size(13).font(fonts::UI_FONT_MEDIUM))
                    .on_press(Message::Outputs(OutputsMessage::CloseEditor))
                    .padding([10, 24])
                    .style(|_: &iced::Theme, status| {
                        let bg = match status {
                            iced::widget::button::Status::Hovered => neon::PRIMARY,
                            _ => iced::Color {
                                a: 0.85,
                                ..neon::PRIMARY
                            },
                        };
                        iced::widget::button::Style {
                            background: Some(iced::Background::Color(bg)),
                            text_color: neon::SURFACE_LOW,
                            border: iced::Border {
                                radius: 12.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .align_y(Alignment::Center)
        )
        .padding([16, 0]),
    ];

    let modal_content = scrollable(editor.spacing(0).width(Length::Fill)).height(Length::Fill);

    let dialog = container(modal_content)
        .padding(32)
        .width(Length::Fixed(900.0))
        .max_height(750.0)
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(neon::SURFACE_CONTAINER_HIGH)),
            border: iced::Border {
                color: iced::Color { a: 0.3, ..accent },
                width: 2.0,
                radius: 20.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 8.0),
                blur_radius: 40.0,
            },
            ..Default::default()
        });

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.7,
            })),
            ..Default::default()
        })
        .into()
}
