//! Overview settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::types::Color;
use crate::ui::components::{section, slider_row, text_row};
use crate::ui::state::AppState;
use crate::ui::theme::SPACING_LG;

/// Create the overview settings page
pub fn overview_page(state: AppState) -> impl IntoElement {
    let settings = state.get_settings();
    let overview = settings.overview;

    let backdrop_hex = overview
        .backdrop_color
        .map(|c| c.to_hex())
        .unwrap_or_default();

    let state_zoom = state.clone();
    let state_backdrop = state.clone();

    rect()
        .width(Size::fill())
        .spacing(SPACING_LG)
        // View section
        .child(section(
            "View",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(slider_row(
                    "Zoom level",
                    "Overview zoom (0.1 = zoomed out, 2.0 = zoomed in)",
                    overview.zoom,
                    0.1,
                    2.0,
                    "x",
                    move |val| {
                        state_zoom.update_settings(|s| s.overview.zoom = val);
                        state_zoom.mark_dirty_and_save(SettingsCategory::Overview);
                    },
                )),
        ))
        // Appearance section
        .child(section(
            "Appearance",
            rect()
                .width(Size::fill())
                .spacing(8.0)
                .child(text_row(
                    "Backdrop color",
                    "Background color (hex, e.g., #000000)",
                    &backdrop_hex,
                    "#00000080",
                    move |val| {
                        if val.is_empty() {
                            state_backdrop.update_settings(|s| s.overview.backdrop_color = None);
                        } else if let Some(color) = Color::from_hex(&val) {
                            state_backdrop
                                .update_settings(|s| s.overview.backdrop_color = Some(color));
                        }
                        state_backdrop.mark_dirty_and_save(SettingsCategory::Overview);
                    },
                )),
        ))
}
