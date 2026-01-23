//! Settings storage - writes KDL configuration files
//!
//! This module provides functions to save settings to the multi-file
//! KDL configuration structure managed by Nirify.
//!
//! # Module Structure
//!
//! - `helpers`: Shared utilities for KDL string conversion
//! - `gradient`: Gradient and color KDL generation
//! - `appearance`: Appearance settings (gaps, focus ring, borders)
//! - `behavior`: Behavior settings (focus follows mouse, etc.)
//! - `input`: Input device settings (keyboard, mouse, touchpad, etc.)
//! - `display`: Display settings (animations, cursor, outputs)
//! - `layout_extras`: Layout extras (shadow, tab indicator, insert hint)
//! - `gestures`: Gesture settings (hot corners, DND)
//! - `misc`: Miscellaneous settings
//! - `workspaces`: Named workspace settings
//! - `rules`: Window and layer rules
//! - `system`: Startup, environment, debug, switch events, recent windows

pub mod builder;

mod appearance;
mod behavior;
mod display;
mod gestures;
mod gradient;
mod helpers;
mod input;
mod keybindings;
mod layout_extras;
mod misc;
mod preferences;
mod rules;
mod system;
mod workspaces;

// Re-export public generators
pub use appearance::generate_appearance_kdl;
pub use behavior::{generate_behavior_kdl, generate_main_kdl};
pub use display::{
    generate_animations_kdl, generate_cursor_kdl, generate_outputs_kdl, generate_overview_kdl,
};
pub use gestures::generate_gestures_kdl;
pub use gradient::{color_or_gradient_to_kdl, gradient_to_kdl};
pub use input::{
    generate_keyboard_kdl, generate_mouse_kdl, generate_tablet_kdl, generate_touch_kdl,
    generate_touchpad_kdl, generate_trackball_kdl, generate_trackpoint_kdl,
};
pub use keybindings::generate_keybindings_kdl;
pub use layout_extras::generate_layout_extras_kdl;
pub use misc::generate_misc_kdl;
pub use preferences::generate_preferences_kdl;
pub use rules::{generate_layer_rules_kdl, generate_window_rules_kdl};
pub use system::{
    generate_debug_kdl, generate_environment_kdl, generate_recent_windows_kdl,
    generate_startup_kdl, generate_switch_events_kdl,
};
pub use workspaces::generate_workspaces_kdl;

use super::error::ConfigError;
use super::models::Settings;
use super::paths::ConfigPaths;
use anyhow::Context;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Strategy for writing configuration files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteStrategy {
    /// Atomic writes using temp file + rename (safe for updates)
    Atomic,
    /// Direct writes (simple for initial creation)
    Direct,
}

/// Atomically write content to a file using a temporary file and rename.
///
/// This function writes to a temporary file first, then atomically renames it
/// to the target path. This prevents file corruption if the process crashes
/// during writing.
///
/// # Security
///
/// Uses a unique temp filename with process ID and timestamp to prevent TOCTOU
/// race conditions. An attacker cannot predict the temp filename, making symlink
/// attacks infeasible.
///
/// # Arguments
/// * `path` - The target file path
/// * `content` - The content to write
///
/// # Returns
/// `Ok(())` on success, or an error if write or rename fails.
pub fn atomic_write(path: &Path, content: &str) -> anyhow::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate unique temp filename with process ID and nanosecond timestamp
    // to prevent TOCTOU attacks - attacker cannot predict the filename
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_extension = format!("tmp.{}.{}", std::process::id(), nanos);
    let temp_path = path.with_extension(temp_extension);

    // Use create_new to atomically create the file, failing if it already exists.
    // With our unique filename, this should never fail due to existing file.
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .with_context(|| format!("Failed to create temporary file {:?}", temp_path))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write temporary file {:?}", temp_path))?;

    // Ensure data is flushed to disk before rename
    file.sync_all()
        .with_context(|| format!("Failed to sync temporary file {:?}", temp_path))?;

    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", temp_path, path))?;

    // Set restrictive permissions (owner read/write only)
    // Config files may contain sensitive information
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        if let Err(e) = std::fs::set_permissions(path, perms) {
            log::warn!("Could not set file permissions on {:?}: {}", path, e);
        }
    }

    Ok(())
}

/// Write content to a file using the specified strategy.
fn write_config(path: &Path, content: &str, strategy: WriteStrategy) -> anyhow::Result<()> {
    match strategy {
        WriteStrategy::Atomic => atomic_write(path, content),
        WriteStrategy::Direct => fs::write(path, content).map_err(Into::into),
    }
    .with_context(|| format!("Failed to write {:?}", path))
}

