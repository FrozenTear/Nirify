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
mod blur;
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
pub use appearance::{generate_appearance_kdl, generate_appearance_kdl_for_settings};
pub use behavior::{generate_behavior_kdl, generate_main_kdl};
pub use blur::generate_blur_kdl;
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
use crate::version::FeatureCompat;
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
///
/// Before writing, the content is re-parsed as a KDL document. A parse failure
/// indicates a generator bug (we produced KDL niri could not read) and aborts
/// the write so we never overwrite a good file with broken output.
fn write_config(path: &Path, content: &str, strategy: WriteStrategy) -> anyhow::Result<()> {
    content
        .parse::<kdl::KdlDocument>()
        .map_err(|e| anyhow::anyhow!("BUG: generated invalid KDL for {:?}: {}", path, e))?;
    match strategy {
        WriteStrategy::Atomic => atomic_write(path, content),
        WriteStrategy::Direct => fs::write(path, content).map_err(Into::into),
    }
    .with_context(|| format!("Failed to write {:?}", path))
}

/// Write a category file, optionally snapshotting the existing file first.
///
/// When `do_backup` is true and the target already exists, the current bytes
/// are copied to a timestamped `.bak` in `backup_dir` before the new content is
/// written (both writes are atomic; the new content passes the KDL parse gate).
fn write_category(
    path: &Path,
    content: &str,
    strategy: WriteStrategy,
    do_backup: bool,
    backup_dir: &Path,
) -> anyhow::Result<()> {
    if do_backup && path.exists() {
        return save_with_backup(path, content, backup_dir);
    }
    write_config(path, content, strategy)
}

