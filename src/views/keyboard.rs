//! Keyboard settings view — neon modal style

use iced::widget::{column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

use super::widgets::{info_text, toggle_row};
use crate::config::models::KeyboardSettings;
use crate::messages::{KeyboardMessage, Message};
use crate::theme::{fonts, neon};

/// Creates the keyboard settings view (styled for modal display)
pub fn view(settings: &KeyboardSettings) -> Element<'_, Message> {
    let xkb_layout = settings.xkb_layout.clone();
    let xkb_variant = settings.xkb_variant.clone();
    let xkb_model = settings.xkb_model.clone();
    let xkb_rules = settings.xkb_rules.clone();
    let xkb_options = settings.xkb_options.clone();
    let xkb_file = settings.xkb_file.clone();
    let track_layout = settings.track_layout.clone();

    let content = column![
        // ── 2-COLUMN: LAYOUT | REPEAT ──
        row![
            // Left: Keyboard Layout
            column![
                modal_section("⌨", "KEYBOARD LAYOUT", neon::SECONDARY),
                info_text("Configure layout using XKB settings."),
                Space::new().height(4),
                styled_text_input("XKB LAYOUT", "e.g., us, de, fr", &xkb_layout, |v| {
                    Message::Keyboard(KeyboardMessage::SetXkbLayout(v))
                }),
                styled_text_input("XKB VARIANT", "e.g., dvorak, colemak", &xkb_variant, |v| {
                    Message::Keyboard(KeyboardMessage::SetXkbVariant(v))
                }),
                styled_text_input("XKB MODEL", "e.g., pc105", &xkb_model, |v| {
                    Message::Keyboard(KeyboardMessage::SetXkbModel(v))
                }),
                styled_text_input("XKB RULES", "e.g., evdev", &xkb_rules, |v| {
                    Message::Keyboard(KeyboardMessage::SetXkbRules(v))
                }),
                styled_text_input(
                    "XKB OPTIONS",
                    "e.g., compose:ralt, caps:escape",
                    &xkb_options,
                    |v| Message::Keyboard(KeyboardMessage::SetXkbOptions(v))
                ),
                styled_text_input(
                    "XKB FILE",
                    "Path to .xkb keymap (overrides above)",
                    &xkb_file,
                    |v| Message::Keyboard(KeyboardMessage::SetXkbFile(v))
                ),
            ]
            .spacing(6)
            .width(Length::FillPortion(1)),
            // Right: Key Repeat + Options
            column![
                modal_section("⟳", "KEY REPEAT", neon::PRIMARY),
                Space::new().height(4),
                styled_slider_int(
                    "REPEAT DELAY",
                    &format!("{}ms", settings.repeat_delay),
                    100..=2000,
                    settings.repeat_delay,
                    |v| Message::Keyboard(KeyboardMessage::SetRepeatDelay(v)),
                ),
                styled_slider_int(
                    "REPEAT RATE",
                    &format!("{}/sec", settings.repeat_rate),
                    1..=100,
                    settings.repeat_rate,
                    |v| Message::Keyboard(KeyboardMessage::SetRepeatRate(v)),
                ),
                Space::new().height(12),
                modal_section("▦", "OPTIONS", neon::TERTIARY),
                Space::new().height(4),
                styled_text_input("TRACK LAYOUT", "global or window", &track_layout, |v| {
                    Message::Keyboard(KeyboardMessage::SetTrackLayout(v))
                }),
                container(toggle_row(
                    "Enable NumLock",
                    "Start with NumLock active on launch",
                    settings.numlock,
                    |v| Message::Keyboard(KeyboardMessage::SetNumlock(v)),
                ),)
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

fn styled_slider_int<'a>(
    label: &'a str,
    display_value: &str,
    range: std::ops::RangeInclusive<i32>,
    value: i32,
    on_slide: impl Fn(i32) -> Message + 'a,
) -> Element<'a, Message> {
    let display_owned = display_value.to_string();
    container(
        column![
            row![
                text(label)
                    .size(10)
                    .font(fonts::UI_FONT_SEMIBOLD)
                    .color(neon::OUTLINE_VARIANT),
                Space::new().width(Length::Fill),
                text(display_owned)
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
