//! Key mapping utilities for converting Slint key events to niri keybinding format
//!
//! This module handles the translation from Slint's keyboard event model
//! to niri's key combination format (e.g., "Mod+Shift+Q", "XF86AudioMute").

use slint::platform::Key;

/// Map Slint's Key enum to niri-compatible key name
///
/// Returns None for keys that shouldn't be captured as the primary key
/// (e.g., modifier keys alone)
pub fn map_key_to_niri(key: &Key) -> Option<&'static str> {
    match key {
        // Function keys
        Key::F1 => Some("F1"),
        Key::F2 => Some("F2"),
        Key::F3 => Some("F3"),
        Key::F4 => Some("F4"),
        Key::F5 => Some("F5"),
        Key::F6 => Some("F6"),
        Key::F7 => Some("F7"),
        Key::F8 => Some("F8"),
        Key::F9 => Some("F9"),
        Key::F10 => Some("F10"),
        Key::F11 => Some("F11"),
        Key::F12 => Some("F12"),
        Key::F13 => Some("F13"),
        Key::F14 => Some("F14"),
        Key::F15 => Some("F15"),
        Key::F16 => Some("F16"),
        Key::F17 => Some("F17"),
        Key::F18 => Some("F18"),
        Key::F19 => Some("F19"),
        Key::F20 => Some("F20"),
        Key::F21 => Some("F21"),
        Key::F22 => Some("F22"),
        Key::F23 => Some("F23"),
        Key::F24 => Some("F24"),

        // Navigation
        Key::UpArrow => Some("Up"),
        Key::DownArrow => Some("Down"),
        Key::LeftArrow => Some("Left"),
        Key::RightArrow => Some("Right"),
        Key::Home => Some("Home"),
        Key::End => Some("End"),
        Key::PageUp => Some("Page_Up"),
        Key::PageDown => Some("Page_Down"),

        // Editing keys
        Key::Backspace => Some("BackSpace"),
        Key::Delete => Some("Delete"),
        Key::Insert => Some("Insert"),
        Key::Return => Some("Return"),
        Key::Tab => Some("Tab"),
        Key::Backtab => Some("ISO_Left_Tab"),
        Key::Escape => Some("Escape"),

        // System keys
        Key::SysReq => Some("Print"),
        Key::ScrollLock => Some("Scroll_Lock"),
        Key::Pause => Some("Pause"),
        Key::Stop => Some("Cancel"),
        Key::Menu => Some("Menu"),

        // Modifier keys - these shouldn't be the primary key
        Key::Shift | Key::ShiftR => None,
        Key::Control | Key::ControlR => None,
        Key::Alt | Key::AltGr => None,
        Key::Meta | Key::MetaR => None,
        Key::CapsLock => None,

        // Other keys - not handled via Key enum
        // Character keys come through the text parameter instead
        _ => None,
    }
}

/// Map a character key (from text event) to niri format
///
/// Most keys come through as text in Slint. This handles standard
/// alphanumeric and symbol keys.
pub fn map_char_to_niri(c: char) -> Option<String> {
    match c {
        // Letters - use lowercase
        'a'..='z' => Some(c.to_string()),
        'A'..='Z' => Some(c.to_ascii_lowercase().to_string()),

        // Numbers
        '0'..='9' => Some(c.to_string()),

        // Common symbols - map to niri names
        '`' => Some("grave".to_string()),
        '~' => Some("asciitilde".to_string()),
        '!' => Some("exclam".to_string()),
        '@' => Some("at".to_string()),
        '#' => Some("numbersign".to_string()),
        '$' => Some("dollar".to_string()),
        '%' => Some("percent".to_string()),
        '^' => Some("asciicircum".to_string()),
        '&' => Some("ampersand".to_string()),
        '*' => Some("asterisk".to_string()),
        '(' => Some("parenleft".to_string()),
        ')' => Some("parenright".to_string()),
        '-' => Some("minus".to_string()),
        '_' => Some("underscore".to_string()),
        '=' => Some("equal".to_string()),
        '+' => Some("plus".to_string()),
        '[' => Some("bracketleft".to_string()),
        '{' => Some("braceleft".to_string()),
        ']' => Some("bracketright".to_string()),
        '}' => Some("braceright".to_string()),
        '\\' => Some("backslash".to_string()),
        '|' => Some("bar".to_string()),
        ';' => Some("semicolon".to_string()),
        ':' => Some("colon".to_string()),
        '\'' => Some("apostrophe".to_string()),
        '"' => Some("quotedbl".to_string()),
        ',' => Some("comma".to_string()),
        '<' => Some("less".to_string()),
        '.' => Some("period".to_string()),
        '>' => Some("greater".to_string()),
        '/' => Some("slash".to_string()),
        '?' => Some("question".to_string()),
        ' ' => Some("space".to_string()),

        _ => None,
    }
}

