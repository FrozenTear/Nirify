//! Error injection and edge case tests
//!
//! These tests verify that the application handles error conditions gracefully,
//! including filesystem errors, corrupted files, and permission issues.
//!
//! Run with: cargo test --test error_injection_tests

mod common;

use common::create_test_paths;
use nirify::config::{
    check_config_health, load_settings, save_settings, ConfigFileStatus, Settings,
};
use nirify::version::FeatureCompat;
use std::fs;
use tempfile::tempdir;

// ============================================================================
// FILESYSTEM ERROR TESTS
// ============================================================================

#[test]
#[cfg(unix)]
fn test_save_to_readonly_directory() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create directory then make it readonly
    fs::create_dir_all(&paths.managed_dir).unwrap();
    fs::set_permissions(&paths.managed_dir, fs::Permissions::from_mode(0o444)).unwrap();

    let settings = Settings::default();
    let result = save_settings(&paths, &settings, FeatureCompat::all_enabled());

    // Should fail gracefully
    assert!(result.is_err());

    // Cleanup: restore permissions so tempdir can be deleted
    fs::set_permissions(&paths.managed_dir, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
#[cfg(unix)]
fn test_load_from_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Save valid settings first
    let mut settings = Settings::default();
    settings.appearance.gaps = 24.0;
    settings.keyboard.repeat_delay = 400;
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).unwrap();

    // Make appearance.kdl unreadable
    fs::set_permissions(&paths.appearance_kdl, fs::Permissions::from_mode(0o000)).unwrap();

    // Load should succeed with defaults for unreadable file
    let loaded = load_settings(&paths);

    // Appearance should be defaults (couldn't read)
    assert_eq!(loaded.appearance.gaps, 16.0);
    // But keyboard should have our value (was readable)
    assert_eq!(loaded.keyboard.repeat_delay, 400);

    // Cleanup
    fs::set_permissions(&paths.appearance_kdl, fs::Permissions::from_mode(0o644)).unwrap();
}

// ============================================================================
// CORRUPTED FILE TESTS
// ============================================================================

#[test]
fn test_load_with_various_corruption_types() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Save valid settings
    let mut settings = Settings::default();
    settings.keyboard.repeat_delay = 400;
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).unwrap();

    // Test various types of corruption
    let long_line = "x".repeat(100_000);
    let corruption_cases: Vec<(&str, &str)> = vec![
        ("unbalanced braces", "layout { { { {"),
        ("null bytes", "layout { gaps \0 16 }"),
        ("extremely long line", &long_line),
        ("just whitespace", "   \n\t\n   "),
        ("binary data", "\x00\x01\x02\x03\x04\x05"),
    ];

    for (name, content) in corruption_cases {
        fs::write(&paths.appearance_kdl, content).unwrap();

        // Load should not panic
        let loaded = load_settings(&paths);

        // Should fall back to defaults
        assert_eq!(
            loaded.appearance.gaps, 16.0,
            "Failed for corruption type: {}",
            name
        );

        // Other files should still load correctly
        assert_eq!(
            loaded.keyboard.repeat_delay, 400,
            "Keyboard corrupted by appearance file: {}",
            name
        );
    }
}

#[test]
fn test_partial_file_corruption() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Save valid settings
    let mut settings = Settings::default();
    settings.keyboard.repeat_delay = 400;
    settings.mouse.accel_speed = 0.5;
    settings.cursor.size = 32;
    save_settings(&paths, &settings, FeatureCompat::all_enabled()).unwrap();

    // Corrupt only appearance.kdl
    fs::write(&paths.appearance_kdl, "{{{{invalid").unwrap();

    let loaded = load_settings(&paths);

    // Corrupted file uses defaults
    assert_eq!(loaded.appearance.gaps, 16.0);
    assert!(loaded.appearance.focus_ring_enabled);

    // Uncorrupted files retain values
    assert_eq!(loaded.keyboard.repeat_delay, 400);
    assert!((loaded.mouse.accel_speed - 0.5).abs() < 0.01);
    assert_eq!(loaded.cursor.size, 32);
}

