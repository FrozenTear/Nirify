//! Gestures and hot corners settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, toggle_row};
use crate::ui::theme::SPACING_LG;

/// Create the gestures settings page
pub fn gestures_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let gestures = &settings.gestures;

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();
    let state3 = state.clone();
    let mut refresh3 = state.refresh.clone();
    let state4 = state.clone();
    let mut refresh4 = state.refresh.clone();
    let state5 = state.clone();
    let mut refresh5 = state.refresh.clone();
    let state6 = state.clone();
    let mut refresh6 = state.refresh.clone();
    let state7 = state.clone();
    let mut refresh7 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.hot_corners.enabled = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
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
                        state2.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.dnd_edge_view_scroll.enabled = val
                        });
                        refresh2.with_mut(|mut v| *v += 1);
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
                        state3.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.dnd_edge_view_scroll.trigger_size = val as i32
                        });
                        refresh3.with_mut(|mut v| *v += 1);
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
                        state4.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.dnd_edge_view_scroll.delay_ms = val as i32
                        });
                        refresh4.with_mut(|mut v| *v += 1);
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
                        state5.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.dnd_edge_workspace_switch.enabled = val
                        });
                        refresh5.with_mut(|mut v| *v += 1);
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
                        state6.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.dnd_edge_workspace_switch.trigger_size = val as i32
                        });
                        refresh6.with_mut(|mut v| *v += 1);
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
                        state7.update_and_save(SettingsCategory::Gestures, |s| {
                            s.gestures.dnd_edge_workspace_switch.delay_ms = val as i32
                        });
                        refresh7.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
