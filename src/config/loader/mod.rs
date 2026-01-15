//! Settings loader - reads KDL configuration files
//!
//! This module provides functions to load settings from the multi-file
//! KDL configuration structure managed by niri-settings.
//!
//! # Module Structure
//!
//! - `helpers`: Shared utilities for KDL parsing
//! - `gradient`: Gradient and color parsing
//! - `appearance`: Appearance settings loader
//! - `behavior`: Behavior settings loader
//! - `input`: Input device settings loaders
//! - `display`: Display settings loaders (animations, cursor, outputs)
//! - `layout_extras`: Layout extras (shadow, tab indicator, insert hint)
//! - `gestures`: Gesture settings (hot corners, DND)
//! - `misc`: Miscellaneous settings
//! - `workspaces`: Named workspace settings
//! - `rules`: Window and layer rules
//! - `system`: Startup, environment, debug, switch events, recent windows
//! - `health`: Config health checking and repair
//! - `import`: Import settings from user's niri config
//!
//! # Error Recovery
//!
//! The loader is designed to be resilient to corrupted config files:
//! - Missing files: Falls back to default values
//! - Corrupted files: Falls back to defaults, logs warning
//! - Out-of-range values: Clamped to valid ranges via `Settings::validate()`
//!
//! Use [`check_config_health`] to diagnose config file issues and
//! [`repair_corrupted_configs`] to back up and regenerate corrupted files.

// Loader helper macros - must be declared first so other modules can use them
#[macro_use]
pub mod macros;

mod appearance;
mod behavior;
mod display;
mod gestures;
mod gradient;
mod health;
mod helpers;
mod import;
mod input;
mod keybindings;
mod layout_extras;
mod misc;
mod rules;
mod system;
mod workspaces;

// Re-export loaders for internal use
pub use appearance::{load_appearance, parse_appearance_from_doc, parse_layout_children};
pub use behavior::{load_behavior, parse_behavior_from_doc};
pub use display::{
    load_animations, load_cursor, load_outputs, load_overview, parse_animations_from_children,
    parse_cursor_from_children, parse_layout_override, parse_output_node_children,
    parse_overview_from_children, parse_single_animation,
};
pub use gestures::{load_gestures, parse_gestures_from_doc};
pub use gradient::{load_color_or_gradient, load_gradient, parse_gradient_from_entries};
pub use input::{
    load_keyboard, load_mouse, load_tablet, load_touch, load_touchpad, load_trackball,
    load_trackpoint, parse_keyboard_from_children, parse_mouse_from_children,
    parse_tablet_from_children, parse_touch_from_children, parse_touchpad_from_children,
    parse_trackball_from_children, parse_trackpoint_from_children,
};
pub use keybindings::load_keybindings;
pub use layout_extras::{load_layout_extras, parse_layout_extras_from_children};
pub use misc::{load_misc, parse_misc_from_doc};
pub use rules::{
    has_flag_in_node, load_layer_rules, load_window_rules, parse_layer_rule_node_children,
    parse_window_rule_node_children,
};
pub use system::{
    load_debug, load_environment, load_recent_windows, load_startup, load_switch_events,
    parse_debug_from_doc, parse_environment_from_doc, parse_recent_windows_from_doc,
    parse_startup_from_doc, parse_switch_events_from_doc,
};
pub use workspaces::{load_workspaces, parse_workspace_node_children};

// Re-export health module items
pub use health::{
    check_config_health, repair_corrupted_configs, ConfigFileStatus, ConfigHealthReport,
};

// Re-export import module items
pub use import::{import_from_niri_config, import_from_niri_config_with_result, ImportResult};

// Re-export FileLoadStatus for tracking individual file load results
pub use helpers::FileLoadStatus;

use super::models::Settings;
use super::paths::ConfigPaths;
use helpers::read_kdl_file_with_status;
use kdl::KdlDocument;
use log::debug;

/// Result of loading settings, including feedback about what was loaded
#[derive(Debug, Clone, Default)]
pub struct LoadResult {
    /// The loaded settings (with defaults for any failed/missing files)
    pub settings: Settings,
    /// Files that were successfully loaded
    pub loaded_files: Vec<String>,
    /// Files that don't exist yet (not an error - just not configured)
    pub missing_files: Vec<String>,
    /// Files that exist but failed to load (parse or read errors)
    pub failed_files: Vec<String>,
    /// Warning messages for UI display
    pub warnings: Vec<String>,
}

