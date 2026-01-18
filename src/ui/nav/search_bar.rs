//! Search bar with action button

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Button, Label, Stack};

use crate::ui::theme::{
    search_bar_style, BG_ELEVATED, BORDER, BORDER_RADIUS_SM, SPACING_MD, SPACING_SM, TEXT_MUTED,
    TEXT_PRIMARY, TEXT_SECONDARY, ACCENT,
};

/// Create the search bar with "Review Changes" button
pub fn search_bar(search_query: RwSignal<String>) -> impl IntoView {
    Stack::horizontal((
        // Search icon placeholder
        Label::derived(|| "🔍".to_string()).style(|s| s.color(TEXT_MUTED)),
        // Search input
        text_input(search_query)
            .placeholder("Search settings...")
            .style(|s| {
                s.flex_grow(1.0)
                    .background(BG_ELEVATED)
                    .border_radius(BORDER_RADIUS_SM)
                    .padding(SPACING_SM)
                    .color(TEXT_PRIMARY)
                    .border(1.0)
                    .border_color(BORDER)
            }),
        // Review Changes button
        Button::new("Review Changes").style(|s| {
            s.padding_horiz(SPACING_MD)
                .padding_vert(SPACING_SM)
                .border_radius(BORDER_RADIUS_SM)
                .background(BG_ELEVATED)
                .color(TEXT_SECONDARY)
                .border(1.0)
                .border_color(BORDER)
        }),
    ))
    .style(search_bar_style)
}