/// Write all settings files using the specified strategy.
///
/// This is the unified function that writes all 19 config files.
fn write_all_settings(
    paths: &ConfigPaths,
    settings: &Settings,
    strategy: WriteStrategy,
) -> anyhow::Result<()> {
    paths.ensure_directories()?;

    // Main entry point
    write_config(&paths.main_kdl, &generate_main_kdl(), strategy)?;

    // Core settings
    write_config(
        &paths.appearance_kdl,
        &generate_appearance_kdl(&settings.appearance, &settings.behavior),
        strategy,
    )?;
    write_config(
        &paths.behavior_kdl,
        &generate_behavior_kdl(&settings.behavior),
        strategy,
    )?;

    // Input settings
    write_config(
        &paths.keyboard_kdl,
        &generate_keyboard_kdl(&settings.keyboard),
        strategy,
    )?;
    write_config(
        &paths.mouse_kdl,
        &generate_mouse_kdl(&settings.mouse),
        strategy,
    )?;
    write_config(
        &paths.touchpad_kdl,
        &generate_touchpad_kdl(&settings.touchpad),
        strategy,
    )?;
    write_config(
        &paths.trackpoint_kdl,
        &generate_trackpoint_kdl(&settings.trackpoint),
        strategy,
    )?;
    write_config(
        &paths.trackball_kdl,
        &generate_trackball_kdl(&settings.trackball),
        strategy,
    )?;
    write_config(
        &paths.tablet_kdl,
        &generate_tablet_kdl(&settings.tablet),
        strategy,
    )?;
    write_config(
        &paths.touch_kdl,
        &generate_touch_kdl(&settings.touch),
        strategy,
    )?;

    // Display settings
    write_config(
        &paths.outputs_kdl,
        &generate_outputs_kdl(&settings.outputs),
        strategy,
    )?;
    write_config(
        &paths.animations_kdl,
        &generate_animations_kdl(&settings.animations),
        strategy,
    )?;
    write_config(
        &paths.cursor_kdl,
        &generate_cursor_kdl(&settings.cursor),
        strategy,
    )?;
    write_config(
        &paths.overview_kdl,
        &generate_overview_kdl(&settings.overview),
        strategy,
    )?;

    // Workspaces
    write_config(
        &paths.workspaces_kdl,
        &generate_workspaces_kdl(&settings.workspaces),
        strategy,
    )?;

    // Keybindings
    write_config(
        &paths.keybindings_kdl,
        &generate_keybindings_kdl(&settings.keybindings),
        strategy,
    )?;

    // Advanced settings
    write_config(
        &paths.layout_extras_kdl,
        &generate_layout_extras_kdl(&settings.layout_extras),
        strategy,
    )?;
    write_config(
        &paths.layer_rules_kdl,
        &generate_layer_rules_kdl(&settings.layer_rules),
        strategy,
    )?;
    write_config(
        &paths.gestures_kdl,
        &generate_gestures_kdl(&settings.gestures),
        strategy,
    )?;
    write_config(
        &paths.misc_kdl,
        &generate_misc_kdl(&settings.miscellaneous),
        strategy,
    )?;
    write_config(
        &paths.window_rules_kdl,
        &generate_window_rules_kdl(&settings.window_rules, settings.preferences.float_settings_app),
        strategy,
    )?;
    write_config(
        &paths.startup_kdl,
        &generate_startup_kdl(&settings.startup),
        strategy,
    )?;
    write_config(
        &paths.environment_kdl,
        &generate_environment_kdl(&settings.environment),
        strategy,
    )?;
    write_config(
        &paths.debug_kdl,
        &generate_debug_kdl(&settings.debug),
        strategy,
    )?;
    write_config(
        &paths.switch_events_kdl,
        &generate_switch_events_kdl(&settings.switch_events),
        strategy,
    )?;
    write_config(
        &paths.recent_windows_kdl,
        &generate_recent_windows_kdl(&settings.recent_windows),
        strategy,
    )?;
    write_config(
        &paths.preferences_kdl,
        &generate_preferences_kdl(&settings.preferences),
        strategy,
    )?;

    Ok(())
}

/// Save all settings to KDL files
///
/// This function writes all settings to their respective KDL files.
/// Should be called after any settings change. Uses atomic writes to
/// prevent file corruption if the process crashes during writing.
pub fn save_settings(paths: &ConfigPaths, settings: &Settings) -> anyhow::Result<()> {
    write_all_settings(paths, settings, WriteStrategy::Atomic)
}

