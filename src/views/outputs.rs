//! Outputs (displays) settings view - list-detail implementation

use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, toggler,
};
use iced::Color as IcedColor;
use iced::{Alignment, Border, Element, Length};
use std::collections::HashMap;

use super::widgets::*;
use crate::config::models::{DefaultColumnDisplay, LayoutOverride, OutputConfig, OutputSettings};
use crate::ipc::FullOutputInfo;
use crate::messages::{Message, OutputsMessage};
use crate::theme::{fonts, muted_text_container};
use crate::types::{CenterFocusedColumn, Color, ColorOrGradient, Transform, VrrMode};

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
    // Find matching output from IPC data
    let ipc_output = available_outputs.iter().find(|o| o.name == output_name);

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
                    || current_mode.starts_with(&m.split(" (").next().unwrap_or(""))
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

    let mut content = column![
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
                    output.scale as f32,
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
                slider_row_int(
                    "Position X",
                    "Horizontal position",
                    output.position_x,
                    -8192,
                    8192,
                    "px",
                    move |v| Message::Outputs(OutputsMessage::SetPositionX(idx, v))
                ),
                slider_row_int(
                    "Position Y",
                    "Vertical position",
                    output.position_y,
                    -8192,
                    8192,
                    "px",
                    move |v| Message::Outputs(OutputsMessage::SetPositionY(idx, v))
                ),
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

    // ── LAYOUT OVERRIDE ──
    content = content.push(Space::new().height(20));
    content = content.push(modal_section("⊡", "LAYOUT OVERRIDE", neon::TERTIARY));
    content = content.push(layout_override_content(output, idx));

    scrollable(content.spacing(0)).height(Length::Fill).into()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Layout Override UI
// ═══════════════════════════════════════════════════════════════════════════════

/// Main layout override content - shows enable button or full override controls
fn layout_override_content(output: &OutputConfig, idx: usize) -> Element<'_, Message> {
    if let Some(lo) = output.layout_override.as_ref() {
        let mut content = column![
            info_text("Override global layout settings for this output. Fields set to \"Use Global\" inherit from the global layout."),
            button(text("Remove All Overrides").size(14))
                .on_press(Message::Outputs(OutputsMessage::SetLayoutOverride(idx, None)))
                .padding([8, 16])
                .style(delete_button_style),
        ]
        .spacing(8);

        content = content.push(gaps_struts_card(lo, idx));
        content = content.push(column_behavior_card(lo, idx));
        content = content.push(sizing_card(lo, idx));
        content = content.push(focus_ring_card(lo, idx));
        content = content.push(border_card(lo, idx));
        content = content.push(shadow_card(lo, idx));

        content.into()
    } else {
        column![
            info_text("Override global layout settings (gaps, borders, focus ring, shadow, etc.) for this specific output."),
            button(text("Enable Layout Override").size(14))
                .on_press(Message::Outputs(OutputsMessage::SetLayoutOverride(idx, Some(LayoutOverride::default()))))
                .padding([8, 16]),
        ]
        .spacing(8)
        .into()
    }
}

/// Helper: send a SetLayoutOverride message with a mutated clone of the current override
fn set_lo_field(
    lo: &LayoutOverride,
    idx: usize,
    mutate: impl FnOnce(&mut LayoutOverride),
) -> Message {
    let mut new_lo = lo.clone();
    mutate(&mut new_lo);
    let result = if new_lo.has_any() { Some(new_lo) } else { None };
    Message::Outputs(OutputsMessage::SetLayoutOverride(idx, result))
}

