//! Keybindings KDL generation
//!
//! Generates KDL configuration for keybindings managed by niri-settings.

use crate::config::models::{KeybindAction, Keybinding, KeybindingsSettings};

/// Generate keybindings.kdl content from settings.
///
/// Creates KDL configuration for all managed keybindings in the format:
/// ```kdl
/// binds {
///     Mod+Space hotkey-overlay-title="App Launcher" {
///         spawn "dmenu_run";
///     }
/// }
/// ```
///
/// # Arguments
/// * `settings` - The keybindings settings to convert
///
/// # Returns
/// A string containing valid KDL configuration for niri keybindings.
pub fn generate_keybindings_kdl(settings: &KeybindingsSettings) -> String {
    // Pre-allocate for typical keybindings config
    let mut content = String::with_capacity(2048);
    content.push_str("// Keybindings - managed by niri-settings-rust\n");
    content.push_str("// Edit these bindings in the niri-settings app\n\n");

    if settings.bindings.is_empty() {
        content.push_str("// No keybindings configured yet.\n");
        content.push_str("// Add keybindings using the niri-settings app.\n");
        return content;
    }

    content.push_str("binds {\n");

    for binding in &settings.bindings {
        content.push_str(&generate_keybinding(binding));
    }

    content.push_str("}\n");
    content
}

/// Generate KDL for a single keybinding
fn generate_keybinding(binding: &Keybinding) -> String {
    let mut line = String::with_capacity(256);

    // Indent + key combo
    line.push_str("    ");
    line.push_str(&binding.key_combo);

    // Optional properties on the same line
    if let Some(ref title) = binding.hotkey_overlay_title {
        line.push_str(&format!(
            " hotkey-overlay-title={}",
            quote_kdl_string(title)
        ));
    }

    if binding.allow_when_locked {
        line.push_str(" allow-when-locked=true");
    }

    if let Some(cooldown) = binding.cooldown_ms {
        line.push_str(&format!(" cooldown-ms={}", cooldown));
    }

    if binding.repeat {
        line.push_str(" repeat=true");
    }

    // Action block
    line.push_str(" {\n");
    line.push_str(&generate_action(&binding.action));
    line.push_str("    }\n");

    line
}

/// Generate KDL for a keybinding action
fn generate_action(action: &KeybindAction) -> String {
    match action {
        KeybindAction::Spawn(args) => {
            let mut line = String::from("        spawn");
            for arg in args {
                line.push(' ');
                line.push_str(&quote_kdl_string(arg));
            }
            line.push_str(";\n");
            line
        }
        KeybindAction::NiriAction(name) => {
            format!("        {};\n", name)
        }
        KeybindAction::NiriActionWithArgs(name, args) => {
            // Actions with args use direct argument syntax
            // e.g., focus-workspace 1
            // e.g., set-column-width "-10%"
            let mut line = format!("        {}", name);
            for arg in args {
                line.push(' ');
                line.push_str(&quote_kdl_string(arg));
            }
            line.push_str(";\n");
            line
        }
    }
}

/// Quote a string for KDL format
///
/// KDL strings need to be quoted if they contain spaces or special characters.
/// This function always quotes for safety and escapes internal quotes.
fn quote_kdl_string(s: &str) -> String {
    // Escape backslashes and quotes, then wrap in quotes
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_keybindings() {
        let settings = KeybindingsSettings::default();
        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("No keybindings configured"));
        assert!(!kdl.contains("binds {"));
    }

    #[test]
    fn test_generate_spawn_keybinding() {
        let settings = KeybindingsSettings {
            bindings: vec![Keybinding {
                id: 1,
                key_combo: "Mod+Space".to_string(),
                hotkey_overlay_title: Some("App Launcher".to_string()),
                allow_when_locked: false,
                cooldown_ms: None,
                repeat: false,
                action: KeybindAction::Spawn(vec!["dmenu_run".to_string()]),
            }],
            ..Default::default()
        };

        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("binds {"));
        assert!(kdl.contains("Mod+Space"));
        assert!(kdl.contains("hotkey-overlay-title=\"App Launcher\""));
        assert!(kdl.contains("spawn \"dmenu_run\""));
    }

    #[test]
    fn test_generate_niri_action() {
        let settings = KeybindingsSettings {
            bindings: vec![Keybinding {
                id: 1,
                key_combo: "Mod+Q".to_string(),
                hotkey_overlay_title: None,
                allow_when_locked: false,
                cooldown_ms: None,
                repeat: false,
                action: KeybindAction::NiriAction("close-window".to_string()),
            }],
            ..Default::default()
        };

        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("Mod+Q"));
        assert!(kdl.contains("close-window;"));
    }

    #[test]
    fn test_generate_with_all_properties() {
        let settings = KeybindingsSettings {
            bindings: vec![Keybinding {
                id: 1,
                key_combo: "XF86AudioMute".to_string(),
                hotkey_overlay_title: Some("Mute".to_string()),
                allow_when_locked: true,
                cooldown_ms: Some(100),
                repeat: true,
                action: KeybindAction::Spawn(vec![
                    "wpctl".to_string(),
                    "set-mute".to_string(),
                    "@DEFAULT_AUDIO_SINK@".to_string(),
                    "toggle".to_string(),
                ]),
            }],
            ..Default::default()
        };

        let kdl = generate_keybindings_kdl(&settings);
        assert!(kdl.contains("XF86AudioMute"));
        assert!(kdl.contains("allow-when-locked=true"));
        assert!(kdl.contains("cooldown-ms=100"));
        assert!(kdl.contains("repeat=true"));
        assert!(kdl.contains("spawn \"wpctl\" \"set-mute\""));
    }

    #[test]
    fn test_quote_kdl_string() {
        assert_eq!(quote_kdl_string("simple"), "\"simple\"");
        assert_eq!(quote_kdl_string("with space"), "\"with space\"");
        assert_eq!(quote_kdl_string("with\"quote"), "\"with\\\"quote\"");
        assert_eq!(quote_kdl_string("path\\to\\file"), "\"path\\\\to\\\\file\"");
    }
}
