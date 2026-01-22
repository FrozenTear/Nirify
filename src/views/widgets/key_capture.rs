//! Key capture widget for keybinding configuration
//!
//! Allows users to press key combinations which are then captured and formatted.

use iced::keyboard::{Key, Modifiers};
use iced::widget::{button, column, container, row, text};
use iced::{Border, Color as IcedColor, Element, Length};

/// State for key capture widget
#[derive(Debug, Clone, PartialEq)]
pub enum KeyCaptureState {
    Idle(String),        // Displaying current binding
    Capturing,           // Waiting for key press
    Captured(String),    // Just captured, showing preview
}

impl Default for KeyCaptureState {
    fn default() -> Self {
        Self::Idle(String::new())
    }
}

/// Messages for key capture interactions
#[derive(Debug, Clone)]
pub enum KeyCaptureMessage {
    StartCapture,
    KeyPressed {
        key: Key,
        modifiers: Modifiers,
    },
    CancelCapture,
    ConfirmCapture,
}

/// Creates a key capture widget for keybinding configuration
///
/// Shows the current binding and allows clicking to capture a new one.
pub fn key_capture_row<'a, Message: Clone + 'a>(
    label: &'a str,
    description: &'a str,
    state: &KeyCaptureState,
    on_change: impl Fn(KeyCaptureMessage) -> Message + 'a + Copy,
) -> Element<'a, Message> {
    // Use owned String to avoid memory leaks (text() accepts Into<String>)
    let display_text: String = match state {
        KeyCaptureState::Idle(binding) if binding.is_empty() => {
            "Click to set...".to_string()
        }
        KeyCaptureState::Idle(binding) => {
            binding.clone()
        }
        KeyCaptureState::Capturing => {
            "Press any key... (ESC to cancel)".to_string()
        }
        KeyCaptureState::Captured(binding) => {
            format!("{} (click Confirm)", binding)
        }
    };

    let is_capturing = matches!(state, KeyCaptureState::Capturing);

    let capture_button = if is_capturing {
        button(text(display_text.clone()))
            .on_press(on_change(KeyCaptureMessage::CancelCapture))
            .padding([8, 16])
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(IcedColor::from_rgb(0.8, 0.6, 0.2))),
                text_color: IcedColor::from_rgb(0.0, 0.0, 0.0),
                border: Border {
                    color: IcedColor::from_rgb(0.9, 0.7, 0.3),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
    } else {
        button(text(display_text))
            .on_press(on_change(KeyCaptureMessage::StartCapture))
            .padding([8, 16])
    };

    let mut content = row![
        // Left side: Label and description
        column![
            text(label).size(16),
            text(description).size(12).color([0.7, 0.7, 0.7]),
        ]
        .spacing(4)
        .width(Length::Fill),
        // Right side: Capture button
        capture_button,
    ]
    .spacing(20)
    .align_y(iced::Alignment::Center);

    // Add confirm/cancel buttons when captured
    if matches!(state, KeyCaptureState::Captured(_)) {
        content = content.push(
            button(text("✓"))
                .on_press(on_change(KeyCaptureMessage::ConfirmCapture))
                .padding([8, 12])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.2, 0.8, 0.2))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    ..Default::default()
                })
        );
        content = content.push(
            button(text("✗"))
                .on_press(on_change(KeyCaptureMessage::CancelCapture))
                .padding([8, 12])
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(IcedColor::from_rgb(0.8, 0.2, 0.2))),
                    text_color: IcedColor::from_rgb(1.0, 1.0, 1.0),
                    ..Default::default()
                })
        );
    }

    container(content)
        .padding(12)
        .into()
}

/// Format a key combination into a human-readable string
pub fn format_key_combination(key: &Key, modifiers: &Modifiers) -> String {
    let mut parts = Vec::new();

    if modifiers.control() {
        parts.push("Ctrl");
    }
    if modifiers.alt() {
        parts.push("Alt");
    }
    if modifiers.shift() {
        parts.push("Shift");
    }
    if modifiers.logo() {
        parts.push("Super");
    }

    // Format the key
    let key_str = match key {
        Key::Named(named) => format!("{:?}", named),
        Key::Character(c) => c.to_uppercase(),
        Key::Unidentified => return "Unknown".to_string(),
    };

    parts.push(&key_str);

    parts.join("+")
}

/// Helper to check if a key should be ignored (modifiers only)
pub fn is_modifier_only(key: &Key) -> bool {
    matches!(
        key,
        Key::Named(iced::keyboard::key::Named::Control)
            | Key::Named(iced::keyboard::key::Named::Alt)
            | Key::Named(iced::keyboard::key::Named::Shift)
            | Key::Named(iced::keyboard::key::Named::Super)
    )
}
