//! Displays screen — monitor arrangement preview + per-output cards
//!
//! Two-section layout: visual monitor preview at top, per-output config cards below.
//! Full output editing is done through a modal overlay.

use iced::widget::{button, column, container, row, scrollable, stack, text, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashMap;

use crate::config::models::{OutputConfig, OutputSettings};
use crate::ipc::FullOutputInfo;
use crate::messages::{Message, OutputsMessage};
use crate::theme::{fonts, neon};
use crate::views;

/// Displays screen with monitor preview + output cards
pub fn view<'a>(
    outputs: &'a OutputSettings,
    _selected_output_index: Option<usize>,
    _sections_expanded: &'a HashMap<String, bool>,
    available_outputs: &'a [FullOutputInfo],
) -> Element<'a, Message> {
    let output_count = outputs.outputs.len();
    let connected_count = available_outputs.len();

    // Calculate canvas size from configured outputs
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
        monitor_preview(outputs, available_outputs),

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

    let Some(layout) = compute_preview_layout(&monitors, 200.0, 960.0) else {
        return Space::new().into();
    };

    let canvas_width = layout.width;
    let canvas_height = layout.height;
    let monitor_layers: Vec<Element<'a, Message>> = layout
        .monitors
        .into_iter()
        .map(|preview| {
            let mon = preview.rect;
            let is_primary = mon.is_primary;
            let enabled = mon.enabled;
            let accent = if is_primary {
                neon::PRIMARY
            } else {
                neon::SECONDARY
            };
            let label = if is_primary {
                format!("PRIMARY:{}", mon.name)
            } else {
                format!("{}", mon.index + 1)
            };

            let monitor_box: Element<'a, Message> = container(
                column![
                    Space::new().height(Length::Fill),
                    text(label)
                        .size(if is_primary { 10 } else { 24 })
                        .font(if is_primary {
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
                    shadow: if is_primary {
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

    let preview_canvas = stack(monitor_layers)
        .width(Length::Fixed(canvas_width))
        .height(Length::Fixed(canvas_height));

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
                text("Add an output to start configuring your displays.")
                    .size(13)
                    .color(neon::ON_SURFACE_VARIANT),
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

    let is_primary = idx == 0;

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
                    text(model_name.clone())
                        .size(15)
                        .font(fonts::UI_FONT_SEMIBOLD),
                    if is_primary {
                        Element::from(
                            container(
                                text("Primary")
                                    .size(9)
                                    .font(fonts::UI_FONT_SEMIBOLD)
                                    .color(neon::SECONDARY),
                            )
                            .padding([2, 8])
                            .style(|_: &iced::Theme| {
                                container::Style {
                                    background: Some(iced::Background::Color(iced::Color {
                                        a: 0.15,
                                        ..neon::SECONDARY
                                    })),
                                    border: iced::Border {
                                        radius: 4.0.into(),
                                        color: iced::Color {
                                            a: 0.3,
                                            ..neon::SECONDARY
                                        },
                                        width: 1.0,
                                    },
                                    ..Default::default()
                                }
                            }),
                        )
                    } else {
                        Space::new().into()
                    },
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text(format!("{} • {}", output.name, resolution))
                    .size(11)
                    .color(neon::ON_SURFACE_VARIANT),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        // Summary rows
        row![
            summary_field("SCALE", &format!("{:.0}%", output.scale * 100.0)),
            summary_field("VRR", &format!("{}", output.vrr)),
            summary_field("TRANSFORM", &format!("{}", output.transform)),
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
        // Configure button
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

fn calculate_canvas_size(outputs: &OutputSettings, available: &[FullOutputInfo]) -> (i32, i32) {
    let monitors: Vec<MonitorRect> = outputs
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, output)| output.enabled)
        .map(|(idx, output)| {
            let (width, height) = configured_output_size(output, available);
            MonitorRect {
                index: idx,
                name: output.name.clone(),
                x: output.position_x,
                y: output.position_y,
                width,
                height,
                enabled: output.enabled,
                is_primary: idx == 0,
            }
        })
        .collect();

    monitor_bounds(&monitors)
        .map(|bounds| (bounds.max_x - bounds.min_x, bounds.max_y - bounds.min_y))
        .unwrap_or((1920, 1080))
}

fn parse_resolution(mode: &str) -> Option<(i32, i32)> {
    let at_idx = mode.find('@').unwrap_or(mode.len());
    let res = &mode[..at_idx];
    let mut parts = res.split('x');
    let w = parts.next()?.parse().ok()?;
    let h = parts.next()?.parse().ok()?;
    Some((w, h))
}

fn collect_monitors(outputs: &OutputSettings, available: &[FullOutputInfo]) -> Vec<MonitorRect> {
    let mut monitors: Vec<MonitorRect> = outputs
        .outputs
        .iter()
        .enumerate()
        .map(|(idx, output)| {
            let (width, height) = configured_output_size(output, available);
            MonitorRect {
                index: idx,
                name: output.name.clone(),
                x: output.position_x,
                y: output.position_y,
                width,
                height,
                enabled: output.enabled,
                is_primary: idx == 0,
            }
        })
        .collect();

    for info in available {
        if monitors.iter().any(|monitor| monitor.name == info.name) {
            continue;
        }

        let (width, height) = ipc_output_size(info);
        monitors.push(MonitorRect {
            index: monitors.len(),
            name: info.name.clone(),
            x: info.position_x(),
            y: info.position_y(),
            width,
            height,
            enabled: true,
            is_primary: false,
        });
    }

    monitors
}

fn configured_output_size(output: &OutputConfig, available: &[FullOutputInfo]) -> (i32, i32) {
    available
        .iter()
        .find(|info| info.name == output.name)
        .map(ipc_output_size)
        .unwrap_or_else(|| parse_resolution(&output.mode).unwrap_or((1920, 1080)))
}

fn ipc_output_size(info: &FullOutputInfo) -> (i32, i32) {
    info.current_mode
        .and_then(|mode_idx| info.modes.get(mode_idx))
        .map(|mode| (mode.width, mode.height))
        .unwrap_or((1920, 1080))
}

fn monitor_bounds(monitors: &[MonitorRect]) -> Option<MonitorBounds> {
    Some(MonitorBounds {
        min_x: monitors.iter().map(|monitor| monitor.x).min()?,
        min_y: monitors.iter().map(|monitor| monitor.y).min()?,
        max_x: monitors
            .iter()
            .map(|monitor| monitor.x + monitor.width)
            .max()?,
        max_y: monitors
            .iter()
            .map(|monitor| monitor.y + monitor.height)
            .max()?,
    })
}

fn compute_preview_layout(
    monitors: &[MonitorRect],
    max_height: f32,
    max_width: f32,
) -> Option<PreviewLayout> {
    let bounds = monitor_bounds(monitors)?;
    let total_width = (bounds.max_x - bounds.min_x) as f32;
    let total_height = (bounds.max_y - bounds.min_y) as f32;

    if total_width <= 0.0 || total_height <= 0.0 {
        return None;
    }

    let scale = (max_height / total_height).min(max_width / total_width);
    let monitors = monitors
        .iter()
        .cloned()
        .map(|rect| PreviewMonitor {
            left: (rect.x - bounds.min_x) as f32 * scale,
            top: (rect.y - bounds.min_y) as f32 * scale,
            width: rect.width as f32 * scale,
            height: rect.height as f32 * scale,
            rect,
        })
        .collect();

    Some(PreviewLayout {
        width: total_width * scale,
        height: total_height * scale,
        monitors,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct MonitorRect {
    index: usize,
    name: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    enabled: bool,
    is_primary: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MonitorBounds {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct PreviewLayout {
    width: f32,
    height: f32,
    monitors: Vec<PreviewMonitor>,
}

#[derive(Clone, Debug, PartialEq)]
struct PreviewMonitor {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    rect: MonitorRect,
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
                text(&output.name).size(22).font(fonts::UI_FONT_SEMIBOLD),
            ]
            .spacing(4)
            .width(Length::Fill),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_canvas_size_includes_negative_coordinates() {
        let outputs = OutputSettings {
            outputs: vec![
                OutputConfig {
                    name: "DP-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position_x: -1920,
                    ..Default::default()
                },
                OutputConfig {
                    name: "HDMI-A-1".to_string(),
                    mode: "1920x1080@60.00".to_string(),
                    position_x: 0,
                    ..Default::default()
                },
            ],
        };

        assert_eq!(calculate_canvas_size(&outputs, &[]), (3840, 1080));
    }

    #[test]
    fn compute_preview_layout_preserves_vertical_offsets() {
        let monitors = vec![
            MonitorRect {
                index: 0,
                name: "DP-1".to_string(),
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                enabled: true,
                is_primary: true,
            },
            MonitorRect {
                index: 1,
                name: "HDMI-A-1".to_string(),
                x: 0,
                y: 1080,
                width: 1920,
                height: 1080,
                enabled: true,
                is_primary: false,
            },
        ];

        let layout = compute_preview_layout(&monitors, 200.0, 960.0).unwrap();
        let top_monitor = layout
            .monitors
            .iter()
            .find(|monitor| monitor.rect.name == "DP-1")
            .unwrap();
        let bottom_monitor = layout
            .monitors
            .iter()
            .find(|monitor| monitor.rect.name == "HDMI-A-1")
            .unwrap();

        assert_eq!(top_monitor.left, bottom_monitor.left);
        assert!(bottom_monitor.top > top_monitor.top);
    }
}