/// Save only the specified categories to KDL files
///
/// This function only writes config files for categories that have been
/// marked as dirty. This significantly reduces disk I/O when users make
/// frequent changes (e.g., dragging sliders).
///
/// # Arguments
/// * `paths` - The configuration paths structure
/// * `settings` - The settings to write
/// * `dirty` - Set of categories that need saving
///
/// # Returns
/// The number of files that were written.
pub fn save_dirty(
    paths: &ConfigPaths,
    settings: &Settings,
    dirty: &std::collections::HashSet<super::dirty::SettingsCategory>,
) -> anyhow::Result<usize> {
    use super::dirty::SettingsCategory;

    if dirty.is_empty() {
        return Ok(0);
    }

    paths.ensure_directories()?;

    let mut files_written = 0;
    let strategy = WriteStrategy::Atomic;

    for category in dirty {
        match category {
            SettingsCategory::Appearance => {
                // Appearance includes some behavior settings (struts)
                write_config(
                    &paths.appearance_kdl,
                    &generate_appearance_kdl(&settings.appearance, &settings.behavior),
                    strategy,
                )?;
            }
            SettingsCategory::Behavior => {
                write_config(
                    &paths.behavior_kdl,
                    &generate_behavior_kdl(&settings.behavior),
                    strategy,
                )?;
            }
            SettingsCategory::Keyboard => {
                write_config(
                    &paths.keyboard_kdl,
                    &generate_keyboard_kdl(&settings.keyboard),
                    strategy,
                )?;
            }
            SettingsCategory::Mouse => {
                write_config(
                    &paths.mouse_kdl,
                    &generate_mouse_kdl(&settings.mouse),
                    strategy,
                )?;
            }
            SettingsCategory::Touchpad => {
                write_config(
                    &paths.touchpad_kdl,
                    &generate_touchpad_kdl(&settings.touchpad),
                    strategy,
                )?;
            }
            SettingsCategory::Trackpoint => {
                write_config(
                    &paths.trackpoint_kdl,
                    &generate_trackpoint_kdl(&settings.trackpoint),
                    strategy,
                )?;
            }
            SettingsCategory::Trackball => {
                write_config(
                    &paths.trackball_kdl,
                    &generate_trackball_kdl(&settings.trackball),
                    strategy,
                )?;
            }
            SettingsCategory::Tablet => {
                write_config(
                    &paths.tablet_kdl,
                    &generate_tablet_kdl(&settings.tablet),
                    strategy,
                )?;
            }
            SettingsCategory::Touch => {
                write_config(
                    &paths.touch_kdl,
                    &generate_touch_kdl(&settings.touch),
                    strategy,
                )?;
            }
            SettingsCategory::Outputs => {
                write_config(
                    &paths.outputs_kdl,
                    &generate_outputs_kdl(&settings.outputs),
                    strategy,
                )?;
            }
            SettingsCategory::Animations => {
                write_config(
                    &paths.animations_kdl,
                    &generate_animations_kdl(&settings.animations),
                    strategy,
                )?;
            }
            SettingsCategory::Cursor => {
                write_config(
                    &paths.cursor_kdl,
                    &generate_cursor_kdl(&settings.cursor),
                    strategy,
                )?;
            }
            SettingsCategory::Overview => {
                write_config(
                    &paths.overview_kdl,
                    &generate_overview_kdl(&settings.overview),
                    strategy,
                )?;
            }
            SettingsCategory::Workspaces => {
                write_config(
                    &paths.workspaces_kdl,
                    &generate_workspaces_kdl(&settings.workspaces),
                    strategy,
                )?;
            }
            SettingsCategory::Keybindings => {
                write_config(
                    &paths.keybindings_kdl,
                    &generate_keybindings_kdl(&settings.keybindings),
                    strategy,
                )?;
            }
            SettingsCategory::LayoutExtras => {
                write_config(
                    &paths.layout_extras_kdl,
                    &generate_layout_extras_kdl(&settings.layout_extras),
                    strategy,
                )?;
            }
            SettingsCategory::Gestures => {
                write_config(
                    &paths.gestures_kdl,
                    &generate_gestures_kdl(&settings.gestures),
                    strategy,
                )?;
            }
            SettingsCategory::LayerRules => {
                write_config(
                    &paths.layer_rules_kdl,
                    &generate_layer_rules_kdl(&settings.layer_rules),
                    strategy,
                )?;
            }
            SettingsCategory::WindowRules => {
                write_config(
                    &paths.window_rules_kdl,
                    &generate_window_rules_kdl(&settings.window_rules, settings.preferences.float_settings_app),
                    strategy,
                )?;
            }
            SettingsCategory::Miscellaneous => {
                write_config(
                    &paths.misc_kdl,
                    &generate_misc_kdl(&settings.miscellaneous),
                    strategy,
                )?;
            }
            SettingsCategory::Startup => {
                write_config(
                    &paths.startup_kdl,
                    &generate_startup_kdl(&settings.startup),
                    strategy,
                )?;
            }
            SettingsCategory::Environment => {
                write_config(
                    &paths.environment_kdl,
                    &generate_environment_kdl(&settings.environment),
                    strategy,
                )?;
            }
            SettingsCategory::Debug => {
                write_config(
                    &paths.debug_kdl,
                    &generate_debug_kdl(&settings.debug),
                    strategy,
                )?;
            }
            SettingsCategory::SwitchEvents => {
                write_config(
                    &paths.switch_events_kdl,
                    &generate_switch_events_kdl(&settings.switch_events),
                    strategy,
                )?;
            }
            SettingsCategory::RecentWindows => {
                write_config(
                    &paths.recent_windows_kdl,
                    &generate_recent_windows_kdl(&settings.recent_windows),
                    strategy,
                )?;
            }
            SettingsCategory::Preferences => {
                write_config(
                    &paths.preferences_kdl,
                    &generate_preferences_kdl(&settings.preferences),
                    strategy,
                )?;
            }
        }
        files_written += 1;
    }

    Ok(files_written)
}

