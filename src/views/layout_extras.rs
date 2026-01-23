//! Layout extras settings view
//!
//! Configure shadows, tab indicators, insert hints, and preset sizes.

use iced::widget::{column, container, scrollable, text, text_input, row, pick_list};
use iced::{Element, Length, Alignment};

use super::widgets::*;
use crate::config::models::{LayoutExtrasSettings, TabIndicatorPosition, DefaultColumnDisplay};
use crate::messages::{LayoutExtrasMessage, Message};
use crate::theme::fonts;

/// Creates the layout extras settings view
pub fn view(settings: &LayoutExtrasSettings) -> Element<'static, Message> {
    // Shadow settings
    let shadow = &settings.shadow;
    let shadow_enabled = shadow.enabled;
    let shadow_softness = shadow.softness;
    let shadow_spread = shadow.spread;
    let shadow_offset_x = shadow.offset_x;
    let shadow_offset_y = shadow.offset_y;
    let shadow_draw_behind = shadow.draw_behind_window;
    let shadow_color = shadow.color.to_hex();
    let shadow_inactive_color = shadow.inactive_color.to_hex();

    // Tab indicator settings
    let tab = &settings.tab_indicator;
    let tab_enabled = tab.enabled;
    let tab_hide_single = tab.hide_when_single_tab;
    let tab_within_column = tab.place_within_column;
    let tab_gap = tab.gap;
    let tab_width = tab.width;
    let tab_length = (tab.length_proportion * 100.0) as i32;
    let tab_corner_radius = tab.corner_radius;
    let tab_gaps_between = tab.gaps_between_tabs;
    let tab_position = tab.position;
    let tab_active_color = tab.active.to_hex();
    let tab_inactive_color = tab.inactive.to_hex();
    let tab_urgent_color = tab.urgent.to_hex();

    // Insert hint settings
    let hint = &settings.insert_hint;
    let hint_enabled = hint.enabled;
    let hint_color = hint.color.to_hex();

    // Default column display
    let default_display = settings.default_column_display;

    let content = column![
        page_title("Layout Extras"),
        info_text(
            "Configure window shadows, tab indicators, insert hints, and preset sizes."
        ),
        subsection_header("Window Shadow"),
        toggle_row(
            "Enable shadow",
            "Show shadow behind windows",
            shadow_enabled,
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetShadowEnabled(value)),
        ),
        toggle_row(
            "Draw behind window",
            "Draw shadow underneath the window (for transparency)",
            shadow_draw_behind,
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetShadowDrawBehindWindow(value)),
        ),
        slider_row_int(
            "Softness",
            "Shadow blur/softness amount",
            shadow_softness,
            0,
            100,
            " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetShadowSoftness(value)),
        ),
        slider_row_int(
            "Spread",
            "Shadow expansion amount",
            shadow_spread,
            0,
            100,
            " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetShadowSpread(value)),
        ),
        slider_row_int(
            "Offset X",
            "Horizontal shadow offset",
            shadow_offset_x,
            -100,
            100,
            " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetShadowOffsetX(value)),
        ),
        slider_row_int(
            "Offset Y",
            "Vertical shadow offset",
            shadow_offset_y,
            -100,
            100,
            " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetShadowOffsetY(value)),
        ),
        color_row("Active color", &shadow_color, |s| Message::LayoutExtras(LayoutExtrasMessage::SetShadowColor(s))),
        color_row("Inactive color", &shadow_inactive_color, |s| Message::LayoutExtras(LayoutExtrasMessage::SetShadowInactiveColor(s))),
        subsection_header("Tab Indicator"),
        info_text("Visual indicator for tabbed windows."),
        toggle_row(
            "Enable tab indicator",
            "Show indicator for tabbed windows",
            tab_enabled,
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorEnabled(value)),
        ),
        toggle_row(
            "Hide when single tab",
            "Don't show indicator when only one tab",
            tab_hide_single,
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorHideWhenSingleTab(value)),
        ),
        toggle_row(
            "Place within column",
            "Position indicator inside the column",
            tab_within_column,
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorPlaceWithinColumn(value)),
        ),
        slider_row_int("Gap", "Distance from window", tab_gap, 0, 50, " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorGap(value))),
        slider_row_int("Width", "Indicator thickness", tab_width, 1, 50, " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorWidth(value))),
        slider_row_int("Length", "Indicator length (% of window)", tab_length, 10, 100, "%",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorLengthProportion(value as f32 / 100.0))),
        slider_row_int("Corner radius", "Rounded corners", tab_corner_radius, 0, 50, " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorCornerRadius(value))),
        slider_row_int("Gaps between tabs", "Space between tab indicators", tab_gaps_between, 0, 50, " px",
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorGapsBetweenTabs(value))),
        row![
            text("Position").size(14).width(Length::Fixed(200.0)),
            pick_list(
                vec![TabIndicatorPosition::Left, TabIndicatorPosition::Right, TabIndicatorPosition::Top, TabIndicatorPosition::Bottom],
                Some(tab_position),
                |value| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorPosition(value)),
            )
            .width(Length::Fixed(150.0)),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
        color_row("Active color", &tab_active_color, |s| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorActiveColor(s))),
        color_row("Inactive color", &tab_inactive_color, |s| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorInactiveColor(s))),
        color_row("Urgent color", &tab_urgent_color, |s| Message::LayoutExtras(LayoutExtrasMessage::SetTabIndicatorUrgentColor(s))),
        subsection_header("Insert Hint"),
        info_text("Visual feedback when inserting windows."),
        toggle_row(
            "Enable insert hint",
            "Show visual hint when inserting windows",
            hint_enabled,
            |value| Message::LayoutExtras(LayoutExtrasMessage::SetInsertHintEnabled(value)),
        ),
        color_row("Hint color", &hint_color, |s| Message::LayoutExtras(LayoutExtrasMessage::SetInsertHintColor(s))),
        subsection_header("Default Column Display"),
        info_text("How new columns display windows by default."),
        row![
            text("Display mode").size(14).width(Length::Fixed(200.0)),
            pick_list(
                vec![DefaultColumnDisplay::Normal, DefaultColumnDisplay::Tabbed],
                Some(default_display),
                |value| Message::LayoutExtras(LayoutExtrasMessage::SetDefaultColumnDisplay(value)),
            )
            .width(Length::Fixed(150.0)),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(iced::Length::Fill))
        .height(iced::Length::Fill)
        .into()
}

/// Create a color input row
fn color_row<F>(label: &str, hex: &str, msg_fn: F) -> Element<'static, Message>
where
    F: Fn(String) -> Message + 'static,
{
    let hex_owned = hex.to_string();

    // Parse color for preview swatch
    let parsed_color = crate::types::Color::from_hex(&hex_owned)
        .map(|c| iced::Color::from_rgba(
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            c.a as f32 / 255.0,
        ))
        .unwrap_or(iced::Color::from_rgb(0.5, 0.5, 0.5));

    row![
        text(label.to_string()).size(14).width(Length::Fixed(200.0)),
        container(text("").size(14))
            .width(24)
            .height(24)
            .style(move |_theme| {
                container::Style {
                    background: Some(iced::Background::Color(parsed_color)),
                    border: iced::Border {
                        color: iced::Color::from_rgba(0.5, 0.5, 0.5, 0.5),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }
            }),
        text_input("#RRGGBB", &hex_owned)
            .on_input(msg_fn)
            .padding(6)
            .font(fonts::MONO_FONT)
            .width(Length::Fixed(100.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}
