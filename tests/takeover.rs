//! Integration tests for first-run import and launch-time absorb.
//!
//! These cover the two data-loss paths: the wizard writing defaults after
//! stripping, and every-launch `smart_replace` dropping hand-edits.

mod common;

use nirify::config::models::normalized_key_combo;
use nirify::config::{absorb_stripped_nodes, first_run_setup, load_settings, SettingsCategory};
use nirify::version::FeatureCompat;
use std::fs;
use tempfile::tempdir;

/// Build ConfigPaths under `base/` without creating `nirify/` first
/// (first-run must import before those directories exist).
fn paths_under(base: &std::path::Path) -> nirify::config::ConfigPaths {
    let mut paths = common::create_test_paths(base);
    // create_test_paths uses `base` as managed_dir and also as niri_config's
    // parent. Re-point managed files into base/nirify so config.kdl sits
    // beside the managed dir the way a real install does.
    let managed = base.join("nirify");
    let input = managed.join("input");
    let advanced = managed.join("advanced");
    paths.niri_config = base.join("config.kdl");
    paths.managed_dir = managed.clone();
    paths.input_dir = input.clone();
    paths.advanced_dir = advanced.clone();
    paths.backup_dir = base.join(".nirify-backups");
    paths.main_kdl = managed.join("main.kdl");
    paths.appearance_kdl = managed.join("appearance.kdl");
    paths.behavior_kdl = managed.join("behavior.kdl");
    paths.keyboard_kdl = input.join("keyboard.kdl");
    paths.mouse_kdl = input.join("mouse.kdl");
    paths.touchpad_kdl = input.join("touchpad.kdl");
    paths.trackpoint_kdl = input.join("trackpoint.kdl");
    paths.trackball_kdl = input.join("trackball.kdl");
    paths.tablet_kdl = input.join("tablet.kdl");
    paths.touch_kdl = input.join("touch.kdl");
    paths.outputs_kdl = managed.join("outputs.kdl");
    paths.animations_kdl = managed.join("animations.kdl");
    paths.cursor_kdl = managed.join("cursor.kdl");
    paths.overview_kdl = managed.join("overview.kdl");
    paths.workspaces_kdl = managed.join("workspaces.kdl");
    paths.keybindings_kdl = managed.join("keybindings.kdl");
    paths.layout_extras_kdl = advanced.join("layout-extras.kdl");
    paths.gestures_kdl = advanced.join("gestures.kdl");
    paths.layer_rules_kdl = advanced.join("layer-rules.kdl");
    paths.window_rules_kdl = advanced.join("window-rules.kdl");
    paths.misc_kdl = advanced.join("misc.kdl");
    paths.startup_kdl = advanced.join("startup.kdl");
    paths.environment_kdl = advanced.join("environment.kdl");
    paths.debug_kdl = advanced.join("debug.kdl");
    paths.switch_events_kdl = advanced.join("switch-events.kdl");
    paths.recent_windows_kdl = advanced.join("recent-windows.kdl");
    paths.preferences_kdl = advanced.join("preferences.kdl");
    paths
}

#[test]
fn wizard_imports_user_config_before_replace_not_defaults() {
    let dir = tempdir().unwrap();
    let paths = paths_under(dir.path());

    fs::write(
        &paths.niri_config,
        r#"
layout { gaps 24 }
output "DP-1" {
    position x=0 y=0
}
binds {
    Mod+Return { spawn "alacritty"; }
}
"#,
    )
    .unwrap();

    // Simulate first run: no managed dir yet.
    if paths.managed_dir.exists() {
        fs::remove_dir_all(&paths.managed_dir).unwrap();
    }

    let result = first_run_setup(&paths, FeatureCompat::all_enabled()).unwrap();
    assert!(
        result.import.has_imports(),
        "wizard must import existing settings, got {:?}",
        result.import.imported_sections
    );

    let loaded = load_settings(&paths);
    assert_eq!(
        loaded.appearance.gaps, 24.0,
        "imported gaps must be written, not the default 16"
    );
    assert_eq!(loaded.outputs.outputs.len(), 1);
    assert_eq!(loaded.outputs.outputs[0].name, "DP-1");
    assert!(loaded
        .keybindings
        .bindings
        .iter()
        .any(|b| b.key_combo == "Mod+Return"));

    let rewritten = fs::read_to_string(&paths.niri_config).unwrap();
    assert!(rewritten.contains("include \"nirify/main.kdl\""));
    assert!(!rewritten.contains("output \"DP-1\""));
    assert!(result.replace.backup_path.exists());
}

