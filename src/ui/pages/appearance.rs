//! Appearance settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Label, Stack};

use crate::ui::components::{color_row, section, slider_row, toggle_row};
use crate::ui::state::AppState;
use crate::ui::theme::{TEXT_PRIMARY, SPACING_LG};

/// Create the appearance settings page
pub fn appearance_page(state: AppState) -> impl IntoView {
    // Create local signals from settings
    let settings = state.get_settings();
    let appearance = settings.appearance;

    let focus_ring_enabled = RwSignal::new(appearance.focus_ring_enabled);
    let focus_ring_width = RwSignal::new(appearance.focus_ring_width as f64);
    let focus_ring_active = RwSignal::new(appearance.focus_ring_active.to_hex());
    let focus_ring_inactive = RwSignal::new(appearance.focus_ring_inactive.to_hex());
    let focus_ring_urgent = RwSignal::new(appearance.focus_ring_urgent.to_hex());

    let border_enabled = RwSignal::new(appearance.border_enabled);
    let border_thickness = RwSignal::new(appearance.border_thickness as f64);

    let gaps = RwSignal::new(appearance.gaps as f64);
    let corner_radius = RwSignal::new(appearance.corner_radius as f64);

    Stack::vertical((
        // Focus Ring section
        section(
            "Focus Ring",
            Stack::vertical((
                toggle_row(
                    "Enable focus ring",
                    Some("Show a colored ring around the focused window"),
                    focus_ring_enabled,
                ),
                slider_row(
                    "Ring width",
                    Some("Thickness of the focus ring in pixels"),
                    focus_ring_width,
                    1.0,
                    10.0,
                    1.0,
                    "px",
                ),
                color_row(
                    "Active color",
                    Some("Color when window is focused"),
                    focus_ring_active,
                ),
                color_row(
                    "Inactive color",
                    Some("Color when window is not focused"),
                    focus_ring_inactive,
                ),
                color_row(
                    "Urgent color",
                    Some("Color when window needs attention"),
                    focus_ring_urgent,
                ),
            )),
        ),
        // Window Border section
        section(
            "Window Border",
            Stack::vertical((toggle_row(
                "Enable window border",
                Some("Show a border around windows (inside the focus ring)"),
                border_enabled,
            ),)),
        ),
        // Gaps section
        section(
            "Gaps",
            Stack::vertical((slider_row(
                "Window gaps",
                Some("Space between windows"),
                gaps,
                0.0,
                32.0,
                1.0,
                "px",
            ),)),
        ),
        // Corners section
        section(
            "Corners",
            Stack::vertical((slider_row(
                "Corner radius",
                Some("Window corner rounding"),
                corner_radius,
                0.0,
                20.0,
                1.0,
                "px",
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
