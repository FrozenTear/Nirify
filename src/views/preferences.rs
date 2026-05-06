//! Preferences settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{info_text, toggle_row};
use crate::messages::{Message, PreferencesMessage};
use crate::theme::{fonts, neon};

/// Creates the preferences settings view
pub fn view(
    float_settings_app: bool,
    show_search_bar: bool,
    search_hotkey: &str,
) -> Element<'static, Message> {
    let search_hotkey_owned = search_hotkey.to_string();

    let content = column![
        // ── 2-COLUMN: WINDOW | NAVIGATION ──
        row![
            // Left: Window Behavior + About
            column![
                modal_section("\u{2699}", "WINDOW BEHAVIOR", neon::SECONDARY),
                info_text("Configure how this settings application behaves."),
                Space::new().height(4),
                container(toggle_row(
                    "Float Settings Window",
                    "Float above other windows instead of tiling normally",
                    float_settings_app,
                    |v| Message::Preferences(PreferencesMessage::SetFloatSettingsApp(v)),
                ),)
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(16),
                modal_section("\u{2139}", "ABOUT", neon::TERTIARY),
                Space::new().height(4),
                container(
                    column![
                        row![
                            text("Niri Settings")
                                .size(14)
                                .font(fonts::UI_FONT_SEMIBOLD)
                                .color(neon::ON_SURFACE),
                            text(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .size(12)
                                .font(fonts::MONO_FONT)
                                .color(neon::OUTLINE),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        Space::new().height(6),
                        text("A native settings application for the niri Wayland compositor.")
                            .size(12)
                            .color(neon::ON_SURFACE_VARIANT),
                        Space::new().height(4),
                        text("Built with iced 0.14").size(11).color(neon::OUTLINE),
                    ]
                    .spacing(2)
                    .padding(4),
                )
                .padding(8)
                .style(crate::theme::card_style),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Navigation
            column![
                modal_section("\u{26A1}", "NAVIGATION", neon::PRIMARY),
                Space::new().height(4),
                container(toggle_row(
                    "Show Search Bar",
                    "When disabled, use the keyboard shortcut to open search as a popup",
                    show_search_bar,
                    |v| Message::Preferences(PreferencesMessage::SetShowSearchBar(v)),
                ),)
                .padding(8)
                .style(crate::theme::card_style),
                Space::new().height(8),
                styled_text_input(
                    "SEARCH KEYBOARD SHORTCUT",
                    "e.g., Ctrl+K, Ctrl+/",
                    &search_hotkey_owned,
                    |v| Message::Preferences(PreferencesMessage::SetSearchHotkey(v)),
                ),
                container(
                    text("Leave empty to disable the keyboard shortcut.")
                        .size(11)
                        .color(neon::OUTLINE),
                )
                .padding([4, 12]),
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
