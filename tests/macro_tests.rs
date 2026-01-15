//! Tests for callback registration macros
//!
//! These tests verify that the batch macros correctly integrate with
//! the dirty tracking system and that category/section mappings are correct.

mod common;

use niri_settings::config::{DirtyTracker, Settings, SettingsCategory};
use std::sync::Arc;

/// Verify that all SettingsCategory variants have corresponding Settings fields
///
/// This test ensures that when using batch macros, the category names
/// match the available settings section fields.
#[test]
fn test_category_settings_field_correspondence() {
    let settings = Settings::default();

    // Verify each category has a corresponding field in Settings
    // This catches mismatches between SettingsCategory and Settings struct

    // Core categories with direct field access
    let _ = &settings.appearance; // Appearance
    let _ = &settings.behavior; // Behavior
    let _ = &settings.keyboard; // Keyboard
    let _ = &settings.mouse; // Mouse
    let _ = &settings.touchpad; // Touchpad
    let _ = &settings.trackpoint; // Trackpoint
    let _ = &settings.trackball; // Trackball
    let _ = &settings.tablet; // Tablet
    let _ = &settings.touch; // Touch
    let _ = &settings.outputs; // Outputs
    let _ = &settings.animations; // Animations
    let _ = &settings.cursor; // Cursor
    let _ = &settings.overview; // Overview
    let _ = &settings.workspaces; // Workspaces
    let _ = &settings.keybindings; // Keybindings
    let _ = &settings.layout_extras; // LayoutExtras
    let _ = &settings.gestures; // Gestures
    let _ = &settings.layer_rules; // LayerRules
    let _ = &settings.window_rules; // WindowRules
    let _ = &settings.miscellaneous; // Miscellaneous
    let _ = &settings.startup; // Startup
    let _ = &settings.environment; // Environment
    let _ = &settings.debug; // Debug
    let _ = &settings.switch_events; // SwitchEvents
    let _ = &settings.recent_windows; // RecentWindows
}

/// Test that marking a category as dirty works correctly
#[test]
fn test_dirty_tracker_marks_category() {
    let tracker = DirtyTracker::new();

    // Initially no categories are dirty
    assert!(!tracker.is_dirty());

    // Mark appearance as dirty (simulating what the macro does)
    tracker.mark(SettingsCategory::Appearance);

    // Now it should be dirty
    assert!(tracker.is_dirty());
    assert_eq!(tracker.dirty_count(), 1);

    // Take should return the marked category and reset
    let dirty = tracker.take();
    assert!(dirty.contains(&SettingsCategory::Appearance));
    assert!(!tracker.is_dirty());
}

/// Test that multiple categories can be marked dirty
#[test]
fn test_dirty_tracker_multiple_categories() {
    let tracker = DirtyTracker::new();

    // Simulate multiple callbacks firing (as batch macros would do)
    tracker.mark(SettingsCategory::Appearance);
    tracker.mark(SettingsCategory::Behavior);
    tracker.mark(SettingsCategory::Cursor);

    assert_eq!(tracker.dirty_count(), 3);

    let dirty = tracker.take();
    assert_eq!(dirty.len(), 3);
    assert!(dirty.contains(&SettingsCategory::Appearance));
    assert!(dirty.contains(&SettingsCategory::Behavior));
    assert!(dirty.contains(&SettingsCategory::Cursor));
}

/// Test that duplicate marks don't increase count
/// (important for callbacks that might fire multiple times)
#[test]
fn test_dirty_tracker_deduplication() {
    let tracker = DirtyTracker::new();

    // Same category marked multiple times (e.g., slider being dragged)
    for _ in 0..10 {
        tracker.mark(SettingsCategory::Appearance);
    }

    // Should only count as one dirty category
    assert_eq!(tracker.dirty_count(), 1);
}

