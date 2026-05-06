//! Behavior settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{info_text, optional_picker_row, optional_slider_row, picker_row, toggle_row};
use crate::config::models::BehaviorSettings;
use crate::config::ColumnWidthType;
use crate::messages::{BehaviorMessage, Message};
use crate::theme::{fonts, neon};
use crate::types::{CenterFocusedColumn, ModKey, WarpMouseMode};

/// Creates the full behavior settings view
pub fn view(settings: &BehaviorSettings) -> Element<'_, Message> {
    let content = column![
        // ── ROW 1: FOCUS | WORKSPACES ──
        row![
            // Left: Focus
            column![
                modal_section("\u{25CE}", "FOCUS", neon::PRIMARY),
                info_text("Control how window focus behaves when moving your mouse."),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Focus follows mouse",
                        "Automatically focus windows when hovering",
                        settings.focus_follows_mouse,
                        |v| Message::Behavior(BehaviorMessage::ToggleFocusFollowsMouse(v)),
                    ),]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(4),
                styled_slider(
                    "MAX SCROLL AMOUNT",
                    &format!(
                        "{}",
                        settings
                            .focus_follows_mouse_max_scroll_amount
                            .map(|v| format!("{:.0}%", v))
                            .unwrap_or_else(|| "off".to_string())
                    ),
                    0.0..=100.0,
                    settings
                        .focus_follows_mouse_max_scroll_amount
                        .unwrap_or(50.0) as f32,
                    1.0,
                    |v| Message::Behavior(BehaviorMessage::SetFocusFollowsMouseMaxScroll(Some(v))),
                ),
                picker_row(
                    "Warp mouse to focus",
                    "Move mouse pointer to newly focused windows",
                    WarpMouseMode::all(),
                    Some(settings.warp_mouse_to_focus),
                    |v| Message::Behavior(BehaviorMessage::SetWarpMouseToFocus(v)),
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Workspaces
            column![
                modal_section("\u{25A6}", "WORKSPACES", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Auto back-and-forth",
                            "Switch to current workspace goes to previous",
                            settings.workspace_auto_back_and_forth,
                            |v| Message::Behavior(
                                BehaviorMessage::ToggleWorkspaceAutoBackAndForth(v)
                            ),
                        ),
                        toggle_row(
                            "Always center single column",
                            "Center when only one column is present",
                            settings.always_center_single_column,
                            |v| Message::Behavior(BehaviorMessage::ToggleAlwaysCenterSingleColumn(
                                v
                            )),
                        ),
                        toggle_row(
                            "Empty workspace above first",
                            "Add an empty workspace above workspace 1",
                            settings.empty_workspace_above_first,
                            |v| Message::Behavior(BehaviorMessage::ToggleEmptyWorkspaceAboveFirst(
                                v
                            )),
                        ),
                    ]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(4),
                picker_row(
                    "Center focused column",
                    "When to center the focused column in viewport",
                    CenterFocusedColumn::all(),
                    Some(settings.center_focused_column),
                    |v| Message::Behavior(BehaviorMessage::SetCenterFocusedColumn(v)),
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(20),
        // ── ROW 2: COLUMNS & STRUTS | MODIFIER KEYS ──
        row![
            // Left: Columns & Struts
            column![
                modal_section("\u{25A4}", "COLUMNS", neon::TERTIARY),
                info_text("Choose how new window columns are sized by default."),
                Space::new().height(4),
                picker_row(
                    "Width type",
                    "Proportion (relative) or Fixed (absolute pixels)",
                    ColumnWidthType::all(),
                    Some(settings.default_column_width_type),
                    |v| Message::Behavior(BehaviorMessage::SetDefaultColumnWidthType(v)),
                ),
                Space::new().height(12),
                modal_section("\u{2B1C}", "STRUTS", neon::SECONDARY),
                info_text("Reserve space at screen edges (for panels/docks)."),
                Space::new().height(4),
                styled_slider(
                    "LEFT STRUT",
                    &format!("{:.0} px", settings.strut_left),
                    0.0..=200.0,
                    settings.strut_left as f32,
                    1.0,
                    |v| Message::Behavior(BehaviorMessage::SetStrutLeft(v)),
                ),
                styled_slider(
                    "RIGHT STRUT",
                    &format!("{:.0} px", settings.strut_right),
                    0.0..=200.0,
                    settings.strut_right as f32,
                    1.0,
                    |v| Message::Behavior(BehaviorMessage::SetStrutRight(v)),
                ),
                styled_slider(
                    "TOP STRUT",
                    &format!("{:.0} px", settings.strut_top),
                    0.0..=200.0,
                    settings.strut_top as f32,
                    1.0,
                    |v| Message::Behavior(BehaviorMessage::SetStrutTop(v)),
                ),
                styled_slider(
                    "BOTTOM STRUT",
                    &format!("{:.0} px", settings.strut_bottom),
                    0.0..=200.0,
                    settings.strut_bottom as f32,
                    1.0,
                    |v| Message::Behavior(BehaviorMessage::SetStrutBottom(v)),
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Modifier Keys
            column![
                modal_section("\u{2328}", "MODIFIER KEYS", neon::PRIMARY),
                info_text("Choose the primary modifier key for niri shortcuts."),
                Space::new().height(4),
                picker_row(
                    "Modifier key",
                    "Primary modifier for compositor shortcuts",
                    ModKey::all(),
                    Some(settings.mod_key),
                    |v| Message::Behavior(BehaviorMessage::SetModKey(v)),
                ),
                optional_picker_row(
                    "Nested modifier key",
                    "Override when niri runs nested",
                    ModKey::all(),
                    settings.mod_key_nested,
                    |v| Message::Behavior(BehaviorMessage::SetModKeyNested(v)),
                ),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Disable power key handling",
                        "Let the system handle power button",
                        settings.disable_power_key_handling,
                        |v| Message::Behavior(BehaviorMessage::ToggleDisablePowerKeyHandling(v)),
                    ),]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
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

/// Focus follows mouse, max scroll, warp mouse to focus
pub fn focus_section(settings: &BehaviorSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{25CE}", "FOCUS", neon::PRIMARY),
        info_text("Control how window focus behaves when moving your mouse."),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Focus follows mouse",
                "Automatically focus windows when hovering",
                settings.focus_follows_mouse,
                |v| Message::Behavior(BehaviorMessage::ToggleFocusFollowsMouse(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        optional_slider_row(
            "Max scroll amount",
            "Limit viewport scrolling (%)",
            settings.focus_follows_mouse_max_scroll_amount,
            0.0,
            100.0,
            "%",
            |v| Message::Behavior(BehaviorMessage::SetFocusFollowsMouseMaxScroll(v)),
        ),
        picker_row(
            "Warp mouse to focus",
            "Move mouse pointer to newly focused windows",
            WarpMouseMode::all(),
            Some(settings.warp_mouse_to_focus),
            |v| Message::Behavior(BehaviorMessage::SetWarpMouseToFocus(v)),
        ),
    ]
    .spacing(6)
    .into()
}

/// Workspace behavior
pub fn workspace_section(settings: &BehaviorSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{25A6}", "WORKSPACES", neon::SECONDARY),
        Space::new().height(4),
        container(
            column![
                toggle_row(
                    "Auto back-and-forth",
                    "Switch to current workspace goes to previous",
                    settings.workspace_auto_back_and_forth,
                    |v| Message::Behavior(BehaviorMessage::ToggleWorkspaceAutoBackAndForth(v)),
                ),
                toggle_row(
                    "Always center single column",
                    "Center when only one column is present",
                    settings.always_center_single_column,
                    |v| Message::Behavior(BehaviorMessage::ToggleAlwaysCenterSingleColumn(v)),
                ),
                toggle_row(
                    "Empty workspace above first",
                    "Add an empty workspace above workspace 1",
                    settings.empty_workspace_above_first,
                    |v| Message::Behavior(BehaviorMessage::ToggleEmptyWorkspaceAboveFirst(v)),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        picker_row(
            "Center focused column",
            "When to center the focused column in viewport",
            CenterFocusedColumn::all(),
            Some(settings.center_focused_column),
            |v| Message::Behavior(BehaviorMessage::SetCenterFocusedColumn(v)),
        ),
    ]
    .spacing(6)
    .into()
}

/// Default column width type
pub fn column_section(settings: &BehaviorSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{25A4}", "COLUMNS", neon::TERTIARY),
        info_text("Choose how new window columns are sized by default."),
        Space::new().height(4),
        picker_row(
            "Width type",
            "Proportion (relative) or Fixed (absolute pixels)",
            ColumnWidthType::all(),
            Some(settings.default_column_width_type),
            |v| Message::Behavior(BehaviorMessage::SetDefaultColumnWidthType(v)),
        ),
    ]
    .spacing(6)
    .into()
}

/// Screen edge struts
pub fn struts_section(settings: &BehaviorSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{2B1C}", "STRUTS", neon::SECONDARY),
        info_text("Reserve space at screen edges (for panels/docks)."),
        Space::new().height(4),
        styled_slider(
            "LEFT STRUT",
            &format!("{:.0} px", settings.strut_left),
            0.0..=200.0,
            settings.strut_left as f32,
            1.0,
            |v| Message::Behavior(BehaviorMessage::SetStrutLeft(v)),
        ),
        styled_slider(
            "RIGHT STRUT",
            &format!("{:.0} px", settings.strut_right),
            0.0..=200.0,
            settings.strut_right as f32,
            1.0,
            |v| Message::Behavior(BehaviorMessage::SetStrutRight(v)),
        ),
        styled_slider(
            "TOP STRUT",
            &format!("{:.0} px", settings.strut_top),
            0.0..=200.0,
            settings.strut_top as f32,
            1.0,
            |v| Message::Behavior(BehaviorMessage::SetStrutTop(v)),
        ),
        styled_slider(
            "BOTTOM STRUT",
            &format!("{:.0} px", settings.strut_bottom),
            0.0..=200.0,
            settings.strut_bottom as f32,
            1.0,
            |v| Message::Behavior(BehaviorMessage::SetStrutBottom(v)),
        ),
    ]
    .spacing(6)
    .into()
}

/// Modifier keys and power button
pub fn modifier_keys_section(settings: &BehaviorSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{2328}", "MODIFIER KEYS", neon::PRIMARY),
        info_text("Choose the primary modifier key for niri shortcuts."),
        Space::new().height(4),
        picker_row(
            "Modifier key",
            "Primary modifier for compositor shortcuts",
            ModKey::all(),
            Some(settings.mod_key),
            |v| Message::Behavior(BehaviorMessage::SetModKey(v)),
        ),
        optional_picker_row(
            "Nested modifier key",
            "Override when niri runs nested",
            ModKey::all(),
            settings.mod_key_nested,
            |v| Message::Behavior(BehaviorMessage::SetModKeyNested(v)),
        ),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Disable power key handling",
                "Let the system handle power button",
                settings.disable_power_key_handling,
                |v| Message::Behavior(BehaviorMessage::ToggleDisablePowerKeyHandling(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
    ]
    .spacing(6)
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

fn styled_slider<'a>(
    label: &'a str,
    display: &str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_slide: impl Fn(f32) -> Message + 'a,
) -> Element<'a, Message> {
    let d = display.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(d)
                    .size(11)
                    .font(fonts::MONO_FONT)
                    .color(neon::SECONDARY),
            ]
            .align_y(Alignment::Center),
            iced::widget::slider(range, value, on_slide)
                .step(step)
                .width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
