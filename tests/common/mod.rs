//! Common test utilities
//!
//! Shared utilities for integration tests.

// Each integration test file compiles this module separately,
// so tests that don't use these helpers emit dead_code warnings.
#![allow(dead_code)]

use nirify::config::ConfigPaths;
use std::fs;
use std::path::Path;

/// Create a mock ConfigPaths pointing to a temp directory
///
/// This sets up the directory structure and returns ConfigPaths
/// suitable for testing save/load operations.
pub fn create_test_paths(base: &Path) -> ConfigPaths {
    let managed_dir = base.to_path_buf();
    let input_dir = managed_dir.join("input");
    let advanced_dir = managed_dir.join("advanced");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&advanced_dir).unwrap();

    ConfigPaths {
        niri_config: base.join("config.kdl"),
        managed_dir: managed_dir.clone(),
        input_dir: input_dir.clone(),
        advanced_dir: advanced_dir.clone(),
        backup_dir: managed_dir.join(".nirify-backups"),
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
        preferences_kdl: advanced_dir.join("preferences.kdl"),
    }
}
