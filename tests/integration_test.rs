//! Integration tests for full app lifecycle
//!
//! Tests the complete flow of:
//! - Fresh start with no config (defaults used)
//! - Saving settings
//! - Loading settings
//! - Modifying and re-saving
//! - Error recovery from corrupted files

mod common;

use common::create_test_paths;
use nirify::config::{
    check_config_health, ensure_required_files_exist, load_settings, repair_corrupted_configs,
    save_settings, ConfigFileStatus, Settings,
};
use nirify::version::FeatureCompat;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_fresh_start_uses_defaults() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Load from non-existent directory - should return defaults
    let settings = load_settings(&paths);

    // Verify defaults are used
    assert!(settings.appearance.focus_ring_enabled);
    assert_eq!(settings.appearance.gaps, 16.0);
    assert!(settings.animations.enabled);
    assert_eq!(settings.keyboard.xkb_layout, "us");
}

#[test]
fn test_full_lifecycle_save_load_modify() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Step 1: Create initial settings with custom values
    let mut settings = Settings::default();
    settings.appearance.gaps = 24.0;
    settings.appearance.focus_ring_width = 6.0;
    settings.appearance.corner_radius = 16.0;
    settings.keyboard.repeat_delay = 400;
    settings.keyboard.repeat_rate = 30;
    settings.mouse.accel_speed = 0.5;
    settings.animations.enabled = false;
    settings.cursor.size = 32;
    settings.overview.zoom = 0.75;
    settings.miscellaneous.prefer_no_csd = true;

    // Step 2: Save settings
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save settings");

    // Verify files were created
    assert!(paths.main_kdl.exists(), "main.kdl should exist");
    assert!(paths.appearance_kdl.exists(), "appearance.kdl should exist");
    assert!(paths.keyboard_kdl.exists(), "keyboard.kdl should exist");
    assert!(paths.misc_kdl.exists(), "misc.kdl should exist");

    // Step 3: Load settings back
    let loaded = load_settings(&paths);

    // Step 4: Verify all values match
    assert_eq!(loaded.appearance.gaps, 24.0);
    assert_eq!(loaded.appearance.focus_ring_width, 6.0);
    assert_eq!(loaded.appearance.corner_radius, 16.0);
    assert_eq!(loaded.keyboard.repeat_delay, 400);
    assert_eq!(loaded.keyboard.repeat_rate, 30);
    assert!((loaded.mouse.accel_speed - 0.5).abs() < 0.01);
    assert!(!loaded.animations.enabled);
    assert_eq!(loaded.cursor.size, 32);
    assert!((loaded.overview.zoom - 0.75).abs() < 0.01);
    assert!(loaded.miscellaneous.prefer_no_csd);

    // Step 5: Modify settings
    let mut modified = loaded.clone();
    modified.appearance.gaps = 32.0;
    modified.keyboard.repeat_delay = 500;
    modified.touchpad.tap = true;
    modified.touchpad.natural_scroll = false;

    // Step 6: Save modified settings
    save_settings(&paths, &modified, FeatureCompat::all_enabled()).expect("Failed to save modified settings");

    // Step 7: Load again and verify modifications persisted
    let reloaded = load_settings(&paths);
    assert_eq!(reloaded.appearance.gaps, 32.0);
    assert_eq!(reloaded.keyboard.repeat_delay, 500);
    assert!(reloaded.touchpad.tap);
    assert!(!reloaded.touchpad.natural_scroll);

    // Original unchanged values should still be correct
    assert_eq!(reloaded.appearance.corner_radius, 16.0);
    assert_eq!(reloaded.cursor.size, 32);
}

#[test]
fn test_partial_config_files() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create only some config files - simulates partial/incomplete setup
    fs::write(
        &paths.appearance_kdl,
        r##"
layout {
    gaps inner=20 outer=10
    focus-ring {
        width 8
        active-color "#ff0000"
    }
}
"##,
    )
    .unwrap();

    // Load settings - should use defaults for missing files
    let settings = load_settings(&paths);

    // Appearance should have custom values
    // Note: inner=20 outer=10 config will use inner value since niri uses single gaps
    assert_eq!(settings.appearance.gaps, 20.0);
    assert_eq!(settings.appearance.focus_ring_width, 8.0);

    // Other settings should be defaults
    assert_eq!(settings.keyboard.xkb_layout, "us");
    assert!(settings.animations.enabled);
    assert_eq!(settings.cursor.size, 24);
}

