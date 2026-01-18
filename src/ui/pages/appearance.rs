//! Appearance settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row, value_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the appearance settings page
pub fn appearance_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let appearance = settings.appearance;

    // Clone state for callbacks
    let state_focus_ring = state.clone();
    let state_focus_width = state.clone();
    let state_border = state.clone();
    let state_border_thickness = state.clone();
    let state_gaps = state.clone();
    let state_corner_radius = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Focus Ring section
        .child(section(
            "Focus Ring",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable focus ring",
                    "Show a colored ring around the focused window",
                    appearance.focus_ring_enabled,
                    move |val| {
                        state_focus_ring.update_settings(|s| s.appearance.focus_ring_enabled = val);
                        state_focus_ring.mark_dirty_and_save(SettingsCategory::Appearance);
                    },
                ))
                .child(slider_row(
                    "Ring width",
                    "Thickness of the focus ring in pixels",
                    appearance.focus_ring_width as f64,
                    1.0,
                    20.0,
                    "px",
                    move |val| {
                        state_focus_width.update_settings(|s| s.appearance.focus_ring_width = val as f32);
                        state_focus_width.mark_dirty_and_save(SettingsCategory::Appearance);
                    },
                ))
                .child(value_row(
                    "Active color",
                    "Color when window is focused",
                    appearance.focus_ring_active.to_hex(),
                ))
                .child(value_row(
                    "Inactive color",
                    "Color when window is not focused",
                    appearance.focus_ring_inactive.to_hex(),
                )),
        ))
        // Window Border section
        .child(section(
            "Window Border",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable window border",
                    "Show a border around windows (inside the focus ring)",
                    appearance.border_enabled,
                    move |val| {
                        state_border.update_settings(|s| s.appearance.border_enabled = val);
                        state_border.mark_dirty_and_save(SettingsCategory::Appearance);
                    },
                ))
                .child(slider_row(
                    "Border thickness",
                    "Thickness of the window border in pixels",
                    appearance.border_thickness as f64,
                    1.0,
                    15.0,
                    "px",
                    move |val| {
                        state_border_thickness.update_settings(|s| s.appearance.border_thickness = val as f32);
                        state_border_thickness.mark_dirty_and_save(SettingsCategory::Appearance);
                    },
                )),
        ))
        // Gaps section
        .child(section(
            "Gaps",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Window gaps",
                    "Space between windows",
                    appearance.gaps as f64,
                    0.0,
                    64.0,
                    "px",
                    move |val| {
                        state_gaps.update_settings(|s| s.appearance.gaps = val as f32);
                        state_gaps.mark_dirty_and_save(SettingsCategory::Appearance);
                    },
                )),
        ))
        // Corners section
        .child(section(
            "Corners",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Corner radius",
                    "Window corner rounding",
                    appearance.corner_radius as f64,
                    0.0,
                    40.0,
                    "px",
                    move |val| {
                        state_corner_radius.update_settings(|s| s.appearance.corner_radius = val as f32);
                        state_corner_radius.mark_dirty_and_save(SettingsCategory::Appearance);
                    },
                )),
        ))
}
