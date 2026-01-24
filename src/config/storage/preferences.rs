//! Storage for application preferences
//!
//! Preferences are app-specific settings (like UI theme) that are not part
//! of niri's configuration.

use crate::config::models::PreferencesSettings;

/// Generate KDL content for preferences
pub fn generate_preferences_kdl(prefs: &PreferencesSettings) -> String {
    let mut lines = vec![
        "// Application preferences (not part of niri config)".to_string(),
        "// This file stores settings specific to Nirify application".to_string(),
        "".to_string(),
        "preferences {".to_string(),
    ];

    // Theme
    lines.push(format!("    theme \"{}\"", prefs.theme));

    // Float settings app (whether this app should float or tile)
    lines.push(format!("    float-settings-app {}", prefs.float_settings_app));

    // Show search bar in navigation
    lines.push(format!("    show-search-bar {}", prefs.show_search_bar));

    // Search hotkey (keyboard shortcut)
    lines.push(format!("    search-hotkey \"{}\"", prefs.search_hotkey));

    lines.push("}".to_string());
    lines.push("".to_string()); // Trailing newline

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_preferences_kdl() {
        let prefs = PreferencesSettings {
            theme: "NiriAmber".to_string(),
            float_settings_app: true,
            show_search_bar: true,
            search_hotkey: "Ctrl+K".to_string(),
        };

        let kdl = generate_preferences_kdl(&prefs);

        assert!(kdl.contains("preferences {"));
        assert!(kdl.contains("theme \"NiriAmber\""));
        assert!(kdl.contains("float-settings-app true"));
        assert!(kdl.contains("show-search-bar true"));
        assert!(kdl.contains("search-hotkey \"Ctrl+K\""));
    }

    #[test]
    fn test_generate_preferences_kdl_catppuccin() {
        let prefs = PreferencesSettings {
            theme: "CatppuccinMocha".to_string(),
            float_settings_app: false,
            show_search_bar: false,
            search_hotkey: "Ctrl+/".to_string(),
        };

        let kdl = generate_preferences_kdl(&prefs);

        assert!(kdl.contains("theme \"CatppuccinMocha\""));
        assert!(kdl.contains("float-settings-app false"));
        assert!(kdl.contains("show-search-bar false"));
        assert!(kdl.contains("search-hotkey \"Ctrl+/\""));
    }
}
