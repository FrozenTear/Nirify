//! Debounced save manager for settings
//!
//! Provides a timer-based debounce mechanism to avoid excessive disk I/O
//! when users rapidly change settings (e.g., dragging sliders).
//!
//! Settings changes are batched and saved after a debounce delay.
//! Only categories that have been marked as dirty are saved, reducing
//! disk I/O from ~20 files to typically 1-3 per change.
//!
//! # Threading
//!
//! File I/O runs on the UI thread via timer callback, but IPC reload runs
//! on a background thread via [`ipc::async_ops::reload_config_async`] to
//! prevent UI freezes when niri is slow or unresponsive.

use crate::config::{save_dirty, ConfigPaths, DirtyTracker, Settings, SettingsCategory};
use crate::constants::{SAVE_DEBOUNCE_MS, TOAST_DISMISS_MS};
use crate::ipc;
use crate::MainWindow;
use log::{debug, info, warn};
use slint::{Timer, TimerMode, Weak};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Manages debounced saving of settings
///
/// When `request_save()` is called, it starts a timer. If called again
/// before the timer fires, the timer is reset. Only when the timer
/// actually fires (after DEBOUNCE_DELAY_MS of inactivity) is the
/// save performed.
///
/// Integrates with `DirtyTracker` to only save categories that have
/// been modified, reducing disk I/O from 25 files to typically 1-3.
///
/// This is wrapped in `Rc` and passed to callbacks, eliminating global state.
pub struct SaveManager {
    timer: Timer,
    ui_weak: Weak<MainWindow>,
    toast_timer: Timer,
    dirty_tracker: Arc<DirtyTracker>,
}

impl SaveManager {
    /// Create a new SaveManager wrapped in Rc
    ///
    /// # Arguments
    ///
    /// * `settings` - The settings to save
    /// * `paths` - Configuration paths for saving
    /// * `ui_weak` - Weak reference to MainWindow for showing toast notifications
    /// * `dirty_tracker` - Shared dirty tracker for selective saving
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        paths: Arc<ConfigPaths>,
        ui_weak: Weak<MainWindow>,
        dirty_tracker: Arc<DirtyTracker>,
    ) -> Rc<Self> {
        let timer = Timer::default();
        let toast_timer = Timer::default();

        // Clone ui_weak for the save timer callback
        let ui_for_save = ui_weak.clone();
        // Clone dirty_tracker for the save timer callback
        let tracker_for_save = Arc::clone(&dirty_tracker);

        timer.start(
            TimerMode::SingleShot,
            std::time::Duration::from_millis(SAVE_DEBOUNCE_MS),
            move || {
                // Take dirty categories before acquiring settings lock
                let dirty = tracker_for_save.take();

                if dirty.is_empty() {
                    debug!("No dirty categories to save");
                    return;
                }

                // Clone ONLY dirty categories while holding lock, then release before disk I/O.
                // This prevents blocking UI callbacks if disk I/O is slow
                // (e.g., network-mounted home directories, disk contention).
                //
                // Instead of cloning the entire Settings struct (~10KB+ with rules),
                // we only clone the categories that will actually be saved.
                let settings_copy = match settings.lock() {
                    Ok(guard) => clone_dirty_categories(&guard, &dirty),
                    Err(poisoned) => {
                        warn!(
                            "Settings mutex poisoned during auto-save - a callback \
                             likely panicked. Recovering data. Check logs for details."
                        );
                        clone_dirty_categories(&poisoned.into_inner(), &dirty)
                    }
                };
                // Lock is released here before disk I/O

                // Log which categories are being saved
                let category_names: Vec<&str> = dirty.iter().map(|c| c.name()).collect();
                debug!("Saving {} categories: {:?}", dirty.len(), category_names);

                match save_dirty(&paths, &settings_copy, &dirty) {
                    Ok(files_written) => {
                        debug!(
                            "Settings auto-saved after debounce ({} files written)",
                            files_written
                        );

                        // Try to reload niri config if running (async to prevent UI freeze)
                        if ipc::is_niri_running() {
                            ipc::async_ops::reload_config_async(move |result| match result {
                                Ok(()) => info!("Niri config reloaded"),
                                Err(e) => debug!("Could not reload niri config: {}", e),
                            });
                        }
                    }
                    Err(e) => {
                        warn!("Auto-save failed: {}", e);
                        show_error_toast(&ui_for_save, "Failed to save settings");
                    }
                }
            },
        );

        // Stop the timer immediately - we'll restart it on request_save
        timer.stop();

        debug!(
            "Save manager initialized with {}ms debounce and dirty tracking",
            SAVE_DEBOUNCE_MS
        );

        Rc::new(Self {
            timer,
            ui_weak,
            toast_timer,
            dirty_tracker,
        })
    }

    /// Request a save operation
    ///
    /// This restarts the debounce timer. The actual save will only occur
    /// after DEBOUNCE_DELAY_MS of no further requests.
    ///
    /// Note: You should call `mark_dirty()` before `request_save()` to indicate
    /// which categories need saving. If no categories are dirty when the timer
    /// fires, the save will be skipped.
    pub fn request_save(&self) {
        self.timer.restart();
    }

    /// Mark a settings category as dirty (needing save)
    ///
    /// This should be called whenever a callback modifies settings, before
    /// calling `request_save()`. Only dirty categories will be written to disk.
    ///
    /// # Arguments
    /// * `category` - The settings category that was modified
    pub fn mark_dirty(&self, category: SettingsCategory) {
        self.dirty_tracker.mark(category);
    }

    /// Get a reference to the dirty tracker
    ///
    /// Useful when you need to mark multiple categories or check dirty state.
    #[allow(dead_code)]
    pub fn dirty_tracker(&self) -> &Arc<DirtyTracker> {
        &self.dirty_tracker
    }

    /// Show an error toast notification with auto-dismiss
    #[allow(dead_code)]
    pub fn show_error(&self, message: &str) {
        if let Some(ui) = self.ui_weak.upgrade() {
            ui.invoke_show_status_toast(message.into(), true);

            // Stop any existing timer to prevent old toast callbacks from firing
            self.toast_timer.stop();

            // Start fresh auto-dismiss timer
            let ui_weak = self.ui_weak.clone();
            self.toast_timer.start(
                TimerMode::SingleShot,
                std::time::Duration::from_millis(TOAST_DISMISS_MS),
                move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_visible(false);
                    }
                },
            );
        }
    }

    /// Show a success toast notification with auto-dismiss
    #[allow(dead_code)]
    pub fn show_success(&self, message: &str) {
        if let Some(ui) = self.ui_weak.upgrade() {
            ui.invoke_show_status_toast(message.into(), false);

            // Stop any existing timer to prevent old toast callbacks from firing
            self.toast_timer.stop();

            // Start fresh auto-dismiss timer
            let ui_weak = self.ui_weak.clone();
            self.toast_timer.start(
                TimerMode::SingleShot,
                std::time::Duration::from_millis(TOAST_DISMISS_MS),
                move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_status_visible(false);
                    }
                },
            );
        }
    }
}