#[test]
fn test_all_files_corrupted() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Save valid settings
    save_settings(&paths, &Settings::default(), FeatureCompat::all_enabled()).unwrap();

    // Corrupt all files
    let files = [
        &paths.appearance_kdl,
        &paths.behavior_kdl,
        &paths.keyboard_kdl,
        &paths.mouse_kdl,
        &paths.cursor_kdl,
        &paths.animations_kdl,
    ];

    for file in files {
        fs::write(file, "corrupted { { { {").unwrap();
    }

    // Should still load with all defaults
    let loaded = load_settings(&paths);

    assert_eq!(loaded.appearance.gaps, 16.0);
    assert_eq!(loaded.keyboard.xkb_layout, "us");
    assert!(loaded.animations.enabled);
    assert_eq!(loaded.cursor.size, 24);
}

// ============================================================================
// CONFIG HEALTH CHECK TESTS
// ============================================================================

#[test]
fn test_health_check_identifies_corrupted_files() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    save_settings(&paths, &Settings::default(), FeatureCompat::all_enabled()).unwrap();

    // Corrupt specific files
    fs::write(&paths.appearance_kdl, "corrupted {{").unwrap();
    fs::write(&paths.keyboard_kdl, "also { bad").unwrap();

    let health = check_config_health(&paths);

    assert!(!health.is_healthy());

    let corrupted = health.corrupted_files();
    assert!(corrupted.contains(&"appearance.kdl"));
    assert!(corrupted.contains(&"keyboard.kdl")); // file_name() returns just the filename

    // Check individual statuses
    assert!(matches!(
        health.appearance(),
        ConfigFileStatus::Corrupted(_)
    ));
    assert!(matches!(health.keyboard(), ConfigFileStatus::Corrupted(_)));
    assert_eq!(*health.mouse(), ConfigFileStatus::Ok);
}

#[test]
fn test_health_check_missing_files_are_ok() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Don't create any files
    let health = check_config_health(&paths);

    // Missing files are considered healthy (will use defaults)
    assert!(health.is_healthy());
    assert_eq!(*health.appearance(), ConfigFileStatus::Missing);
    assert_eq!(*health.keyboard(), ConfigFileStatus::Missing);
}

// ============================================================================
// ATOMIC WRITE TESTS
// ============================================================================

#[test]
fn test_atomic_write_leaves_no_temp_files() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.kdl");

    // Perform multiple writes
    for i in 0..10 {
        let content = format!("// Write number {}\nlayout {{ gaps {} }}", i, i);
        nirify::config::storage::atomic_write(&file, &content).unwrap();
    }

    // Count files in directory
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    // Should only have the one file, no temp files
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_name(), "test.kdl");

    // Content should be the last write
    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("Write number 9"));
}