#[test]
fn launch_absorb_merges_stripped_output_and_bind() {
    let dir = tempdir().unwrap();
    let paths = paths_under(dir.path());
    paths.ensure_directories().unwrap();

    let mut existing = nirify::config::Settings::default();
    existing.appearance.gaps = 16.0;
    existing.outputs.outputs.push(nirify::config::OutputConfig {
        name: "eDP-1".to_string(),
        ..Default::default()
    });
    nirify::config::save_settings(&paths, &existing, FeatureCompat::all_enabled()).unwrap();

    fs::write(
        &paths.niri_config,
        r#"
layout { gaps 99 }
output "HDMI-A-1" {
    position x=1920 y=0
}
binds {
    Mod+Q { close-window; }
}
include "nirify/main.kdl"
"#,
    )
    .unwrap();

    let result = absorb_stripped_nodes(&paths, FeatureCompat::all_enabled()).unwrap();
    assert!(result.adopted.contains(&SettingsCategory::Outputs));
    assert!(result.adopted.contains(&SettingsCategory::Keybindings));
    assert!(!result.adopted.contains(&SettingsCategory::Appearance));

    let loaded = load_settings(&paths);
    assert_eq!(loaded.appearance.gaps, 16.0);
    assert!(loaded.outputs.outputs.iter().any(|o| o.name == "eDP-1"));
    assert!(loaded.outputs.outputs.iter().any(|o| o.name == "HDMI-A-1"));
    assert!(loaded
        .keybindings
        .bindings
        .iter()
        .any(|b| normalized_key_combo(&b.key_combo) == normalized_key_combo("Mod+Q")));
}

