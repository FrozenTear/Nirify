//! Integration tests for the import feature
//!
//! Tests for importing settings from user's existing niri config.kdl

use niri_settings::config::{import_from_niri_config, import_from_niri_config_with_result};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_import_basic_layout_settings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
layout {
    gaps inner=20 outer=10
    focus-ring {
        width 5
    }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    // Note: inner=20 outer=10 - loader reads inner value for backwards compatibility
    assert_eq!(settings.appearance.gaps, 20.0);
    assert_eq!(settings.appearance.focus_ring_width, 5.0);
}

#[test]
fn test_import_handles_corrupted_gracefully() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(&config, "this { is not { valid").unwrap();

    // Should return defaults, not panic
    let settings = import_from_niri_config(&config);
    assert_eq!(settings.appearance.gaps, 16.0); // default
    assert!(settings.appearance.focus_ring_enabled); // default
}

#[test]
fn test_import_handles_missing_file() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("nonexistent.kdl");

    // Should return defaults, not panic
    let settings = import_from_niri_config(&config);
    assert_eq!(settings.appearance.gaps, 16.0); // default
}

#[test]
fn test_import_handles_empty_file() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(&config, "").unwrap();

    // Should return defaults for empty file
    let settings = import_from_niri_config(&config);
    assert_eq!(settings.appearance.gaps, 16.0); // default
}

#[test]
fn test_import_handles_comments_only() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
// This is a comment
// Another comment
/* Block comment */
"#,
    )
    .unwrap();

    // Should return defaults
    let settings = import_from_niri_config(&config);
    assert_eq!(settings.appearance.gaps, 16.0); // default
}

#[test]
fn test_import_focus_ring_disabled() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
layout {
    focus-ring {
        off
    }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(!settings.appearance.focus_ring_enabled);
}

#[test]
fn test_import_border_settings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
layout {
    border {
        width 3
    }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(settings.appearance.border_enabled);
    assert_eq!(settings.appearance.border_thickness, 3.0);
}

#[test]
fn test_import_cursor_settings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
cursor {
    xcursor-size 32
    xcursor-theme "Adwaita"
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.cursor.size, 32);
    assert_eq!(settings.cursor.theme, "Adwaita");
}

#[test]
fn test_import_animations_disabled() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
animations {
    off
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(!settings.animations.enabled);
}

#[test]
fn test_import_animations_slowdown() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
animations {
    slowdown 2.5
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(settings.animations.enabled);
    assert!((settings.animations.slowdown - 2.5).abs() < 0.01);
}

#[test]
fn test_import_input_keyboard() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
input {
    keyboard {
        xkb {
            layout "de"
        }
        repeat-delay 400
        repeat-rate 30
    }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.keyboard.xkb_layout, "de");
    assert_eq!(settings.keyboard.repeat_delay, 400);
    assert_eq!(settings.keyboard.repeat_rate, 30);
}

#[test]
fn test_import_input_touchpad() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
input {
    touchpad {
        tap
        natural-scroll
        accel-speed 0.3
    }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(settings.touchpad.tap);
    assert!(settings.touchpad.natural_scroll);
    assert!((settings.touchpad.accel_speed - 0.3).abs() < 0.01);
}

#[test]
fn test_import_output_settings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
output "DP-1" {
    mode "2560x1440@144"
    scale 1.5
    position x=0 y=0
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.outputs.outputs.len(), 1);
    let output = &settings.outputs.outputs[0];
    assert_eq!(output.name, "DP-1");
    assert_eq!(output.mode, "2560x1440@144");
    assert!((output.scale - 1.5).abs() < 0.01);
}

#[test]
fn test_import_window_rule() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
window-rule {
    match app-id="firefox"
    opacity 0.95
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.window_rules.rules.len(), 1);
    let rule = &settings.window_rules.rules[0];
    assert_eq!(rule.matches.len(), 1);
    assert_eq!(rule.matches[0].app_id, Some("firefox".to_string()));
    assert!((rule.opacity.unwrap() - 0.95).abs() < 0.01);
}

#[test]
fn test_import_multiple_window_rules() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
window-rule {
    match app-id="firefox"
    opacity 0.95
}
window-rule {
    match app-id="kitty"
    open-floating
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.window_rules.rules.len(), 2);
}

#[test]
fn test_import_workspace_config() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
workspace "main"
workspace "dev" {
    open-on-output "DP-1"
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.workspaces.workspaces.len(), 2);
}

#[test]
fn test_import_validates_out_of_range_values() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    // Write values outside valid ranges
    fs::write(
        &config,
        r#"
layout {
    gaps inner=-50 outer=999
    focus-ring {
        width 100
    }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    // Values should be clamped to valid ranges
    // Note: inner=-50 is read first (backwards compat), then clamped to 0
    assert_eq!(settings.appearance.gaps, 0.0); // Clamped to min
    assert_eq!(settings.appearance.focus_ring_width, 16.0); // Clamped to max
}

#[test]
fn test_import_miscellaneous_settings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
prefer-no-csd
screenshot-path "~/Pictures/Screenshots"
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(settings.miscellaneous.prefer_no_csd);
    assert_eq!(
        settings.miscellaneous.screenshot_path,
        "~/Pictures/Screenshots"
    );
}

#[test]
fn test_import_behavior_settings() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
focus-follows-mouse
hotkey-overlay {
    skip-at-startup
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert!(settings.behavior.focus_follows_mouse);
    assert!(settings.miscellaneous.hotkey_overlay_skip_at_startup);
}

// ImportResult tests

#[test]
fn test_import_result_tracks_imported_sections() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
layout {
    gaps inner=20 outer=10
}
cursor {
    xcursor-size 32
}
"#,
    )
    .unwrap();

    let result = import_from_niri_config_with_result(&config);
    assert!(result.has_imports());
    assert!(result.imported_sections.iter().any(|s| s == "appearance"));
    assert!(result.imported_sections.iter().any(|s| s == "cursor"));
}

#[test]
fn test_import_result_tracks_defaulted_sections() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    // Only set appearance, everything else should be defaulted
    fs::write(
        &config,
        r#"
layout {
    gaps inner=20
}
"#,
    )
    .unwrap();

    let result = import_from_niri_config_with_result(&config);
    assert!(result.has_imports());
    // Keyboard should be in defaulted sections (we didn't set it)
    assert!(result.defaulted_sections.iter().any(|s| s == "keyboard"));
}

#[test]
fn test_import_result_empty_config_has_no_imports() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(&config, "// Empty config").unwrap();

    let result = import_from_niri_config_with_result(&config);
    assert!(!result.has_imports());
    assert!(result.imported_sections.is_empty());
}

#[test]
fn test_import_result_summary() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
layout {
    gaps inner=20
}
"#,
    )
    .unwrap();

    let result = import_from_niri_config_with_result(&config);
    let summary = result.summary();
    assert!(summary.contains("appearance"));
}

#[test]
fn test_import_result_tracks_window_rules_count() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
window-rule {
    match app-id="firefox"
    opacity 0.95
}
window-rule {
    match app-id="kitty"
    open-floating
}
"#,
    )
    .unwrap();

    let result = import_from_niri_config_with_result(&config);
    // Should have "window-rules (2)" in imported sections
    assert!(result
        .imported_sections
        .iter()
        .any(|s| s.contains("window-rules") && s.contains("2")));
}

#[test]
fn test_import_result_missing_file_has_warning() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("nonexistent.kdl");

    let result = import_from_niri_config_with_result(&config);
    assert!(!result.warnings.is_empty());
    assert!(result.warnings[0].contains("Could not read"));
}
