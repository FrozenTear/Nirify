//! Overview settings page

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Stack;
use std::rc::Rc;

use crate::config::SettingsCategory;
use crate::types::Color;
use crate::ui::components::{color_row_with_callback, section, slider_row_with_callback};
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

    // Callbacks
    let on_zoom = {
        let state = state.clone();
        Rc::new(move |val: f64| {
            state.update_settings(|s| s.overview.zoom = val);
            state.mark_dirty_and_save(SettingsCategory::Overview);
        })
    };

    let on_backdrop_color = {
        Rc::new(move |val: String| {
            if val.is_empty() {
                state.update_settings(|s| s.overview.backdrop_color = None);
            } else if let Some(color) = Color::from_hex(&val) {
                state.update_settings(|s| s.overview.backdrop_color = Some(color));
            }
            state.mark_dirty_and_save(SettingsCategory::Overview);
        })
    };

    Stack::vertical((
        section(
            "View",
            Stack::vertical((slider_row_with_callback(
                "Zoom level",
                Some("Overview zoom (0.1 = zoomed out, 2.0 = zoomed in)"),
                zoom,
                0.1,
                2.0,
                0.1,
                "x",
                Some(on_zoom),
            ),)),
        ),
        section(
            "Appearance",
            Stack::vertical((color_row_with_callback(
                "Backdrop color",
                Some("Background color behind workspace previews"),
                backdrop_color,
                Some(on_backdrop_color),
            ),)),
        ),
    ))
    .style(|s| s.width_full().gap(SPACING_LG))
}