#[test]
fn wizard_imports_catchall_window_rule_effects() {
    let dir = tempdir().unwrap();
    let paths = paths_under(dir.path());

    fs::write(
        &paths.niri_config,
        r#"
window-rule {
    opacity 0.9
    clip-to-geometry true
    open-maximized true
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

    if paths.managed_dir.exists() {
        fs::remove_dir_all(&paths.managed_dir).unwrap();
    }

    first_run_setup(&paths, FeatureCompat::all_enabled()).unwrap();

    let loaded = load_settings(&paths);
    let catch_all = loaded
        .window_rules
        .rules
        .iter()
        .find(|r| r.is_catch_all())
        .expect("catch-all must survive first_run_setup");
    assert_eq!(catch_all.opacity, Some(0.9));
    assert_eq!(catch_all.clip_to_geometry, Some(true));
    assert_eq!(catch_all.open_maximized, Some(true));
    assert_eq!(catch_all.draw_border_with_background, Some(false));
    assert_eq!(
        catch_all.corner_radius,
        Some(nirify::config::models::CornerRadiusValue::uniform(12.0))
    );
    assert_eq!(loaded.appearance.corner_radius, 12.0);

    let appearance = fs::read_to_string(&paths.appearance_kdl).unwrap();
    assert!(
        !appearance.contains("window-rule"),
        "appearance.kdl must not emit a second radius-only catch-all:\n{appearance}"
    );
    let rules = fs::read_to_string(&paths.window_rules_kdl).unwrap();
    assert!(rules.contains("opacity 0.9"), "{rules}");
    assert!(rules.contains("clip-to-geometry true"), "{rules}");
    assert!(rules.contains("open-maximized true"), "{rules}");
    assert!(rules.contains("geometry-corner-radius 12"), "{rules}");
}

#[test]
fn launch_absorb_adopts_catchall_window_rule() {
    let dir = tempdir().unwrap();
    let paths = paths_under(dir.path());
    paths.ensure_directories().unwrap();

    let mut existing = nirify::config::Settings::default();
    existing.appearance.gaps = 16.0;
    existing.appearance.corner_radius = 8.0;
    nirify::config::save_settings(&paths, &existing, FeatureCompat::all_enabled()).unwrap();

    let appearance_before = fs::read_to_string(&paths.appearance_kdl).unwrap();
    assert!(
        appearance_before.contains("geometry-corner-radius 8"),
        "precondition: appearance emits radius-only catch-all:\n{appearance_before}"
    );

    fs::write(
        &paths.niri_config,
        r#"
window-rule {
    opacity 0.85
    clip-to-geometry true
    open-fullscreen true
    geometry-corner-radius 16
}
include "nirify/main.kdl"
"#,
    )
    .unwrap();

    let result = absorb_stripped_nodes(&paths, FeatureCompat::all_enabled()).unwrap();
    assert!(result.adopted.contains(&SettingsCategory::WindowRules));

    let loaded = load_settings(&paths);
    let catch_all = loaded
        .window_rules
        .rules
        .iter()
        .find(|r| r.is_catch_all())
        .expect("absorb must adopt the catch-all, not strip it");
    assert_eq!(catch_all.opacity, Some(0.85));
    assert_eq!(catch_all.clip_to_geometry, Some(true));
    assert_eq!(catch_all.open_fullscreen, Some(true));
    assert_eq!(
        catch_all.corner_radius,
        Some(nirify::config::models::CornerRadiusValue::uniform(16.0))
    );
    assert_eq!(loaded.appearance.corner_radius, 16.0);

    let appearance_after = fs::read_to_string(&paths.appearance_kdl).unwrap();
    assert!(
        !appearance_after.contains("window-rule"),
        "absorb must stop dual-emitting a radius-only catch-all:\n{appearance_after}"
    );
    let rules = fs::read_to_string(&paths.window_rules_kdl).unwrap();
    assert!(rules.contains("opacity 0.85"), "{rules}");
    assert!(rules.contains("clip-to-geometry true"), "{rules}");
    assert!(rules.contains("open-fullscreen true"), "{rules}");
}

#[test]
fn wizard_imports_window_rule_gradients() {
    let dir = tempdir().unwrap();
    let paths = paths_under(dir.path());

    fs::write(
        &paths.niri_config,
        r##"
window-rule {
    match app-id="kitty"
    focus-ring {
        active-gradient from="#80c8ff" to="#bbddff"
        inactive-color "#333333"
        urgent-gradient from="#ff0000" to="#9b0000"
    }
    border {
        inactive-gradient from="#202020" to="#101010"
        urgent-color "#ff8800"
    }
}
"##,
    )
    .unwrap();

    if paths.managed_dir.exists() {
        fs::remove_dir_all(&paths.managed_dir).unwrap();
    }

    first_run_setup(&paths, FeatureCompat::all_enabled()).unwrap();

    let loaded = load_settings(&paths);
    let rule = loaded
        .window_rules
        .rules
        .iter()
        .find(|r| {
            r.matches
                .iter()
                .any(|m| m.app_id.as_deref() == Some("kitty"))
        })
        .expect("imported rule");
    assert!(
        matches!(
            rule.focus_ring_active,
            Some(nirify::types::ColorOrGradient::Gradient(_))
        ),
        "{:?}",
        rule.focus_ring_active
    );
    assert!(
        matches!(
            rule.focus_ring_urgent,
            Some(nirify::types::ColorOrGradient::Gradient(_))
        ),
        "{:?}",
        rule.focus_ring_urgent
    );

    let rules = fs::read_to_string(&paths.window_rules_kdl).unwrap();
    assert!(rules.contains("active-gradient"), "{rules}");
    assert!(rules.contains("urgent-gradient"), "{rules}");
    assert!(rules.contains("inactive-gradient"), "{rules}");
    assert!(rules.contains("urgent-color"), "{rules}");
}

#[test]
fn launch_absorb_adopts_window_rule_gradients() {
    let dir = tempdir().unwrap();
    let paths = paths_under(dir.path());
    paths.ensure_directories().unwrap();

    let existing = nirify::config::Settings::default();
    nirify::config::save_settings(&paths, &existing, FeatureCompat::all_enabled()).unwrap();

    fs::write(
        &paths.niri_config,
        r##"
window-rule {
    match app-id="absorb-grad"
    focus-ring {
        active-gradient from="#112233" to="#445566" angle=90
        urgent-gradient from="#ff0000" to="#990000"
    }
}
include "nirify/main.kdl"
"##,
    )
    .unwrap();

    let result = absorb_stripped_nodes(&paths, FeatureCompat::all_enabled()).unwrap();
    assert!(result.adopted.contains(&SettingsCategory::WindowRules));

    let loaded = load_settings(&paths);
    let rule = loaded
        .window_rules
        .rules
        .iter()
        .find(|r| {
            r.matches
                .iter()
                .any(|m| m.app_id.as_deref() == Some("absorb-grad"))
        })
        .expect("absorb must adopt the gradient rule, not strip it");
    assert!(
        matches!(
            rule.focus_ring_active,
            Some(nirify::types::ColorOrGradient::Gradient(_))
        ),
        "{:?}",
        rule.focus_ring_active
    );

    let rules = fs::read_to_string(&paths.window_rules_kdl).unwrap();
    assert!(rules.contains("active-gradient"), "{rules}");
    assert!(rules.contains("urgent-gradient"), "{rules}");
    assert!(rules.contains("absorb-grad"), "{rules}");
}
