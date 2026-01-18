//! Gestures and hot corners settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{section, slider_row, toggle_row};
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

    Stack::vertical((
        section(
            "Hot Corners",
            Stack::vertical((toggle_row(
                "Enable hot corners",
                Some("Trigger actions when cursor reaches corners"),
                hot_corners_enabled,
            ),)),
        ),
        section(
            "Edge Scrolling (Drag & Drop)",
            Stack::vertical((
                toggle_row(
                    "Enable edge scroll",
                    Some("Scroll view when dragging to screen edge"),
                    edge_scroll_enabled,
                ),
                slider_row(
                    "Trigger zone size",
                    Some("Edge zone width in pixels"),
                    edge_scroll_size,
                    1.0,
                    100.0,
                    5.0,
                    "px",
                ),
                slider_row(
                    "Trigger delay",
                    Some("Delay before scrolling starts"),
                    edge_scroll_delay,
                    0.0,
                    1000.0,
                    50.0,
                    "ms",
                ),
            )),
        ),
        section(
            "Workspace Switch (Drag & Drop)",
            Stack::vertical((
                toggle_row(
                    "Enable edge workspace switch",
                    Some("Switch workspace when dragging to edge"),
                    edge_workspace_enabled,
                ),
                slider_row(
                    "Trigger zone size",
                    Some("Edge zone width in pixels"),
                    edge_workspace_size,
                    1.0,
                    100.0,
                    5.0,
                    "px",
                ),
                slider_row(
                    "Trigger delay",
                    Some("Delay before switching"),
                    edge_workspace_delay,
                    0.0,
                    1000.0,
                    50.0,
                    "ms",
                ),
            )),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
