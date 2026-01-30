//! SaveManager - Handles debounced auto-save with iced subscriptions
//!
//! This module implements automatic saving of settings with a 300ms debounce.
//! Changes are batched to avoid excessive disk I/O during rapid slider adjustments.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::Task;
use log::{debug, error, info};

use crate::config::{ConfigPaths, DirtyTracker, Settings, SettingsCategory};
use crate::version::FeatureCompat;

/// RAII guard that automatically clears the save_in_progress flag on drop.
/// This prevents the flag from being stuck if the save task panics or returns early.
struct SaveGuard(Arc<Mutex<bool>>);

impl SaveGuard {
    /// Attempts to acquire the save guard. Returns None if a save is already in progress.
    fn new(flag: Arc<Mutex<bool>>) -> Option<Self> {
        {
            let mut lock = flag.lock().expect("save_in_progress mutex poisoned");
            if *lock {
                return None; // Already saving
            }
            *lock = true;
        } // Lock released here
        Some(SaveGuard(flag))
    }
}

impl Drop for SaveGuard {
    fn drop(&mut self) {
        // Clear the flag when guard is dropped (even on panic)
        if let Ok(mut lock) = self.0.lock() {
            *lock = false;
        }
    }
}

/// Manages debounced auto-save operations
#[derive(Clone)]
pub struct SaveManager {
    /// Shared settings state
    settings: Arc<Mutex<Settings>>,

    /// Config paths for saving
    paths: Arc<ConfigPaths>,

    /// Tracks which categories need saving
    dirty_tracker: Arc<DirtyTracker>,

    /// Last time a change was requested
    last_change: Arc<Mutex<Option<Instant>>>,

    /// Whether a save operation is currently in progress
    save_in_progress: Arc<Mutex<bool>>,

    /// Feature compatibility flags based on niri version
    feature_compat: FeatureCompat,
}

impl SaveManager {
    /// Creates a new SaveManager
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        paths: Arc<ConfigPaths>,
        dirty_tracker: Arc<DirtyTracker>,
        feature_compat: FeatureCompat,
    ) -> Self {
        Self {
            settings,
            paths,
            dirty_tracker,
            last_change: Arc::new(Mutex::new(None)),
            save_in_progress: Arc::new(Mutex::new(false)),
            feature_compat,
        }
    }

    /// Records that a change was made (resets the debounce timer)
    pub fn mark_changed(&self) {
        *self.last_change.lock().expect("last_change mutex poisoned") = Some(Instant::now());
    }

    /// Checks if enough time has elapsed to trigger a save
    pub fn should_save(&self) -> bool {
        // Don't save if already saving
        if *self.save_in_progress.lock().expect("save_in_progress mutex poisoned") {
            debug!("Save already in progress, skipping");
            return false;
        }

        // Don't save if nothing is dirty
        if !self.dirty_tracker.is_dirty() {
            return false;
        }

        // Check if 300ms has elapsed since last change
        if let Some(last) = *self.last_change.lock().expect("last_change mutex poisoned") {
            let elapsed = Instant::now().duration_since(last);
            if elapsed >= Duration::from_millis(300) {
                debug!("Debounce timeout reached ({:?}), triggering save", elapsed);
                return true;
            }
        }

        false
    }

    /// Creates an async Task that saves dirty settings to disk
    pub fn save_task(&self) -> Task<SaveResult> {
        let settings = self.settings.clone();
        let paths = self.paths.clone();
        let dirty_tracker = self.dirty_tracker.clone();
        let save_in_progress = self.save_in_progress.clone();
        let feature_compat = self.feature_compat;

        // Try to acquire save guard (returns None if already saving)
        let guard = match SaveGuard::new(save_in_progress) {
            Some(g) => g,
            None => {
                debug!("Save already in progress, skipping");
                return Task::none();
            }
        };

        Task::future(async move {
            // Guard will automatically clear flag on drop (even on panic)
            let _guard = guard;
            let start = Instant::now();

            // Take all dirty categories (this clears them atomically)
            let dirty_set = dirty_tracker.take();

            if dirty_set.is_empty() {
                info!("No dirty categories to save");
                return SaveResult::NothingToSave;
            }

            let dirty_categories_vec: Vec<SettingsCategory> = dirty_set.iter().copied().collect();
            info!("Saving {} dirty categories: {:?}", dirty_set.len(), dirty_categories_vec);

            // Clone settings for async save (releases lock immediately)
            let settings_snapshot = {
                settings.lock().expect("settings mutex poisoned").clone()
            };

            // Validate settings before saving
            let validation_result = crate::config::validation::validate_settings(&settings_snapshot);
            if !validation_result.is_valid() {
                // Log errors but don't block save - warnings are logged in validate_settings
                for err in &validation_result.errors {
                    error!("Validation error (saving anyway): {}", err);
                }
            }

            // Perform actual save (async I/O, no locks held)
            let result: Result<Result<usize, _>, _> = tokio::task::spawn_blocking(move || {
                crate::config::save_dirty(&paths, &settings_snapshot, &dirty_set, feature_compat)
            })
            .await;

            let save_result = match result {
                Ok(Ok(files_written)) => {
                    let elapsed = start.elapsed();
                    info!("Save completed: {} files in {:?}", files_written, elapsed);

                    SaveResult::Success {
                        files_written,
                        categories: dirty_categories_vec,
                    }
                }
                Ok(Err(e)) => {
                    error!("Save failed: {}", e);
                    SaveResult::Error {
                        message: e.to_string(),
                    }
                }
                Err(e) => {
                    error!("Save task panicked: {}", e);
                    SaveResult::Error {
                        message: format!("Save task panicked: {}", e),
                    }
                }
            };

            // Guard automatically clears flag here when it drops
            save_result
        })
    }

    /// Creates a Task that reloads niri config via IPC
    pub fn reload_niri_config_task() -> Task<ReloadResult> {
        Task::future(async move {
            let result: Result<Result<(), _>, _> = tokio::task::spawn_blocking(|| {
                crate::ipc::reload_config()
            })
            .await;

            match result {
                Ok(Ok(())) => {
                    info!("Niri config reloaded successfully");
                    ReloadResult::Success
                }
                Ok(Err(e)) => {
                    error!("Failed to reload niri config: {}", e);
                    ReloadResult::Error {
                        message: e.to_string(),
                    }
                }
                Err(e) => {
                    error!("Reload task panicked: {}", e);
                    ReloadResult::Error {
                        message: format!("Task panicked: {}", e),
                    }
                }
            }
        })
    }
}

/// Result of a save operation
#[derive(Debug, Clone)]
pub enum SaveResult {
    /// Save completed successfully
    Success {
        files_written: usize,
        categories: Vec<SettingsCategory>,
    },
    /// Save failed with error
    Error { message: String },
    /// Nothing needed saving
    NothingToSave,
}

/// Result of niri config reload
#[derive(Debug, Clone)]
pub enum ReloadResult {
    /// Reload successful
    Success,
    /// Reload failed
    Error { message: String },
}