/// Gaps & Struts sub-section card
fn gaps_struts_card(lo: &LayoutOverride, idx: usize) -> Element<'_, Message> {
    let lo1 = lo.clone();
    let lo2 = lo.clone();
    let lo3 = lo.clone();
    let lo4 = lo.clone();
    let lo5 = lo.clone();

    card(
        column![
            subsection_header("Gaps & Struts"),
            optional_slider_row(
                "Gaps",
                "Space between windows (px)",
                lo.gaps,
                0.0,
                64.0,
                "px",
                move |v| set_lo_field(&lo1, idx, |o| o.gaps = v),
            ),
            optional_slider_row(
                "Strut Left",
                "Reserved space on the left edge (px)",
                lo.strut_left,
                0.0,
                500.0,
                "px",
                move |v| set_lo_field(&lo2, idx, |o| o.strut_left = v),
            ),
            optional_slider_row(
                "Strut Right",
                "Reserved space on the right edge (px)",
                lo.strut_right,
                0.0,
                500.0,
                "px",
                move |v| set_lo_field(&lo3, idx, |o| o.strut_right = v),
            ),
            optional_slider_row(
                "Strut Top",
                "Reserved space on the top edge (px)",
                lo.strut_top,
                0.0,
                500.0,
                "px",
                move |v| set_lo_field(&lo4, idx, |o| o.strut_top = v),
            ),
            optional_slider_row(
                "Strut Bottom",
                "Reserved space on the bottom edge (px)",
                lo.strut_bottom,
                0.0,
                500.0,
                "px",
                move |v| set_lo_field(&lo5, idx, |o| o.strut_bottom = v),
            ),
        ]
        .spacing(4),
    )
}

/// Column Behavior sub-section card
fn column_behavior_card(lo: &LayoutOverride, idx: usize) -> Element<'_, Message> {
    let lo1 = lo.clone();
    let lo2 = lo.clone();
    let lo3 = lo.clone();

    card(
        column![
            subsection_header("Column Behavior"),
            optional_picker_row(
                "Center Focused Column",
                "When to auto-center the focused column",
                CenterFocusedColumn::all(),
                lo.center_focused_column,
                move |v| set_lo_field(&lo1, idx, |o| o.center_focused_column = v),
            ),
            optional_bool_picker(
                "Always Center Single Column",
                "Center a single column even when it fits",
                lo.always_center_single_column,
                move |v| set_lo_field(&lo2, idx, |o| o.always_center_single_column = v),
            ),
            optional_picker_row(
                "Default Column Display",
                "How new columns are displayed",
                &[DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed],
                lo.default_column_display,
                move |v| set_lo_field(&lo3, idx, |o| o.default_column_display = v),
            ),
        ]
        .spacing(4),
    )
}

/// Default Sizing sub-section card
fn sizing_card(lo: &LayoutOverride, idx: usize) -> Element<'_, Message> {
    let lo1 = lo.clone();

    card(
        column![
            subsection_header("Default Sizing"),
            optional_slider_row(
                "Column Width (Proportion)",
                "Default column width as a fraction of screen width",
                lo.default_column_width_proportion,
                0.1,
                1.0,
                "",
                move |v| set_lo_field(&lo1, idx, |o| o.default_column_width_proportion = v),
            ),
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Column Width (Fixed)",
                    "Default column width in pixels",
                    lo.default_column_width_fixed.map(|v| v as f32),
                    200.0,
                    4000.0,
                    "px",
                    move |v| {
                        set_lo_field(&lo_c, idx, |o| {
                            o.default_column_width_fixed = v.map(|f| f as i32)
                        })
                    },
                )
            },
            info_text(
                "Preset column widths and window heights can be configured via KDL config files"
            ),
        ]
        .spacing(4),
    )
}

/// Focus Ring sub-section card
fn focus_ring_card(lo: &LayoutOverride, idx: usize) -> Element<'_, Message> {
    let lo1 = lo.clone();
    let lo2 = lo.clone();
    let lo3 = lo.clone();

    card(
        column![
            subsection_header("Focus Ring"),
            optional_bool_picker(
                "Enabled",
                "Show focus ring around focused window",
                lo.focus_ring_enabled,
                move |v| set_lo_field(&lo1, idx, |o| o.focus_ring_enabled = v),
            ),
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Width",
                    "Focus ring thickness (px)",
                    lo.focus_ring_width.map(|v| v as f32),
                    1.0,
                    16.0,
                    "px",
                    move |v| set_lo_field(&lo_c, idx, |o| o.focus_ring_width = v.map(|f| f as i32)),
                )
            },
            optional_color_or_gradient_row(
                "Active Color",
                "Color of the focus ring on the focused window",
                lo.focus_ring_active.as_ref(),
                move |v| set_lo_field(&lo2, idx, |o| o.focus_ring_active = v),
            ),
            optional_color_or_gradient_row(
                "Inactive Color",
                "Color of the focus ring on unfocused windows",
                lo.focus_ring_inactive.as_ref(),
                move |v| set_lo_field(&lo3, idx, |o| o.focus_ring_inactive = v),
            ),
        ]
        .spacing(4),
    )
}

