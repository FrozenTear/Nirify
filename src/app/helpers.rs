//! Helper functions for the App module
//!
//! Contains utility functions used across multiple handlers.

use crate::types::{Color, ColorOrGradient, Gradient};
use crate::views::widgets::GradientPickerMessage;

/// Formats a key press event into a niri-compatible key combo string
/// e.g., "Mod+Shift+Return" or "Ctrl+Alt+Delete"
pub fn format_key_combo(key: &iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) -> String {
    use iced::keyboard::{Key, key::Named};

    // Skip if this is just a modifier key by itself
    let is_modifier_key = matches!(
        key,
        Key::Named(Named::Shift | Named::Control | Named::Alt | Named::Super)
    );
    if is_modifier_key {
        return String::new();
    }

    let mut parts = Vec::new();

    // Add modifiers in niri's expected order
    // Note: niri uses "Mod" for Super/Logo key
    if modifiers.logo() {
        parts.push("Mod");
    }
    if modifiers.control() {
        parts.push("Ctrl");
    }
    if modifiers.alt() {
        parts.push("Alt");
    }
    if modifiers.shift() {
        parts.push("Shift");
    }

    // Add the key name
    let key_name = match key {
        Key::Named(named) => match named {
            Named::Enter => "Return",
            Named::Tab => "Tab",
            Named::Space => "space",
            Named::Backspace => "BackSpace",
            Named::Delete => "Delete",
            Named::Escape => "Escape",
            Named::Home => "Home",
            Named::End => "End",
            Named::PageUp => "Page_Up",
            Named::PageDown => "Page_Down",
            Named::ArrowUp => "Up",
            Named::ArrowDown => "Down",
            Named::ArrowLeft => "Left",
            Named::ArrowRight => "Right",
            Named::F1 => "F1",
            Named::F2 => "F2",
            Named::F3 => "F3",
            Named::F4 => "F4",
            Named::F5 => "F5",
            Named::F6 => "F6",
            Named::F7 => "F7",
            Named::F8 => "F8",
            Named::F9 => "F9",
            Named::F10 => "F10",
            Named::F11 => "F11",
            Named::F12 => "F12",
            Named::Insert => "Insert",
            Named::PrintScreen => "Print",
            Named::ScrollLock => "Scroll_Lock",
            Named::Pause => "Pause",
            Named::AudioVolumeUp => "XF86AudioRaiseVolume",
            Named::AudioVolumeDown => "XF86AudioLowerVolume",
            Named::AudioVolumeMute => "XF86AudioMute",
            Named::MediaPlayPause => "XF86AudioPlay",
            Named::MediaStop => "XF86AudioStop",
            Named::MediaTrackNext => "XF86AudioNext",
            Named::MediaTrackPrevious => "XF86AudioPrev",
            Named::BrightnessUp => "XF86MonBrightnessUp",
            Named::BrightnessDown => "XF86MonBrightnessDown",
            _ => return String::new(), // Unknown named key
        },
        Key::Character(c) => {
            // For character keys, uppercase for consistent display
            let s = c.as_str();
            if s.len() == 1 {
                // Single character - uppercase it for display
                let upper = s.to_uppercase();
                if parts.is_empty() {
                    return upper;
                } else {
                    return format!("{}+{}", parts.join("+"), upper);
                }
            } else {
                return String::new();
            }
        }
        Key::Unidentified => return String::new(),
    };

    parts.push(key_name);
    parts.join("+")
}

impl super::App {
    /// Helper to apply GradientPickerMessage to a ColorOrGradient field
    pub(super) fn apply_gradient_message(&self, target: &mut ColorOrGradient, msg: GradientPickerMessage) {
        match msg {
            GradientPickerMessage::ToggleSolidGradient(is_gradient) => {
                *target = if is_gradient {
                    // Convert to gradient
                    match target {
                        ColorOrGradient::Color(color) => {
                            ColorOrGradient::Gradient(Gradient {
                                from: *color,
                                to: *color,
                                angle: 0,
                                ..Default::default()
                            })
                        }
                        ColorOrGradient::Gradient(_) => target.clone(),
                    }
                } else {
                    // Convert to solid color
                    match target {
                        ColorOrGradient::Color(_) => target.clone(),
                        ColorOrGradient::Gradient(gradient) => {
                            ColorOrGradient::Color(gradient.from)
                        }
                    }
                };
            }
            GradientPickerMessage::SetFromColor(hex) => {
                if let Some(color) = Color::from_hex(&hex) {
                    match target {
                        ColorOrGradient::Color(c) => *c = color,
                        ColorOrGradient::Gradient(g) => g.from = color,
                    }
                }
            }
            GradientPickerMessage::SetToColor(hex) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    if let Some(color) = Color::from_hex(&hex) {
                        gradient.to = color;
                    }
                }
            }
            GradientPickerMessage::SetAngle(angle) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.angle = angle;
                }
            }
            GradientPickerMessage::SetColorSpace(color_space) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.color_space = color_space;
                }
            }
            GradientPickerMessage::SetRelativeTo(relative_to) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.relative_to = relative_to;
                }
            }
            GradientPickerMessage::SetHueInterpolation(hue_interp) => {
                if let ColorOrGradient::Gradient(gradient) = target {
                    gradient.hue_interpolation = Some(hue_interp);
                }
            }
        }
    }
}
