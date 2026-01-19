//! Overview settings page

use freya::prelude::*;

use crate::config::SettingsCategory;
use crate::types::Color;
use crate::ui::app::ReactiveState;
use crate::ui::components::{section, slider_row, text_row};
use crate::ui::theme::SPACING_LG;

/// Create the overview settings page
pub fn overview_page(state: ReactiveState) -> impl IntoElement {
    let settings = state.get_settings();
    let overview = &settings.overview;

    let backdrop_hex = overview
        .backdrop_color
        .map(|c| c.to_hex())
        .unwrap_or_default();

    let state1 = state.clone();
    let mut refresh1 = state.refresh.clone();
    let state2 = state.clone();
    let mut refresh2 = state.refresh.clone();

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
                        state1.update_and_save(SettingsCategory::Overview, |s| {
                            s.overview.zoom = val
                        });
                        refresh1.with_mut(|mut v| *v += 1);
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
                        state2.update_and_save(SettingsCategory::Overview, |s| {
                            if val.is_empty() {
                                s.overview.backdrop_color = None;
                            } else if let Some(color) = Color::from_hex(&val) {
                                s.overview.backdrop_color = Some(color);
                            }
                        });
                        refresh2.with_mut(|mut v| *v += 1);
                    },
                )),
        ))
}
