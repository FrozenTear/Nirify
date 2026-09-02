//! Integration tests for the import feature
//!
//! Tests for importing settings from user's existing niri config.kdl

use nirify::config::models::{normalized_key_combo, KeybindAction};
use nirify::config::storage::generate_window_rules_kdl;
use nirify::config::{import_from_niri_config, import_from_niri_config_with_result};
use nirify::types::ColorOrGradient;
use nirify::version::FeatureCompat;
use serial_test::serial;
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
    assert_eq!(output.scale, Some(1.5));
}

#[test]
fn test_import_explicit_scale_1_is_some() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
output "eDP-1" {
    scale 1.0
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.outputs.outputs[0].scale, Some(1.0));
}

#[test]
fn test_import_unset_scale_stays_none() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
output "eDP-1" {
    mode "1920x1080@60"
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.outputs.outputs[0].scale, None);
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
fn test_import_catchall_window_rule_keeps_full_effects() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
window-rule {
    opacity 0.9
    clip-to-geometry true
    open-maximized true
    open-floating false
    draw-border-with-background false
    geometry-corner-radius 12
}
window-rule {
    match app-id="firefox"
    opacity 0.95
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.window_rules.rules.len(), 2);

    let catch_all = settings
        .window_rules
        .rules
        .iter()
        .find(|r| r.is_catch_all())
        .expect("catch-all must be imported, not discarded");
    assert_eq!(catch_all.opacity, Some(0.9));
    assert_eq!(catch_all.clip_to_geometry, Some(true));
    assert_eq!(catch_all.open_maximized, Some(true));
    assert_eq!(catch_all.open_floating, Some(false));
    assert_eq!(catch_all.draw_border_with_background, Some(false));
    assert_eq!(
        catch_all.corner_radius,
        Some(nirify::config::models::CornerRadiusValue::uniform(12.0))
    );
    assert_eq!(settings.appearance.corner_radius, 12.0);

    let matched = settings
        .window_rules
        .rules
        .iter()
        .find(|r| !r.is_catch_all())
        .expect("matched rule still imported");
    assert_eq!(matched.matches[0].app_id.as_deref(), Some("firefox"));
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
        nirify::config::models::ScreenshotPathConfig::Custom("~/Pictures/Screenshots".to_string())
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
fn test_import_duplicate_mod_q_last_wins() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r#"
binds {
    Mod+Q { close-window; }
    Mod+Q { spawn "kitty"; }
}
"#,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.keybindings.bindings.len(), 1);
    assert_eq!(
        normalized_key_combo(&settings.keybindings.bindings[0].key_combo),
        normalized_key_combo("Mod+Q")
    );
    assert!(
        matches!(
            &settings.keybindings.bindings[0].action,
            KeybindAction::Spawn(args) if args == &["kitty"]
        ),
        "first-run import must keep the last Mod+Q action, got {:?}",
        settings.keybindings.bindings[0].action
    );
}

fn with_xdg_niri_home<F>(f: F)
where
    F: FnOnce(&std::path::Path),
{
    let tmp = tempdir().unwrap();
    let niri_dir = tmp.path().join("niri");
    fs::create_dir_all(&niri_dir).unwrap();
    let old = std::env::var_os("XDG_CONFIG_HOME");
    std::env::set_var("XDG_CONFIG_HOME", tmp.path());
    f(&niri_dir);
    match old {
        Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
        None => std::env::remove_var("XDG_CONFIG_HOME"),
    }
}

#[test]
#[serial]
fn test_import_includes_are_positional_later_same_file_wins() {
    with_xdg_niri_home(|niri_dir| {
        fs::write(niri_dir.join("early.kdl"), "layout { gaps 8 }\n").unwrap();
        let config = niri_dir.join("config.kdl");
        fs::write(
            &config,
            r#"
include "early.kdl"
layout { gaps 24 }
"#,
        )
        .unwrap();

        let settings = import_from_niri_config(&config);
        assert_eq!(
            settings.appearance.gaps, 24.0,
            "content after include must override the include (niri positionality)"
        );
    });
}

#[test]
#[serial]
fn test_import_includes_are_positional_later_include_wins() {
    with_xdg_niri_home(|niri_dir| {
        fs::write(niri_dir.join("late.kdl"), "layout { gaps 8 }\n").unwrap();
        let config = niri_dir.join("config.kdl");
        fs::write(
            &config,
            r#"
layout { gaps 24 }
include "late.kdl"
"#,
        )
        .unwrap();

        let settings = import_from_niri_config(&config);
        assert_eq!(
            settings.appearance.gaps, 8.0,
            "include after same-file nodes must override (niri positionality)"
        );
    });
}

#[test]
fn test_import_window_rule_gradients_survive_save() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("config.kdl");

    fs::write(
        &config,
        r##"
window-rule {
    match app-id="firefox"
    focus-ring {
        active-gradient from="#80c8ff" to="#bbddff" angle=45
        inactive-gradient from="#404040" to="#202020"
        urgent-gradient from="#ff0000" to="#9b0000"
    }
    border {
        active-color "#ff0000"
        urgent-gradient from="#ff8800" to="#884400"
    }
    tab-indicator {
        active-gradient from="#ffffff" to="#aaaaaa"
        urgent-color "#0000ff"
    }
}
"##,
    )
    .unwrap();

    let settings = import_from_niri_config(&config);
    assert_eq!(settings.window_rules.rules.len(), 1);
    let rule = &settings.window_rules.rules[0];
    assert!(
        matches!(rule.focus_ring_active, Some(ColorOrGradient::Gradient(_))),
        "{:?}",
        rule.focus_ring_active
    );
    assert!(
        matches!(rule.focus_ring_inactive, Some(ColorOrGradient::Gradient(_))),
        "{:?}",
        rule.focus_ring_inactive
    );
    assert!(
        matches!(rule.focus_ring_urgent, Some(ColorOrGradient::Gradient(_))),
        "{:?}",
        rule.focus_ring_urgent
    );
    assert!(
        matches!(rule.border_urgent, Some(ColorOrGradient::Gradient(_))),
        "{:?}",
        rule.border_urgent
    );
    assert!(
        matches!(
            rule.tab_indicator.as_ref().and_then(|t| t.active.as_ref()),
            Some(ColorOrGradient::Gradient(_))
        ),
        "{:?}",
        rule.tab_indicator
    );
    assert!(
        matches!(
            rule.tab_indicator.as_ref().and_then(|t| t.urgent.as_ref()),
            Some(ColorOrGradient::Color(_))
        ),
        "urgent solid on tab-indicator must also round-trip: {:?}",
        rule.tab_indicator
    );

    let saved =
        generate_window_rules_kdl(&settings.window_rules, false, FeatureCompat::all_enabled());
    assert!(saved.contains("active-gradient"), "{saved}");
    assert!(saved.contains("inactive-gradient"), "{saved}");
    assert!(saved.contains("urgent-gradient"), "{saved}");
    assert!(saved.contains("urgent-color"), "{saved}");
}

#[test]
fn test_import_result_missing_file_has_warning() {
    let dir = tempdir().unwrap();
    let config = dir.path().join("nonexistent.kdl");

    let result = import_from_niri_config_with_result(&config);
    assert!(!result.warnings.is_empty());
    assert!(result.warnings[0].contains("Could not read"));
}
