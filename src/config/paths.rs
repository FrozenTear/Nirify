use super::error::ConfigError;
use super::registry::ConfigFile;
use crate::constants::CONFIG_DIR_NAME;
use std::path::PathBuf;

/// Holds all paths for config files
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    /// User's niri config: ~/.config/niri/config.kdl
    pub niri_config: PathBuf,

    /// Our managed directory: ~/.config/niri/niri-settings/
    pub managed_dir: PathBuf,

    /// Input subdirectory: ~/.config/niri/niri-settings/input/
    pub input_dir: PathBuf,

    /// Advanced subdirectory: ~/.config/niri/niri-settings/advanced/
    pub advanced_dir: PathBuf,

    /// Backup directory: ~/.config/niri/niri-settings/.backup/
    pub backup_dir: PathBuf,

    // Core config files
    pub main_kdl: PathBuf,
    pub appearance_kdl: PathBuf,
    pub behavior_kdl: PathBuf,

    // Input config files
    pub keyboard_kdl: PathBuf,
    pub mouse_kdl: PathBuf,
    pub touchpad_kdl: PathBuf,
    pub trackpoint_kdl: PathBuf,
    pub trackball_kdl: PathBuf,
    pub tablet_kdl: PathBuf,
    pub touch_kdl: PathBuf,

    // Display & visual
    pub outputs_kdl: PathBuf,
    pub animations_kdl: PathBuf,
    pub cursor_kdl: PathBuf,
    pub overview_kdl: PathBuf,

    // Workspaces
    pub workspaces_kdl: PathBuf,

    // Keybindings
    pub keybindings_kdl: PathBuf,

    // Advanced
    pub layout_extras_kdl: PathBuf,
    pub gestures_kdl: PathBuf,
    pub layer_rules_kdl: PathBuf,
    pub window_rules_kdl: PathBuf,
    pub misc_kdl: PathBuf,
    pub startup_kdl: PathBuf,
    pub environment_kdl: PathBuf,
    pub debug_kdl: PathBuf,
    pub switch_events_kdl: PathBuf,
    pub recent_windows_kdl: PathBuf,

    /// App preferences file: ~/.config/niri/niri-settings/app-prefs.json
    pub preferences_json: PathBuf,
}

impl ConfigPaths {
    /// Create new ConfigPaths based on XDG config directory
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::ConfigDirNotFound)?;

        let niri_dir = config_dir.join("niri");
        let managed_dir = niri_dir.join(CONFIG_DIR_NAME);
        let input_dir = managed_dir.join("input");
        let advanced_dir = managed_dir.join("advanced");
        let backup_dir = managed_dir.join(".backup");

        // Build paths - using references to avoid clones
        let main_kdl = managed_dir.join("main.kdl");
        let appearance_kdl = managed_dir.join("appearance.kdl");
        let behavior_kdl = managed_dir.join("behavior.kdl");
        let outputs_kdl = managed_dir.join("outputs.kdl");
        let animations_kdl = managed_dir.join("animations.kdl");
        let cursor_kdl = managed_dir.join("cursor.kdl");
        let overview_kdl = managed_dir.join("overview.kdl");

        let keyboard_kdl = input_dir.join("keyboard.kdl");
        let mouse_kdl = input_dir.join("mouse.kdl");
        let touchpad_kdl = input_dir.join("touchpad.kdl");
        let trackpoint_kdl = input_dir.join("trackpoint.kdl");
        let trackball_kdl = input_dir.join("trackball.kdl");
        let tablet_kdl = input_dir.join("tablet.kdl");
        let touch_kdl = input_dir.join("touch.kdl");

        let workspaces_kdl = managed_dir.join("workspaces.kdl");
        let keybindings_kdl = managed_dir.join("keybindings.kdl");

        let layout_extras_kdl = advanced_dir.join("layout-extras.kdl");
        let gestures_kdl = advanced_dir.join("gestures.kdl");
        let layer_rules_kdl = advanced_dir.join("layer-rules.kdl");
        let window_rules_kdl = advanced_dir.join("window-rules.kdl");
        let misc_kdl = advanced_dir.join("misc.kdl");
        let startup_kdl = advanced_dir.join("startup.kdl");
        let environment_kdl = advanced_dir.join("environment.kdl");
        let debug_kdl = advanced_dir.join("debug.kdl");
        let switch_events_kdl = advanced_dir.join("switch-events.kdl");
        let recent_windows_kdl = advanced_dir.join("recent-windows.kdl");
        let preferences_json = managed_dir.join("app-prefs.json");

        Ok(Self {
            niri_config: niri_dir.join("config.kdl"),
            managed_dir,
            input_dir,
            advanced_dir,
            backup_dir,

            // Core
            main_kdl,
            appearance_kdl,
            behavior_kdl,

            // Input
            keyboard_kdl,
            mouse_kdl,
            touchpad_kdl,
            trackpoint_kdl,
            trackball_kdl,
            tablet_kdl,
            touch_kdl,

            // Display
            outputs_kdl,
            animations_kdl,
            cursor_kdl,
            overview_kdl,

            // Workspaces
            workspaces_kdl,

            // Keybindings
            keybindings_kdl,

            // Advanced
            layout_extras_kdl,
            gestures_kdl,
            layer_rules_kdl,
            window_rules_kdl,
            misc_kdl,
            startup_kdl,
            environment_kdl,
            debug_kdl,
            switch_events_kdl,
            recent_windows_kdl,
            preferences_json,
        })
    }

    /// Create all necessary directories if they don't exist
    pub fn ensure_directories(&self) -> Result<(), ConfigError> {
        for dir in [
            &self.managed_dir,
            &self.input_dir,
            &self.advanced_dir,
            &self.backup_dir,
        ] {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| ConfigError::create_dir_error(dir, e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Check if this is the first run (our directory doesn't exist)
    pub fn is_first_run(&self) -> bool {
        !self.managed_dir.exists()
    }

    /// Get the full path for a config file using the registry
    ///
    /// This is the preferred method for getting config file paths as it uses
    /// the centralized ConfigFile registry.
    pub fn path_for(&self, file: ConfigFile) -> PathBuf {
        file.full_path(&self.managed_dir)
    }

    /// Get path for a config file by name (legacy method)
    ///
    /// Returns the full path for a config file given its file name.
    /// Prefer using `path_for(ConfigFile)` for new code.
    pub fn path_for_file(&self, file_name: &str) -> Option<PathBuf> {
        ConfigFile::from_file_name(file_name).map(|f| self.path_for(f))
    }

    /// Check if the user's config.kdl contains our include line
    ///
    /// Uses streaming search to avoid reading entire file for large configs.
    pub fn has_include_line(&self) -> bool {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = match File::open(&self.niri_config) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.contains("niri-settings/main.kdl") {
                return true;
            }
        }
        false
    }
}

// Note: Default trait intentionally not implemented for ConfigPaths
// because it requires fallible operations (directory resolution).
// Use ConfigPaths::new() instead and handle the error appropriately.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_paths_creation() {
        let paths = ConfigPaths::new().unwrap();
        assert!(paths.managed_dir.ends_with("niri-settings"));
        assert!(paths.main_kdl.ends_with("main.kdl"));
        assert!(paths.keyboard_kdl.ends_with("keyboard.kdl"));
    }
}
