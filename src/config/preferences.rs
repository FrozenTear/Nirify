//! App preferences (UI theme, etc.)
//!
//! Stores niri-settings app preferences separately from niri configuration.
//! These are settings for the settings app itself, not for niri.

use crate::ui::theme::ThemePreset;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// App preferences stored separately from niri settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    /// UI theme for the settings app
    #[serde(default)]
    pub theme: ThemePreset,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: ThemePreset::CatppuccinMocha,
        }
    }
}

/// Old format from Slint version for backward compatibility
#[derive(Debug, Deserialize)]
struct LegacyPreferences {
    #[serde(default)]
    theme_flavor: u8,
}

/// Load app preferences from the given path
/// Handles both new format and legacy Slint format
pub fn load_preferences(prefs_path: &Path) -> AppPreferences {
    match fs::read_to_string(prefs_path) {
        Ok(content) => {
            // Try new format first
            if let Ok(prefs) = serde_json::from_str::<AppPreferences>(&content) {
                return prefs;
            }

            // Try legacy format (from Slint version)
            if let Ok(legacy) = serde_json::from_str::<LegacyPreferences>(&content) {
                let theme = match legacy.theme_flavor {
                    0 => ThemePreset::CatppuccinLatte,  // Light theme
                    1 => ThemePreset::Nord,             // Frappe -> Nord (similar blue tones)
                    2 => ThemePreset::TokyoNight,       // Macchiato -> TokyoNight
                    _ => ThemePreset::CatppuccinMocha,  // Mocha (default dark)
                };
                log::info!("Migrated legacy theme_flavor {} to {:?}", legacy.theme_flavor, theme);
                return AppPreferences { theme };
            }

            log::warn!("Failed to parse preferences, using defaults");
            AppPreferences::default()
        }
        Err(_) => {
            log::debug!("No preferences file found, using defaults");
            AppPreferences::default()
        }
    }
}

/// Save app preferences to the given path
pub fn save_preferences(prefs_path: &Path, prefs: &AppPreferences) -> Result<(), std::io::Error> {
    let content = serde_json::to_string_pretty(prefs)?;

    // Ensure parent directory exists
    if let Some(parent) = prefs_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(prefs_path, content)?;
    log::debug!("Saved preferences to {:?}", prefs_path);
    Ok(())
}
