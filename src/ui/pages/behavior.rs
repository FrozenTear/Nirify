//! Behavior settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the behavior settings page
pub fn behavior_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let behavior = settings.behavior;

    let state_focus = state.clone();
    let state_back_forth = state.clone();
    let state_center = state.clone();
    let state_empty_ws = state.clone();
    let state_col_width = state.clone();
    let state_strut_l = state.clone();
    let state_strut_r = state.clone();
    let state_strut_t = state.clone();
    let state_strut_b = state.clone();
    let state_power = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Focus section
        .child(section(
            "Focus",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Focus follows mouse",
                    "Automatically focus window under cursor",
                    behavior.focus_follows_mouse,
                    move |val| {
                        state_focus.update_settings(|s| s.behavior.focus_follows_mouse = val);
                        state_focus.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                ))
                .child(toggle_row(
                    "Workspace back and forth",
                    "Switch to previous workspace with same key",
                    behavior.workspace_auto_back_and_forth,
                    move |val| {
                        state_back_forth
                            .update_settings(|s| s.behavior.workspace_auto_back_and_forth = val);
                        state_back_forth.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                )),
        ))
        // Layout section
        .child(section(
            "Layout",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Center single column",
                    "Always center when only one column exists",
                    behavior.always_center_single_column,
                    move |val| {
                        state_center
                            .update_settings(|s| s.behavior.always_center_single_column = val);
                        state_center.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                ))
                .child(toggle_row(
                    "Empty workspace above first",
                    "Add empty workspace above the first one",
                    behavior.empty_workspace_above_first,
                    move |val| {
                        state_empty_ws
                            .update_settings(|s| s.behavior.empty_workspace_above_first = val);
                        state_empty_ws.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                ))
                .child(slider_row(
                    "Default column width",
                    "Width proportion for new columns (0.5 = half)",
                    behavior.default_column_width_proportion as f64,
                    0.1,
                    2.0,
                    "",
                    move |val| {
                        state_col_width
                            .update_settings(|s| s.behavior.default_column_width_proportion = val as f32);
                        state_col_width.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                )),
        ))
        // Screen Margins section
        .child(section(
            "Screen Margins (Struts)",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Left margin",
                    "Reserved space on left edge",
                    behavior.strut_left as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state_strut_l.update_settings(|s| s.behavior.strut_left = val as f32);
                        state_strut_l.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                ))
                .child(slider_row(
                    "Right margin",
                    "Reserved space on right edge",
                    behavior.strut_right as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state_strut_r.update_settings(|s| s.behavior.strut_right = val as f32);
                        state_strut_r.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                ))
                .child(slider_row(
                    "Top margin",
                    "Reserved space on top edge",
                    behavior.strut_top as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state_strut_t.update_settings(|s| s.behavior.strut_top = val as f32);
                        state_strut_t.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                ))
                .child(slider_row(
                    "Bottom margin",
                    "Reserved space on bottom edge",
                    behavior.strut_bottom as f64,
                    0.0,
                    500.0,
                    "px",
                    move |val| {
                        state_strut_b.update_settings(|s| s.behavior.strut_bottom = val as f32);
                        state_strut_b.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                )),
        ))
        // System section
        .child(section(
            "System",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Disable power key handling",
                    "Let the system handle the power button",
                    behavior.disable_power_key_handling,
                    move |val| {
                        state_power
                            .update_settings(|s| s.behavior.disable_power_key_handling = val);
                        state_power.mark_dirty_and_save(SettingsCategory::Behavior);
                    },
                )),
        ))
}