/// Border sub-section card
fn border_card(lo: &LayoutOverride, idx: usize) -> Element<'_, Message> {
    let lo1 = lo.clone();
    let lo2 = lo.clone();
    let lo3 = lo.clone();

    card(
        column![
            subsection_header("Border"),
            optional_bool_picker(
                "Enabled",
                "Show border around windows",
                lo.border_enabled,
                move |v| set_lo_field(&lo1, idx, |o| o.border_enabled = v),
            ),
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Width",
                    "Border thickness (px)",
                    lo.border_width.map(|v| v as f32),
                    1.0,
                    8.0,
                    "px",
                    move |v| set_lo_field(&lo_c, idx, |o| o.border_width = v.map(|f| f as i32)),
                )
            },
            optional_color_or_gradient_row(
                "Active Color",
                "Border color on the focused window",
                lo.border_active.as_ref(),
                move |v| set_lo_field(&lo2, idx, |o| o.border_active = v),
            ),
            optional_color_or_gradient_row(
                "Inactive Color",
                "Border color on unfocused windows",
                lo.border_inactive.as_ref(),
                move |v| set_lo_field(&lo3, idx, |o| o.border_inactive = v),
            ),
        ]
        .spacing(4),
    )
}

/// Shadow sub-section card
fn shadow_card(lo: &LayoutOverride, idx: usize) -> Element<'_, Message> {
    let lo1 = lo.clone();
    let lo6 = lo.clone();

    card(
        column![
            subsection_header("Shadow"),
            optional_bool_picker(
                "Enabled",
                "Show shadow behind windows",
                lo.shadow_enabled,
                move |v| set_lo_field(&lo1, idx, |o| o.shadow_enabled = v),
            ),
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Softness",
                    "Shadow blur radius (px)",
                    lo.shadow_softness.map(|v| v as f32),
                    0.0,
                    100.0,
                    "px",
                    move |v| set_lo_field(&lo_c, idx, |o| o.shadow_softness = v.map(|f| f as i32)),
                )
            },
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Spread",
                    "Shadow expansion (px)",
                    lo.shadow_spread.map(|v| v as f32),
                    0.0,
                    100.0,
                    "px",
                    move |v| set_lo_field(&lo_c, idx, |o| o.shadow_spread = v.map(|f| f as i32)),
                )
            },
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Offset X",
                    "Horizontal shadow offset (px)",
                    lo.shadow_offset_x.map(|v| v as f32),
                    -100.0,
                    100.0,
                    "px",
                    move |v| set_lo_field(&lo_c, idx, |o| o.shadow_offset_x = v.map(|f| f as i32)),
                )
            },
            {
                let lo_c = lo.clone();
                optional_slider_row(
                    "Offset Y",
                    "Vertical shadow offset (px)",
                    lo.shadow_offset_y.map(|v| v as f32),
                    -100.0,
                    100.0,
                    "px",
                    move |v| set_lo_field(&lo_c, idx, |o| o.shadow_offset_y = v.map(|f| f as i32)),
                )
            },
            optional_color_row(
                "Color",
                "Shadow color",
                lo.shadow_color.as_ref(),
                move |v| set_lo_field(&lo6, idx, |o| o.shadow_color = v),
            ),
        ]
        .spacing(4),
    )
}