#[test]
fn test_corrupted_file_recovery() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // First, save valid settings
    let mut settings = Settings::default();
    settings.appearance.gaps = 24.0;
    settings.keyboard.repeat_delay = 400;
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save settings");

    // Now corrupt one file with invalid KDL
    fs::write(&paths.appearance_kdl, "this is { not valid kdl {{{{").unwrap();

    // Load should succeed, using defaults for corrupted file
    let loaded = load_settings(&paths);

    // Corrupted file (appearance) should have defaults
    assert_eq!(loaded.appearance.gaps, 16.0); // Default, not 24.0
    assert!(loaded.appearance.focus_ring_enabled); // Default

    // Other files should still load correctly
    assert_eq!(loaded.keyboard.repeat_delay, 400); // Our saved value
}

#[test]
fn test_validation_on_load() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Write file with out-of-range values
    fs::write(
        &paths.appearance_kdl,
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

    // Load settings - validation should clamp values
    let settings = load_settings(&paths);

    // Values should be clamped to valid ranges
    // Note: inner=-50 outer=999 - the loader reads inner first, which gets clamped to 0
    assert_eq!(settings.appearance.gaps, 0.0); // GAP_SIZE_MIN (from clamping -50)
    assert_eq!(settings.appearance.focus_ring_width, 16.0); // FOCUS_RING_WIDTH_MAX
}

#[test]
fn test_window_rules_lifecycle() {
    use nirify::config::models::{OpenBehavior, WindowRule, WindowRuleMatch};

    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create settings with window rules
    let mut settings = Settings::default();
    // Disable float preference to test user-defined rules in isolation
    settings.preferences.float_settings_app = false;
    settings.window_rules.rules.push(WindowRule {
        id: 1,
        name: "Firefox Floating".to_string(),
        matches: vec![WindowRuleMatch {
            app_id: Some("firefox".to_string()),
            ..Default::default()
        }],
        open_behavior: OpenBehavior::Floating,
        opacity: Some(0.95),
        corner_radius: Some(12),
        ..Default::default()
    });
    settings.window_rules.next_id = 2;

    // Save and reload
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");
    let loaded = load_settings(&paths);

    // Verify window rule was preserved
    assert_eq!(loaded.window_rules.rules.len(), 1);
    let rule = &loaded.window_rules.rules[0];
    assert_eq!(rule.matches[0].app_id, Some("firefox".to_string()));
    assert_eq!(rule.open_behavior, OpenBehavior::Floating);
    assert!((rule.opacity.unwrap() - 0.95).abs() < 0.01);
    assert_eq!(rule.corner_radius, Some(12));
}

#[test]
fn test_output_settings_lifecycle() {
    use nirify::config::models::OutputConfig;
    use nirify::types::{Transform, VrrMode};

    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create settings with output config
    let mut settings = Settings::default();
    settings.outputs.outputs.push(OutputConfig {
        name: "DP-1".to_string(),
        enabled: true,
        scale: 1.5,
        mode: "2560x1440@144".to_string(),
        position_x: 0,
        position_y: 0,
        transform: Transform::Normal,
        vrr: VrrMode::On,
        focus_at_startup: true,
        backdrop_color: None,
        ..Default::default()
    });
    settings.outputs.outputs.push(OutputConfig {
        name: "HDMI-A-1".to_string(),
        enabled: false,
        ..Default::default()
    });

    // Save and reload
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");
    let loaded = load_settings(&paths);

    // Verify outputs were preserved
    assert_eq!(loaded.outputs.outputs.len(), 2);

    let dp1 = loaded
        .outputs
        .outputs
        .iter()
        .find(|o| o.name == "DP-1")
        .unwrap();
    assert!(dp1.enabled);
    assert!((dp1.scale - 1.5).abs() < 0.01);
    assert_eq!(dp1.mode, "2560x1440@144");
    assert_eq!(dp1.vrr, VrrMode::On);
    assert!(dp1.focus_at_startup);

    let hdmi = loaded
        .outputs
        .outputs
        .iter()
        .find(|o| o.name == "HDMI-A-1")
        .unwrap();
    assert!(!hdmi.enabled);
}

#[test]
fn test_check_config_health_all_valid() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Save valid settings
    let settings = Settings::default();
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");

    // Check health - all should be Ok
    let health = check_config_health(&paths);
    assert!(health.is_healthy());
    assert!(health.corrupted_files().is_empty());
    assert!(health.unreadable_files().is_empty());

    // Individual file checks
    assert_eq!(*health.appearance(), ConfigFileStatus::Ok);
    assert_eq!(*health.behavior(), ConfigFileStatus::Ok);
    assert_eq!(*health.keyboard(), ConfigFileStatus::Ok);
}

#[test]
fn test_check_config_health_missing_files() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Don't create any files - all should be Missing
    let health = check_config_health(&paths);

    // Missing files are considered healthy (will use defaults)
    assert!(health.is_healthy());
    assert_eq!(*health.appearance(), ConfigFileStatus::Missing);
    assert_eq!(*health.keyboard(), ConfigFileStatus::Missing);
}

