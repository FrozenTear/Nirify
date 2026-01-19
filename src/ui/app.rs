//! Main application view
//!
//! Obsidian Editorial - A refined settings interface with centered navigation

use freya::prelude::*;

use crate::config::{Settings, SettingsCategory};
use crate::ui::nav::{footer, header, subnav};
use crate::ui::pages::*;
use crate::ui::state::{AppState, Category, NavGroup};
use crate::ui::theme::*;

/// Reactive state holder that wraps AppState with a refresh mechanism
#[derive(Clone)]
pub struct ReactiveState {
    pub app_state: AppState,
    pub settings: Settings,
    /// Public refresh signal - callbacks should clone this as `mut` and call set()
    pub refresh: State<u64>,
}

impl ReactiveState {
    pub fn new(app_state: AppState, settings: Settings, refresh: State<u64>) -> Self {
        Self {
            app_state,
            settings,
            refresh,
        }
    }

    /// Get current settings
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    /// Update settings and save (caller handles refresh trigger)
    pub fn update_and_save<F>(&self, category: SettingsCategory, f: F)
    where
        F: FnOnce(&mut Settings),
    {
        self.app_state.update_settings(f);
        self.app_state.mark_dirty_and_save(category);
    }
}

/// Create the main application view
pub fn app_view(state: AppState) -> impl IntoElement {
    // Initialize Freya theme (required for built-in components like Switch, Slider, Input)
    let _theme = use_init_theme(|| DARK_THEME);

    let current_category = use_state(|| Category::Appearance);
    let current_nav_group = use_state(|| NavGroup::Appearance);

    // Refresh counter - incrementing this triggers re-renders
    let refresh = use_state(|| 0u64);

    // Read current refresh value to create dependency
    let _refresh_val = *refresh.read();

    // Get fresh settings on each render
    let settings = state.get_settings();
    let reactive_state = ReactiveState::new(state, settings, refresh);

    rect()
        .content(Content::flex())
        .direction(Direction::Vertical)
        .width(Size::fill())
        .height(Size::fill())
        .background(BG_DEEP)
        // Main navigation header with centered title and nav
        .child(header(current_nav_group, current_category))
        // Sub-navigation bar with category pills
        .child(subnav(current_nav_group, current_category))
        // Content area - takes remaining space
        .child(content_area(reactive_state, current_category))
        // Minimal footer
        .child(footer())
}

/// Content area with the current page
fn content_area(state: ReactiveState, current_category: State<Category>) -> impl IntoElement {
    let cat = *current_category.read();

    rect()
        .content(Content::flex())
        .width(Size::fill())
        .height(Size::flex(1.0))
        .background(BG_BASE)
        .child(
            ScrollView::new()
                .width(Size::fill())
                .height(Size::fill())
                .child(
                    // Outer container - centers the content horizontally
                    rect()
                        .content(Content::flex())
                        .width(Size::fill())
                        .cross_align(Alignment::Center)
                        .padding((SPACING_3XL, SPACING_2XL, SPACING_3XL, SPACING_2XL))
                        .child(
                            // Max-width container for content - wider for better use of space
                            rect()
                                .width(Size::fill())
                                .max_width(Size::px(1000.0))
                                .child(render_page(state, cat)),
                        ),
                ),
        )
}

/// Render the appropriate page based on category
fn render_page(state: ReactiveState, category: Category) -> Element {
    match category {
        Category::Appearance => appearance_page(state).into_element(),
        Category::Keyboard => keyboard_page(state).into_element(),
        Category::Mouse => mouse_page(state).into_element(),
        Category::Touchpad => touchpad_page(state).into_element(),
        Category::Trackpoint => trackpoint_page(state).into_element(),
        Category::Trackball => trackball_page(state).into_element(),
        Category::Tablet => tablet_page(state).into_element(),
        Category::Touch => touch_page(state).into_element(),
        Category::Outputs => outputs_page(state).into_element(),
        Category::Animations => animations_page(state).into_element(),
        Category::Cursor => cursor_page(state).into_element(),
        Category::Overview => overview_page(state).into_element(),
        Category::RecentWindows => recent_windows_page(state).into_element(),
        Category::Behavior => behavior_page(state).into_element(),
        Category::LayoutExtras => layout_extras_page(state).into_element(),
        Category::Workspaces => workspaces_page(state).into_element(),
        Category::WindowRules => window_rules_page(state).into_element(),
        Category::LayerRules => layer_rules_page(state).into_element(),
        Category::Gestures => gestures_page(state).into_element(),
        Category::Keybindings => keybindings_page(state).into_element(),
        Category::Startup => startup_page(state).into_element(),
        Category::Environment => environment_page(state).into_element(),
        Category::SwitchEvents => switch_events_page(state).into_element(),
        Category::Miscellaneous => miscellaneous_page(state).into_element(),
        Category::Debug => debug_page(state).into_element(),
    }
}
