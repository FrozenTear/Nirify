//! Keyboard settings view

use iced::widget::{column, container, scrollable, text, text_input};
use iced::{Element, Length};

use super::widgets::{page_title, info_text, section_header, slider_row_int, spacer, card};
use crate::config::models::KeyboardSettings;
use crate::messages::{KeyboardMessage, Message};
use crate::theme::muted_text_container;

/// Helper to create a text input row with string value (owned)
/// Takes the string by value for keyboard settings
fn keyboard_text_input<'a>(
    label: &'static str,
    description: &'static str,
    value: String,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(16),
        container(text(description).size(12)).style(muted_text_container),
        text_input("", &value).on_input(on_change).padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Creates the keyboard settings view
pub fn view(settings: &KeyboardSettings) -> Element<'_, Message> {
    let xkb_layout = settings.xkb_layout.clone();
    let repeat_rate = settings.repeat_rate;
    let repeat_delay = settings.repeat_delay;
    let track_layout = settings.track_layout.clone();

    let content = column![
        page_title("Keyboard Layout"),
        info_text(
            "Configure keyboard layout using XKB settings. The layout determines key mapping and language support."
        ),
        card(column![
            keyboard_text_input(
                "XKB layout",
                "Keyboard layout (e.g., us, de, fr)",
                xkb_layout,
                |value| Message::Keyboard(KeyboardMessage::SetXkbLayout(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Key Repeat"),
        info_text(
            "Controls how quickly keys repeat when held down. Delay is how long before repeat starts, rate is how fast keys repeat."
        ),
        card(column![
            slider_row_int(
                "Repeat delay",
                "Delay before key repeat starts in milliseconds",
                repeat_delay,
                100,
                2000,
                " ms",
                |value| Message::Keyboard(KeyboardMessage::SetRepeatDelay(value)),
            ),
            slider_row_int(
                "Repeat rate",
                "Number of key repeats per second",
                repeat_rate,
                1,
                100,
                " /sec",
                |value| Message::Keyboard(KeyboardMessage::SetRepeatRate(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        section_header("Layout Tracking"),
        info_text(
            "Configure how keyboard layouts are tracked across windows. Options: 'global' (one layout for all windows), 'window' (per-window layout)."
        ),
        card(column![
            keyboard_text_input(
                "Track layout",
                "Layout tracking mode (global or window)",
                track_layout,
                |value| Message::Keyboard(KeyboardMessage::SetTrackLayout(value)),
            ),
        ].spacing(0).width(Length::Fill)),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20).width(Length::Fill))
        .height(Length::Fill)
        .into()
}
