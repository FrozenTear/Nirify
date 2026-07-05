//! Dirty tracking for settings categories
//!
//! This module provides category-based dirty tracking to enable selective
//! saving of configuration files. Instead of rewriting all 25+ config files
//! on every change, only the files for categories that were actually modified
//! are saved.
//!
//! # Usage
//!
//! The `DirtyTracker` is shared across the application and is marked when
//! callbacks modify settings. When the save timer fires, only dirty categories
//! are written to disk.

use std::collections::HashSet;
use std::sync::Mutex;

/// Categories of settings that map to individual KDL config files.
///
/// Each variant corresponds to one or more `.kdl` files that need to be
/// written when settings in that category change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsCategory {
    /// appearance.kdl - gaps, focus ring, borders, corner radius
    Appearance,
    /// behavior.kdl - focus follows mouse, warp mouse, struts
    Behavior,
    /// input/keyboard.kdl - XKB layout, repeat rate/delay
    Keyboard,
    /// input/mouse.kdl - acceleration, scroll settings
    Mouse,
    /// input/touchpad.kdl - tap-to-click, natural scroll
    Touchpad,
    /// input/trackpoint.kdl - trackpoint settings
    Trackpoint,
    /// input/trackball.kdl - trackball settings
    Trackball,
    /// input/tablet.kdl - tablet/pen settings
    Tablet,
    /// input/touch.kdl - touch screen settings
    Touch,
    /// outputs.kdl - display configuration
    Outputs,
    /// animations.kdl - window animations
    Animations,
    /// cursor.kdl - cursor theme, size
    Cursor,
    /// overview.kdl - overview mode settings
    Overview,
    /// blur.kdl - top-level background blur (niri 26.04+)
    Blur,
    /// workspaces.kdl - named workspaces
    Workspaces,
    /// keybindings.kdl - keyboard shortcuts
    Keybindings,
    /// advanced/layout-extras.kdl - additional layout settings
    LayoutExtras,
    /// advanced/gestures.kdl - touchpad gestures
    Gestures,
    /// advanced/layer-rules.kdl - layer shell rules
    LayerRules,
    /// advanced/window-rules.kdl - window matching rules
    WindowRules,
    /// advanced/misc.kdl - miscellaneous settings
    Miscellaneous,
    /// advanced/startup.kdl - spawn-at-startup commands
    Startup,
    /// advanced/environment.kdl - environment variables
    Environment,
    /// advanced/debug.kdl - debug options
    Debug,
    /// advanced/switch-events.kdl - lid/tablet mode switch
    SwitchEvents,
    /// advanced/recent-windows.kdl - recent window switcher
    RecentWindows,
    /// advanced/preferences.kdl - app preferences (theme, etc.)
    Preferences,
}