impl LoadResult {
    /// Returns true if any files failed to load
    pub fn has_failures(&self) -> bool {
        !self.failed_files.is_empty()
    }

    /// Returns true if all existing files were loaded successfully
    pub fn is_healthy(&self) -> bool {
        self.failed_files.is_empty()
    }

    /// Get a summary message for display
    pub fn summary(&self) -> String {
        if self.failed_files.is_empty() {
            format!(
                "Loaded {} config files ({} not yet configured)",
                self.loaded_files.len(),
                self.missing_files.len()
            )
        } else {
            format!(
                "Loaded {} config files, {} failed to load",
                self.loaded_files.len(),
                self.failed_files.len()
            )
        }
    }

    /// Track a file load result
    fn track(&mut self, filename: &str, status: &FileLoadStatus) {
        match status {
            FileLoadStatus::Loaded(_) => {
                self.loaded_files.push(filename.to_string());
            }
            FileLoadStatus::Missing => {
                self.missing_files.push(filename.to_string());
            }
            FileLoadStatus::ParseError(msg) => {
                self.failed_files.push(filename.to_string());
                self.warnings
                    .push(format!("{}: parse error - {}", filename, msg));
            }
            FileLoadStatus::ReadError(msg) => {
                self.failed_files.push(filename.to_string());
                self.warnings
                    .push(format!("{}: read error - {}", filename, msg));
            }
        }
    }
}

/// Load settings from all KDL files, falling back to defaults for missing/invalid files
pub fn load_settings(paths: &ConfigPaths) -> Settings {
    load_settings_with_result(paths).settings
}

