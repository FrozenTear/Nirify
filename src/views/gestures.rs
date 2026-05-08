//! Gestures settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

use super::widgets::toggle_row;
use crate::config::models::GestureSettings;
use crate::messages::{GesturesMessage, Message};
use crate::theme::{fonts, neon};

pub fn view(settings: &GestureSettings) -> Element<'static, Message> {
    let hot_corners_enabled = settings.hot_corners.enabled;
    let hot_corner_tl = settings.hot_corners.top_left;
    let hot_corner_tr = settings.hot_corners.top_right;
    let hot_corner_bl = settings.hot_corners.bottom_left;
    let hot_corner_br = settings.hot_corners.bottom_right;

    let dnd_scroll_enabled = settings.dnd_edge_view_scroll.enabled;
    let dnd_scroll_trigger = settings.dnd_edge_view_scroll.trigger_size;
    let dnd_scroll_delay = settings.dnd_edge_view_scroll.delay_ms;
    let dnd_scroll_speed = settings.dnd_edge_view_scroll.max_speed;

    let dnd_workspace_enabled = settings.dnd_edge_workspace_switch.enabled;
    let dnd_workspace_trigger = settings.dnd_edge_workspace_switch.trigger_size;
    let dnd_workspace_delay = settings.dnd_edge_workspace_switch.delay_ms;
    let dnd_workspace_speed = settings.dnd_edge_workspace_switch.max_speed;

    let content = column![
        // -- 2-COLUMN: HOT CORNERS | DND EDGE SCROLL --
        row![
            // Left column: Hot Corners
            column![
                modal_section("\u{25f0}", "HOT CORNERS", neon::SECONDARY),
                Space::new().height(4),
                container(
                    column![
                        toggle_row(
                            "Enable hot corners",
                            "Trigger overview from screen corners",
                            hot_corners_enabled,
                            |v| Message::Gestures(GesturesMessage::SetHotCornersEnabled(v))
                        ),
                        toggle_row(
                            "Top Left",
                            "Trigger from top-left corner",
                            hot_corner_tl,
                            |v| Message::Gestures(GesturesMessage::SetHotCornerTopLeft(v))
                        ),
                        toggle_row(
                            "Top Right",
                            "Trigger from top-right corner",
                            hot_corner_tr,
                            |v| Message::Gestures(GesturesMessage::SetHotCornerTopRight(v))
                        ),
                        toggle_row(
                            "Bottom Left",
                            "Trigger from bottom-left corner",
                            hot_corner_bl,
                            |v| Message::Gestures(GesturesMessage::SetHotCornerBottomLeft(v))
                        ),
                        toggle_row(
                            "Bottom Right",
                            "Trigger from bottom-right corner",
                            hot_corner_br,
                            |v| Message::Gestures(GesturesMessage::SetHotCornerBottomRight(v))
                        ),
                    ]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right column: DnD Edge View Scroll
            column![
                modal_section("\u{21c4}", "DND EDGE VIEW SCROLL", neon::PRIMARY),
                Space::new().height(4),
                container(
                    column![toggle_row(
                        "Enable edge scroll",
                        "Scroll view when dragging to edges",
                        dnd_scroll_enabled,
                        |v| Message::Gestures(GesturesMessage::SetDndScrollEnabled(v))
                    ),]
                    .spacing(0)
                )
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(8),
                styled_slider_int(
                    "TRIGGER WIDTH",
                    &format!("{}px", dnd_scroll_trigger),
                    10..=200,
                    dnd_scroll_trigger,
                    |v| Message::Gestures(GesturesMessage::SetDndScrollTriggerWidth(v))
                ),
                styled_slider_int(
                    "DELAY",
                    &format!("{}ms", dnd_scroll_delay),
                    0..=2000,
                    dnd_scroll_delay,
                    |v| Message::Gestures(GesturesMessage::SetDndScrollDelayMs(v))
                ),
                styled_slider_int(
                    "MAX SPEED",
                    &format!("{}px/s", dnd_scroll_speed),
                    100..=5000,
                    dnd_scroll_speed,
                    |v| Message::Gestures(GesturesMessage::SetDndScrollMaxSpeed(v))
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(12),
        // -- DND WORKSPACE SWITCH (full width) --
        modal_section("\u{21c5}", "DND EDGE WORKSPACE SWITCH", neon::TERTIARY),
        Space::new().height(4),
        row![
            column![container(
                column![toggle_row(
                    "Enable workspace switch",
                    "Switch workspace when dragging to edges",
                    dnd_workspace_enabled,
                    |v| Message::Gestures(GesturesMessage::SetDndWorkspaceEnabled(v))
                ),]
                .spacing(0)
            )
            .padding(8)
            .style(crate::theme::card_style),]
            .spacing(6)
            .width(Length::FillPortion(1)),
            column![
                styled_slider_int(
                    "TRIGGER HEIGHT",
                    &format!("{}px", dnd_workspace_trigger),
                    10..=200,
                    dnd_workspace_trigger,
                    |v| Message::Gestures(GesturesMessage::SetDndWorkspaceTriggerHeight(v))
                ),
                styled_slider_int(
                    "DELAY",
                    &format!("{}ms", dnd_workspace_delay),
                    0..=2000,
                    dnd_workspace_delay,
                    |v| Message::Gestures(GesturesMessage::SetDndWorkspaceDelayMs(v))
                ),
                styled_slider_int(
                    "MAX SPEED",
                    &format!("{}px/s", dnd_workspace_speed),
                    100..=5000,
                    dnd_workspace_speed,
                    |v| Message::Gestures(GesturesMessage::SetDndWorkspaceMaxSpeed(v))
                ),
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

// -- Helpers --

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

fn styled_slider_int<'a>(
    label: &'a str,
    display: &str,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'a,
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
            iced::widget::slider(range, value, on_slide).width(Length::Fill),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