impl SettingsCategory {
    /// Get all categories as a slice for iteration
    pub const fn all() -> &'static [SettingsCategory] {
        &[
            SettingsCategory::Appearance,
            SettingsCategory::Behavior,
            SettingsCategory::Keyboard,
            SettingsCategory::Mouse,
            SettingsCategory::Touchpad,
            SettingsCategory::Trackpoint,
            SettingsCategory::Trackball,
            SettingsCategory::Tablet,
            SettingsCategory::Touch,
            SettingsCategory::Outputs,
            SettingsCategory::Animations,
            SettingsCategory::Cursor,
            SettingsCategory::Overview,
            SettingsCategory::Blur,
            SettingsCategory::Workspaces,
            SettingsCategory::Keybindings,
            SettingsCategory::LayoutExtras,
            SettingsCategory::Gestures,
            SettingsCategory::LayerRules,
            SettingsCategory::WindowRules,
            SettingsCategory::Miscellaneous,
            SettingsCategory::Startup,
            SettingsCategory::Environment,
            SettingsCategory::Debug,
            SettingsCategory::SwitchEvents,
            SettingsCategory::RecentWindows,
            SettingsCategory::Preferences,
        ]
    }

    /// Get a human-readable name for logging
    pub const fn name(&self) -> &'static str {
        match self {
            SettingsCategory::Appearance => "Appearance",
            SettingsCategory::Behavior => "Behavior",
            SettingsCategory::Keyboard => "Keyboard",
            SettingsCategory::Mouse => "Mouse",
            SettingsCategory::Touchpad => "Touchpad",
            SettingsCategory::Trackpoint => "Trackpoint",
            SettingsCategory::Trackball => "Trackball",
            SettingsCategory::Tablet => "Tablet",
            SettingsCategory::Touch => "Touch",
            SettingsCategory::Outputs => "Outputs",
            SettingsCategory::Animations => "Animations",
            SettingsCategory::Cursor => "Cursor",
            SettingsCategory::Overview => "Overview",
            SettingsCategory::Blur => "Blur",
            SettingsCategory::Workspaces => "Workspaces",
            SettingsCategory::Keybindings => "Keybindings",
            SettingsCategory::LayoutExtras => "Layout Extras",
            SettingsCategory::Gestures => "Gestures",
            SettingsCategory::LayerRules => "Layer Rules",
            SettingsCategory::WindowRules => "Window Rules",
            SettingsCategory::Miscellaneous => "Miscellaneous",
            SettingsCategory::Startup => "Startup",
            SettingsCategory::Environment => "Environment",
            SettingsCategory::Debug => "Debug",
            SettingsCategory::SwitchEvents => "Switch Events",
            SettingsCategory::RecentWindows => "Recent Windows",
            SettingsCategory::Preferences => "Preferences",
        }
    }

    /// Map a relative KDL file name (as used by `load_settings_with_result`'s
    /// `track()` calls) to the category whose save writes that file.
    ///
    /// Returns `None` for names that no single category owns.
    pub fn from_relative_path(p: &str) -> Option<Self> {
        Some(match p {
            "keybindings.kdl" => SettingsCategory::Keybindings,
            "appearance.kdl" => SettingsCategory::Appearance,
            "behavior.kdl" => SettingsCategory::Behavior,
            "input/keyboard.kdl" => SettingsCategory::Keyboard,
            "input/mouse.kdl" => SettingsCategory::Mouse,
            "input/touchpad.kdl" => SettingsCategory::Touchpad,
            "input/trackpoint.kdl" => SettingsCategory::Trackpoint,
            "input/trackball.kdl" => SettingsCategory::Trackball,
            "input/tablet.kdl" => SettingsCategory::Tablet,
            "input/touch.kdl" => SettingsCategory::Touch,
            "animations.kdl" => SettingsCategory::Animations,
            "cursor.kdl" => SettingsCategory::Cursor,
            "overview.kdl" => SettingsCategory::Overview,
            "blur.kdl" => SettingsCategory::Blur,
            "outputs.kdl" => SettingsCategory::Outputs,
            "advanced/layout-extras.kdl" => SettingsCategory::LayoutExtras,
            "advanced/gestures.kdl" => SettingsCategory::Gestures,
            "advanced/misc.kdl" => SettingsCategory::Miscellaneous,
            "workspaces.kdl" => SettingsCategory::Workspaces,
            "advanced/layer-rules.kdl" => SettingsCategory::LayerRules,
            "advanced/window-rules.kdl" => SettingsCategory::WindowRules,
            "advanced/startup.kdl" => SettingsCategory::Startup,
            "advanced/environment.kdl" => SettingsCategory::Environment,
            "advanced/debug.kdl" => SettingsCategory::Debug,
            "advanced/switch-events.kdl" => SettingsCategory::SwitchEvents,
            "advanced/recent-windows.kdl" => SettingsCategory::RecentWindows,
            "advanced/preferences.kdl" => SettingsCategory::Preferences,
            _ => return None,
        })
    }
}

