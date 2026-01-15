//! Category-Section binding for compile-time safety
//!
//! This module provides a sealed trait pattern that binds SettingsCategory
//! enum variants to their corresponding Settings struct fields at compile time.
//!
//! This eliminates the runtime mismatch risk where `Appearance` category could
//! accidentally be paired with `behavior` section in callback macros.
//!
//! # Usage
//!
//! Instead of passing separate category and section identifiers:
//! ```ignore
//! // OLD (error-prone): category and section must match manually
//! register_bool_callbacks!(ui, settings, save_manager, Appearance, appearance, [...]);
//! ```
//!
//! Use a single marker type that encodes both:
//! ```ignore
//! // NEW (compile-time safe): Appearance marker provides both
//! register_bool_callbacks!(ui, settings, save_manager, Appearance, [...]);
//! ```

use super::dirty::SettingsCategory;
use super::models::{
    AnimationSettings, AppearanceSettings, BehaviorSettings, CursorSettings, DebugSettings,
    EnvironmentSettings, GestureSettings, KeybindingsSettings, KeyboardSettings,
    LayerRulesSettings, LayoutExtrasSettings, MiscSettings, MouseSettings, OutputSettings,
    OverviewSettings, RecentWindowsSettings, Settings, StartupSettings, SwitchEventsSettings,
    TabletSettings, TouchSettings, TouchpadSettings, TrackballSettings, TrackpointSettings,
    WindowRulesSettings, WorkspacesSettings,
};

/// Trait binding a marker type to its SettingsCategory and Settings field.
///
/// This trait is sealed - only the marker types in this module implement it.
/// This prevents misuse and ensures compile-time safety.
pub trait CategorySection: private::Sealed {
    /// The settings struct type for this category
    type Section;

    /// The SettingsCategory enum variant
    const CATEGORY: SettingsCategory;

    /// Get immutable reference to this category's section
    fn section(settings: &Settings) -> &Self::Section;

    /// Get mutable reference to this category's section
    fn section_mut(settings: &mut Settings) -> &mut Self::Section;
}

// Sealed trait pattern - prevents external implementations
mod private {
    pub trait Sealed {}
}

// Macro to implement CategorySection for a marker type
macro_rules! impl_category_section {
    ($marker:ident, $section_type:ty, $category:ident, $field:ident) => {
        /// Marker type for compile-time category-section binding
        pub struct $marker;

        impl private::Sealed for $marker {}

        impl CategorySection for $marker {
            type Section = $section_type;
            const CATEGORY: SettingsCategory = SettingsCategory::$category;

            #[inline]
            fn section(settings: &Settings) -> &Self::Section {
                &settings.$field
            }

            #[inline]
            fn section_mut(settings: &mut Settings) -> &mut Self::Section {
                &mut settings.$field
            }
        }
    };
}

// Implement for all 25 categories
impl_category_section!(Appearance, AppearanceSettings, Appearance, appearance);
impl_category_section!(Behavior, BehaviorSettings, Behavior, behavior);
impl_category_section!(Keyboard, KeyboardSettings, Keyboard, keyboard);
impl_category_section!(Mouse, MouseSettings, Mouse, mouse);
impl_category_section!(Touchpad, TouchpadSettings, Touchpad, touchpad);
impl_category_section!(Trackpoint, TrackpointSettings, Trackpoint, trackpoint);
impl_category_section!(Trackball, TrackballSettings, Trackball, trackball);
impl_category_section!(Tablet, TabletSettings, Tablet, tablet);
impl_category_section!(Touch, TouchSettings, Touch, touch);
impl_category_section!(Outputs, OutputSettings, Outputs, outputs);
impl_category_section!(Animations, AnimationSettings, Animations, animations);
impl_category_section!(Cursor, CursorSettings, Cursor, cursor);
impl_category_section!(Overview, OverviewSettings, Overview, overview);
impl_category_section!(Workspaces, WorkspacesSettings, Workspaces, workspaces);
impl_category_section!(Keybindings, KeybindingsSettings, Keybindings, keybindings);
impl_category_section!(
    LayoutExtras,
    LayoutExtrasSettings,
    LayoutExtras,
    layout_extras
);
impl_category_section!(Gestures, GestureSettings, Gestures, gestures);
impl_category_section!(LayerRules, LayerRulesSettings, LayerRules, layer_rules);
impl_category_section!(WindowRules, WindowRulesSettings, WindowRules, window_rules);
impl_category_section!(Miscellaneous, MiscSettings, Miscellaneous, miscellaneous);
impl_category_section!(Startup, StartupSettings, Startup, startup);
impl_category_section!(Environment, EnvironmentSettings, Environment, environment);
impl_category_section!(Debug, DebugSettings, Debug, debug);
impl_category_section!(
    SwitchEvents,
    SwitchEventsSettings,
    SwitchEvents,
    switch_events
);
impl_category_section!(
    RecentWindows,
    RecentWindowsSettings,
    RecentWindows,
    recent_windows
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appearance_binding() {
        let mut settings = Settings::default();

        // Verify the trait correctly accesses the field
        assert_eq!(Appearance::CATEGORY, SettingsCategory::Appearance);

        // Verify section access
        let section = Appearance::section(&settings);
        assert!(section.focus_ring_enabled);

        // Verify mutable access
        let section_mut = Appearance::section_mut(&mut settings);
        section_mut.focus_ring_enabled = false;
        assert!(!settings.appearance.focus_ring_enabled);
    }

    #[test]
    fn test_all_categories_have_correct_mapping() {
        // This test ensures the macro correctly maps each category
        let settings = Settings::default();

        // Spot-check a few categories
        assert_eq!(Behavior::CATEGORY, SettingsCategory::Behavior);
        assert_eq!(Mouse::CATEGORY, SettingsCategory::Mouse);
        assert_eq!(Debug::CATEGORY, SettingsCategory::Debug);

        // Verify we can access sections
        let _ = Behavior::section(&settings);
        let _ = Mouse::section(&settings);
        let _ = Debug::section(&settings);
    }
}