/// Optional color row for `Option<Color>` fields
///
/// Shows a toggler to enable/disable, and when enabled shows a hex color input with preview.
fn optional_color_row<'a>(
    label: &'a str,
    description: &'a str,
    value: Option<&Color>,
    on_change: impl Fn(Option<Color>) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let is_enabled = value.is_some();
    let color = value.cloned().unwrap_or_default();
    let hex_value = color.to_hex();

    let on_change_toggle = on_change.clone();
    let on_change_input = on_change.clone();

    let mut content = column![row![
        column![
            text(label).size(15).font(fonts::UI_FONT_MEDIUM),
            container(text(description).size(11)).style(muted_text_container),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(is_enabled).on_toggle(move |enabled| {
            if enabled {
                on_change_toggle(Some(Color::default()))
            } else {
                on_change_toggle(None)
            }
        }),
    ]
    .spacing(12)
    .align_y(Alignment::Center),]
    .spacing(6)
    .padding(12);

    if is_enabled {
        let preview_color = IcedColor::from_rgb8(color.r, color.g, color.b);
        let preview = container(text(""))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(preview_color)),
                border: Border {
                    color: IcedColor::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        let hex_input = text_input("", &hex_value)
            .on_input(move |hex| {
                if let Some(c) = Color::from_hex(&hex) {
                    on_change_input(Some(c))
                } else {
                    Message::None
                }
            })
            .padding(8)
            .width(Length::Fixed(100.0))
            .font(fonts::MONO_FONT);

        content = content.push(
            row![preview, hex_input]
                .spacing(8)
                .align_y(Alignment::Center),
        );
    }

    content.into()
}

/// Optional color-or-gradient row for `Option<ColorOrGradient>` fields
///
/// Shows a toggler to enable/disable. When enabled, shows a hex color input.
/// Only supports solid colors in the UI; gradients set via KDL are preserved
/// until the user changes the color.
fn optional_color_or_gradient_row<'a>(
    label: &'a str,
    description: &'a str,
    value: Option<&ColorOrGradient>,
    on_change: impl Fn(Option<ColorOrGradient>) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let is_enabled = value.is_some();
    let color = value.map(|cog| *cog.primary_color()).unwrap_or_default();
    let hex_value = color.to_hex();
    let is_gradient = value.map(|v| v.is_gradient()).unwrap_or(false);

    let on_change_toggle = on_change.clone();
    let on_change_input = on_change.clone();

    let mut content = column![row![
        column![
            text(label).size(15).font(fonts::UI_FONT_MEDIUM),
            container(text(description).size(11)).style(muted_text_container),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(is_enabled).on_toggle(move |enabled| {
            if enabled {
                on_change_toggle(Some(ColorOrGradient::Color(Color::default())))
            } else {
                on_change_toggle(None)
            }
        }),
    ]
    .spacing(12)
    .align_y(Alignment::Center),]
    .spacing(6)
    .padding(12);

    if is_enabled {
        let preview_color = IcedColor::from_rgb8(color.r, color.g, color.b);
        let preview = container(text(""))
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(preview_color)),
                border: Border {
                    color: IcedColor::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        let hex_input = text_input("", &hex_value)
            .on_input(move |hex| {
                if let Some(c) = Color::from_hex(&hex) {
                    on_change_input(Some(ColorOrGradient::Color(c)))
                } else {
                    Message::None
                }
            })
            .padding(8)
            .width(Length::Fixed(100.0))
            .font(fonts::MONO_FONT);

        let mut input_row = row![preview, hex_input]
            .spacing(8)
            .align_y(Alignment::Center);

        if is_gradient {
            input_row = input_row.push(
                container(text("gradient (edit via KDL)").size(11)).style(muted_text_container),
            );
        }

        content = content.push(input_row);
    }

    content.into()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Button Styles
// ═══════════════════════════════════════════════════════════════════════════════

/// Style for delete buttons - uses theme danger color
fn delete_button_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let danger = theme.palette().danger;
    let bg = match status {
        button::Status::Hovered => iced::Color { a: 0.6, ..danger },
        button::Status::Pressed => iced::Color { a: 0.7, ..danger },
        _ => iced::Color { a: 0.5, ..danger },
    };
    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: iced::Color::WHITE,
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
        ..Default::default()
    }
}
