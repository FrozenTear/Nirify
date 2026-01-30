//! Property-based tests using proptest
//!
//! These tests use randomized inputs to find edge cases that manual tests miss.
//! Run with: cargo test --test property_tests

mod common;

use common::create_test_paths;
use nirify::config::models::{AppearanceSettings, BehaviorSettings, Settings};
use nirify::config::storage::{
    generate_appearance_kdl, generate_behavior_kdl, generate_cursor_kdl, generate_keyboard_kdl,
    generate_mouse_kdl,
};
use nirify::config::{load_settings, save_settings};
use nirify::version::FeatureCompat;
use nirify::constants::*;
use nirify::types::Color;
use proptest::prelude::*;
use tempfile::tempdir;

// ============================================================================
// COLOR TESTS
// ============================================================================

proptest! {
    /// Any valid RGBA color should roundtrip through hex format
    #[test]
    fn color_hex_roundtrip(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255, a in 0u8..=255) {
        let color = Color { r, g, b, a };
        let hex = color.to_hex();
        let parsed = Color::from_hex(&hex).expect("Should parse generated hex");
        prop_assert_eq!(color, parsed, "Color roundtrip failed for {:?} -> {} -> {:?}", color, hex, parsed);
    }

    /// Valid 6-digit hex strings should parse correctly
    #[test]
    fn color_from_valid_6digit_hex(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        let parsed = Color::from_hex(&hex).expect("Should parse valid 6-digit hex");
        prop_assert_eq!(parsed.r, r);
        prop_assert_eq!(parsed.g, g);
        prop_assert_eq!(parsed.b, b);
        prop_assert_eq!(parsed.a, 255);
    }

    /// Valid 8-digit hex strings should parse correctly
    #[test]
    fn color_from_valid_8digit_hex(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255, a in 0u8..=255) {
        let hex = format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a);
        let parsed = Color::from_hex(&hex).expect("Should parse valid 8-digit hex");
        prop_assert_eq!(parsed.r, r);
        prop_assert_eq!(parsed.g, g);
        prop_assert_eq!(parsed.b, b);
        prop_assert_eq!(parsed.a, a);
    }

    /// 3-digit shorthand should expand correctly (each digit doubled)
    #[test]
    fn color_from_3digit_shorthand(r in 0u8..=15, g in 0u8..=15, b in 0u8..=15) {
        let hex = format!("#{:x}{:x}{:x}", r, g, b);
        let parsed = Color::from_hex(&hex).expect("Should parse 3-digit shorthand");
        // Each digit should be doubled: 0xA -> 0xAA
        prop_assert_eq!(parsed.r, r * 17);
        prop_assert_eq!(parsed.g, g * 17);
        prop_assert_eq!(parsed.b, b * 17);
        prop_assert_eq!(parsed.a, 255);
    }
}

// ============================================================================
// SETTINGS VALIDATION TESTS
// ============================================================================

