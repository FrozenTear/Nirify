//! Gestures and hot corners settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row_with_callback, toggle_row_with_callback};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the gestures settings page
pub fn gestures_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let gestures = settings.gestures;

    // Hot corners
    let hot_corners_enabled = RwSignal::new(gestures.hot_corners.enabled);

    // DnD edge view scroll
    let edge_scroll_enabled = RwSignal::new(gestures.dnd_edge_view_scroll.enabled);
    let edge_scroll_size = RwSignal::new(gestures.dnd_edge_view_scroll.trigger_size as f64);
    let edge_scroll_delay = RwSignal::new(gestures.dnd_edge_view_scroll.delay_ms as f64);

    // DnD edge workspace switch
    let edge_workspace_enabled = RwSignal::new(gestures.dnd_edge_workspace_switch.enabled);
    let edge_workspace_size = RwSignal::new(gestures.dnd_edge_workspace_switch.trigger_size as f64);
    let edge_workspace_delay = RwSignal::new(gestures.dnd_edge_workspace_switch.delay_ms as f64);

    // Callbacks
    let on_hot_corners = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.gestures.hot_corners.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    let on_edge_scroll_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.gestures.dnd_edge_view_scroll.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    let on_edge_scroll_size = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.gestures.dnd_edge_view_scroll.trigger_size = val as i32);
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    let on_edge_scroll_delay = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.gestures.dnd_edge_view_scroll.delay_ms = val as i32);
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    let on_edge_workspace_enabled = {
        let state = state.clone();
        Rc::new(move |val: bool| {
            state.update_settings(|s| s.gestures.dnd_edge_workspace_switch.enabled = val);
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    let on_edge_workspace_size = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| {
                s.gestures.dnd_edge_workspace_switch.trigger_size = val as i32
            });
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    let on_edge_workspace_delay = {
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.gestures.dnd_edge_workspace_switch.delay_ms = val as i32);
            state.mark_dirty_and_save(SettingsCategory::Gestures);
        })
    };

    Stack::vertical((
        section(
            "Hot Corners",
            Stack::vertical((toggle_row_with_callback(
                "Enable hot corners",
                Some("Trigger actions when cursor reaches corners"),
                hot_corners_enabled,
                Some(on_hot_corners),
            ),)),
        ),
        section(
            "Edge Scrolling (Drag & Drop)",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable edge scroll",
                    Some("Scroll view when dragging to screen edge"),
                    edge_scroll_enabled,
                    Some(on_edge_scroll_enabled),
                ),
                slider_row_with_callback(
                    "Trigger zone size",
                    Some("Edge zone width in pixels"),
                    edge_scroll_size,
                    1.0,
                    100.0,
                    5.0,
                    "px",
                    Some(on_edge_scroll_size),
                ),
                slider_row_with_callback(
                    "Trigger delay",
                    Some("Delay before scrolling starts"),
                    edge_scroll_delay,
                    0.0,
                    1000.0,
                    50.0,
                    "ms",
                    Some(on_edge_scroll_delay),
                ),
            )),
        ),
        section(
            "Workspace Switch (Drag & Drop)",
            Stack::vertical((
                toggle_row_with_callback(
                    "Enable edge workspace switch",
                    Some("Switch workspace when dragging to edge"),
                    edge_workspace_enabled,
                    Some(on_edge_workspace_enabled),
                ),
                slider_row_with_callback(
                    "Trigger zone size",
                    Some("Edge zone width in pixels"),
                    edge_workspace_size,
                    1.0,
                    100.0,
                    5.0,
                    "px",
                    Some(on_edge_workspace_size),
                ),
                slider_row_with_callback(
                    "Trigger delay",
                    Some("Delay before switching"),
                    edge_workspace_delay,
                    0.0,
                    1000.0,
                    50.0,
                    "ms",
                    Some(on_edge_workspace_delay),
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