/// Helper function to show error toast from within timer callbacks
///
/// Creates its own auto-dismiss timer since it doesn't have access to SaveManager's toast_timer.
/// Uses `Timer::single_shot` which doesn't require keeping a handle alive.
fn show_error_toast(ui_weak: &Weak<MainWindow>, message: &str) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.invoke_show_status_toast(message.into(), true);

        // Start auto-dismiss timer using single_shot (no handle needed)
        let ui_weak_for_dismiss = ui_weak.clone();
        Timer::single_shot(
            std::time::Duration::from_millis(TOAST_DISMISS_MS),
            move || {
                if let Some(ui) = ui_weak_for_dismiss.upgrade() {
                    ui.set_status_visible(false);
                }
            },
        );
    }
}

/// Clone only the dirty categories from Settings into a new Settings struct.
///
/// This avoids cloning the entire Settings struct (which can be large with
/// many window rules, keybindings, etc.) when only a few categories changed.
/// Non-dirty categories are left at their Default values since save_dirty
/// won't access them anyway.
fn clone_dirty_categories(
    source: &Settings,
    dirty: &std::collections::HashSet<SettingsCategory>,
) -> Settings {
    let mut result = Settings::default();

    for category in dirty {
        match category {
            SettingsCategory::Appearance => {
                result.appearance = source.appearance.clone();
                // Appearance KDL also needs behavior for struts
                result.behavior = source.behavior.clone();
            }
            SettingsCategory::Behavior => {
                result.behavior = source.behavior.clone();
            }
            SettingsCategory::Keyboard => {
                result.keyboard = source.keyboard.clone();
            }
            SettingsCategory::Mouse => {
                result.mouse = source.mouse.clone();
            }
            SettingsCategory::Touchpad => {
                result.touchpad = source.touchpad.clone();
            }
            SettingsCategory::Trackpoint => {
                result.trackpoint = source.trackpoint.clone();
            }
            SettingsCategory::Trackball => {
                result.trackball = source.trackball.clone();
            }
            SettingsCategory::Tablet => {
                result.tablet = source.tablet.clone();
            }
            SettingsCategory::Touch => {
                result.touch = source.touch.clone();
            }
            SettingsCategory::Outputs => {
                result.outputs = source.outputs.clone();
            }
            SettingsCategory::Animations => {
                result.animations = source.animations.clone();
            }
            SettingsCategory::Cursor => {
                result.cursor = source.cursor.clone();
            }
            SettingsCategory::Overview => {
                result.overview = source.overview.clone();
            }
            SettingsCategory::Workspaces => {
                result.workspaces = source.workspaces.clone();
            }
            SettingsCategory::Keybindings => {
                result.keybindings = source.keybindings.clone();
            }
            SettingsCategory::LayoutExtras => {
                result.layout_extras = source.layout_extras.clone();
            }
            SettingsCategory::Gestures => {
                result.gestures = source.gestures.clone();
            }
            SettingsCategory::LayerRules => {
                result.layer_rules = source.layer_rules.clone();
            }
            SettingsCategory::WindowRules => {
                result.window_rules = source.window_rules.clone();
            }
            SettingsCategory::Miscellaneous => {
                result.miscellaneous = source.miscellaneous.clone();
            }
            SettingsCategory::Startup => {
                result.startup = source.startup.clone();
            }
            SettingsCategory::Environment => {
                result.environment = source.environment.clone();
            }
            SettingsCategory::Debug => {
                result.debug = source.debug.clone();
            }
            SettingsCategory::SwitchEvents => {
                result.switch_events = source.switch_events.clone();
            }
            SettingsCategory::RecentWindows => {
                result.recent_windows = source.recent_windows.clone();
            }
        }
    }

    result
}
