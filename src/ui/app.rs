//! Main application view
//!
//! Crystalline Dark - A refined settings interface
//!
//! Layout structure (top to bottom):
//! ┌─────────────────────────────────────┐
//! │ Header                              │ (title + primary nav)
//! ├─────────────────────────────────────┤
//! │ Secondary Nav                       │ (subcategory pills)
//! ├─────────────────────────────────────┤
//! │ Search Bar                          │ (search + actions)
//! ├─────────────────────────────────────┤
//! │                                     │
//! │ Content Area (scrollable)           │
//! │                                     │
//! ├─────────────────────────────────────┤
//! │ Footer                              │ (status + close)
//! └─────────────────────────────────────┘

use floem::prelude::*;
use floem::views::{dyn_view, Scroll, Stack};

use crate::ui::nav::{footer, header, search_bar, sidebar};
use crate::ui::pages::*;
use crate::ui::state::{AppState, Category};
use crate::ui::theme::{content_style, BG_DEEP};

/// Create the main application view
pub fn app_view(state: AppState) -> impl IntoView {
    let nav_group = state.nav_group;
    let category = state.category;
    let search_query = state.search_query;
    let state_for_content = state.clone();

    Stack::vertical((
        // Header with title and primary navigation
        header(nav_group, category),
        // Secondary navigation with subcategory pills
        sidebar(nav_group, category),
        // Search bar with actions
        search_bar(search_query),
        // Scrollable content area
        Scroll::new(
            dyn_view(move || {
                let cat = category.get();
                let state = state_for_content.clone();

                match cat {
                    // Appearance
                    Category::Appearance => appearance_page(state).into_any(),

                    // Input devices
                    Category::Keyboard => keyboard_page(state).into_any(),
                    Category::Mouse => mouse_page(state).into_any(),
                    Category::Touchpad => touchpad_page(state).into_any(),
                    Category::Trackpoint => trackpoint_page(state).into_any(),
                    Category::Trackball => trackball_page(state).into_any(),
                    Category::Tablet => tablet_page(state).into_any(),
                    Category::Touch => touch_page(state).into_any(),
                    Category::Outputs => outputs_page(state).into_any(),

                    // Visuals
                    Category::Animations => animations_page(state).into_any(),
                    Category::Cursor => cursor_page(state).into_any(),
                    Category::Overview => overview_page(state).into_any(),
                    Category::RecentWindows => recent_windows_page(state).into_any(),

                    // Behavior
                    Category::Behavior => behavior_page(state).into_any(),
                    Category::LayoutExtras => layout_extras_page(state).into_any(),
                    Category::Workspaces => workspaces_page(state).into_any(),

                    // Rules
                    Category::WindowRules => window_rules_page(state).into_any(),
                    Category::LayerRules => layer_rules_page(state).into_any(),
                    Category::Gestures => gestures_page(state).into_any(),

                    // System
                    Category::Keybindings => keybindings_page(state).into_any(),
                    Category::Startup => startup_page(state).into_any(),
                    Category::Environment => environment_page(state).into_any(),
                    Category::SwitchEvents => switch_events_page(state).into_any(),
                    Category::Miscellaneous => miscellaneous_page(state).into_any(),
                    Category::Debug => debug_page(state).into_any(),
                }
            })
            .style(|s| s.width_full()),
        )
        .style(content_style),
        // Footer with status and close button
        footer(),
    ))
    .style(|s| s.width_full().height_full().background(BG_DEEP))
}
