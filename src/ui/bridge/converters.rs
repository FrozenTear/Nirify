//! Type conversion utilities for UI-backend communication
//!
//! This module provides functions to convert between Slint types
//! and our internal types.

use crate::types::Color;
use crate::KeyPart;

/// Convert Slint color to our Color type
#[inline]
pub fn slint_color_to_color(color: slint::Color) -> Color {
    Color {
        r: color.red(),
        g: color.green(),
        b: color.blue(),
        a: color.alpha(),
    }
}

/// Convert our Color type to Slint color
#[inline]
pub fn color_to_slint_color(color: &Color) -> slint::Color {
    slint::Color::from_argb_u8(color.a, color.r, color.g, color.b)
}

/// Known modifier key names used in niri keybindings
const MODIFIER_KEYS: &[&str] = &[
    "Mod",
    "Super",
    "Ctrl",
    "Control",
    "Alt",
    "Shift",
    "Meta",
    "ISO_Level3_Shift",
    "ISO_Level5_Shift",
];

/// Check if a key name is a modifier
#[inline]
fn is_modifier(key: &str) -> bool {
    MODIFIER_KEYS.iter().any(|&m| m.eq_ignore_ascii_case(key))
}

/// Parse a key combination string into parts for badge display
///
/// Takes a combo like "Mod+Shift+Q" and returns a Vec of KeyPart structs,
/// each tagged as modifier or regular key.
///
/// Examples:
/// - "Mod+Q" -> [KeyPart("Mod", true), KeyPart("Q", false)]
/// - "Ctrl+Shift+F1" -> [KeyPart("Ctrl", true), KeyPart("Shift", true), KeyPart("F1", false)]
/// - "XF86AudioMute" -> [KeyPart("XF86AudioMute", false)]
pub fn parse_key_combo_parts(combo: &str) -> Vec<KeyPart> {
    if combo.is_empty() {
        return Vec::new();
    }

    combo
        .split('+')
        .map(|part| {
            let trimmed = part.trim();
            KeyPart {
                text: trimmed.into(),
                is_modifier: is_modifier(trimmed),
            }
        })
        .collect()
}

/// Convert a Vec of KeyPart to a Slint ModelRc for UI binding
pub fn key_parts_to_model(parts: Vec<KeyPart>) -> slint::ModelRc<KeyPart> {
    slint::ModelRc::new(slint::VecModel::from(parts))
}