/// Tracks which settings categories have been modified since the last save.
///
/// Thread-safe via internal `Mutex`. Categories are marked when callbacks
/// modify settings, and the set is atomically taken (cleared and returned)
/// when the save timer fires.
///
/// # Example
///
/// ```ignore
/// let tracker = DirtyTracker::new();
///
/// // In a callback:
/// tracker.mark(SettingsCategory::Appearance);
///
/// // When save timer fires:
/// let dirty = tracker.take();
/// for category in dirty {
///     save_category(category);
/// }
/// ```
pub struct DirtyTracker {
    dirty: Mutex<HashSet<SettingsCategory>>,
}

impl DirtyTracker {
    /// Create a new empty dirty tracker
    pub fn new() -> Self {
        Self {
            dirty: Mutex::new(HashSet::new()),
        }
    }

    /// Mark a category as dirty (needing save)
    ///
    /// This is called from callbacks when settings change.
    /// Recovers from poisoned mutex to ensure dirty tracking continues even after panics.
    pub fn mark(&self, category: SettingsCategory) {
        let mut set = match self.dirty.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("DirtyTracker mutex poisoned in mark(), recovering");
                poisoned.into_inner()
            }
        };
        set.insert(category);
        log::trace!("Marked {} as dirty", category.name());
    }

    /// Mark multiple categories as dirty
    ///
    /// Useful when a single UI action affects multiple config files.
    /// Recovers from poisoned mutex to ensure dirty tracking continues even after panics.
    pub fn mark_many(&self, categories: &[SettingsCategory]) {
        let mut set = match self.dirty.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("DirtyTracker mutex poisoned in mark_many(), recovering");
                poisoned.into_inner()
            }
        };
        for category in categories {
            set.insert(*category);
        }
        if !categories.is_empty() {
            log::trace!("Marked {} categories as dirty", categories.len());
        }
    }

    /// Take and clear all dirty categories
    ///
    /// Returns the set of categories that were dirty, and resets the tracker
    /// to empty. This is called when the save timer fires.
    pub fn take(&self) -> HashSet<SettingsCategory> {
        match self.dirty.lock() {
            Ok(mut set) => std::mem::take(&mut *set),
            Err(poisoned) => {
                // Recover from poisoned mutex by taking the data anyway
                log::warn!("DirtyTracker mutex was poisoned, recovering");
                std::mem::take(&mut *poisoned.into_inner())
            }
        }
    }

    /// Take and clear all dirty categories except those in `excluded`.
    ///
    /// Returns the set of categories that were dirty and not excluded, and
    /// clears exactly those from the tracker. Excluded categories that were
    /// dirty remain marked. Used to keep load-blocked categories from being
    /// written to disk while still allowing them to accumulate edits.
    pub fn take_except(&self, excluded: &HashSet<SettingsCategory>) -> HashSet<SettingsCategory> {
        let mut set = match self.dirty.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("DirtyTracker mutex was poisoned in take_except, recovering");
                poisoned.into_inner()
            }
        };
        let taken: HashSet<SettingsCategory> = set.difference(excluded).copied().collect();
        for c in &taken {
            set.remove(c);
        }
        taken
    }

    /// Check if any category is dirty
    ///
    /// Useful for quick checks without taking the set.
    pub fn is_dirty(&self) -> bool {
        match self.dirty.lock() {
            Ok(set) => !set.is_empty(),
            Err(poisoned) => !poisoned.into_inner().is_empty(),
        }
    }

    /// Peek at the dirty categories without clearing them
    ///
    /// Returns a clone of the current dirty set. Useful for generating
    /// diffs or previewing what would be saved.
    pub fn peek(&self) -> HashSet<SettingsCategory> {
        match self.dirty.lock() {
            Ok(set) => set.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Get count of dirty categories (for logging)
    pub fn dirty_count(&self) -> usize {
        match self.dirty.lock() {
            Ok(set) => set.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// Mark all categories as dirty
    ///
    /// Used when we need to force a full save (e.g., first run, repair).
    /// Recovers from poisoned mutex to ensure dirty tracking continues even after panics.
    pub fn mark_all(&self) {
        let mut set = match self.dirty.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("DirtyTracker mutex poisoned in mark_all(), recovering");
                poisoned.into_inner()
            }
        };
        for category in SettingsCategory::all() {
            set.insert(*category);
        }
        log::debug!("Marked all {} categories as dirty", set.len());
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_and_take() {
        let tracker = DirtyTracker::new();
        assert!(!tracker.is_dirty());

        tracker.mark(SettingsCategory::Appearance);
        assert!(tracker.is_dirty());
        assert_eq!(tracker.dirty_count(), 1);

        tracker.mark(SettingsCategory::Behavior);
        assert_eq!(tracker.dirty_count(), 2);

        let dirty = tracker.take();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&SettingsCategory::Appearance));
        assert!(dirty.contains(&SettingsCategory::Behavior));

        // After take, should be empty
        assert!(!tracker.is_dirty());
        assert_eq!(tracker.dirty_count(), 0);
    }

    #[test]
    fn test_mark_many() {
        let tracker = DirtyTracker::new();

        tracker.mark_many(&[
            SettingsCategory::Mouse,
            SettingsCategory::Touchpad,
            SettingsCategory::Keyboard,
        ]);

        assert_eq!(tracker.dirty_count(), 3);
    }

    #[test]
    fn test_mark_all() {
        let tracker = DirtyTracker::new();
        tracker.mark_all();

        let dirty = tracker.take();
        assert_eq!(dirty.len(), SettingsCategory::all().len());
    }

    #[test]
    fn test_duplicate_marks() {
        let tracker = DirtyTracker::new();

        // Marking the same category twice should not increase count
        tracker.mark(SettingsCategory::Cursor);
        tracker.mark(SettingsCategory::Cursor);
        tracker.mark(SettingsCategory::Cursor);

        assert_eq!(tracker.dirty_count(), 1);
    }

    #[test]
    fn test_category_all_count() {
        // Ensure we have all 27 categories
        assert_eq!(SettingsCategory::all().len(), 27);
    }

    #[test]
    fn take_except_leaves_excluded_marked() {
        let tracker = DirtyTracker::new();
        tracker.mark(SettingsCategory::Appearance);
        tracker.mark(SettingsCategory::Mouse);

        let mut excluded = HashSet::new();
        excluded.insert(SettingsCategory::Mouse);

        let taken = tracker.take_except(&excluded);
        assert_eq!(taken.len(), 1);
        assert!(taken.contains(&SettingsCategory::Appearance));

        // Mouse stays marked
        assert!(tracker.is_dirty());
        assert!(tracker.peek().contains(&SettingsCategory::Mouse));
        assert!(!tracker.peek().contains(&SettingsCategory::Appearance));
    }

    #[test]
    fn from_relative_path_covers_all_tracked_files() {
        let tracked = [
            "keybindings.kdl",
            "appearance.kdl",
            "behavior.kdl",
            "input/keyboard.kdl",
            "input/mouse.kdl",
            "input/touchpad.kdl",
            "input/trackpoint.kdl",
            "input/trackball.kdl",
            "input/tablet.kdl",
            "input/touch.kdl",
            "animations.kdl",
            "cursor.kdl",
            "overview.kdl",
            "blur.kdl",
            "outputs.kdl",
            "advanced/layout-extras.kdl",
            "advanced/gestures.kdl",
            "advanced/misc.kdl",
            "workspaces.kdl",
            "advanced/layer-rules.kdl",
            "advanced/window-rules.kdl",
            "advanced/startup.kdl",
            "advanced/environment.kdl",
            "advanced/debug.kdl",
            "advanced/switch-events.kdl",
            "advanced/recent-windows.kdl",
            "advanced/preferences.kdl",
        ];
        for name in tracked {
            assert!(
                SettingsCategory::from_relative_path(name).is_some(),
                "expected Some for {}",
                name
            );
        }
        assert!(SettingsCategory::from_relative_path("bogus.kdl").is_none());
    }
}
