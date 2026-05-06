//! Appearance settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{gradient_picker, info_text, toggle_row};
use crate::config::models::AppearanceSettings;
use crate::messages::{AppearanceMessage, Message};
use crate::theme::{fonts, neon};

/// Creates the full appearance settings view
pub fn view(settings: &AppearanceSettings) -> Element<'_, Message> {
    let content = column![
        // ── 2-COLUMN: FOCUS RING | BORDER ──
        row![
            // Left: Focus Ring
            column![
                modal_section("\u{25CE}", "FOCUS RING", neon::PRIMARY),
                info_text(
                    "The focus ring is a colored outline around the currently focused window."
                ),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Enable focus ring",
                        "Show a colored ring around the focused window",
                        settings.focus_ring_enabled,
                        |v| Message::Appearance(AppearanceMessage::ToggleFocusRing(v)),
                    ),]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(4),
                styled_slider(
                    "RING WIDTH",
                    &format!("{:.0} px", settings.focus_ring_width),
                    1.0..=20.0,
                    settings.focus_ring_width as f32,
                    1.0,
                    |v| Message::Appearance(AppearanceMessage::SetFocusRingWidth(v)),
                ),
                Space::new().height(4),
                container(
                    column![
                        gradient_picker(
                            "Active window color",
                            "Color or gradient for the active window",
                            &settings.focus_ring_active,
                            |msg| Message::Appearance(AppearanceMessage::FocusRingActive(msg)),
                        ),
                        gradient_picker(
                            "Inactive window color",
                            "Color or gradient for inactive windows",
                            &settings.focus_ring_inactive,
                            |msg| Message::Appearance(AppearanceMessage::FocusRingInactive(msg)),
                        ),
                        gradient_picker(
                            "Urgent window color",
                            "Color or gradient for urgent windows",
                            &settings.focus_ring_urgent,
                            |msg| Message::Appearance(AppearanceMessage::FocusRingUrgent(msg)),
                        ),
                    ]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Border
            column![
                modal_section("\u{25A7}", "BORDER", neon::TERTIARY),
                info_text("Window borders are drawn inside the window geometry."),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Enable border",
                        "Show a colored border around windows",
                        settings.border_enabled,
                        |v| Message::Appearance(AppearanceMessage::ToggleBorder(v)),
                    ),]
                    .spacing(0),
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(4),
                styled_slider(
                    "BORDER THICKNESS",
                    &format!("{:.0} px", settings.border_thickness),
                    1.0..=20.0,
                    settings.border_thickness as f32,
                    1.0,
                    |v| Message::Appearance(AppearanceMessage::SetBorderThickness(v)),
                ),
                Space::new().height(4),
                container(
                    column![
                        gradient_picker(
                            "Active window border",
                            "Color or gradient for the active window border",
                            &settings.border_active,
                            |msg| Message::Appearance(AppearanceMessage::BorderActive(msg)),
                        ),
                        gradient_picker(
                            "Inactive window border",
                            "Color or gradient for inactive window borders",
                            &settings.border_inactive,
                            |msg| Message::Appearance(AppearanceMessage::BorderInactive(msg)),
                        ),
                        gradient_picker(
                            "Urgent window border",
                            "Color or gradient for urgent window borders",
                            &settings.border_urgent,
                            |msg| Message::Appearance(AppearanceMessage::BorderUrgent(msg)),
                        ),
                    ]
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
        Space::new().height(20),
        // ── GAPS & RADIUS (full width) ──
        modal_section("\u{2B1C}", "GAPS & CORNERS", neon::SECONDARY),
        Space::new().height(4),
        row![
            styled_slider(
                "WINDOW GAPS",
                &format!("{:.0} px", settings.gaps),
                0.0..=64.0,
                settings.gaps as f32,
                1.0,
                |v| Message::Appearance(AppearanceMessage::SetGaps(v)),
            ),
            styled_slider(
                "CORNER RADIUS",
                &format!("{:.0} px", settings.corner_radius),
                0.0..=32.0,
                settings.corner_radius as f32,
                1.0,
                |v| Message::Appearance(AppearanceMessage::SetCornerRadius(v)),
            ),
        ]
        .spacing(16),
    ]
    .spacing(0)
    .width(Length::Fill);

    scrollable(container(content).padding(8).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

/// Focus ring settings — toggle, width, active/inactive/urgent colors
pub fn focus_ring_section(settings: &AppearanceSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{25CE}", "FOCUS RING", neon::PRIMARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Enable focus ring",
                "Show a colored ring around the focused window",
                settings.focus_ring_enabled,
                |v| Message::Appearance(AppearanceMessage::ToggleFocusRing(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        styled_slider(
            "RING WIDTH",
            &format!("{:.0} px", settings.focus_ring_width),
            1.0..=20.0,
            settings.focus_ring_width as f32,
            1.0,
            |v| Message::Appearance(AppearanceMessage::SetFocusRingWidth(v)),
        ),
        container(
            column![
                gradient_picker(
                    "Active window color",
                    "Color or gradient for the active window",
                    &settings.focus_ring_active,
                    |msg| Message::Appearance(AppearanceMessage::FocusRingActive(msg)),
                ),
                gradient_picker(
                    "Inactive window color",
                    "Color or gradient for inactive windows",
                    &settings.focus_ring_inactive,
                    |msg| Message::Appearance(AppearanceMessage::FocusRingInactive(msg)),
                ),
                gradient_picker(
                    "Urgent window color",
                    "Color or gradient for urgent windows",
                    &settings.focus_ring_urgent,
                    |msg| Message::Appearance(AppearanceMessage::FocusRingUrgent(msg)),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
    ]
    .spacing(6)
    .into()
}

/// Window border settings — toggle, thickness, active/inactive/urgent colors
pub fn border_section(settings: &AppearanceSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{25A7}", "BORDER", neon::TERTIARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Enable border",
                "Show a colored border around windows",
                settings.border_enabled,
                |v| Message::Appearance(AppearanceMessage::ToggleBorder(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        styled_slider(
            "BORDER THICKNESS",
            &format!("{:.0} px", settings.border_thickness),
            1.0..=20.0,
            settings.border_thickness as f32,
            1.0,
            |v| Message::Appearance(AppearanceMessage::SetBorderThickness(v)),
        ),
        container(
            column![
                gradient_picker(
                    "Active window border",
                    "Color or gradient for the active window border",
                    &settings.border_active,
                    |msg| Message::Appearance(AppearanceMessage::BorderActive(msg)),
                ),
                gradient_picker(
                    "Inactive window border",
                    "Color or gradient for inactive window borders",
                    &settings.border_inactive,
                    |msg| Message::Appearance(AppearanceMessage::BorderInactive(msg)),
                ),
                gradient_picker(
                    "Urgent window border",
                    "Color or gradient for urgent window borders",
                    &settings.border_urgent,
                    |msg| Message::Appearance(AppearanceMessage::BorderUrgent(msg)),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
    ]
    .spacing(6)
    .into()
}

/// Gaps & corner radius settings
pub fn gaps_section(settings: &AppearanceSettings) -> Element<'_, Message> {
    column![
        modal_section("\u{2B1C}", "GAPS & CORNERS", neon::SECONDARY),
        Space::new().height(4),
        styled_slider(
            "WINDOW GAPS",
            &format!("{:.0} px", settings.gaps),
            0.0..=64.0,
            settings.gaps as f32,
            1.0,
            |v| Message::Appearance(AppearanceMessage::SetGaps(v)),
        ),
        styled_slider(
            "CORNER RADIUS",
            &format!("{:.0} px", settings.corner_radius),
            0.0..=32.0,
            settings.corner_radius as f32,
            1.0,
            |v| Message::Appearance(AppearanceMessage::SetCornerRadius(v)),
        ),
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
