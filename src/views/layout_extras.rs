//! Layout extras settings view — neon modal style
//!
//! Configure shadows, tab indicators, insert hints, and preset sizes.

use iced::widget::{column, container, pick_list, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{info_text, toggle_row};
use crate::config::models::{DefaultColumnDisplay, LayoutExtrasSettings, TabIndicatorPosition};
use crate::messages::{LayoutExtrasMessage, Message};
use crate::theme::{fonts, neon};

/// Creates the full layout extras view
pub fn view(settings: &LayoutExtrasSettings) -> Element<'static, Message> {
    let content = column![
        // ── ROW 1: SHADOWS | TAB INDICATOR (top) ──
        row![
            // Left: Shadows
            column![shadow_section(settings),]
                .spacing(0)
                .width(Length::FillPortion(1)),
            // Right: Tab Indicator
            column![tab_indicator_section(settings),]
                .spacing(0)
                .width(Length::FillPortion(1)),
        ]
        .spacing(32)
        .align_y(Alignment::Start),
        Space::new().height(20),
        // ── ROW 2: INSERT HINT | COLUMN DISPLAY ──
        row![
            column![insert_hint_section(settings),]
                .spacing(0)
                .width(Length::FillPortion(1)),
            column![column_display_section(settings),]
                .spacing(0)
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

/// Window shadow settings
pub fn shadow_section(settings: &LayoutExtrasSettings) -> Element<'static, Message> {
    let shadow = &settings.shadow;
    let shadow_color = shadow.color.to_hex();
    let shadow_inactive_color = shadow.inactive_color.to_hex();

    column![
        modal_section("\u{25A0}", "SHADOWS", neon::PRIMARY),
        Space::new().height(4),
        container(
            column![
                toggle_row(
                    "Enable shadow",
                    "Show shadow behind windows",
                    shadow.enabled,
                    |v| Message::LayoutExtras(LayoutExtrasMessage::SetShadowEnabled(v)),
                ),
                toggle_row(
                    "Draw behind window",
                    "Draw shadow underneath (for transparency)",
                    shadow.draw_behind_window,
                    |v| Message::LayoutExtras(LayoutExtrasMessage::SetShadowDrawBehindWindow(v)),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        styled_slider_int(
            "SOFTNESS",
            &format!("{} px", shadow.softness),
            0..=100,
            shadow.softness,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetShadowSoftness(v)),
        ),
        styled_slider_int(
            "SPREAD",
            &format!("{} px", shadow.spread),
            0..=100,
            shadow.spread,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetShadowSpread(v)),
        ),
        styled_slider_int(
            "OFFSET X",
            &format!("{} px", shadow.offset_x),
            -100..=100,
            shadow.offset_x,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetShadowOffsetX(v)),
        ),
        styled_slider_int(
            "OFFSET Y",
            &format!("{} px", shadow.offset_y),
            -100..=100,
            shadow.offset_y,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetShadowOffsetY(v)),
        ),
        Space::new().height(4),
        color_input("ACTIVE COLOR", &shadow_color, |s| Message::LayoutExtras(
            LayoutExtrasMessage::SetShadowColor(s)
        ),),
        color_input("INACTIVE COLOR", &shadow_inactive_color, |s| {
            Message::LayoutExtras(LayoutExtrasMessage::SetShadowInactiveColor(s))
        },),
    ]
    .spacing(6)
    .into()
}

/// Tab indicator settings
pub fn tab_indicator_section(settings: &LayoutExtrasSettings) -> Element<'static, Message> {
    let tab = &settings.tab_indicator;
    let tab_length = (tab.length_proportion * 100.0) as i32;
    let tab_active_color = tab.active.to_hex();
    let tab_inactive_color = tab.inactive.to_hex();
    let tab_urgent_color = tab.urgent.to_hex();

    column![
        modal_section("\u{25A4}", "TAB INDICATOR", neon::SECONDARY),
        Space::new().height(4),
        container(
            column![
                toggle_row(
                    "Enable tab indicator",
                    "Show indicator for tabbed windows",
                    tab.enabled,
                    |v| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorEnabled(v)),
                ),
                toggle_row(
                    "Hide when single tab",
                    "Don't show when only one tab",
                    tab.hide_when_single_tab,
                    |v| Message::LayoutExtras(
                        LayoutExtrasMessage::SetTabIndicatorHideWhenSingleTab(v)
                    ),
                ),
                toggle_row(
                    "Place within column",
                    "Position inside the column",
                    tab.place_within_column,
                    |v| Message::LayoutExtras(
                        LayoutExtrasMessage::SetTabIndicatorPlaceWithinColumn(v)
                    ),
                ),
            ]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        styled_slider_int("GAP", &format!("{} px", tab.gap), 0..=50, tab.gap, |v| {
            Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorGap(v))
        },),
        styled_slider_int(
            "WIDTH",
            &format!("{} px", tab.width),
            1..=50,
            tab.width,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorWidth(v)),
        ),
        styled_slider_int(
            "LENGTH",
            &format!("{}%", tab_length),
            10..=100,
            tab_length,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorLengthProportion(
                v as f32 / 100.0
            )),
        ),
        styled_slider_int(
            "CORNER RADIUS",
            &format!("{} px", tab.corner_radius),
            0..=50,
            tab.corner_radius,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorCornerRadius(v)),
        ),
        styled_slider_int(
            "GAPS BETWEEN TABS",
            &format!("{} px", tab.gaps_between_tabs),
            0..=50,
            tab.gaps_between_tabs,
            |v| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorGapsBetweenTabs(v)),
        ),
        container(
            column![row![
                text("POSITION")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                pick_list(
                    vec![
                        TabIndicatorPosition::Left,
                        TabIndicatorPosition::Right,
                        TabIndicatorPosition::Top,
                        TabIndicatorPosition::Bottom,
                    ],
                    Some(tab.position),
                    |v| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorPosition(v)),
                )
                .width(Length::Fixed(120.0)),
            ]
            .align_y(Alignment::Center),]
            .spacing(4),
        )
        .padding(12)
        .style(crate::theme::card_style),
        Space::new().height(4),
        color_input(
            "ACTIVE COLOR",
            &tab_active_color,
            |s| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorActiveColor(s)),
        ),
        color_input("INACTIVE COLOR", &tab_inactive_color, |s| {
            Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorInactiveColor(s))
        },),
        color_input(
            "URGENT COLOR",
            &tab_urgent_color,
            |s| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorUrgentColor(s)),
        ),
    ]
    .spacing(6)
    .into()
}

