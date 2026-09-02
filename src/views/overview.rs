//! Workspace overview (toggle-overview) settings — neon modal style

use iced::widget::{column, container, row, slider, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::config::models::OverviewSettings;
use crate::messages::{Message, OverviewMessage};
use crate::theme::{fonts, neon};
use crate::views::widgets::{info_text, toggle_row};

/// Overview zoom, backdrop, and workspace-shadow controls.
pub fn section(settings: &OverviewSettings) -> Element<'_, Message> {
    let zoom = settings.zoom;
    let backdrop_color = settings
        .backdrop_color
        .as_ref()
        .map(|c| c.to_hex())
        .unwrap_or_default();
    let shadow_enabled = settings
        .workspace_shadow
        .as_ref()
        .map(|s| s.enabled)
        .unwrap_or(false);

    let mut content = column![
        modal_section("\u{29C9}", "WORKSPACE OVERVIEW", neon::PRIMARY),
        info_text("Appearance of the workspace overview (toggle-overview)."),
        Space::new().height(4),
        styled_slider(
            "ZOOM LEVEL",
            &format!("{:.2}x", zoom),
            crate::constants::OVERVIEW_ZOOM_MIN as f32..=crate::constants::OVERVIEW_ZOOM_MAX as f32,
            zoom as f32,
            0.05,
            |v| Message::Overview(OverviewMessage::SetZoom(v as f64)),
        ),
        container(
            text("How much to scale down windows in overview (0.1 = 10%, 1.0 = 100%)")
                .size(11)
                .color(neon::OUTLINE),
        )
        .padding([0, 12]),
        Space::new().height(8),
        styled_text_input(
            "BACKDROP COLOR",
            "#00000080",
            &backdrop_color,
            |v| {
                let color = if v.is_empty() { None } else { Some(v) };
                Message::Overview(OverviewMessage::SetBackdropColor(color))
            },
        ),
        container(
            text("Background color behind workspaces (hex with alpha, e.g. #00000080). Leave empty for default.")
                .size(11)
                .color(neon::OUTLINE),
        )
        .padding([0, 12]),
        Space::new().height(8),
        container(toggle_row(
            "Workspace Shadow",
            "Add a shadow behind workspaces in overview (niri 25.05+)",
            shadow_enabled,
            |v| Message::Overview(OverviewMessage::ToggleWorkspaceShadow(v)),
        ))
        .padding(8)
        .style(crate::theme::card_style),
    ]
    .spacing(6);

    if let Some(ref shadow) = settings.workspace_shadow {
        if shadow.enabled {
            let shadow_color = shadow.color.to_hex();
            content = content.push(styled_slider(
                "SHADOW SOFTNESS",
                &format!("{}", shadow.softness),
                0.0..=200.0,
                shadow.softness as f32,
                1.0,
                |v| Message::Overview(OverviewMessage::SetWorkspaceShadowSoftness(v as i32)),
            ));
            content = content.push(styled_slider(
                "SHADOW SPREAD",
                &format!("{}", shadow.spread),
                0.0..=200.0,
                shadow.spread as f32,
                1.0,
                |v| Message::Overview(OverviewMessage::SetWorkspaceShadowSpread(v as i32)),
            ));
            content = content.push(styled_slider(
                "SHADOW OFFSET X",
                &format!("{}", shadow.offset_x),
                -100.0..=100.0,
                shadow.offset_x as f32,
                1.0,
                |v| Message::Overview(OverviewMessage::SetWorkspaceShadowOffsetX(v as i32)),
            ));
            content = content.push(styled_slider(
                "SHADOW OFFSET Y",
                &format!("{}", shadow.offset_y),
                -100.0..=100.0,
                shadow.offset_y as f32,
                1.0,
                |v| Message::Overview(OverviewMessage::SetWorkspaceShadowOffsetY(v as i32)),
            ));
            content = content.push(styled_text_input(
                "SHADOW COLOR",
                "#00000050",
                &shadow_color,
                |v| Message::Overview(OverviewMessage::SetWorkspaceShadowColor(v)),
            ));
        }
    }

    content.into()
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
            slider(range, value, on_slide)
                .step(step)
                .width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}

fn styled_text_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let value_owned = value.to_string();
    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            text_input(placeholder, &value_owned)
                .on_input(on_change)
                .padding(10)
                .size(13),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
