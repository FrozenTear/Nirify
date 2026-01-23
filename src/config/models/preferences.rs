//! Application preferences (not niri config)
//!
//! Settings that are specific to this application, not part of niri's configuration.

/// Application preferences
#[derive(Debug, Clone, PartialEq)]
pub struct PreferencesSettings {
    /// Selected UI theme ("NiriAmber" or "CatppuccinMocha")
    pub theme: String,
    /// Whether the settings app should float (true) or tile (false)
    pub float_settings_app: bool,
}

impl Default for PreferencesSettings {
    fn default() -> Self {
        Self {
            theme: "NiriAmber".to_string(),
            float_settings_app: true, // Float by default
        }
    }
}
