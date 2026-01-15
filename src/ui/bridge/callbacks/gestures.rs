//! Gestures UI callbacks
//!
//! Handles hot corners and drag-and-drop gesture settings.

use crate::config::{Settings, SettingsCategory};
use crate::MainWindow;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::macros::{register_bool_callback, register_clamped_callback, SaveManager};

/// Set up gestures callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    setup_hot_corners_callbacks(ui, settings.clone(), Rc::clone(&save_manager));
    setup_dnd_edge_scroll_callbacks(ui, settings.clone(), Rc::clone(&save_manager));
    setup_dnd_workspace_switch_callbacks(ui, settings, save_manager);
}

/// Hot corners callbacks
fn setup_hot_corners_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Hot corners enabled
    register_bool_callback!(
        ui,
        on_hot_corners_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.hot_corners.enabled,
        "Hot corners enabled"
    );

    // Top left corner
    register_bool_callback!(
        ui,
        on_hot_corner_top_left_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.hot_corners.top_left,
        "Hot corner top left"
    );

    // Top right corner
    register_bool_callback!(
        ui,
        on_hot_corner_top_right_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.hot_corners.top_right,
        "Hot corner top right"
    );

    // Bottom left corner
    register_bool_callback!(
        ui,
        on_hot_corner_bottom_left_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.hot_corners.bottom_left,
        "Hot corner bottom left"
    );

    // Bottom right corner
    register_bool_callback!(
        ui,
        on_hot_corner_bottom_right_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.hot_corners.bottom_right,
        "Hot corner bottom right"
    );
}

/// DND edge scroll callbacks
fn setup_dnd_edge_scroll_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Edge scroll enabled
    register_bool_callback!(
        ui,
        on_dnd_edge_scroll_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.dnd_edge_view_scroll.enabled,
        "DND edge scroll enabled"
    );

    // Trigger width (uses trigger_size internally)
    register_clamped_callback!(
        ui,
        on_dnd_edge_scroll_trigger_width_changed,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        10,
        100,
        |s| s.gestures.dnd_edge_view_scroll.trigger_size,
        "DND edge scroll trigger width: {}px"
    );

    // Delay
    register_clamped_callback!(
        ui,
        on_dnd_edge_scroll_delay_changed,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        0,
        500,
        |s| s.gestures.dnd_edge_view_scroll.delay_ms,
        "DND edge scroll delay: {}ms"
    );

    // Max speed
    register_clamped_callback!(
        ui,
        on_dnd_edge_scroll_max_speed_changed,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        100,
        5000,
        |s| s.gestures.dnd_edge_view_scroll.max_speed,
        "DND edge scroll max speed: {}px/s"
    );
}

/// DND workspace switch callbacks
fn setup_dnd_workspace_switch_callbacks(
    ui: &MainWindow,
    settings: Arc<Mutex<Settings>>,
    save_manager: Rc<SaveManager>,
) {
    // Workspace switch enabled
    register_bool_callback!(
        ui,
        on_dnd_workspace_switch_toggled,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        |s| s.gestures.dnd_edge_workspace_switch.enabled,
        "DND workspace switch enabled"
    );

    // Trigger height (uses trigger_size internally)
    register_clamped_callback!(
        ui,
        on_dnd_workspace_switch_trigger_height_changed,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        20,
        150,
        |s| s.gestures.dnd_edge_workspace_switch.trigger_size,
        "DND workspace switch trigger height: {}px"
    );

    // Delay
    register_clamped_callback!(
        ui,
        on_dnd_workspace_switch_delay_changed,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        0,
        500,
        |s| s.gestures.dnd_edge_workspace_switch.delay_ms,
        "DND workspace switch delay: {}ms"
    );

    // Max speed
    register_clamped_callback!(
        ui,
        on_dnd_workspace_switch_max_speed_changed,
        settings,
        save_manager,
        SettingsCategory::Gestures,
        100,
        5000,
        |s| s.gestures.dnd_edge_workspace_switch.max_speed,
        "DND workspace switch max speed: {}px/s"
    );
}