/// Save content to file with automatic backup.
///
/// If the target file already exists, creates a timestamped backup in the
/// specified backup directory before writing new content.
///
/// # Arguments
/// * `path` - The path to write the file to
/// * `content` - The content to write
/// * `backup_dir` - Directory to store backup files
///
/// # Returns
/// `Ok(())` on success, or an error if backup or write fails.
///
/// # Errors
/// Returns an error if:
/// - The backup directory cannot be written to
/// - The target file cannot be written
/// - The path has no valid filename
pub fn save_with_backup(path: &Path, content: &str, backup_dir: &Path) -> anyhow::Result<()> {
    // Atomically read existing content (combines exists check + read)
    if let Ok(existing_content) = fs::read(path) {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ConfigError::InvalidConfig("Path has no valid filename".to_string()))?;
        // Use microsecond precision (%.6f) to prevent filename collisions during rapid saves
        let timestamp = Local::now().format("%Y-%m-%dT%H-%M-%S%.6f");
        let backup_name = format!("{}.{}.bak", filename, timestamp);
        let backup_path = backup_dir.join(&backup_name);

        // Use atomic_write for backup (not fs::copy which has TOCTOU)
        atomic_write(&backup_path, &String::from_utf8_lossy(&existing_content))
            .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;
    }

    // Use atomic_write instead of fs::write
    atomic_write(path, content).with_context(|| format!("Failed to write config file {:?}", path))
}

