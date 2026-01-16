//! Appearance-related UI callbacks
//!
//! Handles focus ring, border, gaps, and corner radius settings.

use crate::config::category_section::Appearance;
use crate::config::SettingsCategory;
use crate::constants::*;
use crate::MainWindow;
use log::{debug, error};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::converters::slint_color_to_color;
use super::super::macros::{
    register_bool_callbacks, register_clamped_callbacks, register_color_or_gradient_callbacks,
    SaveManager,
};
use crate::config::Settings;

/// Set up appearance-related callbacks
pub fn setup(ui: &MainWindow, settings: Arc<Mutex<Settings>>, save_manager: Rc<SaveManager>) {
    // Boolean toggles
    register_bool_callbacks!(
        ui,
        settings,
        save_manager,
        Appearance,
        [
            (
                on_focus_ring_toggled,
                focus_ring_enabled,
                "Focus ring enabled"
            ),
            (on_border_toggled, border_enabled, "Border enabled"),
        ]
    );

    // Clamped numeric values (sliders)
    register_clamped_callbacks!(
        ui,
        settings,
        save_manager,
        Appearance,
        [
            (
                on_focus_ring_width_changed,
                focus_ring_width,
                FOCUS_RING_WIDTH_MIN,
                FOCUS_RING_WIDTH_MAX,
                "Focus ring width: {}px"
            ),
            (
                on_border_thickness_changed,
                border_thickness,
                BORDER_THICKNESS_MIN,
                BORDER_THICKNESS_MAX,
                "Border thickness: {}px"
            ),
            (
                on_gaps_inner_changed,
                gaps_inner,
                GAP_SIZE_MIN,
                GAP_SIZE_MAX,
                "Inner gaps: {}px"
            ),
            (
                on_gaps_outer_changed,
                gaps_outer,
                GAP_SIZE_MIN,
                GAP_SIZE_MAX,
                "Outer gaps: {}px"
            ),
            (
                on_corner_radius_changed,
                corner_radius,
                CORNER_RADIUS_MIN,
                CORNER_RADIUS_MAX,
                "Corner radius: {}px"
            ),
        ]
    );

    // Color or gradient values
    register_color_or_gradient_callbacks!(
        ui,
        settings,
        save_manager,
        Appearance,
        [
            (
                on_focus_ring_active_color_changed,
                focus_ring_active,
                "Focus ring active color changed"
            ),
            (
                on_focus_ring_inactive_color_changed,
                focus_ring_inactive,
                "Focus ring inactive color changed"
            ),
            (
                on_border_active_color_changed,
                border_active,
                "Border active color changed"
            ),
            (
                on_border_inactive_color_changed,
                border_inactive,
                "Border inactive color changed"
            ),
            (
                on_focus_ring_urgent_color_changed,
                focus_ring_urgent,
                "Focus ring urgent color changed"
            ),
            (
                on_border_urgent_color_changed,
                border_urgent,
                "Border urgent color changed"
            ),
        ]
    );

    {
        let settings = settings.clone();
        let save_manager = Rc::clone(&save_manager);
        ui.on_background_color_changed(move |color| match settings.lock() {
            Ok(mut s) => {
                s.appearance.background_color = Some(slint_color_to_color(color));
                debug!("Background color changed");
                save_manager.mark_dirty(SettingsCategory::Appearance);
                save_manager.request_save();
            }
            Err(e) => error!("Settings lock error: {}", e),
        });
    }
}
