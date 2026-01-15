//! Helper functions for KDL generation
//!
//! Contains shared utilities for converting Rust types to KDL string representations.

use crate::types::{AccelProfile, ScrollMethod};
use std::borrow::Cow;

/// Check if a character requires escaping for KDL
fn needs_escape(c: char) -> bool {
    matches!(c, '\\' | '"' | '\n' | '\t' | '\r') || c.is_control()
}

/// Escape special characters in strings for safe KDL insertion.
///
/// This function escapes characters that could cause injection vulnerabilities
/// or malformed KDL when user-provided strings are inserted into configuration.
///
/// Handles:
/// - Backslash, quotes, newlines, tabs, carriage returns (standard escapes)
/// - Null bytes and other control characters (0x00-0x1F) are replaced with
///   Unicode replacement character to prevent malformed output
///
/// Uses `Cow` to avoid allocation when no escaping is needed (most common case).
///
/// # Arguments
/// * `s` - The string to escape
///
/// # Returns
/// A `Cow<str>` - borrowed if no escaping needed, owned if characters were escaped.
pub fn escape_kdl_string(s: &str) -> Cow<'_, str> {
    // Fast path: check if any escaping is needed
    if !s.chars().any(needs_escape) {
        return Cow::Borrowed(s);
    }

    // Slow path: escape using iterator adapters (no per-char Vec allocation)
    let mut result = String::with_capacity(s.len() + 8); // Reserve extra for escapes
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            // Replace null and other control characters with replacement character
            c if c.is_control() => result.push('\u{FFFD}'),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

/// Convert AccelProfile to KDL string representation.
pub fn accel_profile_to_kdl(profile: AccelProfile) -> &'static str {
    match profile {
        AccelProfile::Adaptive => "adaptive",
        AccelProfile::Flat => "flat",
    }
}

/// Convert ScrollMethod to KDL string representation.
pub fn scroll_method_to_kdl(method: ScrollMethod) -> &'static str {
    match method {
        ScrollMethod::TwoFinger => "two-finger",
        ScrollMethod::Edge => "edge",
        ScrollMethod::OnButtonDown => "on-button-down",
        ScrollMethod::NoScroll => "no-scroll",
    }
}

/// Convert TapButtonMap to KDL string representation.
pub fn tap_button_map_to_kdl(map: crate::types::TapButtonMap) -> &'static str {
    match map {
        crate::types::TapButtonMap::LeftRightMiddle => "left-right-middle",
        crate::types::TapButtonMap::LeftMiddleRight => "left-middle-right",
    }
}

/// Convert ClickMethod to KDL string representation.
pub fn click_method_to_kdl(method: crate::types::ClickMethod) -> &'static str {
    match method {
        crate::types::ClickMethod::ButtonAreas => "button-areas",
        crate::types::ClickMethod::Clickfinger => "clickfinger",
    }
}

/// Write common input device settings (shared between mouse and touchpad).
///
/// Writes boolean flags and numeric settings that are common to both input devices.
pub fn write_common_input_settings(
    content: &mut String,
    natural_scroll: bool,
    left_handed: bool,
    middle_emulation: bool,
    accel_speed: f64,
    accel_profile: AccelProfile,
    scroll_factor: f64,
) {
    if natural_scroll {
        content.push_str("        natural-scroll\n");
    }
    if left_handed {
        content.push_str("        left-handed\n");
    }
    if middle_emulation {
        content.push_str("        middle-emulation\n");
    }
    content.push_str(&format!("        accel-speed {:.2}\n", accel_speed));
    content.push_str(&format!(
        "        accel-profile \"{}\"\n",
        accel_profile_to_kdl(accel_profile)
    ));
    content.push_str(&format!("        scroll-factor {:.2}\n", scroll_factor));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_kdl_string_no_escape_needed() {
        let input = "hello world";
        let result = escape_kdl_string(input);
        // Should return borrowed string (no allocation)
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_escape_kdl_string_backslash() {
        let result = escape_kdl_string(r"path\to\file");
        assert_eq!(result, r"path\\to\\file");
    }

    #[test]
    fn test_escape_kdl_string_quotes() {
        let result = escape_kdl_string(r#"say "hello""#);
        assert_eq!(result, r#"say \"hello\""#);
    }

    #[test]
    fn test_escape_kdl_string_newlines() {
        let result = escape_kdl_string("line1\nline2");
        assert_eq!(result, "line1\\nline2");
    }

    #[test]
    fn test_escape_kdl_string_tabs() {
        let result = escape_kdl_string("col1\tcol2");
        assert_eq!(result, "col1\\tcol2");
    }

    #[test]
    fn test_escape_kdl_string_carriage_return() {
        let result = escape_kdl_string("line1\rline2");
        assert_eq!(result, "line1\\rline2");
    }

    #[test]
    fn test_escape_kdl_string_control_characters() {
        // Null byte should be replaced with replacement character
        let result = escape_kdl_string("hello\0world");
        assert_eq!(result, "hello\u{FFFD}world");
    }

    #[test]
    fn test_escape_kdl_string_mixed() {
        let result = escape_kdl_string("say \"hi\"\nbye\\end");
        assert_eq!(result, "say \\\"hi\\\"\\nbye\\\\end");
    }

    #[test]
    fn test_escape_kdl_string_empty() {
        let result = escape_kdl_string("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_escape_kdl_string_unicode() {
        // Unicode should pass through unchanged
        let result = escape_kdl_string("héllo 世界 🎉");
        assert_eq!(result, "héllo 世界 🎉");
    }
}
