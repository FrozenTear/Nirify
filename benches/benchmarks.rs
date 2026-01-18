//! Performance benchmarks for niri-settings
//!
//! Run with: cargo bench
//!
//! Results are output to target/criterion/report/index.html

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use niri_settings::config::models::*;
use niri_settings::config::storage::*;
use niri_settings::config::{
    load_settings, save_dirty, save_settings, DirtyTracker, Settings, SettingsCategory,
};
use std::collections::HashSet;
use tempfile::tempdir;

// ============================================================================
// SETTINGS VALIDATION BENCHMARKS
// ============================================================================

fn bench_settings_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings_validate");

    // Test with default settings (already valid)
    group.bench_function("defaults", |b| {
        b.iter(|| {
            let mut settings = black_box(Settings::default());
            settings.validate();
        })
    });

    // Test with out-of-range values (needs clamping)
    group.bench_function("needs_clamping", |b| {
        b.iter(|| {
            let mut settings = Settings::default();
            settings.appearance.gaps = -100.0; // Niri uses a single gaps value
            settings.appearance.focus_ring_width = 100.0;
            settings.keyboard.repeat_delay = 10000;
            settings.keyboard.repeat_rate = -50;
            settings.mouse.accel_speed = 5.0;
            settings.animations.slowdown = 100.0;
            settings.cursor.size = 500;
            black_box(&mut settings).validate();
        })
    });

    group.finish();
}

// ============================================================================
// KDL GENERATION BENCHMARKS
// ============================================================================

fn bench_kdl_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("kdl_generation");

    // Appearance KDL (most commonly changed)
    group.bench_function("appearance_default", |b| {
        let appearance = AppearanceSettings::default();
        let behavior = BehaviorSettings::default();
        b.iter(|| generate_appearance_kdl(black_box(&appearance), black_box(&behavior)))
    });

    // Keyboard KDL
    group.bench_function("keyboard", |b| {
        let keyboard = KeyboardSettings::default();
        b.iter(|| generate_keyboard_kdl(black_box(&keyboard)))
    });

    // Mouse KDL
    group.bench_function("mouse", |b| {
        let mouse = MouseSettings::default();
        b.iter(|| generate_mouse_kdl(black_box(&mouse)))
    });

    // Animations KDL
    group.bench_function("animations", |b| {
        let animations = AnimationSettings::default();
        b.iter(|| generate_animations_kdl(black_box(&animations)))
    });

    group.finish();
}

// ============================================================================
// DIRTY TRACKER BENCHMARKS
// ============================================================================