/// Build a niri key combination string from modifiers and key
///
/// # Arguments
/// * `key_name` - The base key name (e.g., "q", "F1", "Return")
/// * `has_mod` - Whether Mod (Super/Meta) is held
/// * `has_ctrl` - Whether Ctrl is held
/// * `has_alt` - Whether Alt is held
/// * `has_shift` - Whether Shift is held
///
/// # Returns
/// A key combo string like "Mod+Shift+q" or "Ctrl+Alt+Delete"
pub fn build_key_combo(
    key_name: &str,
    has_mod: bool,
    has_ctrl: bool,
    has_alt: bool,
    has_shift: bool,
) -> String {
    let mut parts = Vec::with_capacity(5);

    // Niri convention: Mod, Ctrl, Alt, Shift, then key
    if has_mod {
        parts.push("Mod");
    }
    if has_ctrl {
        parts.push("Ctrl");
    }
    if has_alt {
        parts.push("Alt");
    }
    if has_shift {
        parts.push("Shift");
    }
    parts.push(key_name);

    parts.join("+")
}

/// Check if a captured key combo is valid for niri
///
/// A valid combo typically requires at least one modifier for regular keys,
/// or can be a standalone special key like XF86 keys.
pub fn is_valid_combo(combo: &str) -> bool {
    if combo.is_empty() {
        return false;
    }

    // XF86 keys can stand alone
    if combo.starts_with("XF86") {
        return true;
    }

    // Check if it has at least one modifier
    let has_modifier = combo.contains("Mod+")
        || combo.contains("Ctrl+")
        || combo.contains("Alt+")
        || combo.contains("Shift+")
        || combo.contains("Super+");

    // Single character keys need a modifier
    let parts: Vec<&str> = combo.split('+').collect();
    if let Some(last) = parts.last() {
        // Single character or short key names need modifiers
        if last.len() <= 2 && !has_modifier {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_key_combo() {
        assert_eq!(build_key_combo("q", true, false, false, false), "Mod+q");
        assert_eq!(
            build_key_combo("Return", true, false, false, true),
            "Mod+Shift+Return"
        );
        assert_eq!(
            build_key_combo("F1", false, true, true, false),
            "Ctrl+Alt+F1"
        );
        assert_eq!(
            build_key_combo("Delete", true, true, true, true),
            "Mod+Ctrl+Alt+Shift+Delete"
        );
    }

    #[test]
    fn test_map_char_to_niri() {
        assert_eq!(map_char_to_niri('a'), Some("a".to_string()));
        assert_eq!(map_char_to_niri('A'), Some("a".to_string()));
        assert_eq!(map_char_to_niri('5'), Some("5".to_string()));
        assert_eq!(map_char_to_niri('-'), Some("minus".to_string()));
        assert_eq!(map_char_to_niri(' '), Some("space".to_string()));
    }

    #[test]
    fn test_is_valid_combo() {
        assert!(is_valid_combo("Mod+q"));
        assert!(is_valid_combo("Ctrl+Alt+Delete"));
        assert!(is_valid_combo("XF86AudioMute"));
        assert!(!is_valid_combo("q")); // Single key without modifier
        assert!(!is_valid_combo("")); // Empty
    }
}
