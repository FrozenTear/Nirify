//! Application state management for niri-settings
//!
//! Uses Freya's reactive state system (use_state) for UI state management.
//! Global app state is managed through a shared Arc<Mutex<>> for settings.

use std::sync::{Arc, Mutex};

use crate::config::{save_dirty, ConfigPaths, DirtyTracker, Settings, SettingsCategory};

/// Navigation category for the sidebar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Category {
    #[default]
    Appearance,
    Keyboard,
    Mouse,
    Touchpad,
    Trackpoint,
    Trackball,
    Tablet,
    Touch,
    Outputs,
    Animations,
    Cursor,
    Overview,
    RecentWindows,
    Behavior,
    LayoutExtras,
    Workspaces,
    WindowRules,
    LayerRules,
    Gestures,
    Keybindings,
    Startup,
    Environment,
    SwitchEvents,
    Miscellaneous,
    Debug,
}

impl Category {
    /// Get the display label for this category
    pub fn label(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Keyboard => "Keyboard",
            Self::Mouse => "Mouse",
            Self::Touchpad => "Touchpad",
            Self::Trackpoint => "Trackpoint",
            Self::Trackball => "Trackball",
            Self::Tablet => "Tablet",
            Self::Touch => "Touch",
            Self::Outputs => "Displays",
            Self::Animations => "Animations",
            Self::Cursor => "Cursor",
            Self::Overview => "Overview",
            Self::RecentWindows => "Recent Windows",
            Self::Behavior => "Behavior",
            Self::LayoutExtras => "Layout Extras",
            Self::Workspaces => "Workspaces",
            Self::WindowRules => "Window Rules",
            Self::LayerRules => "Layer Rules",
            Self::Gestures => "Gestures",
            Self::Keybindings => "Keybindings",
            Self::Startup => "Startup",
            Self::Environment => "Environment",
            Self::SwitchEvents => "Switch Events",
            Self::Miscellaneous => "Miscellaneous",
            Self::Debug => "Debug",
        }
    }

    /// Get the navigation group for this category
    pub fn nav_group(&self) -> NavGroup {
        match self {
            Self::Appearance => NavGroup::Appearance,
            Self::Keyboard
            | Self::Mouse
            | Self::Touchpad
            | Self::Trackpoint
            | Self::Trackball
            | Self::Tablet
            | Self::Touch
            | Self::Outputs => NavGroup::Input,
            Self::Animations | Self::Cursor | Self::Overview | Self::RecentWindows => {
                NavGroup::Visuals
            }
            Self::Behavior | Self::LayoutExtras | Self::Workspaces => NavGroup::Behavior,
            Self::WindowRules | Self::LayerRules | Self::Gestures => NavGroup::Rules,
            Self::Keybindings
            | Self::Startup
            | Self::Environment
            | Self::SwitchEvents
            | Self::Miscellaneous
            | Self::Debug => NavGroup::System,
        }
    }
}

/// Navigation group (top-level tabs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavGroup {
    #[default]
    Appearance,
    Input,
    Visuals,
    Behavior,
    Rules,
    System,
}

impl NavGroup {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Input => "Input",
            Self::Visuals => "Visuals",
            Self::Behavior => "Behavior",
            Self::Rules => "Rules",
            Self::System => "System",
        }
    }

    pub fn all() -> &'static [NavGroup] {
        &[
            Self::Appearance,
            Self::Input,
            Self::Visuals,
            Self::Behavior,
            Self::Rules,
            Self::System,
        ]
    }

    /// Get the categories in this group
    pub fn categories(&self) -> &'static [Category] {
        match self {
            Self::Appearance => &[Category::Appearance],
            Self::Input => &[
                Category::Keyboard,
                Category::Mouse,
                Category::Touchpad,
                Category::Trackpoint,
                Category::Trackball,
                Category::Tablet,
                Category::Touch,
                Category::Outputs,
            ],
            Self::Visuals => &[
                Category::Animations,
                Category::Cursor,
                Category::Overview,
                Category::RecentWindows,
            ],
            Self::Behavior => &[
                Category::Behavior,
                Category::LayoutExtras,
                Category::Workspaces,
            ],
            Self::Rules => &[
                Category::WindowRules,
                Category::LayerRules,
                Category::Gestures,
            ],
            Self::System => &[
                Category::Keybindings,
                Category::Startup,
                Category::Environment,
                Category::SwitchEvents,
                Category::Miscellaneous,
                Category::Debug,
            ],
        }
    }
}

/// Global application state - shared across the application
///
/// In Freya, local UI state uses `use_state()` hooks within components.
/// This struct holds the shared application data that persists across
/// component re-renders.
#[derive(Clone)]
pub struct AppState {
    /// Settings data (shared with config system)
    pub settings: Arc<Mutex<Settings>>,
    /// Config paths
    pub paths: Arc<ConfigPaths>,
    /// Dirty tracker for selective saves
    pub dirty_tracker: Arc<DirtyTracker>,
}

impl AppState {
    pub fn new(settings: Arc<Mutex<Settings>>, paths: Arc<ConfigPaths>) -> Self {
        Self {
            settings,
            paths,
            dirty_tracker: Arc::new(DirtyTracker::new()),
        }
    }

    /// Mark a settings category as dirty and save immediately
    ///
    /// Changes are saved right away to provide immediate feedback.
    /// The DirtyTracker ensures only changed categories are written.
    pub fn mark_dirty_and_save(&self, category: SettingsCategory) {
        self.dirty_tracker.mark(category);
        self.perform_save();
    }

    /// Perform the save of all dirty categories
    fn perform_save(&self) {
        let dirty = self.dirty_tracker.take();
        if dirty.is_empty() {
            return;
        }

        let settings = self.get_settings();
        match save_dirty(&self.paths, &settings, &dirty) {
            Ok(count) => {
                log::debug!("Auto-saved {} category files", count);
            }
            Err(e) => {
                log::error!("Auto-save failed: {}", e);
            }
        }
    }

    /// Get a clone of the current settings
    pub fn get_settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    /// Update settings with a closure
    pub fn update_settings<F>(&self, f: F)
    where
        F: FnOnce(&mut Settings),
    {
        let mut settings = self.settings.lock().unwrap();
        f(&mut settings);
    }
}