/// Initialize all configuration files with the provided settings.
///
/// Creates all necessary directories and writes all KDL configuration files
/// based on the provided settings. This is typically called on first run
/// or when resetting configuration to defaults.
///
/// # Arguments
/// * `paths` - The configuration paths structure
/// * `settings` - The settings to write to files
///
/// # Returns
/// `Ok(())` on success, or an error if any file write fails.
///
/// # Errors
/// Returns an error if:
/// - Directory creation fails
/// - Any configuration file cannot be written
pub fn initialize_config_files(paths: &ConfigPaths, settings: &Settings) -> anyhow::Result<()> {
    write_all_settings(paths, settings, WriteStrategy::Direct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{AppearanceSettings, BehaviorSettings};
    use crate::types::Color;

    #[test]
    fn test_generate_main_kdl_contains_includes() {
        let content = generate_main_kdl();
        assert!(content.contains("include \"appearance.kdl\""));
        assert!(content.contains("include \"behavior.kdl\""));
        assert!(content.contains("include \"input/keyboard.kdl\""));
        assert!(content.contains("Nirify managed"));
    }

    #[test]
    fn test_generate_appearance_kdl_with_defaults() {
        let appearance = AppearanceSettings::default();
        let behavior = BehaviorSettings::default();
        let content = generate_appearance_kdl(&appearance, &behavior);

        // Should contain layout block
        assert!(content.contains("layout {"));
        // Should contain gaps (single value format)
        assert!(content.contains("gaps "));
        // Focus ring should be enabled by default
        assert!(content.contains("focus-ring {"));
        assert!(content.contains("active"));
    }

    #[test]
    fn test_generate_appearance_kdl_focus_ring_disabled() {
        let appearance = AppearanceSettings {
            focus_ring_enabled: false,
            ..Default::default()
        };
        let behavior = BehaviorSettings::default();
        let content = generate_appearance_kdl(&appearance, &behavior);

        // When focus ring is disabled, we don't output the block at all
        // (niri's default is no focus ring)
        assert!(!content.contains("focus-ring {"));
    }

    #[test]
    fn test_generate_appearance_kdl_with_struts() {
        let appearance = AppearanceSettings::default();
        let behavior = BehaviorSettings {
            strut_left: 50.0,
            strut_top: 30.0,
            ..Default::default()
        };
        let content = generate_appearance_kdl(&appearance, &behavior);

        assert!(content.contains("struts {"));
        assert!(content.contains("left 50"));
        assert!(content.contains("top 30"));
    }

    #[test]
    fn test_generate_appearance_kdl_corner_radius() {
        let appearance = AppearanceSettings {
            corner_radius: 16.0,
            ..Default::default()
        };
        let behavior = BehaviorSettings::default();
        let content = generate_appearance_kdl(&appearance, &behavior);

        assert!(content.contains("window-rule {"));
        assert!(content.contains("geometry-corner-radius 16"));
    }

    #[test]
    fn test_generate_behavior_kdl_defaults() {
        let behavior = BehaviorSettings::default();
        let content = generate_behavior_kdl(&behavior);

        // Default settings shouldn't include these
        assert!(!content.contains("focus-follows-mouse"));
        assert!(!content.contains("warp-mouse-to-focus"));
    }

    #[test]
    fn test_generate_behavior_kdl_with_options() {
        let behavior = BehaviorSettings {
            focus_follows_mouse: true,
            warp_mouse_to_focus: crate::types::WarpMouseMode::CenterXY,
            ..Default::default()
        };
        let content = generate_behavior_kdl(&behavior);

        assert!(content.contains("focus-follows-mouse"));
        assert!(content.contains("warp-mouse-to-focus"));
    }

    #[test]
    fn test_generate_keyboard_kdl() {
        use crate::config::models::KeyboardSettings;
        let keyboard = KeyboardSettings::default();
        let content = generate_keyboard_kdl(&keyboard);

        assert!(content.contains("input {"));
        assert!(content.contains("keyboard {"));
        assert!(content.contains("xkb {"));
        assert!(content.contains("layout \"us\""));
        assert!(content.contains("repeat-delay"));
        assert!(content.contains("repeat-rate"));
    }

    #[test]
    fn test_generate_animations_kdl_enabled() {
        use crate::config::models::AnimationSettings;
        let animations = AnimationSettings::default();
        let content = generate_animations_kdl(&animations);

        assert!(content.contains("animations {"));
        // Should NOT contain "off" when enabled
        assert!(!content.contains("off"));
    }

    #[test]
    fn test_generate_animations_kdl_disabled() {
        use crate::config::models::AnimationSettings;
        let animations = AnimationSettings {
            enabled: false,
            ..Default::default()
        };
        let content = generate_animations_kdl(&animations);

        assert!(content.contains("off"));
    }

    #[test]
    fn test_generate_cursor_kdl() {
        use crate::config::models::CursorSettings;
        let cursor = CursorSettings::default();
        let content = generate_cursor_kdl(&cursor);

        assert!(content.contains("cursor {"));
        assert!(content.contains("xcursor-size"));
    }

    #[test]
    fn test_generate_overview_kdl() {
        use crate::config::models::OverviewSettings;
        let overview = OverviewSettings::default();
        let content = generate_overview_kdl(&overview);

        assert!(content.contains("overview {"));
        assert!(content.contains("zoom"));
    }

    #[test]
    fn test_generate_overview_kdl_with_backdrop() {
        use crate::config::models::OverviewSettings;
        let overview = OverviewSettings {
            backdrop_color: Some(Color::from_hex("#ff0000").unwrap()),
            ..Default::default()
        };
        let content = generate_overview_kdl(&overview);

        assert!(content.contains("backdrop-color \"#ff0000\""));
    }
}
