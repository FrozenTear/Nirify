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
    /// Whether to show the search bar in navigation
    pub show_search_bar: bool,
    /// Keyboard shortcut for opening search (e.g., "Ctrl+K", "Ctrl+/", or empty to disable)
    pub search_hotkey: String,
}

impl Default for PreferencesSettings {
    fn default() -> Self {
        Self {
            theme: "NiriAmber".to_string(),
            float_settings_app: true, // Float by default
            show_search_bar: true,    // Show search bar by default
            search_hotkey: "Ctrl+K".to_string(),
        }
    }
}
