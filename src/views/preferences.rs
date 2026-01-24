//! Preferences settings view
//!
//! Application preferences that control how Nirify behaves.

use iced::widget::{column, container, row, text, text_input, toggler};
use iced::{Alignment, Element, Length};

use super::widgets::*;
use crate::messages::{Message, PreferencesMessage};

/// Creates the preferences settings view
pub fn view(float_settings_app: bool, show_search_bar: bool, search_hotkey: &str) -> Element<'static, Message> {
    let mut content = column![
        page_title("Preferences"),
        info_text("Configure how this settings application behaves."),
    ]
    .spacing(4);

    // Window Behavior Section
    content = content.push(spacer(16.0));
    content = content.push(subsection_header("Window Behavior"));

    // Float toggle
    content = content.push(
        container(
            row![
                column![
                    text("Float Settings Window").size(14),
                    text("When enabled, this settings app floats above other windows instead of tiling normally.")
                        .size(12)
                        .color([0.6, 0.6, 0.6]),
                ]
                .spacing(4)
                .width(Length::Fill),
                toggler(float_settings_app)
                    .on_toggle(|v| Message::Preferences(PreferencesMessage::SetFloatSettingsApp(v))),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .padding([16, 20])
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.15, 0.15, 0.15, 0.5,
            ))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    // Navigation Section
    content = content.push(spacer(16.0));
    content = content.push(subsection_header("Navigation"));

    // Show search bar toggle
    content = content.push(
        container(
            row![
                column![
                    text("Show Search Bar").size(14),
                    text("When disabled, use the keyboard shortcut to open search as a popup.")
                        .size(12)
                        .color([0.6, 0.6, 0.6]),
                ]
                .spacing(4)
                .width(Length::Fill),
                toggler(show_search_bar)
                    .on_toggle(|v| Message::Preferences(PreferencesMessage::SetShowSearchBar(v))),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .padding([16, 20])
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.15, 0.15, 0.15, 0.5,
            ))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    // Search hotkey (need to own the string for the closure)
    let search_hotkey_owned = search_hotkey.to_string();
    content = content.push(spacer(8.0));
    content = content.push(
        container(
            row![
                column![
                    text("Search Keyboard Shortcut").size(14),
                    text("Keyboard shortcut to open search (e.g., Ctrl+K, Ctrl+/). Leave empty to disable.")
                        .size(12)
                        .color([0.6, 0.6, 0.6]),
                ]
                .spacing(4)
                .width(Length::Fill),
                text_input("Ctrl+K", &search_hotkey_owned)
                    .on_input(|v| Message::Preferences(PreferencesMessage::SetSearchHotkey(v)))
                    .padding(8)
                    .width(Length::Fixed(120.0)),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .padding([16, 20])
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.15, 0.15, 0.15, 0.5,
            ))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    content = content.push(spacer(24.0));
    content = content.push(subsection_header("About"));
    content = content.push(
        container(
            column![
                row![
                    text("Niri Settings").size(16),
                    text("v0.2.0").size(14).color([0.5, 0.5, 0.5]),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                spacer(8.0),
                text("A native settings application for the niri Wayland compositor.")
                    .size(13)
                    .color([0.6, 0.6, 0.6]),
                spacer(4.0),
                text("Built with iced 0.14")
                    .size(12)
                    .color([0.5, 0.5, 0.5]),
            ]
            .spacing(4)
            .padding([16, 20])
        )
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.15, 0.15, 0.15, 0.5,
            ))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
    );

    iced::widget::scrollable(
        iced::widget::container(content)
            .padding(20)
            .width(iced::Length::Fill)
    )
    .height(iced::Length::Fill)
    .into()
}
