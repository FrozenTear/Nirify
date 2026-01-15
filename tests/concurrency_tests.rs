//! Concurrency and thread safety tests
//!
//! These tests verify that shared state (Settings, DirtyTracker) behaves
//! correctly under concurrent access.
//!
//! Run with: cargo test --test concurrency_tests

mod common;

use common::create_test_paths;
use niri_settings::config::{
    load_settings, save_dirty, save_settings, DirtyTracker, Settings, SettingsCategory,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::tempdir;

// ============================================================================
// DIRTY TRACKER CONCURRENCY TESTS
// ============================================================================

#[test]
fn test_dirty_tracker_concurrent_marks() {
    let tracker = Arc::new(DirtyTracker::new());

    // Spawn multiple threads that mark different categories
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let tracker = Arc::clone(&tracker);
            let category = match i % 5 {
                0 => SettingsCategory::Appearance,
                1 => SettingsCategory::Mouse,
                2 => SettingsCategory::Keyboard,
                3 => SettingsCategory::Touchpad,
                _ => SettingsCategory::Cursor,
            };

            thread::spawn(move || {
                for _ in 0..100 {
                    tracker.mark(category);
                    thread::yield_now();
                }
            })
        })
        .collect();

    // Wait for all threads
    for h in handles {
        h.join().expect("Thread panicked");
    }

    // Should have exactly 5 categories marked (deduplication)
    let dirty = tracker.take();
    assert_eq!(dirty.len(), 5);
    assert!(dirty.contains(&SettingsCategory::Appearance));
    assert!(dirty.contains(&SettingsCategory::Mouse));
    assert!(dirty.contains(&SettingsCategory::Keyboard));
    assert!(dirty.contains(&SettingsCategory::Touchpad));
    assert!(dirty.contains(&SettingsCategory::Cursor));
}

#[test]
fn test_dirty_tracker_concurrent_mark_and_take() {
    let tracker = Arc::new(DirtyTracker::new());

    // Spawn writers that continuously mark categories
    let writer_handles: Vec<_> = (0..5)
        .map(|i| {
            let tracker = Arc::clone(&tracker);
            let category = match i % 3 {
                0 => SettingsCategory::Appearance,
                1 => SettingsCategory::Mouse,
                _ => SettingsCategory::Keyboard,
            };

            thread::spawn(move || {
                for _ in 0..50 {
                    tracker.mark(category);
                    thread::sleep(std::time::Duration::from_micros(100));
                }
            })
        })
        .collect();

    // Spawn readers that take the dirty set
    let reader_handles: Vec<_> = (0..3)
        .map(|_| {
            let tracker = Arc::clone(&tracker);

            thread::spawn(move || {
                let mut total_taken = 0;
                for _ in 0..10 {
                    let taken = tracker.take();
                    total_taken += taken.len();
                    thread::sleep(std::time::Duration::from_millis(1));
                }
                total_taken
            })
        })
        .collect();

    // Wait for writers
    for h in writer_handles {
        h.join().expect("Writer thread panicked");
    }

    // Wait for readers and sum results
    let totals: Vec<_> = reader_handles
        .into_iter()
        .map(|h| h.join().expect("Reader thread panicked"))
        .collect();

    // At least some takes should have found dirty entries
    let total: usize = totals.iter().sum();
    assert!(total > 0, "Expected some dirty entries to be taken");
}

#[test]
fn test_dirty_tracker_take_clears_state() {
    let tracker = Arc::new(DirtyTracker::new());

    // Mark some categories
    tracker.mark(SettingsCategory::Appearance);
    tracker.mark(SettingsCategory::Mouse);
    tracker.mark(SettingsCategory::Keyboard);

    assert!(tracker.is_dirty());
    assert_eq!(tracker.dirty_count(), 3);

    // Take should return all and clear
    let taken = tracker.take();
    assert_eq!(taken.len(), 3);

    // Should now be empty
    assert!(!tracker.is_dirty());
    assert_eq!(tracker.dirty_count(), 0);

    // Second take should be empty
    let taken2 = tracker.take();
    assert!(taken2.is_empty());
}

// ============================================================================
// SETTINGS MUTEX CONCURRENCY TESTS
// ============================================================================

