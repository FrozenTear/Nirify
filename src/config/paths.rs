use super::error::ConfigError;
use super::registry::ConfigFile;
use crate::constants::CONFIG_DIR_NAME;
use std::path::PathBuf;

/// Holds all paths for config files
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    /// User's niri config: ~/.config/niri/config.kdl
    pub niri_config: PathBuf,

    /// Our managed directory: ~/.config/niri/nirify/
    pub managed_dir: PathBuf,

    /// Input subdirectory: ~/.config/niri/nirify/input/
    pub input_dir: PathBuf,

    /// Advanced subdirectory: ~/.config/niri/nirify/advanced/
    pub advanced_dir: PathBuf,

    /// Backup directory: ~/.config/niri/nirify/.backup/
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
    pub preferences_kdl: PathBuf,
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
        let preferences_kdl = advanced_dir.join("preferences.kdl");

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
            preferences_kdl,
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

            // Set restrictive permissions on backup directory (owner only)
            // Backups may contain sensitive configuration data
            #[cfg(unix)]
            if dir == &self.backup_dir {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o700);
                if let Err(e) = std::fs::set_permissions(dir, perms) {
                    log::warn!("Could not set backup directory permissions: {}", e);
                }
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
            if line.contains("nirify/main.kdl") {
                return true;
            }
        }
        false
    }

    /// Add the include line to the user's config.kdl
    ///
    /// Creates a backup of the original file and adds the include line at the end.
    /// Returns Ok(()) if successful or if the include line already exists.
    pub fn add_include_line(&self) -> Result<(), ConfigError> {
        use std::fs;
        use std::io::Write;

        // Check if include already exists
        if self.has_include_line() {
            log::info!("Include line already present in config.kdl");
            return Ok(());
        }

        // Create backup
        if self.niri_config.exists() {
            let backup_name = format!(
                "config.kdl.backup.{}",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );
            let backup_path = self.backup_dir.join(backup_name);
            fs::create_dir_all(&self.backup_dir)?;
            fs::copy(&self.niri_config, &backup_path)?;
            log::info!("Created backup at {:?}", backup_path);
        }

        // Read existing content
        let existing_content = if self.niri_config.exists() {
            fs::read_to_string(&self.niri_config)?
        } else {
            String::new()
        };

        // Append include line (relative path works regardless of XDG_CONFIG_HOME)
        let include_line = format!(
            "\n// Managed by Nirify - do not remove this line\ninclude \"{}/main.kdl\"\n",
            crate::constants::CONFIG_DIR_NAME
        );

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.niri_config)?;

        file.write_all(existing_content.as_bytes())?;
        file.write_all(include_line.as_bytes())?;

        log::info!("Added include line to {:?}", self.niri_config);
        Ok(())
    }
}

/// Create fallback ConfigPaths for error state display.
///
/// This is used when normal initialization fails, allowing the app to
/// display an error dialog instead of crashing. The paths point to a
/// temporary directory and should not be used for actual configuration.
impl Default for ConfigPaths {
    fn default() -> Self {
        let temp = std::env::temp_dir().join("nirify-error-fallback");
        let input_dir = temp.join("input");
        let advanced_dir = temp.join("advanced");

        Self {
            niri_config: temp.join("config.kdl"),
            managed_dir: temp.clone(),
            input_dir: input_dir.clone(),
            advanced_dir: advanced_dir.clone(),
            backup_dir: temp.join(".backup"),

            // Core
            main_kdl: temp.join("main.kdl"),
            appearance_kdl: temp.join("appearance.kdl"),
            behavior_kdl: temp.join("behavior.kdl"),

            // Input
            keyboard_kdl: input_dir.join("keyboard.kdl"),
            mouse_kdl: input_dir.join("mouse.kdl"),
            touchpad_kdl: input_dir.join("touchpad.kdl"),
            trackpoint_kdl: input_dir.join("trackpoint.kdl"),
            trackball_kdl: input_dir.join("trackball.kdl"),
            tablet_kdl: input_dir.join("tablet.kdl"),
            touch_kdl: input_dir.join("touch.kdl"),

            // Display
            outputs_kdl: temp.join("outputs.kdl"),
            animations_kdl: temp.join("animations.kdl"),
            cursor_kdl: temp.join("cursor.kdl"),
            overview_kdl: temp.join("overview.kdl"),

            // Workspaces
            workspaces_kdl: temp.join("workspaces.kdl"),

            // Keybindings
            keybindings_kdl: temp.join("keybindings.kdl"),

            // Advanced
            layout_extras_kdl: advanced_dir.join("layout-extras.kdl"),
            gestures_kdl: advanced_dir.join("gestures.kdl"),
            layer_rules_kdl: advanced_dir.join("layer-rules.kdl"),
            window_rules_kdl: advanced_dir.join("window-rules.kdl"),
            misc_kdl: advanced_dir.join("misc.kdl"),
            startup_kdl: advanced_dir.join("startup.kdl"),
            environment_kdl: advanced_dir.join("environment.kdl"),
            debug_kdl: advanced_dir.join("debug.kdl"),
            switch_events_kdl: advanced_dir.join("switch-events.kdl"),
            recent_windows_kdl: advanced_dir.join("recent-windows.kdl"),
            preferences_kdl: advanced_dir.join("preferences.kdl"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_paths_creation() {
        let paths = ConfigPaths::new().unwrap();
        assert!(paths.managed_dir.ends_with("nirify"));
        assert!(paths.main_kdl.ends_with("main.kdl"));
        assert!(paths.keyboard_kdl.ends_with("keyboard.kdl"));
    }
}
