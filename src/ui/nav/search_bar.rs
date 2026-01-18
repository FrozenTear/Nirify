//! Search bar with action button

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Button, Label, Stack};

use crate::ui::theme::{
    search_bar_style, BORDER_RADIUS_SM, OVERLAY0, SPACING_MD, SPACING_SM, SUBTEXT1, SURFACE0,
    SURFACE1, TEXT,
};

/// Create the search bar with "Review Changes" button
pub fn search_bar(search_query: RwSignal<String>) -> impl IntoView {
    Stack::horizontal((
        // Search icon placeholder
        Label::derived(|| "🔍".to_string()).style(|s| s.color(OVERLAY0)),
        // Search input
        text_input(search_query)
            .placeholder("Search settings...")
            .style(|s| {
                s.flex_grow(1.0)
                    .background(SURFACE0)
                    .border_radius(BORDER_RADIUS_SM)
                    .padding(SPACING_SM)
                    .color(TEXT)
            }),
        // Review Changes button
        Button::new("Review Changes").style(|s| {
            s.padding_horiz(SPACING_MD)
                .padding_vert(SPACING_SM)
                .border_radius(BORDER_RADIUS_SM)
                .background(SURFACE1)
                .color(SUBTEXT1)
        }),
    ))
    .style(search_bar_style)
}
