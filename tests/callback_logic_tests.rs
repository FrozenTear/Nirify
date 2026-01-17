//! Tests for callback logic
//!
//! These tests verify the underlying logic that callbacks use, including:
//! - Value clamping (simulating what register_clamped_callback does)
//! - Settings field updates
//! - Save-dirty integration
//!
//! Note: We can't test Slint callbacks directly without the full runtime,
//! but we can test the logic they invoke.

mod common;

use common::create_test_paths;
use niri_settings::config::{
    load_settings, save_dirty, save_settings, DirtyTracker, Settings, SettingsCategory,
};
use niri_settings::constants::*;
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::tempdir;

/// Test that clamping logic works correctly for appearance values
/// This simulates what register_clamped_callback! does
#[test]
fn test_appearance_clamping_logic() {
    // Simulate callback clamping behavior
    let clamp_focus_ring = |val: f32| val.clamp(FOCUS_RING_WIDTH_MIN, FOCUS_RING_WIDTH_MAX);
    let clamp_gaps = |val: f32| val.clamp(GAP_SIZE_MIN, GAP_SIZE_MAX);
    let clamp_corner_radius = |val: f32| val.clamp(CORNER_RADIUS_MIN, CORNER_RADIUS_MAX);

    // Test focus ring width clamping
    assert_eq!(clamp_focus_ring(0.0), FOCUS_RING_WIDTH_MIN); // Below min
    assert_eq!(clamp_focus_ring(100.0), FOCUS_RING_WIDTH_MAX); // Above max
    assert_eq!(clamp_focus_ring(4.0), 4.0); // Within range

    // Test gaps clamping
    assert_eq!(clamp_gaps(-10.0), GAP_SIZE_MIN); // Negative
    assert_eq!(clamp_gaps(1000.0), GAP_SIZE_MAX); // Way above max
    assert_eq!(clamp_gaps(16.0), 16.0); // Valid value

    // Test corner radius clamping
    assert_eq!(clamp_corner_radius(-5.0), CORNER_RADIUS_MIN);
    assert_eq!(clamp_corner_radius(200.0), CORNER_RADIUS_MAX);
    assert_eq!(clamp_corner_radius(12.0), 12.0);
}

/// Test that clamping logic works for input device values
#[test]
fn test_input_clamping_logic() {
    let clamp_accel = |val: f64| val.clamp(ACCEL_SPEED_MIN, ACCEL_SPEED_MAX);
    let clamp_scroll_factor = |val: f64| val.clamp(SCROLL_FACTOR_MIN, SCROLL_FACTOR_MAX);

    // Accel speed: -1.0 to 1.0
    assert_eq!(clamp_accel(-2.0), ACCEL_SPEED_MIN);
    assert_eq!(clamp_accel(2.0), ACCEL_SPEED_MAX);
    assert_eq!(clamp_accel(0.5), 0.5);

    // Scroll factor: 0.1 to 10.0
    assert_eq!(clamp_scroll_factor(0.0), SCROLL_FACTOR_MIN);
    assert_eq!(clamp_scroll_factor(100.0), SCROLL_FACTOR_MAX);
    assert_eq!(clamp_scroll_factor(1.5), 1.5);
}

/// Test that clamping logic works for keyboard values
#[test]
fn test_keyboard_clamping_logic() {
    let clamp_delay = |val: i32| val.clamp(REPEAT_DELAY_MIN, REPEAT_DELAY_MAX);
    let clamp_rate = |val: i32| val.clamp(REPEAT_RATE_MIN, REPEAT_RATE_MAX);

    // Repeat delay: 100-2000ms
    assert_eq!(clamp_delay(50), REPEAT_DELAY_MIN);
    assert_eq!(clamp_delay(5000), REPEAT_DELAY_MAX);
    assert_eq!(clamp_delay(600), 600);

    // Repeat rate: 1-100
    assert_eq!(clamp_rate(0), REPEAT_RATE_MIN);
    assert_eq!(clamp_rate(200), REPEAT_RATE_MAX);
    assert_eq!(clamp_rate(25), 25);
}

/// Test save_dirty with a single category
#[test]
fn test_save_dirty_single_category() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Create initial settings
    let mut settings = Settings::default();
    settings.appearance.gaps = 24.0;

    // Save all first to create files
    save_settings(&paths, &settings).expect("Initial save failed");

    // Modify appearance
    settings.appearance.gaps = 32.0;

    // Save only appearance
    let mut dirty = HashSet::new();
    dirty.insert(SettingsCategory::Appearance);

    let files_written = save_dirty(&paths, &settings, &dirty).expect("save_dirty failed");
    assert_eq!(files_written, 1, "Should write exactly 1 file");

    // Verify the change persisted
    let loaded = load_settings(&paths);
    assert_eq!(loaded.appearance.gaps, 32.0);
}

