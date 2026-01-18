//! Appearance settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Label, Stack, Scroll};

use crate::ui::components::{section, slider_row, toggle_row, color_row};
use crate::ui::state::AppState;
use crate::ui::theme::{content_style, TEXT_PRIMARY, SPACING_LG};

/// Create the appearance settings page
pub fn appearance_page(state: AppState) -> impl IntoView {
    // Create local signals from settings
    let settings = state.get_settings();
    let appearance = settings.appearance;

    let focus_ring_enabled = RwSignal::new(appearance.focus_ring_enabled);
    let focus_ring_width = RwSignal::new(appearance.focus_ring_width as f64);
    let focus_ring_color = RwSignal::new(appearance.focus_ring_active.to_hex());

    let border_enabled = RwSignal::new(appearance.border_enabled);
    let border_thickness = RwSignal::new(appearance.border_thickness as f64);
    let border_color = RwSignal::new(appearance.border_active.to_hex());

    let gaps = RwSignal::new(appearance.gaps as f64);
    let corner_radius = RwSignal::new(appearance.corner_radius as f64);

    Scroll::new(
        Stack::vertical((
            // Page title
            Label::derived(|| "Appearance".to_string())
                .style(|s| s.font_size(24.0).font_bold().color(TEXT_PRIMARY).margin_bottom(SPACING_LG)),

            // Focus Ring section
            section(
                "Focus Ring",
                Some("Visual indicator around focused windows"),
                Stack::vertical((
                    toggle_row("Enable Focus Ring", None, focus_ring_enabled),
                    slider_row("Width", Some("Ring width in pixels"), focus_ring_width, 1.0, 10.0, 1.0),
                    color_row("Active Color", Some("Color when window is focused"), focus_ring_color),
                )),
            ),

            // Window Border section
            section(
                "Window Border",
                Some("Border around windows"),
                Stack::vertical((
                    toggle_row("Enable Border", None, border_enabled),
                    slider_row("Thickness", Some("Border thickness in pixels"), border_thickness, 1.0, 10.0, 1.0),
                    color_row("Active Color", Some("Color when window is focused"), border_color),
                )),
            ),

            // Spacing section
            section(
                "Spacing",
                Some("Gaps and corner radius"),
                Stack::vertical((
                    slider_row("Gaps", Some("Space between windows"), gaps, 0.0, 32.0, 1.0),
                    slider_row("Corner Radius", Some("Window corner rounding"), corner_radius, 0.0, 20.0, 1.0),
                )),
            ),
        ))
        .style(|s| s.width_full()),
    )
    .style(content_style)
}