#[test]
fn test_check_config_health_corrupted() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create a corrupted file
    fs::write(&paths.appearance_kdl, "this { is not {{ valid kdl").unwrap();

    // Check health
    let health = check_config_health(&paths);
    assert!(!health.is_healthy());

    let corrupted = health.corrupted_files();
    assert_eq!(corrupted.len(), 1);
    assert!(corrupted.contains(&"appearance.kdl"));

    // Verify the error message is captured
    if let ConfigFileStatus::Corrupted(msg) = health.appearance() {
        assert!(!msg.is_empty());
    } else {
        panic!("Expected Corrupted status for appearance.kdl");
    }
}

#[test]
fn test_repair_corrupted_configs() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // First save valid settings
    let mut settings = Settings::default();
    settings.keyboard.repeat_delay = 400;
    settings.mouse.accel_speed = 0.5;
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");

    // Corrupt one file
    fs::write(&paths.appearance_kdl, "corrupted {{ data").unwrap();

    // Verify it's corrupted
    let health = check_config_health(&paths);
    assert!(!health.is_healthy());
    assert_eq!(health.corrupted_files(), vec!["appearance.kdl"]);

    // Load settings (will fall back to defaults for appearance)
    let loaded = load_settings(&paths);

    // Appearance should have defaults, but keyboard/mouse should have our values
    assert_eq!(loaded.appearance.gaps, 16.0); // Default
    assert_eq!(loaded.keyboard.repeat_delay, 400); // Our value preserved
    assert!((loaded.mouse.accel_speed - 0.5).abs() < 0.01); // Our value preserved

    // Repair corrupted configs
    let repaired = repair_corrupted_configs(&paths, &loaded, FeatureCompat::all_enabled()).expect("Repair failed");
    assert_eq!(repaired.len(), 1);
    assert!(repaired.contains(&"appearance.kdl".to_string()));

    // Verify backup was created
    let backup_files: Vec<_> = fs::read_dir(&paths.backup_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(backup_files.len(), 1);
    let backup_name = backup_files[0].file_name();
    let backup_str = backup_name.to_string_lossy();
    assert!(backup_str.contains("appearance.kdl"));
    assert!(backup_str.contains(".corrupted.bak"));

    // Health should now be good
    let health_after = check_config_health(&paths);
    assert!(health_after.is_healthy());
    assert_eq!(*health_after.appearance(), ConfigFileStatus::Ok);
}

#[test]
fn test_repair_no_corrupted_files() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Save valid settings
    let settings = Settings::default();
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");

    // Try to repair - should do nothing
    let repaired = repair_corrupted_configs(&paths, &settings, FeatureCompat::all_enabled()).expect("Repair failed");
    assert!(repaired.is_empty());

    // Backup directory should be empty (or not exist)
    if paths.backup_dir.exists() {
        let count = fs::read_dir(&paths.backup_dir).unwrap().count();
        assert_eq!(count, 0);
    }
}

#[test]
fn test_ensure_required_files_creates_missing() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create directories but only some files (simulating upgrade from older version)
    fs::create_dir_all(&paths.managed_dir).unwrap();
    fs::create_dir_all(&paths.input_dir).unwrap();
    fs::create_dir_all(&paths.advanced_dir).unwrap();

    // Save only a subset of files (simulating older version)
    let settings = Settings::default();
    fs::write(&paths.appearance_kdl, "layout { gaps 16 }").unwrap();
    fs::write(&paths.behavior_kdl, "// behavior").unwrap();
    // Deliberately leave startup.kdl, environment.kdl, etc. missing

    // Verify startup.kdl doesn't exist
    assert!(!paths.startup_kdl.exists());
    assert!(!paths.environment_kdl.exists());

    // Call ensure_required_files_exist
    let created = ensure_required_files_exist(&paths, &settings, FeatureCompat::all_enabled())
        .expect("Should create missing files");

    // Should have created the missing files
    assert!(!created.is_empty());
    assert!(created.iter().any(|f| f.contains("startup.kdl")));

    // Now verify the files exist
    assert!(paths.startup_kdl.exists());
    assert!(paths.environment_kdl.exists());

    // Files that already existed should not be in the created list
    assert!(!created.iter().any(|f| f == "appearance.kdl"));
    assert!(!created.iter().any(|f| f == "behavior.kdl"));
}

#[test]
fn test_ensure_required_files_does_nothing_when_all_exist() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create all files by doing a full save
    let settings = Settings::default();
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");

    // Call ensure_required_files_exist
    let created = ensure_required_files_exist(&paths, &settings, FeatureCompat::all_enabled())
        .expect("Should succeed");

    // Should not have created anything since all files exist
    assert!(created.is_empty());
}
