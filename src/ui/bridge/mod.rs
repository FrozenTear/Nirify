//! Bridge between Slint UI and Rust logic
//!
//! This module handles callbacks and data synchronization between the UI layer
//! and the backend settings management.
//!
//! # Threading Model
//!
//! This application uses a carefully designed threading model for safety and performance:
//!
//! ## `Arc<Mutex<Settings>>` for settings storage
//!
//! Settings use `Arc<Mutex<>>` (not `Rc<RefCell<>>`) because they're accessed from
//! multiple contexts:
//!
//! 1. **Async reload operations**: `SaveManager` uses async file I/O via `async_ops`
//!    which runs on background threads. The settings mutex must be thread-safe.
//!
//! 2. **Exit handling**: Settings are accessed during application shutdown after
//!    the Slint event loop has stopped, potentially from a different thread context.
//!
//! 3. **IPC operations**: Niri config reload (`ipc::async_ops::reload_config_async`)
//!    runs asynchronously and may trigger callbacks that access settings.
//!
//! ## `Rc<SaveManager>` for save coordination
//!
//! The SaveManager uses `Rc` (not `Arc`) because:
//!
//! - Slint's UI thread is single-threaded; all callbacks run on the main thread
//! - SaveManager is only accessed from UI callbacks, never from background threads
//! - Using `Rc` avoids unnecessary atomic overhead for single-threaded access
//!
//! ## `Arc<DirtyTracker>` for change tracking
//!
//! DirtyTracker uses `Arc` because it's shared across callbacks and may be
//! accessed during async save operations to determine which categories need saving.
//!
//! # Module Structure
//!
//! - `callbacks/` - UI event handlers organized by category
//! - `converters` - Type conversion utilities (Slint <-> Rust types)
//! - `indices` - Enum-to-index mappings for comboboxes
//! - `macros` - Helper macros to reduce callback boilerplate
//! - `save_manager` - Debounced save management
//! - `sync` - UI state synchronization from settings

mod callbacks;
mod converters;
mod indices;
pub mod key_mapping;
mod macros;
mod save_manager;
mod sync;
mod sync_macros;

use crate::config::{load_settings, save_settings, ConfigPaths, DirtyTracker, Settings};
use crate::diff::{generate_diff, CategoryDiff, ConfigDiff};
use crate::ipc;
use crate::{CategoryDiffItem, DiffLineItem, MainWindow};
use log::{debug, error, info, warn};
use save_manager::SaveManager;
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// Re-export for external use
pub use converters::{color_to_slint_color, slint_color_to_color};
pub use sync::sync_ui_from_settings;

/// Set up all UI callbacks
///
/// This function registers all callback handlers for UI events.
/// It should be called once during application initialization.
/// It also creates the SaveManager for debounced auto-saving.
///
/// # Arguments
///
/// * `ui` - The main window reference
/// * `settings` - Thread-safe settings storage
/// * `paths` - Configuration paths
pub fn setup_callbacks(ui: &MainWindow, settings: Arc<Mutex<Settings>>, paths: Arc<ConfigPaths>) {
    // First sync UI from settings
    match settings.lock() {
        Ok(s) => sync_ui_from_settings(ui, &s),
        Err(e) => error!("Failed to lock settings for initial sync: {}", e),
    }

    // Create dirty tracker and save manager (passed to callbacks, no global state)
    let dirty_tracker = Arc::new(DirtyTracker::new());
    let ui_weak = ui.as_weak();
    let save_manager = SaveManager::new(
        settings.clone(),
        paths.clone(),
        ui_weak,
        Arc::clone(&dirty_tracker),
    );
    info!("Auto-save enabled with 300ms debounce and dirty tracking");

    // Dynamic page callbacks (migrated from static)
    callbacks::animations::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::appearance::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::behavior::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::cursor::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::debug::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::gestures::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::keyboard::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::layout_extras::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::miscellaneous::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::mouse::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::overview::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::switch_events::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::touchpad::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::window_rules::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::layer_rules::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::workspaces::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::trackpoint::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::trackball::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::tablet::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::touch::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::startup::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::environment::setup(ui, settings.clone(), Rc::clone(&save_manager));
    callbacks::recent_windows::setup(ui, settings.clone(), Rc::clone(&save_manager));

    // Static callbacks (not yet migrated to dynamic)
    callbacks::outputs::setup(ui, settings.clone(), Rc::clone(&save_manager));

    callbacks::keybindings::setup(ui, settings.clone(), paths.clone(), save_manager);

    // Backups only needs paths (no settings modification, just file operations)
    callbacks::backups::setup(ui, paths.clone());

    // Diff view callbacks (needs dirty tracker, settings, and paths)
    setup_diff_view_callbacks(ui, settings, paths, Arc::clone(&dirty_tracker));
}

