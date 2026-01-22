//! Storage for application preferences
//!
//! Preferences are app-specific settings (like UI theme) that are not part
//! of niri's configuration.

use crate::config::models::PreferencesSettings;

/// Generate KDL content for preferences
pub fn generate_preferences_kdl(prefs: &PreferencesSettings) -> String {
    let mut lines = vec![
        "// Application preferences (not part of niri config)".to_string(),
        "// This file stores settings specific to niri-settings application".to_string(),
        "".to_string(),
        "preferences {".to_string(),
    ];

    // Theme
    lines.push(format!("    theme \"{}\"", prefs.theme));

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
        };

        let kdl = generate_preferences_kdl(&prefs);

        assert!(kdl.contains("preferences {"));
        assert!(kdl.contains("theme \"NiriAmber\""));
    }

    #[test]
    fn test_generate_preferences_kdl_catppuccin() {
        let prefs = PreferencesSettings {
            theme: "CatppuccinMocha".to_string(),
        };

        let kdl = generate_preferences_kdl(&prefs);

        assert!(kdl.contains("theme \"CatppuccinMocha\""));
    }
}
