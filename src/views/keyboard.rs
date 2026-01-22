//! Keyboard settings view

use iced::widget::{column, container, scrollable, text, text_input};
use iced::Element;

use super::widgets::*;
use crate::config::models::KeyboardSettings;
use crate::messages::{KeyboardMessage, Message};

/// Helper to create a text input row with string value
/// Takes the string by value and returns it as part of the Element
fn text_input_row<'a, Message: Clone + 'a>(
    label: &'static str,
    description: &'static str,
    value: String,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    column![
        text(label).size(16),
        text(description).size(12).color([0.7, 0.7, 0.7]),
        // text_input internally stores what it needs
        text_input("", &value).on_input(on_change).padding(8),
    ]
    .spacing(6)
    .padding(12)
    .into()
}

/// Creates the keyboard settings view
pub fn view(settings: KeyboardSettings) -> Element<'static, Message> {
    let xkb_layout = settings.xkb_layout;
    let repeat_rate = settings.repeat_rate;
    let repeat_delay = settings.repeat_delay;
    let track_layout = settings.track_layout;

    let content = column![
        section_header("Keyboard Layout"),
        info_text(
            "Configure keyboard layout using XKB settings. The layout determines key mapping and language support."
        ),
        text_input_row(
            "XKB layout",
            "Keyboard layout (e.g., us, de, fr)",
            xkb_layout,
            |value| Message::Keyboard(KeyboardMessage::SetXkbLayout(value)),
        ),
        spacer(16.0),

        section_header("Key Repeat"),
        info_text(
            "Controls how quickly keys repeat when held down. Delay is how long before repeat starts, rate is how fast keys repeat."
        ),
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
        spacer(16.0),

        section_header("Layout Tracking"),
        info_text(
            "Configure how keyboard layouts are tracked across windows. Options: 'global' (one layout for all windows), 'window' (per-window layout)."
        ),
        text_input_row(
            "Track layout",
            "Layout tracking mode (global or window)",
            track_layout,
            |value| Message::Keyboard(KeyboardMessage::SetTrackLayout(value)),
        ),
        spacer(32.0),
    ]
    .spacing(4);

    scrollable(container(content).padding(20)).into()
}