/// Set up diff view callbacks for the Review Changes dialog
fn setup_diff_view_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    paths: Arc<ConfigPaths>,
    dirty_tracker: Arc<DirtyTracker>,
) {
    // Review Changes button clicked - generate and show diff
    let ui_weak = ui.as_weak();
    let settings_for_review = settings.clone();
    let paths_for_review = paths.clone();
    let tracker_for_review = Arc::clone(&dirty_tracker);
    ui.on_review_changes_requested(move || {
        if let Some(ui) = ui_weak.upgrade() {
            // Get dirty categories
            let dirty = tracker_for_review.peek();

            if dirty.is_empty() {
                // No pending changes
                ui.set_diff_categories(ModelRc::default());
                ui.set_diff_total_additions(0);
                ui.set_diff_total_deletions(0);
                ui.set_diff_view_visible(true);
                debug!("Review Changes: no pending changes");
                return;
            }

            // Clone settings for diff generation
            let settings_copy = match settings_for_review.lock() {
                Ok(s) => s.clone(),
                Err(poisoned) => {
                    warn!("Settings mutex poisoned during diff generation");
                    poisoned.into_inner().clone()
                }
            };

            // Generate diff
            let config_diff = generate_diff(&settings_copy, &paths_for_review, &dirty);
            debug!(
                "Generated diff: {} categories, +{} -{}",
                config_diff.categories.len(),
                config_diff.total_additions,
                config_diff.total_deletions
            );

            // Convert to Slint types
            let slint_categories = config_diff_to_slint(&config_diff);

            // Update UI
            ui.set_diff_categories(slint_categories);
            ui.set_diff_total_additions(config_diff.total_additions);
            ui.set_diff_total_deletions(config_diff.total_deletions);
            ui.set_diff_view_visible(true);
        }
    });

    // Save Now button - force immediate save
    let ui_weak = ui.as_weak();
    let settings_for_save = settings.clone();
    let paths_for_save = paths.clone();
    let tracker_for_save = Arc::clone(&dirty_tracker);
    ui.on_diff_save_now(move || {
        if let Some(ui) = ui_weak.upgrade() {
            // Take dirty categories (marks them as clean)
            let dirty = tracker_for_save.take();

            if dirty.is_empty() {
                ui.set_diff_view_visible(false);
                return;
            }

            // Clone settings and save
            let settings_copy = match settings_for_save.lock() {
                Ok(s) => s.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            };

            match save_settings(&paths_for_save, &settings_copy) {
                Ok(()) => {
                    info!("Settings saved via Review Changes dialog");
                    // Try to reload niri config (async to prevent UI freeze)
                    if ipc::is_niri_running() {
                        ipc::async_ops::reload_config_async(|result| {
                            if let Err(e) = result {
                                debug!("Could not reload niri config: {}", e);
                            }
                        });
                    }
                }
                Err(e) => {
                    error!("Failed to save settings: {}", e);
                }
            }

            ui.set_diff_view_visible(false);
        }
    });

    // Discard All button - reload settings from disk
    let ui_weak = ui.as_weak();
    let settings_for_discard = settings;
    let paths_for_discard = paths;
    let tracker_for_discard = dirty_tracker;
    ui.on_diff_discard_all(move || {
        if let Some(ui) = ui_weak.upgrade() {
            // Clear dirty tracker
            tracker_for_discard.take();

            // Reload settings from disk
            let fresh_settings = load_settings(&paths_for_discard);

            // Update the settings
            match settings_for_discard.lock() {
                Ok(mut s) => {
                    *s = fresh_settings.clone();
                    // Re-sync UI from settings
                    sync::sync_ui_from_settings(&ui, &fresh_settings);
                    info!("Discarded all changes, reloaded from disk");
                }
                Err(poisoned) => {
                    let mut s = poisoned.into_inner();
                    *s = fresh_settings.clone();
                    sync::sync_ui_from_settings(&ui, &fresh_settings);
                    warn!("Discarded all changes (mutex was poisoned)");
                }
            }

            ui.set_diff_view_visible(false);
        }
    });

    // Close button
    let ui_weak = ui.as_weak();
    ui.on_diff_close(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_diff_view_visible(false);
        }
    });
}

/// Convert ConfigDiff to Slint model
fn config_diff_to_slint(diff: &ConfigDiff) -> ModelRc<CategoryDiffItem> {
    let items: Vec<CategoryDiffItem> = diff.categories.iter().map(category_diff_to_slint).collect();
    ModelRc::new(VecModel::from(items))
}

/// Convert CategoryDiff to Slint CategoryDiffItem
fn category_diff_to_slint(cat: &CategoryDiff) -> CategoryDiffItem {
    let lines: Vec<DiffLineItem> = cat
        .lines
        .iter()
        .map(|line| DiffLineItem {
            line_type: line.line_type.to_int(),
            old_text: line.old_text.clone().into(),
            new_text: line.new_text.clone().into(),
            line_num: line.line_num,
        })
        .collect();

    CategoryDiffItem {
        name: cat.name.clone().into(),
        file_path: cat.file_path.display().to_string().into(),
        has_changes: cat.has_changes,
        additions: cat.additions,
        deletions: cat.deletions,
        lines: ModelRc::new(VecModel::from(lines)),
    }
}
