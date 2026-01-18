//! Overview settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;

use crate::ui::components::{color_row, section, slider_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the overview settings page
pub fn overview_page(state: AppState) -> impl IntoView {
    let settings = state.get_settings();
    let overview = settings.overview;

    let zoom = RwSignal::new(overview.zoom);
    let backdrop_color = RwSignal::new(
        overview
            .backdrop_color
            .map(|c| c.to_hex())
            .unwrap_or_default(),
    );

    Stack::vertical((
        section(
            "View",
            Stack::vertical((slider_row(
                "Zoom level",
                Some("Overview zoom (0.1 = zoomed out, 2.0 = zoomed in)"),
                zoom,
                0.1,
                2.0,
                0.1,
                "x",
            ),)),
        ),
        section(
            "Appearance",
            Stack::vertical((color_row(
                "Backdrop color",
                Some("Background color behind workspace previews"),
                backdrop_color,
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