fn bench_dirty_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("dirty_tracker");

    // Single mark
    group.bench_function("mark_single", |b| {
        let tracker = DirtyTracker::new();
        b.iter(|| {
            tracker.mark(black_box(SettingsCategory::Appearance));
        })
    });

    // Multiple marks (simulating slider drag)
    group.bench_function("mark_100_same", |b| {
        let tracker = DirtyTracker::new();
        b.iter(|| {
            for _ in 0..100 {
                tracker.mark(black_box(SettingsCategory::Appearance));
            }
            tracker.take()
        })
    });

    // Mark different categories
    group.bench_function("mark_all_categories", |b| {
        let tracker = DirtyTracker::new();
        let categories = [
            SettingsCategory::Appearance,
            SettingsCategory::Behavior,
            SettingsCategory::Keyboard,
            SettingsCategory::Mouse,
            SettingsCategory::Touchpad,
            SettingsCategory::Animations,
            SettingsCategory::Cursor,
        ];
        b.iter(|| {
            for cat in &categories {
                tracker.mark(black_box(*cat));
            }
            tracker.take()
        })
    });

    // Take operation
    group.bench_function("take", |b| {
        b.iter_batched(
            || {
                let tracker = DirtyTracker::new();
                tracker.mark(SettingsCategory::Appearance);
                tracker.mark(SettingsCategory::Mouse);
                tracker.mark(SettingsCategory::Keyboard);
                tracker
            },
            |tracker| tracker.take(),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

// ============================================================================
// SAVE/LOAD BENCHMARKS
// ============================================================================

fn bench_save_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_load");

    // Configure for I/O-bound tests
    group.sample_size(50);

    // Save all settings
    group.bench_function("save_all", |b| {
        let dir = tempdir().unwrap();
        let paths = create_test_paths(dir.path());
        let settings = Settings::default();

        b.iter(|| save_settings(black_box(&paths), black_box(&settings)).unwrap())
    });

    // Load all settings
    group.bench_function("load_all", |b| {
        let dir = tempdir().unwrap();
        let paths = create_test_paths(dir.path());
        let settings = Settings::default();
        save_settings(&paths, &settings).unwrap();

        b.iter(|| load_settings(black_box(&paths)))
    });

    // Save single dirty category
    group.bench_function("save_dirty_single", |b| {
        let dir = tempdir().unwrap();
        let paths = create_test_paths(dir.path());
        let settings = Settings::default();
        save_settings(&paths, &settings).unwrap();

        let mut dirty = HashSet::new();
        dirty.insert(SettingsCategory::Appearance);

        b.iter(|| save_dirty(black_box(&paths), black_box(&settings), black_box(&dirty)).unwrap())
    });

    // Save multiple dirty categories
    group.bench_function("save_dirty_three", |b| {
        let dir = tempdir().unwrap();
        let paths = create_test_paths(dir.path());
        let settings = Settings::default();
        save_settings(&paths, &settings).unwrap();

        let mut dirty = HashSet::new();
        dirty.insert(SettingsCategory::Appearance);
        dirty.insert(SettingsCategory::Mouse);
        dirty.insert(SettingsCategory::Keyboard);

        b.iter(|| save_dirty(black_box(&paths), black_box(&settings), black_box(&dirty)).unwrap())
    });

    group.finish();
}

// ============================================================================
// CLONE BENCHMARKS
// ============================================================================

fn bench_settings_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("settings_clone");

    // Clone entire settings (what we want to avoid)
    group.bench_function("full_clone", |b| {
        let settings = Settings::default();
        b.iter(|| black_box(&settings).clone())
    });

    // Clone just appearance (what save_dirty does)
    group.bench_function("appearance_only", |b| {
        let settings = Settings::default();
        b.iter(|| black_box(&settings).appearance.clone())
    });

    // Clone with many window rules
    group.bench_function("with_50_window_rules", |b| {
        let mut settings = Settings::default();
        for i in 0..50 {
            settings.window_rules.rules.push(WindowRule {
                id: i,
                name: format!("Rule {}", i),
                ..Default::default()
            });
        }
        b.iter(|| black_box(&settings).clone())
    });

    group.finish();
}

// ============================================================================
// COLOR PARSING BENCHMARKS
// ============================================================================

fn bench_color_parsing(c: &mut Criterion) {
    use niri_settings::types::Color;

    let mut group = c.benchmark_group("color_parsing");

    // Parse 6-digit hex
    group.bench_function("parse_6digit", |b| {
        b.iter(|| Color::from_hex(black_box("#ff5500")))
    });

    // Parse 8-digit hex
    group.bench_function("parse_8digit", |b| {
        b.iter(|| Color::from_hex(black_box("#ff5500cc")))
    });

    // Parse 3-digit shorthand
    group.bench_function("parse_3digit", |b| {
        b.iter(|| Color::from_hex(black_box("#f50")))
    });

    // Format to hex
    group.bench_function("to_hex", |b| {
        let color = Color {
            r: 255,
            g: 85,
            b: 0,
            a: 204,
        };
        b.iter(|| black_box(&color).to_hex())
    });

    group.finish();
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_paths(base: &std::path::Path) -> niri_settings::config::ConfigPaths {
    use std::fs;

    let managed_dir = base.to_path_buf();
    let input_dir = managed_dir.join("input");
    let advanced_dir = managed_dir.join("advanced");

    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&advanced_dir).unwrap();

    niri_settings::config::ConfigPaths {
        niri_config: base.join("config.kdl"),
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
    }
}

// ============================================================================
// CRITERION SETUP
// ============================================================================

criterion_group!(
    benches,
    bench_settings_validate,
    bench_kdl_generation,
    bench_dirty_tracker,
    bench_save_load,
    bench_settings_clone,
    bench_color_parsing,
);

criterion_main!(benches);