proptest! {
    /// Settings::validate() should always produce values within valid ranges
    #[test]
    fn validate_clamps_appearance_values(
        gaps in -1000.0f32..1000.0,
        focus_ring_width in -100.0f32..100.0,
        border_thickness in -100.0f32..100.0,
        corner_radius in -100.0f32..100.0,
    ) {
        let mut settings = Settings::default();
        settings.appearance.gaps = gaps;
        settings.appearance.focus_ring_width = focus_ring_width;
        settings.appearance.border_thickness = border_thickness;
        settings.appearance.corner_radius = corner_radius;

        settings.validate();

        prop_assert!(settings.appearance.gaps >= GAP_SIZE_MIN);
        prop_assert!(settings.appearance.gaps <= GAP_SIZE_MAX);
        prop_assert!(settings.appearance.focus_ring_width >= FOCUS_RING_WIDTH_MIN);
        prop_assert!(settings.appearance.focus_ring_width <= FOCUS_RING_WIDTH_MAX);
        prop_assert!(settings.appearance.border_thickness >= BORDER_THICKNESS_MIN);
        prop_assert!(settings.appearance.border_thickness <= BORDER_THICKNESS_MAX);
        prop_assert!(settings.appearance.corner_radius >= CORNER_RADIUS_MIN);
        prop_assert!(settings.appearance.corner_radius <= CORNER_RADIUS_MAX);
    }

    /// Settings::validate() should clamp keyboard values
    #[test]
    fn validate_clamps_keyboard_values(
        repeat_delay in -1000i32..10000,
        repeat_rate in -100i32..500,
    ) {
        let mut settings = Settings::default();
        settings.keyboard.repeat_delay = repeat_delay;
        settings.keyboard.repeat_rate = repeat_rate;

        settings.validate();

        prop_assert!(settings.keyboard.repeat_delay >= REPEAT_DELAY_MIN);
        prop_assert!(settings.keyboard.repeat_delay <= REPEAT_DELAY_MAX);
        prop_assert!(settings.keyboard.repeat_rate >= REPEAT_RATE_MIN);
        prop_assert!(settings.keyboard.repeat_rate <= REPEAT_RATE_MAX);
    }

    /// Settings::validate() should clamp mouse/touchpad acceleration
    #[test]
    fn validate_clamps_accel_values(
        mouse_accel in -10.0f64..10.0,
        touchpad_accel in -10.0f64..10.0,
        mouse_scroll in -10.0f64..100.0,
        touchpad_scroll in -10.0f64..100.0,
    ) {
        let mut settings = Settings::default();
        settings.mouse.accel_speed = mouse_accel;
        settings.touchpad.accel_speed = touchpad_accel;
        settings.mouse.scroll_factor = mouse_scroll;
        settings.touchpad.scroll_factor = touchpad_scroll;

        settings.validate();

        prop_assert!(settings.mouse.accel_speed >= ACCEL_SPEED_MIN);
        prop_assert!(settings.mouse.accel_speed <= ACCEL_SPEED_MAX);
        prop_assert!(settings.touchpad.accel_speed >= ACCEL_SPEED_MIN);
        prop_assert!(settings.touchpad.accel_speed <= ACCEL_SPEED_MAX);
        prop_assert!(settings.mouse.scroll_factor >= SCROLL_FACTOR_MIN);
        prop_assert!(settings.mouse.scroll_factor <= SCROLL_FACTOR_MAX);
        prop_assert!(settings.touchpad.scroll_factor >= SCROLL_FACTOR_MIN);
        prop_assert!(settings.touchpad.scroll_factor <= SCROLL_FACTOR_MAX);
    }

    /// Settings::validate() should clamp cursor and animation values
    #[test]
    fn validate_clamps_cursor_and_animation(
        cursor_size in -100i32..500,
        slowdown in -10.0f64..100.0,
        overview_zoom in -1.0f64..2.0,
    ) {
        let mut settings = Settings::default();
        settings.cursor.size = cursor_size;
        settings.animations.slowdown = slowdown;
        settings.overview.zoom = overview_zoom;

        settings.validate();

        prop_assert!(settings.cursor.size >= CURSOR_SIZE_MIN);
        prop_assert!(settings.cursor.size <= CURSOR_SIZE_MAX);
        prop_assert!(settings.animations.slowdown >= ANIMATION_SLOWDOWN_MIN);
        prop_assert!(settings.animations.slowdown <= ANIMATION_SLOWDOWN_MAX);
        prop_assert!(settings.overview.zoom >= OVERVIEW_ZOOM_MIN);
        prop_assert!(settings.overview.zoom <= OVERVIEW_ZOOM_MAX);
    }
}

// ============================================================================
// KDL GENERATION TESTS
// ============================================================================

proptest! {
    /// Generated appearance KDL should always be valid KDL
    #[test]
    fn appearance_kdl_is_valid(
        gaps in 0.0f32..64.0,
        focus_ring_width in 1.0f32..16.0,
        border_thickness in 1.0f32..8.0,
        corner_radius in 0.0f32..32.0,
        focus_ring_enabled in any::<bool>(),
        border_enabled in any::<bool>(),
    ) {
        let appearance = AppearanceSettings {
            gaps,
            focus_ring_width,
            border_thickness,
            corner_radius,
            focus_ring_enabled,
            border_enabled,
            ..Default::default()
        };
        let behavior = BehaviorSettings::default();

        let kdl_str = generate_appearance_kdl(&appearance, &behavior);

        // Should parse without error
        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok(), "Generated KDL failed to parse:\n{}\nError: {:?}", kdl_str, result.err());
    }

    /// Generated behavior KDL should always be valid KDL
    #[test]
    fn behavior_kdl_is_valid(
        focus_follows_mouse in any::<bool>(),
        strut_left in 0.0f32..500.0,
        strut_right in 0.0f32..500.0,
        strut_top in 0.0f32..500.0,
        strut_bottom in 0.0f32..500.0,
    ) {
        let behavior = BehaviorSettings {
            focus_follows_mouse,
            strut_left,
            strut_right,
            strut_top,
            strut_bottom,
            ..Default::default()
        };

        let kdl_str = generate_behavior_kdl(&behavior);

        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok(), "Generated behavior KDL failed to parse:\n{}\nError: {:?}", kdl_str, result.err());
    }

    /// Generated keyboard KDL should always be valid KDL
    #[test]
    fn keyboard_kdl_is_valid(
        repeat_delay in 100i32..2000,
        repeat_rate in 1i32..100,
    ) {
        use nirify::config::models::KeyboardSettings;

        let keyboard = KeyboardSettings {
            repeat_delay,
            repeat_rate,
            ..Default::default()
        };

        let kdl_str = generate_keyboard_kdl(&keyboard);

        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok(), "Generated keyboard KDL failed to parse:\n{}\nError: {:?}", kdl_str, result.err());
    }

    /// Generated mouse KDL should always be valid KDL
    #[test]
    fn mouse_kdl_is_valid(
        accel_speed in -1.0f64..1.0,
        scroll_factor in 0.1f64..10.0,
        natural_scroll in any::<bool>(),
    ) {
        use nirify::config::models::MouseSettings;

        let mouse = MouseSettings {
            accel_speed,
            scroll_factor,
            natural_scroll,
            ..Default::default()
        };

        let kdl_str = generate_mouse_kdl(&mouse);

        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok(), "Generated mouse KDL failed to parse:\n{}\nError: {:?}", kdl_str, result.err());
    }

    /// Generated cursor KDL should always be valid KDL
    #[test]
    fn cursor_kdl_is_valid(
        size in 16i32..64,
        hide_when_typing in any::<bool>(),
    ) {
        use nirify::config::models::CursorSettings;

        let cursor = CursorSettings {
            size,
            hide_when_typing,
            theme: "default".to_string(),
            ..Default::default()
        };

        let kdl_str = generate_cursor_kdl(&cursor);

        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok(), "Generated cursor KDL failed to parse:\n{}\nError: {:?}", kdl_str, result.err());
    }
}