/// Write all settings files using the specified strategy.
///
/// This is the unified function that writes all config files.
/// The `compat` parameter controls which version-dependent features are written.
fn write_all_settings(
    paths: &ConfigPaths,
    settings: &Settings,
    strategy: WriteStrategy,
    compat: FeatureCompat,
) -> anyhow::Result<()> {
    paths.ensure_directories()?;

    // Main entry point (includes version-aware file list)
    write_config(&paths.main_kdl, &generate_main_kdl(compat), strategy)?;

    // Core settings
    write_config(
        &paths.appearance_kdl,
        &generate_appearance_kdl_for_settings(
            &settings.appearance,
            &settings.behavior,
            &settings.window_rules,
        ),
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
        &generate_tablet_kdl(&settings.tablet, compat),
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
        &generate_layer_rules_kdl(&settings.layer_rules, compat),
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
        &generate_window_rules_kdl(
            &settings.window_rules,
            settings.preferences.float_settings_app,
            compat,
        ),
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

    // Recent windows requires niri 25.11+
    if compat.recent_windows {
        write_config(
            &paths.recent_windows_kdl,
            &generate_recent_windows_kdl(&settings.recent_windows),
            strategy,
        )?;
    }

    // Top-level blur requires niri 26.04+
    if compat.blur {
        write_config(
            &paths.path_for(crate::config::registry::ConfigFile::Blur),
            &generate_blur_kdl(&settings.blur),
            strategy,
        )?;
    }

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
///
/// # Arguments
/// * `paths` - The configuration paths structure
/// * `settings` - The settings to write
/// * `compat` - Feature compatibility flags based on detected niri version
pub fn save_settings(
    paths: &ConfigPaths,
    settings: &Settings,
    compat: FeatureCompat,
) -> anyhow::Result<()> {
    write_all_settings(paths, settings, WriteStrategy::Atomic, compat)
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
/// * `compat` - Feature compatibility flags based on detected niri version
/// * `backup_first` - Categories whose existing file should be snapshotted to a
///   `.bak` before being overwritten (first-write-per-session backup policy)
///
/// # Returns
/// The number of files that were written.
pub fn save_dirty(
    paths: &ConfigPaths,
    settings: &Settings,
    dirty: &std::collections::HashSet<super::dirty::SettingsCategory>,
    compat: FeatureCompat,
    backup_first: &std::collections::HashSet<super::dirty::SettingsCategory>,
) -> anyhow::Result<usize> {
    use super::dirty::SettingsCategory;

    if dirty.is_empty() {
        return Ok(0);
    }

    paths.ensure_directories()?;

    let mut files_written = 0;
    let strategy = WriteStrategy::Atomic;

    // Ensure main.kdl exists (it includes all other config files)
    // This is essential for niri to load our managed configuration
    if !paths.main_kdl.exists() {
        write_config(&paths.main_kdl, &generate_main_kdl(compat), strategy)?;
        files_written += 1;
    }

    // blur.kdl has no dedicated ConfigPaths field; resolve it via the registry.
    // Bound outside the loop so the borrowed &Path lives for the whole match.
    let blur_path = paths.path_for(crate::config::registry::ConfigFile::Blur);

    for category in dirty {
        // Compute the target path and generated content for this category.
        // `continue` skips categories that must not be written for the detected
        // niri version (they are not counted as written).
        let (path, content): (&Path, String) = match category {
            SettingsCategory::Appearance => (
                &paths.appearance_kdl,
                // Appearance includes some behavior settings (struts)
                generate_appearance_kdl_for_settings(
                    &settings.appearance,
                    &settings.behavior,
                    &settings.window_rules,
                ),
            ),
            SettingsCategory::Behavior => (
                &paths.behavior_kdl,
                generate_behavior_kdl(&settings.behavior),
            ),
            SettingsCategory::Keyboard => (
                &paths.keyboard_kdl,
                generate_keyboard_kdl(&settings.keyboard),
            ),
            SettingsCategory::Mouse => (&paths.mouse_kdl, generate_mouse_kdl(&settings.mouse)),
            SettingsCategory::Touchpad => (
                &paths.touchpad_kdl,
                generate_touchpad_kdl(&settings.touchpad),
            ),
            SettingsCategory::Trackpoint => (
                &paths.trackpoint_kdl,
                generate_trackpoint_kdl(&settings.trackpoint),
            ),
            SettingsCategory::Trackball => (
                &paths.trackball_kdl,
                generate_trackball_kdl(&settings.trackball),
            ),
            SettingsCategory::Tablet => (
                &paths.tablet_kdl,
                generate_tablet_kdl(&settings.tablet, compat),
            ),
            SettingsCategory::Touch => (&paths.touch_kdl, generate_touch_kdl(&settings.touch)),
            SettingsCategory::Outputs => {
                (&paths.outputs_kdl, generate_outputs_kdl(&settings.outputs))
            }
            SettingsCategory::Animations => (
                &paths.animations_kdl,
                generate_animations_kdl(&settings.animations),
            ),
            SettingsCategory::Cursor => (&paths.cursor_kdl, generate_cursor_kdl(&settings.cursor)),
            SettingsCategory::Overview => (
                &paths.overview_kdl,
                generate_overview_kdl(&settings.overview),
            ),
            SettingsCategory::Workspaces => (
                &paths.workspaces_kdl,
                generate_workspaces_kdl(&settings.workspaces),
            ),
            SettingsCategory::Keybindings => (
                &paths.keybindings_kdl,
                generate_keybindings_kdl(&settings.keybindings),
            ),
            SettingsCategory::LayoutExtras => (
                &paths.layout_extras_kdl,
                generate_layout_extras_kdl(&settings.layout_extras),
            ),
            SettingsCategory::Gestures => (
                &paths.gestures_kdl,
                generate_gestures_kdl(&settings.gestures),
            ),
            SettingsCategory::LayerRules => (
                &paths.layer_rules_kdl,
                generate_layer_rules_kdl(&settings.layer_rules, compat),
            ),
            SettingsCategory::WindowRules => (
                &paths.window_rules_kdl,
                generate_window_rules_kdl(
                    &settings.window_rules,
                    settings.preferences.float_settings_app,
                    compat,
                ),
            ),
            SettingsCategory::Miscellaneous => {
                (&paths.misc_kdl, generate_misc_kdl(&settings.miscellaneous))
            }
            SettingsCategory::Startup => {
                (&paths.startup_kdl, generate_startup_kdl(&settings.startup))
            }
            SettingsCategory::Environment => (
                &paths.environment_kdl,
                generate_environment_kdl(&settings.environment),
            ),
            SettingsCategory::Debug => (&paths.debug_kdl, generate_debug_kdl(&settings.debug)),
            SettingsCategory::SwitchEvents => (
                &paths.switch_events_kdl,
                generate_switch_events_kdl(&settings.switch_events),
            ),
            SettingsCategory::RecentWindows => {
                // Recent windows requires niri 25.11+
                if compat.recent_windows {
                    (
                        &paths.recent_windows_kdl,
                        generate_recent_windows_kdl(&settings.recent_windows),
                    )
                } else {
                    // Skip writing this file, don't count as written
                    continue;
                }
            }
            SettingsCategory::Blur => {
                // Top-level blur requires niri 26.04+
                if compat.blur {
                    (blur_path.as_path(), generate_blur_kdl(&settings.blur))
                } else {
                    // Skip writing this file, don't count as written
                    continue;
                }
            }
            SettingsCategory::Preferences => (
                &paths.preferences_kdl,
                generate_preferences_kdl(&settings.preferences),
            ),
        };

        let do_backup = backup_first.contains(category);
        write_category(path, &content, strategy, do_backup, &paths.backup_dir)?;
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
/// - The new content fails the KDL parse gate (see [`write_config`])
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

        // Snapshot existing bytes verbatim (no parse gate: preserve whatever was
        // on disk, including hand edits or corrupt content).
        atomic_write(&backup_path, &String::from_utf8_lossy(&existing_content))
            .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;
    }

    // Final write goes through the parse gate + atomic write.
    write_config(path, content, WriteStrategy::Atomic)
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
/// * `compat` - Feature compatibility flags based on detected niri version
///
/// # Returns
/// `Ok(())` on success, or an error if any file write fails.
///
/// # Errors
/// Returns an error if:
/// - Directory creation fails
/// - Any configuration file cannot be written
pub fn initialize_config_files(
    paths: &ConfigPaths,
    settings: &Settings,
    compat: FeatureCompat,
) -> anyhow::Result<()> {
    write_all_settings(paths, settings, WriteStrategy::Direct, compat)
}

#[cfg(test)]
// Test setup mutates a couple fields after default() for readability.
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::config::models::{AppearanceSettings, BehaviorSettings};
    use crate::types::Color;

    #[test]
    fn test_generate_main_kdl_contains_includes() {
        let content = generate_main_kdl(FeatureCompat::all_enabled());
        assert!(content.contains("include \"appearance.kdl\""));
        assert!(content.contains("include \"behavior.kdl\""));
        assert!(content.contains("include \"input/keyboard.kdl\""));
        assert!(content.contains("Nirify managed"));
        assert!(content.contains("include \"advanced/recent-windows.kdl\""));
        assert!(content.contains("include \"blur.kdl\""));
    }

    #[test]
    fn test_generate_main_kdl_skips_recent_windows_when_disabled() {
        let compat = FeatureCompat {
            recent_windows: false,
            background_effects: false,
            blur: false,
            map_to_focused_output: false,
        };
        let content = generate_main_kdl(compat);
        assert!(!content.contains("include \"advanced/recent-windows.kdl\""));
        assert!(content.contains("recent-windows.kdl requires niri 25.11+"));
    }

    #[test]
    fn test_generate_main_kdl_skips_blur_when_disabled() {
        let compat = FeatureCompat {
            recent_windows: true,
            background_effects: true,
            blur: false,
            map_to_focused_output: false,
        };
        let content = generate_main_kdl(compat);
        assert!(!content.contains("include \"blur.kdl\""));
        assert!(content.contains("blur.kdl requires niri 26.04+"));
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

        // The focus-ring block is always emitted so the disabled state (and
        // styling) round-trips and overrides niri's default (which is ON).
        assert!(content.contains("focus-ring {"));
        assert!(content.contains("off"));
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

    #[test]
    fn write_config_rejects_invalid_generated_kdl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.kdl");
        let err = write_config(&path, "node {", WriteStrategy::Atomic).unwrap_err();
        assert!(
            err.to_string().contains("generated invalid KDL"),
            "unexpected error: {}",
            err
        );
        assert!(!path.exists(), "file must not be written on parse failure");
    }

    #[test]
    fn save_dirty_backs_up_only_first_write() {
        use crate::config::{ConfigPaths, Settings, SettingsCategory};
        use std::collections::HashSet;

        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();

        let mut paths = ConfigPaths::default();
        paths.managed_dir = base.to_path_buf();
        paths.input_dir = base.join("input");
        paths.advanced_dir = base.join("advanced");
        paths.backup_dir = base.join("backups");
        paths.main_kdl = base.join("main.kdl");
        paths.appearance_kdl = base.join("appearance.kdl");

        std::fs::create_dir_all(base).unwrap();
        std::fs::write(&paths.appearance_kdl, "// sentinel\nlayout {\n}\n").unwrap();

        let settings = Settings::default();
        let compat = crate::version::FeatureCompat::from_version(None);

        let mut dirty = HashSet::new();
        dirty.insert(SettingsCategory::Appearance);
        let mut backup_first = HashSet::new();
        backup_first.insert(SettingsCategory::Appearance);

        let count_baks = |backup_dir: &std::path::Path| -> usize {
            std::fs::read_dir(backup_dir)
                .map(|rd| {
                    rd.flatten()
                        .filter(|e| {
                            let n = e.file_name();
                            let n = n.to_string_lossy();
                            n.starts_with("appearance.kdl.") && n.ends_with(".bak")
                        })
                        .count()
                })
                .unwrap_or(0)
        };

        save_dirty(&paths, &settings, &dirty, compat, &backup_first).unwrap();
        assert_eq!(count_baks(&paths.backup_dir), 1);

        // The single backup preserves the original sentinel content
        let bak = std::fs::read_dir(&paths.backup_dir)
            .unwrap()
            .flatten()
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("appearance.kdl.")
            })
            .unwrap();
        let content = std::fs::read_to_string(bak.path()).unwrap();
        assert!(content.contains("sentinel"));

        // Second save without backup_first must not add another .bak
        save_dirty(&paths, &settings, &dirty, compat, &HashSet::new()).unwrap();
        assert_eq!(count_baks(&paths.backup_dir), 1);
    }
}
