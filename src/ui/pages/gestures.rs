//! Gestures and hot corners settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the gestures settings page
pub fn gestures_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let gestures = settings.gestures;

    let state_hot_corners = state.clone();
    let state_scroll_en = state.clone();
    let state_scroll_size = state.clone();
    let state_scroll_delay = state.clone();
    let state_ws_en = state.clone();
    let state_ws_size = state.clone();
    let state_ws_delay = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // Hot Corners section
        .child(section(
            "Hot Corners",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable hot corners",
                    "Trigger actions when cursor reaches corners",
                    gestures.hot_corners.enabled,
                    move |val| {
                        state_hot_corners.update_settings(|s| s.gestures.hot_corners.enabled = val);
                        state_hot_corners.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                )),
        ))
        // Edge Scrolling section
        .child(section(
            "Edge Scrolling (Drag & Drop)",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable edge scroll",
                    "Scroll view when dragging to screen edge",
                    gestures.dnd_edge_view_scroll.enabled,
                    move |val| {
                        state_scroll_en
                            .update_settings(|s| s.gestures.dnd_edge_view_scroll.enabled = val);
                        state_scroll_en.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                ))
                .child(slider_row(
                    "Trigger zone size",
                    "Edge zone width in pixels",
                    gestures.dnd_edge_view_scroll.trigger_size as f64,
                    1.0,
                    100.0,
                    "px",
                    move |val| {
                        state_scroll_size.update_settings(|s| {
                            s.gestures.dnd_edge_view_scroll.trigger_size = val as i32
                        });
                        state_scroll_size.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                ))
                .child(slider_row(
                    "Trigger delay",
                    "Delay before scrolling starts",
                    gestures.dnd_edge_view_scroll.delay_ms as f64,
                    0.0,
                    1000.0,
                    "ms",
                    move |val| {
                        state_scroll_delay
                            .update_settings(|s| s.gestures.dnd_edge_view_scroll.delay_ms = val as i32);
                        state_scroll_delay.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                )),
        ))
        // Workspace Switch section
        .child(section(
            "Workspace Switch (Drag & Drop)",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(toggle_row(
                    "Enable edge workspace switch",
                    "Switch workspace when dragging to edge",
                    gestures.dnd_edge_workspace_switch.enabled,
                    move |val| {
                        state_ws_en
                            .update_settings(|s| s.gestures.dnd_edge_workspace_switch.enabled = val);
                        state_ws_en.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                ))
                .child(slider_row(
                    "Trigger zone size",
                    "Edge zone width in pixels",
                    gestures.dnd_edge_workspace_switch.trigger_size as f64,
                    1.0,
                    100.0,
                    "px",
                    move |val| {
                        state_ws_size.update_settings(|s| {
                            s.gestures.dnd_edge_workspace_switch.trigger_size = val as i32
                        });
                        state_ws_size.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                ))
                .child(slider_row(
                    "Trigger delay",
                    "Delay before switching",
                    gestures.dnd_edge_workspace_switch.delay_ms as f64,
                    0.0,
                    1000.0,
                    "ms",
                    move |val| {
                        state_ws_delay.update_settings(|s| {
                            s.gestures.dnd_edge_workspace_switch.delay_ms = val as i32
                        });
                        state_ws_delay.mark_dirty_and_save(SettingsCategory::Gestures);
                    },
                )),
        ))
}