/// Test save_dirty with multiple categories
#[test]
fn test_save_dirty_multiple_categories() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    let mut settings = Settings::default();
    save_settings(&paths, &settings).expect("Initial save failed");

    // Modify multiple categories
    settings.appearance.focus_ring_width = 6.0;
    settings.cursor.size = 32;
    settings.keyboard.repeat_delay = 400;

    // Mark all three as dirty
    let mut dirty = HashSet::new();
    dirty.insert(SettingsCategory::Appearance);
    dirty.insert(SettingsCategory::Cursor);
    dirty.insert(SettingsCategory::Keyboard);

    let files_written = save_dirty(&paths, &settings, &dirty).expect("save_dirty failed");
    assert_eq!(files_written, 3, "Should write exactly 3 files");

    // Verify all changes persisted
    let loaded = load_settings(&paths);
    assert_eq!(loaded.appearance.focus_ring_width, 6.0);
    assert_eq!(loaded.cursor.size, 32);
    assert_eq!(loaded.keyboard.repeat_delay, 400);
}

/// Test save_dirty with empty dirty set (should be a no-op)
#[test]
fn test_save_dirty_empty() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    let settings = Settings::default();
    save_settings(&paths, &settings).expect("Initial save failed");

    // Empty dirty set
    let dirty = HashSet::new();
    let files_written = save_dirty(&paths, &settings, &dirty).expect("save_dirty failed");
    assert_eq!(files_written, 0, "Should write 0 files for empty dirty set");
}

/// Test that DirtyTracker integrates correctly with save_dirty
#[test]
fn test_dirty_tracker_save_integration() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    let mut settings = Settings::default();
    save_settings(&paths, &settings).expect("Initial save failed");

    // Simulate callback behavior: modify, mark dirty
    let tracker = Arc::new(DirtyTracker::new());

    settings.mouse.accel_speed = 0.5;
    tracker.mark(SettingsCategory::Mouse);

    settings.touchpad.tap = false;
    tracker.mark(SettingsCategory::Touchpad);

    // Take dirty set (as SaveManager does)
    let dirty = tracker.take();
    assert_eq!(dirty.len(), 2);

    // Save dirty categories
    let files_written = save_dirty(&paths, &settings, &dirty).expect("save_dirty failed");
    assert_eq!(files_written, 2);

    // Tracker should be empty now
    assert!(!tracker.is_dirty());

    // Verify changes persisted
    let loaded = load_settings(&paths);
    assert!((loaded.mouse.accel_speed - 0.5).abs() < 0.01);
    assert!(!loaded.touchpad.tap);
}

/// Test settings validation clamps out-of-range values
#[test]
fn test_settings_validate_clamps_values() {
    let mut settings = Settings::default();

    // Set invalid values (simulating corrupted config or manual edit)
    settings.appearance.gaps = -100.0;
    settings.appearance.focus_ring_width = 0.0;
    settings.keyboard.repeat_delay = 10;
    settings.keyboard.repeat_rate = 500;
    settings.mouse.accel_speed = 5.0;
    settings.animations.slowdown = 100.0;
    settings.cursor.size = 500;

    // Validate should clamp all values
    settings.validate();

    // Check all values are clamped to valid ranges
    assert_eq!(settings.appearance.gaps, GAP_SIZE_MIN);
    assert_eq!(settings.appearance.focus_ring_width, FOCUS_RING_WIDTH_MIN);
    assert_eq!(settings.keyboard.repeat_delay, REPEAT_DELAY_MIN);
    assert_eq!(settings.keyboard.repeat_rate, REPEAT_RATE_MAX);
    assert_eq!(settings.mouse.accel_speed, ACCEL_SPEED_MAX);
    assert_eq!(settings.animations.slowdown, ANIMATION_SLOWDOWN_MAX);
    assert_eq!(settings.cursor.size, CURSOR_SIZE_MAX);
}

/// Test that boolean callbacks don't need clamping (sanity check)
#[test]
fn test_boolean_values_no_clamping() {
    let mut settings = Settings::default();

    // Booleans just toggle
    settings.appearance.focus_ring_enabled = true;
    assert!(settings.appearance.focus_ring_enabled);

    settings.appearance.focus_ring_enabled = false;
    assert!(!settings.appearance.focus_ring_enabled);

    // Validate shouldn't change booleans
    settings.validate();
    assert!(!settings.appearance.focus_ring_enabled);
}

/// Test callback pattern: modify -> mark dirty -> save cycle
#[test]
fn test_full_callback_save_cycle() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Initial state
    let mut settings = Settings::default();
    save_settings(&paths, &settings).expect("Initial save failed");

    let tracker = Arc::new(DirtyTracker::new());

    // Simulate rapid-fire callbacks (like dragging a slider)
    for i in 1..=10 {
        settings.appearance.gaps = i as f32;
        tracker.mark(SettingsCategory::Appearance);
    }

    // Despite 10 marks, only 1 category is dirty (deduplication)
    assert_eq!(tracker.dirty_count(), 1);

    // Final value should be 10
    assert_eq!(settings.appearance.gaps, 10.0);

    // Save
    let dirty = tracker.take();
    save_dirty(&paths, &settings, &dirty).expect("save_dirty failed");

    // Verify final value persisted
    let loaded = load_settings(&paths);
    assert_eq!(loaded.appearance.gaps, 10.0);
}