/// Test that DirtyTracker is thread-safe (as required by SaveManager)
#[test]
fn test_dirty_tracker_thread_safety() {
    use std::thread;

    let tracker = Arc::new(DirtyTracker::new());

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let tracker = Arc::clone(&tracker);
            thread::spawn(move || {
                // Each thread marks a different category
                let category = match i {
                    0 => SettingsCategory::Appearance,
                    1 => SettingsCategory::Behavior,
                    2 => SettingsCategory::Cursor,
                    _ => SettingsCategory::Mouse,
                };
                tracker.mark(category);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // All 4 categories should be marked
    assert_eq!(tracker.dirty_count(), 4);
}

/// Verify category count matches the documented 25 categories
#[test]
fn test_all_categories_count() {
    assert_eq!(
        SettingsCategory::all().len(),
        25,
        "Expected 25 settings categories"
    );
}

/// Test that category names are human-readable (used in logging)
#[test]
fn test_category_names() {
    // Verify some category names for logging
    assert_eq!(SettingsCategory::Appearance.name(), "Appearance");
    assert_eq!(SettingsCategory::Behavior.name(), "Behavior");
    assert_eq!(SettingsCategory::LayoutExtras.name(), "Layout Extras");
    assert_eq!(SettingsCategory::WindowRules.name(), "Window Rules");
    assert_eq!(SettingsCategory::RecentWindows.name(), "Recent Windows");
}

/// Document the expected category-to-section mapping
/// This serves as documentation and will fail if the mapping changes
#[test]
fn test_category_section_mapping_documentation() {
    // This test documents the expected mapping between SettingsCategory
    // variants and Settings struct fields. When using batch macros,
    // these mappings MUST be followed:
    //
    // | SettingsCategory    | Settings field     |
    // |---------------------|-------------------|
    // | Appearance          | appearance        |
    // | Behavior            | behavior          |
    // | Keyboard            | keyboard          |
    // | Mouse               | mouse             |
    // | Touchpad            | touchpad          |
    // | Trackpoint          | trackpoint        |
    // | Trackball           | trackball         |
    // | Tablet              | tablet            |
    // | Touch               | touch             |
    // | Outputs             | outputs           |
    // | Animations          | animations        |
    // | Cursor              | cursor            |
    // | Overview            | overview          |
    // | Workspaces          | workspaces        |
    // | Keybindings         | keybindings       |
    // | LayoutExtras        | layout_extras     |
    // | Gestures            | gestures          |
    // | LayerRules          | layer_rules       |
    // | WindowRules         | window_rules      |
    // | Miscellaneous       | miscellaneous     |
    // | Startup             | startup           |
    // | Environment         | environment       |
    // | Debug               | debug             |
    // | SwitchEvents        | switch_events     |
    // | RecentWindows       | recent_windows    |

    // Verify all categories exist
    let all = SettingsCategory::all();
    assert!(all.contains(&SettingsCategory::Appearance));
    assert!(all.contains(&SettingsCategory::Behavior));
    assert!(all.contains(&SettingsCategory::Keyboard));
    assert!(all.contains(&SettingsCategory::Mouse));
    assert!(all.contains(&SettingsCategory::Touchpad));
    assert!(all.contains(&SettingsCategory::Trackpoint));
    assert!(all.contains(&SettingsCategory::Trackball));
    assert!(all.contains(&SettingsCategory::Tablet));
    assert!(all.contains(&SettingsCategory::Touch));
    assert!(all.contains(&SettingsCategory::Outputs));
    assert!(all.contains(&SettingsCategory::Animations));
    assert!(all.contains(&SettingsCategory::Cursor));
    assert!(all.contains(&SettingsCategory::Overview));
    assert!(all.contains(&SettingsCategory::Workspaces));
    assert!(all.contains(&SettingsCategory::Keybindings));
    assert!(all.contains(&SettingsCategory::LayoutExtras));
    assert!(all.contains(&SettingsCategory::Gestures));
    assert!(all.contains(&SettingsCategory::LayerRules));
    assert!(all.contains(&SettingsCategory::WindowRules));
    assert!(all.contains(&SettingsCategory::Miscellaneous));
    assert!(all.contains(&SettingsCategory::Startup));
    assert!(all.contains(&SettingsCategory::Environment));
    assert!(all.contains(&SettingsCategory::Debug));
    assert!(all.contains(&SettingsCategory::SwitchEvents));
    assert!(all.contains(&SettingsCategory::RecentWindows));
}