/// Insert hint settings
pub fn insert_hint_section(settings: &LayoutExtrasSettings) -> Element<'static, Message> {
    let hint = &settings.insert_hint;
    let hint_color = hint.color.to_hex();

    column![
        modal_section("\u{25C7}", "INSERT HINT", neon::TERTIARY),
        Space::new().height(4),
        container(
            column![toggle_row(
                "Enable insert hint",
                "Show visual hint when inserting windows",
                hint.enabled,
                |v| Message::LayoutExtras(LayoutExtrasMessage::SetInsertHintEnabled(v)),
            ),]
            .spacing(0),
        )
        .padding(8)
        .style(crate::theme::card_style),
        Space::new().height(4),
        color_input("HINT COLOR", &hint_color, |s| Message::LayoutExtras(
            LayoutExtrasMessage::SetInsertHintColor(s)
        ),),
    ]
    .spacing(6)
    .into()
}

/// Default column display mode
pub fn column_display_section(settings: &LayoutExtrasSettings) -> Element<'static, Message> {
    column![
        modal_section("\u{25A6}", "COLUMN DISPLAY", neon::PRIMARY),
        info_text("How new columns display windows by default."),
        Space::new().height(4),
        container(
            column![row![
                text("DISPLAY MODE")
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                pick_list(
                    vec![DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed],
                    Some(settings.default_column_display),
                    |v| Message::LayoutExtras(LayoutExtrasMessage::SetDefaultColumnDisplay(v)),
                )
                .width(Length::Fixed(120.0)),
            ]
            .align_y(Alignment::Center),]
            .spacing(4),
        )
        .padding(12)
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

fn styled_slider_int<'a>(
    label: &'a str,
    display_value: &str,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    let d = display_value.to_string();
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

fn color_input<'a>(
    label: &'a str,
    hex: &str,
    msg_fn: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let hex_owned = hex.to_string();

    let parsed_color = crate::types::Color::from_hex(&hex_owned)
        .map(|c| {
            iced::Color::from_rgba(
                c.r as f32 / 255.0,
                c.g as f32 / 255.0,
                c.b as f32 / 255.0,
                c.a as f32 / 255.0,
            )
        })
        .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

    container(
        column![
            text(label)
                .size(10)
                .font(fonts::UI_FONT_SEMIBOLD)
                .color(neon::OUTLINE_VARIANT),
            row![
                container(text("").size(14)).width(24).height(24).style(
                    move |theme: &iced::Theme| {
                        let border_color = {
                            let tc = theme.palette().text;
                            iced::Color { a: 0.3, ..tc }
                        };
                        container::Style {
                            background: Some(iced::Background::Color(parsed_color)),
                            border: iced::Border {
                                color: border_color,
                                width: 1.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }
                    }
                ),
                text_input("#RRGGBB", &hex_owned)
                    .on_input(msg_fn)
                    .padding(6)
                    .font(fonts::MONO_FONT)
                    .size(12)
                    .width(Length::Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(4),
    )
    .padding(12)
    .style(crate::theme::card_style)
    .into()
}
