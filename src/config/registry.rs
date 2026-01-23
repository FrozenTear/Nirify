//! Config file registry - single source of truth for config file mappings
//!
//! This module provides a central registry for all config files managed by
//! Nirify. It eliminates duplication of file paths across ConfigPaths,
//! ConfigHealthReport, loader, and storage modules.

use std::path::{Path, PathBuf};

/// All config file types managed by Nirify
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigFile {
    // Core settings
    Appearance,
    Behavior,

    // Input settings
    Keyboard,
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,

    // Display & visual
    Outputs,
    Animations,
    Cursor,
    Overview,

    // Workspaces
    Workspaces,

    // Keybindings
    Keybindings,

    // Advanced settings
    LayoutExtras,
    Gestures,
    LayerRules,
    WindowRules,
    Misc,
    Startup,
    Environment,
    Debug,
    SwitchEvents,
    RecentWindows,
}

impl ConfigFile {
    /// All config files in the order they should be processed
    pub const ALL: &'static [ConfigFile] = &[
        Self::Appearance,
        Self::Behavior,
        Self::Keyboard,
        Self::Mouse,
        Self::Touchpad,
        Self::Trackpoint,
        Self::Trackball,
        Self::Tablet,
        Self::Touch,
        Self::Outputs,
        Self::Animations,
        Self::Cursor,
        Self::Overview,
        Self::Workspaces,
        Self::Keybindings,
        Self::LayoutExtras,
        Self::Gestures,
        Self::LayerRules,
        Self::WindowRules,
        Self::Misc,
        Self::Startup,
        Self::Environment,
        Self::Debug,
        Self::SwitchEvents,
        Self::RecentWindows,
    ];

    /// Files included in config health checks (excludes keybindings which is loaded from niri config)
    pub const HEALTH_CHECK: &'static [ConfigFile] = &[
        Self::Appearance,
        Self::Behavior,
        Self::Keyboard,
        Self::Mouse,
        Self::Touchpad,
        Self::Animations,
        Self::Cursor,
        Self::Overview,
        Self::Outputs,
        Self::LayoutExtras,
        Self::Gestures,
        Self::Misc,
        Self::Workspaces,
        Self::LayerRules,
        Self::WindowRules,
        Self::Startup,
        Self::Environment,
        Self::Debug,
        Self::SwitchEvents,
        Self::RecentWindows,
    ];

    /// Relative path from managed_dir
    pub fn relative_path(&self) -> &'static str {
        match self {
            // Core
            Self::Appearance => "appearance.kdl",
            Self::Behavior => "behavior.kdl",

            // Input (in input/ subdirectory)
            Self::Keyboard => "input/keyboard.kdl",
            Self::Mouse => "input/mouse.kdl",
            Self::Touchpad => "input/touchpad.kdl",
            Self::Trackpoint => "input/trackpoint.kdl",
            Self::Trackball => "input/trackball.kdl",
            Self::Tablet => "input/tablet.kdl",
            Self::Touch => "input/touch.kdl",

            // Display & visual
            Self::Outputs => "outputs.kdl",
            Self::Animations => "animations.kdl",
            Self::Cursor => "cursor.kdl",
            Self::Overview => "overview.kdl",

            // Workspaces
            Self::Workspaces => "workspaces.kdl",

            // Keybindings
            Self::Keybindings => "keybindings.kdl",

            // Advanced (in advanced/ subdirectory)
            Self::LayoutExtras => "advanced/layout-extras.kdl",
            Self::Gestures => "advanced/gestures.kdl",
            Self::LayerRules => "advanced/layer-rules.kdl",
            Self::WindowRules => "advanced/window-rules.kdl",
            Self::Misc => "advanced/misc.kdl",
            Self::Startup => "advanced/startup.kdl",
            Self::Environment => "advanced/environment.kdl",
            Self::Debug => "advanced/debug.kdl",
            Self::SwitchEvents => "advanced/switch-events.kdl",
            Self::RecentWindows => "advanced/recent-windows.kdl",
        }
    }

    /// File name only (without directory path)
    pub fn file_name(&self) -> &'static str {
        match self {
            Self::Appearance => "appearance.kdl",
            Self::Behavior => "behavior.kdl",
            Self::Keyboard => "keyboard.kdl",
            Self::Mouse => "mouse.kdl",
            Self::Touchpad => "touchpad.kdl",
            Self::Trackpoint => "trackpoint.kdl",
            Self::Trackball => "trackball.kdl",
            Self::Tablet => "tablet.kdl",
            Self::Touch => "touch.kdl",
            Self::Outputs => "outputs.kdl",
            Self::Animations => "animations.kdl",
            Self::Cursor => "cursor.kdl",
            Self::Overview => "overview.kdl",
            Self::Workspaces => "workspaces.kdl",
            Self::Keybindings => "keybindings.kdl",
            Self::LayoutExtras => "layout-extras.kdl",
            Self::Gestures => "gestures.kdl",
            Self::LayerRules => "layer-rules.kdl",
            Self::WindowRules => "window-rules.kdl",
            Self::Misc => "misc.kdl",
            Self::Startup => "startup.kdl",
            Self::Environment => "environment.kdl",
            Self::Debug => "debug.kdl",
            Self::SwitchEvents => "switch-events.kdl",
            Self::RecentWindows => "recent-windows.kdl",
        }
    }

    /// Get the full path for this config file
    pub fn full_path(&self, managed_dir: &Path) -> PathBuf {
        managed_dir.join(self.relative_path())
    }

    /// Look up a ConfigFile by its file name
    pub fn from_file_name(name: &str) -> Option<Self> {
        Self::ALL.iter().find(|f| f.file_name() == name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_files_count() {
        assert_eq!(ConfigFile::ALL.len(), 25);
    }

    #[test]
    fn test_relative_paths() {
        assert_eq!(ConfigFile::Appearance.relative_path(), "appearance.kdl");
        assert_eq!(ConfigFile::Keyboard.relative_path(), "input/keyboard.kdl");
        assert_eq!(
            ConfigFile::LayoutExtras.relative_path(),
            "advanced/layout-extras.kdl"
        );
    }

    #[test]
    fn test_from_file_name() {
        assert_eq!(
            ConfigFile::from_file_name("appearance.kdl"),
            Some(ConfigFile::Appearance)
        );
        assert_eq!(
            ConfigFile::from_file_name("layout-extras.kdl"),
            Some(ConfigFile::LayoutExtras)
        );
        assert_eq!(ConfigFile::from_file_name("nonexistent.kdl"), None);
    }

    #[test]
    fn test_full_path() {
        let managed_dir = Path::new("/home/user/.config/niri/Nirify");
        assert_eq!(
            ConfigFile::Keyboard.full_path(managed_dir),
            PathBuf::from("/home/user/.config/niri/Nirify/input/keyboard.kdl")
        );
    }
}
