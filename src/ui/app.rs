//! Main application view
//!
//! Crystalline Dark - A refined settings interface built with Freya

use freya::prelude::*;

use crate::ui::nav::{footer, header, sidebar};
use crate::ui::pages::*;
use crate::ui::state::{AppState, Category, NavGroup};
use crate::ui::theme::*;

/// Create the main application view
pub fn app_view(state: AppState) -> impl IntoElement {
    // Initialize Freya theme (required for built-in components like Switch, Slider, Input)
    let _theme = use_init_theme(|| DARK_THEME);

    let current_category = use_state(|| Category::Appearance);
    let current_nav_group = use_state(|| NavGroup::Appearance);

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(BG_DEEP)
        .child(header(current_nav_group, current_category))
        .child(sidebar(current_nav_group, current_category))
        .child(content_area(state.clone(), current_category))
        .child(footer())
}

/// Content area with the current page
fn content_area(state: AppState, current_category: State<Category>) -> impl IntoElement {
    let cat = *current_category.read();

    ScrollView::new()
        .width(Size::fill())
        .height(Size::flex(1.0))
        .child(
            rect()
                .width(Size::fill())
                .padding((SPACING_2XL, SPACING_2XL, SPACING_2XL, SPACING_2XL))
                .background(BG_BASE)
                .child(render_page(state, cat)),
        )
}

/// Render the appropriate page based on category
fn render_page(state: AppState, category: Category) -> Element {
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
