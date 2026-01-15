//! Layout extras UI callbacks
//!
//! Handles shadow settings, tab indicator, and insert hint.

use crate::config::{Settings, SettingsCategory};
use crate::MainWindow;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{
    register_bool_callback, register_clamped_callback, register_color_callback,
    register_color_or_gradient_callback, register_enum_callback, SaveManager,
};

/// Set up layout extras callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    setup_shadow_callbacks(ui, settings.clone(), Rc::clone(&save_manager));
    setup_tab_indicator_callbacks(ui, settings.clone(), Rc::clone(&save_manager));
    setup_insert_hint_callbacks(ui, settings, save_manager);
}

/// Shadow settings callbacks
fn setup_shadow_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Shadow enabled
    register_bool_callback!(
        ui,
        on_shadow_toggled,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.shadow.enabled,
        "Shadow enabled"
    );

    // Shadow softness
    register_clamped_callback!(
        ui,
        on_shadow_softness_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        0,
        100,
        |s| s.layout_extras.shadow.softness,
        "Shadow softness: {}px"
    );

    // Shadow spread
    register_clamped_callback!(
        ui,
        on_shadow_spread_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        0,
        50,
        |s| s.layout_extras.shadow.spread,
        "Shadow spread: {}px"
    );

    // Shadow offset X
    register_clamped_callback!(
        ui,
        on_shadow_offset_x_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        -50,
        50,
        |s| s.layout_extras.shadow.offset_x,
        "Shadow offset X: {}px"
    );

    // Shadow offset Y
    register_clamped_callback!(
        ui,
        on_shadow_offset_y_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        -50,
        50,
        |s| s.layout_extras.shadow.offset_y,
        "Shadow offset Y: {}px"
    );

    // Shadow draw behind
    register_bool_callback!(
        ui,
        on_shadow_draw_behind_toggled,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.shadow.draw_behind_window,
        "Shadow draw behind"
    );

    // Shadow color
    register_color_callback!(
        ui,
        on_shadow_color_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.shadow.color,
        "Shadow color changed"
    );

    // Shadow inactive color
    register_color_callback!(
        ui,
        on_shadow_inactive_color_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.shadow.inactive_color,
        "Shadow inactive color changed"
    );
}

/// Tab indicator settings callbacks
fn setup_tab_indicator_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Tab indicator enabled
    register_bool_callback!(
        ui,
        on_tab_indicator_toggled,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.enabled,
        "Tab indicator enabled"
    );

    // Hide when single tab
    register_bool_callback!(
        ui,
        on_tab_indicator_hide_single_toggled,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.hide_when_single_tab,
        "Tab indicator hide single"
    );

    // Place within column
    register_bool_callback!(
        ui,
        on_tab_indicator_within_column_toggled,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.place_within_column,
        "Tab indicator within column"
    );

    // Gap
    register_clamped_callback!(
        ui,
        on_tab_indicator_gap_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        0,
        20,
        |s| s.layout_extras.tab_indicator.gap,
        "Tab indicator gap: {}px"
    );

    // Width
    register_clamped_callback!(
        ui,
        on_tab_indicator_width_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        1,
        20,
        |s| s.layout_extras.tab_indicator.width,
        "Tab indicator width: {}px"
    );

    // Position (index-based)
    register_enum_callback!(
        ui,
        on_tab_indicator_position_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.position,
        crate::config::models::TabIndicatorPosition,
        "Tab indicator position"
    );

    // Corner radius
    register_clamped_callback!(
        ui,
        on_tab_indicator_corner_radius_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        0,
        20,
        |s| s.layout_extras.tab_indicator.corner_radius,
        "Tab indicator corner radius: {}px"
    );

    // Gaps between tabs
    register_clamped_callback!(
        ui,
        on_tab_indicator_gaps_between_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        0,
        20,
        |s| s.layout_extras.tab_indicator.gaps_between_tabs,
        "Tab indicator gaps between tabs: {}px"
    );

    // Active color
    register_color_or_gradient_callback!(
        ui,
        on_tab_indicator_active_color_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.active,
        "Tab indicator active color changed"
    );

    // Inactive color
    register_color_or_gradient_callback!(
        ui,
        on_tab_indicator_inactive_color_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.inactive,
        "Tab indicator inactive color changed"
    );

    // Urgent color
    register_color_or_gradient_callback!(
        ui,
        on_tab_indicator_urgent_color_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.tab_indicator.urgent,
        "Tab indicator urgent color changed"
    );
}

/// Insert hint settings callbacks
fn setup_insert_hint_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Insert hint enabled
    register_bool_callback!(
        ui,
        on_insert_hint_toggled,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.insert_hint.enabled,
        "Insert hint enabled"
    );

    // Insert hint color
    register_color_or_gradient_callback!(
        ui,
        on_insert_hint_color_changed,
        settings,
        save_manager,
        SettingsCategory::LayoutExtras,
        |s| s.layout_extras.insert_hint.color,
        "Insert hint color changed"
    );
}