/// Load settings with detailed result including load status for each file
///
/// This is the preferred function when you need feedback about what was loaded.
/// Use `load_settings()` when you only need the settings themselves.
pub fn load_settings_with_result(paths: &ConfigPaths) -> LoadResult {
    let mut result = LoadResult::default();

    // Load keybindings from our managed keybindings.kdl file
    load_keybindings(&paths.keybindings_kdl, &mut result.settings.keybindings);
    // Derive status from what load_keybindings already determined (avoids re-reading file)
    let kb_status = if !paths.keybindings_kdl.exists() {
        FileLoadStatus::Missing
    } else if result.settings.keybindings.loaded {
        // Create empty doc for status - track() only checks the variant, not the contents
        FileLoadStatus::Loaded(KdlDocument::default())
    } else if let Some(err) = &result.settings.keybindings.error {
        if err.contains("Could not read") {
            FileLoadStatus::ReadError(err.clone())
        } else {
            FileLoadStatus::ParseError(err.clone())
        }
    } else {
        // File exists but no bindings found - treat as loaded (empty is valid)
        FileLoadStatus::Loaded(KdlDocument::default())
    };
    result.track("keybindings.kdl", &kb_status);

    // Only attempt to load managed settings if our directory exists
    if !paths.managed_dir.exists() {
        debug!("Managed config directory doesn't exist, using defaults");
        return result;
    }

    // Helper macro to load a file and track its status
    macro_rules! load_and_track {
        ($path:expr, $filename:expr, $loader:ident, $settings:expr) => {{
            let status = read_kdl_file_with_status($path);
            if let Some(doc) = status.document() {
                // Call the parse function directly instead of the loader
                // to avoid double-reading the file
                $loader(doc, $settings);
            }
            result.track($filename, &status);
        }};
    }

    // Load each category, tracking status for each file
    // Appearance has special handling for corner_radius from window-rule block
    {
        let status = read_kdl_file_with_status(&paths.appearance_kdl);
        if let Some(doc) = status.document() {
            parse_appearance_from_doc(doc, &mut result.settings);
            // Window rule for corner radius (global) - only first one in managed file
            if let Some(wr) = doc.get("window-rule") {
                if let Some(wr_children) = wr.children() {
                    if let Some(cr) =
                        super::parser::get_i64(wr_children, &["geometry-corner-radius"])
                    {
                        result.settings.appearance.corner_radius = cr as f32;
                    }
                }
            }
        }
        result.track("appearance.kdl", &status);
    }
    load_and_track!(
        &paths.behavior_kdl,
        "behavior.kdl",
        parse_behavior_from_doc,
        &mut result.settings
    );

    // Input devices - these have different parse function signatures
    {
        let status = read_kdl_file_with_status(&paths.keyboard_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(kb) = children.get("keyboard") {
                        if let Some(kb_children) = kb.children() {
                            parse_keyboard_from_children(kb_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/keyboard.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.mouse_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(mouse) = children.get("mouse") {
                        if let Some(mouse_children) = mouse.children() {
                            parse_mouse_from_children(mouse_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/mouse.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.touchpad_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(tp) = children.get("touchpad") {
                        if let Some(tp_children) = tp.children() {
                            parse_touchpad_from_children(tp_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/touchpad.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.trackpoint_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(tp) = children.get("trackpoint") {
                        if let Some(tp_children) = tp.children() {
                            parse_trackpoint_from_children(tp_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/trackpoint.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.trackball_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(tb) = children.get("trackball") {
                        if let Some(tb_children) = tb.children() {
                            parse_trackball_from_children(tb_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/trackball.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.tablet_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(tablet) = children.get("tablet") {
                        if let Some(tablet_children) = tablet.children() {
                            parse_tablet_from_children(tablet_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/tablet.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.touch_kdl);
        if let Some(doc) = status.document() {
            if let Some(input) = doc.get("input") {
                if let Some(children) = input.children() {
                    if let Some(touch) = children.get("touch") {
                        if let Some(touch_children) = touch.children() {
                            parse_touch_from_children(touch_children, &mut result.settings);
                        }
                    }
                }
            }
        }
        result.track("input/touch.kdl", &status);
    }

    // Display settings
    {
        let status = read_kdl_file_with_status(&paths.animations_kdl);
        if let Some(doc) = status.document() {
            if let Some(anims) = doc.get("animations") {
                if let Some(children) = anims.children() {
                    parse_animations_from_children(children, &mut result.settings);
                }
            }
        }
        result.track("animations.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.cursor_kdl);
        if let Some(doc) = status.document() {
            if let Some(cursor) = doc.get("cursor") {
                if let Some(children) = cursor.children() {
                    parse_cursor_from_children(children, &mut result.settings);
                }
            }
        }
        result.track("cursor.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.overview_kdl);
        if let Some(doc) = status.document() {
            if let Some(overview) = doc.get("overview") {
                if let Some(children) = overview.children() {
                    parse_overview_from_children(children, &mut result.settings);
                }
            }
        }
        result.track("overview.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.outputs_kdl);
        if let Some(doc) = status.document() {
            result.settings.outputs.outputs.clear();
            for node in doc.nodes() {
                if node.name().value() == "output" {
                    let name = node
                        .entries()
                        .first()
                        .and_then(|e| e.value().as_string())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    let mut output = crate::config::models::OutputConfig {
                        name,
                        ..Default::default()
                    };
                    if let Some(o_children) = node.children() {
                        parse_output_node_children(o_children, &mut output);
                    }
                    result.settings.outputs.outputs.push(output);
                }
            }
        }
        result.track("outputs.kdl", &status);
    }

    // Advanced settings
    {
        let status = read_kdl_file_with_status(&paths.layout_extras_kdl);
        if let Some(doc) = status.document() {
            if let Some(layout) = doc.get("layout") {
                if let Some(children) = layout.children() {
                    parse_layout_extras_from_children(children, &mut result.settings);
                }
            }
        }
        result.track("advanced/layout-extras.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.gestures_kdl);
        if let Some(doc) = status.document() {
            parse_gestures_from_doc(doc, &mut result.settings);
        }
        result.track("advanced/gestures.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.misc_kdl);
        if let Some(doc) = status.document() {
            parse_misc_from_doc(doc, &mut result.settings);
        }
        result.track("advanced/misc.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.workspaces_kdl);
        if let Some(doc) = status.document() {
            result.settings.workspaces.workspaces.clear();
            let mut next_id = 0u32;
            for node in doc.nodes() {
                if node.name().value() == "workspace" {
                    let name = node
                        .entries()
                        .first()
                        .and_then(|e| e.value().as_string())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    let mut ws = crate::config::models::NamedWorkspace {
                        id: next_id,
                        name,
                        ..Default::default()
                    };
                    next_id += 1;
                    if let Some(children) = node.children() {
                        parse_workspace_node_children(children, &mut ws);
                    }
                    result.settings.workspaces.workspaces.push(ws);
                }
            }
        }
        result.track("workspaces.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.layer_rules_kdl);
        if let Some(doc) = status.document() {
            result.settings.layer_rules.rules.clear();
            let mut next_id = 0u32;
            for node in doc.nodes() {
                if node.name().value() == "layer-rule" {
                    let mut rule = crate::config::models::LayerRule {
                        id: next_id,
                        name: format!("Layer Rule {}", next_id + 1),
                        ..Default::default()
                    };
                    next_id += 1;
                    if let Some(children) = node.children() {
                        parse_layer_rule_node_children(children, &mut rule);
                    }
                    result.settings.layer_rules.rules.push(rule);
                }
            }
        }
        result.track("advanced/layer-rules.kdl", &status);
    }

    {
        let status = read_kdl_file_with_status(&paths.window_rules_kdl);
        if let Some(doc) = status.document() {
            result.settings.window_rules.rules.clear();
            let mut next_id = 0u32;
            for node in doc.nodes() {
                if node.name().value() == "window-rule" {
                    let mut rule = crate::config::models::WindowRule {
                        id: next_id,
                        name: format!("Rule {}", next_id + 1),
                        ..Default::default()
                    };
                    next_id += 1;
                    if let Some(children) = node.children() {
                        parse_window_rule_node_children(children, &mut rule);
                    }
                    result.settings.window_rules.rules.push(rule);
                }
            }
        }
        result.track("advanced/window-rules.kdl", &status);
    }

    // System settings
    load_and_track!(
        &paths.startup_kdl,
        "advanced/startup.kdl",
        parse_startup_from_doc,
        &mut result.settings
    );
    load_and_track!(
        &paths.environment_kdl,
        "advanced/environment.kdl",
        parse_environment_from_doc,
        &mut result.settings
    );
    load_and_track!(
        &paths.debug_kdl,
        "advanced/debug.kdl",
        parse_debug_from_doc,
        &mut result.settings
    );
    load_and_track!(
        &paths.switch_events_kdl,
        "advanced/switch-events.kdl",
        parse_switch_events_from_doc,
        &mut result.settings
    );
    load_and_track!(
        &paths.recent_windows_kdl,
        "advanced/recent-windows.kdl",
        parse_recent_windows_from_doc,
        &mut result.settings
    );

    // Validate and clamp all values to valid ranges
    result.settings.validate();

    debug!("{}", result.summary());

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_load_nonexistent_file() {
        let mut settings = Settings::default();
        load_appearance(Path::new("/nonexistent/path.kdl"), &mut settings);
        // Should not panic, settings unchanged
        assert!(settings.appearance.focus_ring_enabled);
    }

    #[test]
    fn test_load_appearance_kdl() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("appearance.kdl");

        std::fs::write(
            &path,
            r##"
layout {
    gaps 24
    focus-ring {
        width 6
        active-color "#ff0000"
    }
    border { off }
}
"##,
        )
        .unwrap();

        let mut settings = Settings::default();
        load_appearance(&path, &mut settings);

        assert_eq!(settings.appearance.gaps_inner, 24.0);
        assert_eq!(settings.appearance.gaps_outer, 24.0);
        assert_eq!(settings.appearance.focus_ring_width, 6.0);
        assert_eq!(settings.appearance.focus_ring_active.primary_color().r, 255);
        assert!(!settings.appearance.border_enabled);
    }

    #[test]
    fn test_load_animations_off() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("animations.kdl");

        std::fs::write(
            &path,
            r#"
animations {
    off
    slowdown 2.0
}
"#,
        )
        .unwrap();

        let mut settings = Settings::default();
        load_animations(&path, &mut settings);

        assert!(!settings.animations.enabled);
        assert_eq!(settings.animations.slowdown, 2.0);
    }

    #[test]
    fn test_settings_roundtrip() {
        use super::super::paths::ConfigPaths;
        use super::super::storage::save_settings;

        let dir = tempdir().unwrap();

        // Create a mock ConfigPaths pointing to our temp directory
        let managed_dir = dir.path().to_path_buf();
        let input_dir = managed_dir.join("input");
        let advanced_dir = managed_dir.join("advanced");

        std::fs::create_dir_all(&input_dir).unwrap();
        std::fs::create_dir_all(&advanced_dir).unwrap();

        let paths = ConfigPaths {
            niri_config: dir.path().join("config.kdl"),
            managed_dir: managed_dir.clone(),
            input_dir: input_dir.clone(),
            advanced_dir: advanced_dir.clone(),
            backup_dir: managed_dir.join(".backup"),
            main_kdl: managed_dir.join("main.kdl"),
            appearance_kdl: managed_dir.join("appearance.kdl"),
            behavior_kdl: managed_dir.join("behavior.kdl"),
            keyboard_kdl: input_dir.join("keyboard.kdl"),
            mouse_kdl: input_dir.join("mouse.kdl"),
            touchpad_kdl: input_dir.join("touchpad.kdl"),
            trackpoint_kdl: input_dir.join("trackpoint.kdl"),
            trackball_kdl: input_dir.join("trackball.kdl"),
            tablet_kdl: input_dir.join("tablet.kdl"),
            touch_kdl: input_dir.join("touch.kdl"),
            outputs_kdl: managed_dir.join("outputs.kdl"),
            animations_kdl: managed_dir.join("animations.kdl"),
            cursor_kdl: managed_dir.join("cursor.kdl"),
            overview_kdl: managed_dir.join("overview.kdl"),
            workspaces_kdl: managed_dir.join("workspaces.kdl"),
            keybindings_kdl: managed_dir.join("keybindings.kdl"),
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
        };

        // Create custom settings with non-default values
        let mut original = Settings::default();
        original.appearance.gaps_inner = 24.0;
        // Note: gaps_outer is ignored in output since niri uses single gap value
        original.appearance.focus_ring_width = 6.0;
        original.appearance.corner_radius = 16.0;
        original.appearance.border_enabled = false;
        original.keyboard.repeat_delay = 400;
        original.keyboard.repeat_rate = 30;
        original.mouse.accel_speed = 0.5;
        original.animations.enabled = false;
        original.animations.slowdown = 2.5;
        original.cursor.size = 32;
        original.overview.zoom = 0.75;

        // Save settings
        save_settings(&paths, &original).expect("Failed to save settings");

        // Load settings back
        let loaded = load_settings(&paths);

        // Compare key values (not full equality due to potential rounding)
        // Note: niri uses a single gap value, so inner and outer become the same
        assert_eq!(loaded.appearance.gaps_inner, original.appearance.gaps_inner);
        assert_eq!(loaded.appearance.gaps_outer, original.appearance.gaps_inner);
        assert_eq!(
            loaded.appearance.focus_ring_width,
            original.appearance.focus_ring_width
        );
        assert_eq!(
            loaded.appearance.corner_radius,
            original.appearance.corner_radius
        );
        assert_eq!(
            loaded.appearance.border_enabled,
            original.appearance.border_enabled
        );
        assert_eq!(loaded.keyboard.repeat_delay, original.keyboard.repeat_delay);
        assert_eq!(loaded.keyboard.repeat_rate, original.keyboard.repeat_rate);
        assert!((loaded.mouse.accel_speed - original.mouse.accel_speed).abs() < 0.01);
        assert_eq!(loaded.animations.enabled, original.animations.enabled);
        assert!((loaded.animations.slowdown - original.animations.slowdown).abs() < 0.01);
        assert_eq!(loaded.cursor.size, original.cursor.size);
        assert!((loaded.overview.zoom - original.overview.zoom).abs() < 0.01);
    }

    #[test]
    fn test_validate_clamps_values() {
        let mut settings = Settings::default();

        // Set invalid values
        settings.appearance.gaps_inner = -100.0;
        settings.appearance.gaps_outer = 1000.0;
        settings.appearance.focus_ring_width = 0.0;
        settings.keyboard.repeat_delay = 50;
        settings.mouse.accel_speed = 5.0;
        settings.animations.slowdown = 100.0;
        settings.cursor.size = 200;

        // Validate
        settings.validate();

        // Check values are clamped to valid ranges
        assert_eq!(settings.appearance.gaps_inner, 0.0); // GAP_SIZE_MIN
        assert_eq!(settings.appearance.gaps_outer, 64.0); // GAP_SIZE_MAX
        assert_eq!(settings.appearance.focus_ring_width, 1.0); // FOCUS_RING_WIDTH_MIN
        assert_eq!(settings.keyboard.repeat_delay, 100); // REPEAT_DELAY_MIN
        assert_eq!(settings.mouse.accel_speed, 1.0); // ACCEL_SPEED_MAX
        assert_eq!(settings.animations.slowdown, 10.0); // ANIMATION_SLOWDOWN_MAX
        assert_eq!(settings.cursor.size, 64); // CURSOR_SIZE_MAX
    }
}
