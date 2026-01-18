//! Behavior settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the behavior settings page
pub fn behavior_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let behavior = settings.behavior;

    let focus_follows_mouse = RwSignal::new(behavior.focus_follows_mouse);
    let always_center_single = RwSignal::new(behavior.always_center_single_column);
    let empty_workspace_above = RwSignal::new(behavior.empty_workspace_above_first);
    let workspace_back_and_forth = RwSignal::new(behavior.workspace_auto_back_and_forth);
    let disable_power_key = RwSignal::new(behavior.disable_power_key_handling);

    let strut_left = RwSignal::new(behavior.strut_left as f64);
    let strut_right = RwSignal::new(behavior.strut_right as f64);
    let strut_top = RwSignal::new(behavior.strut_top as f64);
    let strut_bottom = RwSignal::new(behavior.strut_bottom as f64);

    let default_column_width = RwSignal::new(behavior.default_column_width_proportion as f64);

    // Callbacks
    let on_focus_follows_mouse = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.behavior.focus_follows_mouse = val);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_workspace_back_and_forth = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.behavior.workspace_auto_back_and_forth = val);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_always_center_single = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.behavior.always_center_single_column = val);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_empty_workspace_above = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.behavior.empty_workspace_above_first = val);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_default_column_width = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.behavior.default_column_width_proportion = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_strut_left = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.behavior.strut_left = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_strut_right = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.behavior.strut_right = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_strut_top = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.behavior.strut_top = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_strut_bottom = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.behavior.strut_bottom = val as f32);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    let on_disable_power_key = {
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.behavior.disable_power_key_handling = val);
            state.mark_dirty_and_save(SettingsCategory::Behavior);
        })
    };

    Stack::vertical((
        section(
            "Focus",
            Stack::vertical((
                toggle_row_with_callback(
                    "Focus follows mouse",
                    Some("Automatically focus window under cursor"),
                    focus_follows_mouse,
                    Some(on_focus_follows_mouse),
                ),
                toggle_row_with_callback(
                    "Workspace back and forth",
                    Some("Switch to previous workspace with same key"),
                    workspace_back_and_forth,
                    Some(on_workspace_back_and_forth),
                ),
            )),
        ),
        section(
            "Layout",
            Stack::vertical((
                toggle_row_with_callback(
                    "Center single column",
                    Some("Always center when only one column exists"),
                    always_center_single,
                    Some(on_always_center_single),
                ),
                toggle_row_with_callback(
                    "Empty workspace above first",
                    Some("Add empty workspace above the first one"),
                    empty_workspace_above,
                    Some(on_empty_workspace_above),
                ),
                slider_row_with_callback(
                    "Default column width",
                    Some("Width proportion for new columns (0.5 = half)"),
                    default_column_width,
                    0.1,
                    2.0,
                    0.1,
                    "",
                    Some(on_default_column_width),
                ),
            )),
        ),
        section(
            "Screen Margins (Struts)",
            Stack::vertical((
                slider_row_with_callback(
                    "Left margin",
                    Some("Reserved space on left edge"),
                    strut_left,
                    0.0,
                    500.0,
                    10.0,
                    "px",
                    Some(on_strut_left),
                ),
                slider_row_with_callback(
                    "Right margin",
                    Some("Reserved space on right edge"),
                    strut_right,
                    0.0,
                    500.0,
                    10.0,
                    "px",
                    Some(on_strut_right),
                ),
                slider_row_with_callback(
                    "Top margin",
                    Some("Reserved space on top edge"),
                    strut_top,
                    0.0,
                    500.0,
                    10.0,
                    "px",
                    Some(on_strut_top),
                ),
                slider_row_with_callback(
                    "Bottom margin",
                    Some("Reserved space on bottom edge"),
                    strut_bottom,
                    0.0,
                    500.0,
                    10.0,
                    "px",
                    Some(on_strut_bottom),
                ),
            )),
        ),
        section(
            "System",
            Stack::vertical((toggle_row_with_callback(
                "Disable power key handling",
                Some("Let the system handle the power button"),
                disable_power_key,
                Some(on_disable_power_key),
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