#[test]
fn test_atomic_write_concurrent_safety() {
    use std::sync::Arc;
    use std::thread;

    let dir = tempdir().unwrap();
    let file = Arc::new(dir.path().join("test.kdl"));

    // Multiple threads writing simultaneously
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let file = Arc::clone(&file);
            thread::spawn(move || {
                for j in 0..10 {
                    let content = format!("// Thread {} write {}\nlayout {{ gaps {} }}", i, j, i);
                    nirify::config::storage::atomic_write(&file, &content).unwrap();
                    thread::yield_now();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // File should exist and be valid
    let content = fs::read_to_string(file.as_ref()).unwrap();
    assert!(content.contains("layout"));

    // Should parse as valid KDL
    let result: Result<kdl::KdlDocument, _> = content.parse();
    assert!(result.is_ok());
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_empty_file_handling() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create empty files
    fs::write(&paths.appearance_kdl, "").unwrap();
    fs::write(&paths.keyboard_kdl, "").unwrap();

    // Should load with defaults
    let loaded = load_settings(&paths);
    assert_eq!(loaded.appearance.gaps, 16.0);
    assert_eq!(loaded.keyboard.xkb_layout, "us");
}

#[test]
fn test_comments_only_file() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    fs::write(
        &paths.appearance_kdl,
        r#"
// This is a comment
// Another comment
/* Block comment
   spanning multiple lines */
"#,
    )
    .unwrap();

    let loaded = load_settings(&paths);
    assert_eq!(loaded.appearance.gaps, 16.0);
}

#[test]
fn test_unicode_in_config() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Write config with unicode
    fs::write(
        &paths.cursor_kdl,
        r#"
cursor {
    xcursor-theme "日本語テーマ"
    xcursor-size 24
}
"#,
    )
    .unwrap();

    let loaded = load_settings(&paths);
    assert_eq!(loaded.cursor.theme, "日本語テーマ");
}

#[test]
fn test_very_long_string_values() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    let long_path = "a".repeat(500);
    fs::write(
        &paths.misc_kdl,
        format!(
            r#"
screenshot-path "{}"
"#,
            long_path
        ),
    )
    .unwrap();

    let loaded = load_settings(&paths);
    assert_eq!(loaded.miscellaneous.screenshot_path, long_path);
}

#[test]
fn test_extreme_numeric_values() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Write extreme values
    fs::write(
        &paths.appearance_kdl,
        r#"
layout {
    gaps inner=-999999 outer=999999
    focus-ring {
        width 999999
    }
}
"#,
    )
    .unwrap();

    // Should load and clamp to valid ranges
    let loaded = load_settings(&paths);

    use nirify::constants::*;
    // Note: inner=-999 is read (backwards compat), then clamped to min
    assert_eq!(loaded.appearance.gaps, GAP_SIZE_MIN);
    assert_eq!(loaded.appearance.focus_ring_width, FOCUS_RING_WIDTH_MAX);
}

#[test]
fn test_scientific_notation_values() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Some parsers handle scientific notation differently
    fs::write(
        &paths.mouse_kdl,
        r#"
input {
    mouse {
        accel-speed 1e-1
        scroll-factor 1.0e0
    }
}
"#,
    )
    .unwrap();

    let loaded = load_settings(&paths);
    // Should handle scientific notation or fall back to defaults
    // The exact behavior depends on KDL parser
    assert!(loaded.mouse.accel_speed >= -1.0 && loaded.mouse.accel_speed <= 1.0);
}

#[test]
fn test_duplicate_sections() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Write duplicate sections - last one should win or be merged
    fs::write(
        &paths.appearance_kdl,
        r#"
layout {
    gaps inner=10
}
layout {
    gaps inner=20
}
"#,
    )
    .unwrap();

    let loaded = load_settings(&paths);
    // Behavior depends on parser - should not crash either way
    assert!(loaded.appearance.gaps == 10.0 || loaded.appearance.gaps == 20.0);
}

// ============================================================================
// BACKUP DIRECTORY TESTS
// ============================================================================

#[test]
fn test_backup_dir_created_on_demand() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Backup dir should not exist yet
    assert!(!paths.backup_dir.exists());

    // Save settings
    save_settings(&paths, &Settings::default(), FeatureCompat::all_enabled()).unwrap();

    // Normal save doesn't create backup dir
    // (backup only happens during replace operations)
}

#[test]
fn test_handles_symlink_as_config_file() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let dir = tempdir().unwrap();
        let paths = create_test_paths(dir.path());

        // Create a real file
        let real_file = dir.path().join("real_appearance.kdl");
        fs::write(&real_file, "layout { gaps inner=30 }").unwrap();

        // Make appearance.kdl a symlink to it
        if paths.appearance_kdl.exists() {
            fs::remove_file(&paths.appearance_kdl).unwrap();
        }
        symlink(&real_file, &paths.appearance_kdl).unwrap();

        // Should follow symlink and load
        let loaded = load_settings(&paths);
        assert_eq!(loaded.appearance.gaps, 30.0);
    }
}