#[test]
fn test_concurrent_settings_modifications() {
    let settings = Arc::new(Mutex::new(Settings::default()));
    let tracker = Arc::new(DirtyTracker::new());

    // Spawn threads that modify different parts of settings
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let settings = Arc::clone(&settings);
            let tracker = Arc::clone(&tracker);

            thread::spawn(move || {
                for j in 0..50 {
                    {
                        let mut s = settings.lock().unwrap();
                        match i % 3 {
                            0 => {
                                s.appearance.gaps_inner = (i * 50 + j) as f32;
                                tracker.mark(SettingsCategory::Appearance);
                            }
                            1 => {
                                s.keyboard.repeat_delay = 100 + (i * 10 + j);
                                tracker.mark(SettingsCategory::Keyboard);
                            }
                            _ => {
                                s.mouse.accel_speed = (j as f64) / 100.0;
                                tracker.mark(SettingsCategory::Mouse);
                            }
                        }
                    }
                    thread::yield_now();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("Thread panicked");
    }

    // Tracker should have exactly 3 categories (deduplication)
    let dirty = tracker.take();
    assert_eq!(dirty.len(), 3);
}

#[test]
fn test_mutex_poisoning_recovery() {
    let settings = Arc::new(Mutex::new(Settings::default()));

    // Spawn a thread that panics while holding the lock
    let settings_clone = Arc::clone(&settings);
    let handle = thread::spawn(move || {
        let mut s = settings_clone.lock().unwrap();
        s.appearance.gaps_inner = 42.0;
        s.keyboard.repeat_delay = 999;
        panic!("Intentional panic to test poisoning recovery");
    });

    // Wait for panic
    let result = handle.join();
    assert!(result.is_err(), "Thread should have panicked");

    // Lock should be poisoned, but we can recover
    let recovered = match settings.lock() {
        Ok(s) => s.clone(),
        Err(poisoned) => {
            // This is the recovery path
            poisoned.into_inner().clone()
        }
    };

    // Should have the values that were set before the panic
    assert_eq!(recovered.appearance.gaps_inner, 42.0);
    assert_eq!(recovered.keyboard.repeat_delay, 999);
}

#[test]
fn test_concurrent_save_same_file() {
    let dir = tempdir().unwrap();
    let paths = create_test_paths(dir.path());

    // Initial save
    let settings = Settings::default();
    save_settings(&paths, &settings).expect("Initial save failed");

    let paths = Arc::new(paths);

    // Multiple threads trying to save simultaneously
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let paths = Arc::clone(&paths);

            thread::spawn(move || {
                let mut settings = Settings::default();
                settings.appearance.gaps_inner = (i * 10) as f32;

                let mut dirty = HashSet::new();
                dirty.insert(SettingsCategory::Appearance);

                // This should not corrupt files due to atomic writes
                save_dirty(&paths, &settings, &dirty).expect("Save failed");
            })
        })
        .collect();

    for h in handles {
        h.join().expect("Thread panicked");
    }

    // File should be valid and readable
    let loaded = load_settings(&paths);
    // Value should be one of the values written (whichever finished last)
    assert!(
        loaded.appearance.gaps_inner >= 0.0 && loaded.appearance.gaps_inner <= 40.0,
        "Unexpected gaps value: {}",
        loaded.appearance.gaps_inner
    );

    // The file should be valid KDL
    let content = std::fs::read_to_string(&paths.appearance_kdl).expect("Failed to read file");
    let parsed: Result<kdl::KdlDocument, _> = content.parse();
    assert!(
        parsed.is_ok(),
        "Corrupted KDL after concurrent writes: {}",
        content
    );
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[test]
fn test_rapid_dirty_mark_and_take() {
    let tracker = Arc::new(DirtyTracker::new());
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Writer thread
    let tracker_w = Arc::clone(&tracker);
    let stop_w = Arc::clone(&stop);
    let writer = thread::spawn(move || {
        let categories = [
            SettingsCategory::Appearance,
            SettingsCategory::Behavior,
            SettingsCategory::Keyboard,
            SettingsCategory::Mouse,
            SettingsCategory::Touchpad,
        ];
        let mut i = 0;
        while !stop_w.load(std::sync::atomic::Ordering::Relaxed) {
            tracker_w.mark(categories[i % categories.len()]);
            i += 1;
            if i % 100 == 0 {
                thread::yield_now();
            }
        }
        i
    });

    // Reader thread
    let tracker_r = Arc::clone(&tracker);
    let stop_r = Arc::clone(&stop);
    let reader = thread::spawn(move || {
        let mut total = 0;
        while !stop_r.load(std::sync::atomic::Ordering::Relaxed) {
            let taken = tracker_r.take();
            total += taken.len();
            thread::sleep(std::time::Duration::from_micros(500));
        }
        total
    });

    // Let it run for a short time
    thread::sleep(std::time::Duration::from_millis(100));
    stop.store(true, std::sync::atomic::Ordering::Relaxed);

    let writes = writer.join().expect("Writer panicked");
    let reads = reader.join().expect("Reader panicked");

    // Should have done significant work
    assert!(writes > 100, "Expected more writes, got {}", writes);
    assert!(reads > 0, "Expected some reads, got {}", reads);
}

#[test]
fn test_settings_lock_contention() {
    let settings = Arc::new(Mutex::new(Settings::default()));
    let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let handles: Vec<_> = (0..8)
        .map(|_| {
            let settings = Arc::clone(&settings);
            let counter = Arc::clone(&counter);

            thread::spawn(move || {
                for _ in 0..100 {
                    {
                        let mut s = settings.lock().unwrap();
                        s.appearance.gaps_inner += 0.1;
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                    // Small yield to increase contention
                    thread::yield_now();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("Thread panicked");
    }

    // All increments should have succeeded
    let total = counter.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(total, 800, "Expected 800 increments, got {}", total);

    // Final value should reflect all increments
    let final_value = settings.lock().unwrap().appearance.gaps_inner;
    let expected = 16.0 + (800.0 * 0.1); // default + increments
    assert!(
        (final_value - expected).abs() < 0.01,
        "Expected ~{}, got {}",
        expected,
        final_value
    );
}