// ============================================================================
// SAVE/LOAD ROUNDTRIP TESTS
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))] // Fewer cases for I/O tests

    /// Settings should roundtrip through save/load
    #[test]
    fn settings_save_load_roundtrip(
        gaps in 0.0f32..64.0,
        focus_ring_width in 1.0f32..16.0,
        repeat_delay in 100i32..2000,
        repeat_rate in 1i32..100,
        cursor_size in 16i32..64,
        animations_enabled in any::<bool>(),
    ) {
        let dir = tempdir().unwrap();
        let paths = create_test_paths(dir.path());

        let mut settings = Settings::default();
        settings.appearance.gaps = gaps;
        settings.appearance.focus_ring_width = focus_ring_width;
        settings.keyboard.repeat_delay = repeat_delay;
        settings.keyboard.repeat_rate = repeat_rate;
        settings.cursor.size = cursor_size;
        settings.animations.enabled = animations_enabled;

        // Save
        save_settings(&paths, &settings, FeatureCompat::all_enabled()).expect("Failed to save");

        // Load
        let loaded = load_settings(&paths);

        // Verify values match
        // Note: floats are converted to integers in KDL output, so tolerance is 1.0
        prop_assert!((loaded.appearance.gaps - gaps).abs() < 1.0, "gaps mismatch");
        prop_assert!((loaded.appearance.focus_ring_width - focus_ring_width).abs() < 1.0, "focus_ring_width mismatch");
        prop_assert_eq!(loaded.keyboard.repeat_delay, repeat_delay);
        prop_assert_eq!(loaded.keyboard.repeat_rate, repeat_rate);
        prop_assert_eq!(loaded.cursor.size, cursor_size);
        prop_assert_eq!(loaded.animations.enabled, animations_enabled);
    }
}

// ============================================================================
// STRING PROPERTY TESTS
// ============================================================================

proptest! {
    /// XKB layout strings should be handled correctly
    #[test]
    fn xkb_layout_roundtrip(layout in "[a-z]{2,5}") {
        use nirify::config::models::KeyboardSettings;

        let keyboard = KeyboardSettings {
            xkb_layout: layout.clone(),
            ..Default::default()
        };

        let kdl_str = generate_keyboard_kdl(&keyboard);

        // Should contain our layout
        let expected_layout = format!("layout \"{}\"", layout);
        prop_assert!(kdl_str.contains(&expected_layout), "Expected layout string not found");

        // Should be valid KDL
        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok());
    }

    /// Cursor theme names should be handled correctly
    #[test]
    fn cursor_theme_roundtrip(theme in "[A-Za-z][A-Za-z0-9_-]{0,30}") {
        use nirify::config::models::CursorSettings;

        let cursor = CursorSettings {
            theme: theme.clone(),
            ..Default::default()
        };

        let kdl_str = generate_cursor_kdl(&cursor);

        // Should contain our theme
        let expected_theme = format!("xcursor-theme \"{}\"", theme);
        prop_assert!(kdl_str.contains(&expected_theme), "Expected theme string not found");

        // Should be valid KDL
        let result: Result<kdl::KdlDocument, _> = kdl_str.parse();
        prop_assert!(result.is_ok());
    }
}
