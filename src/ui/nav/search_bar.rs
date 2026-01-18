//! Search bar component with refined styling
//!
//! Features a search icon, text input, and action button

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{text_input, Label, Stack};

use crate::ui::theme::{button_secondary_style, search_bar_style, search_input_style, TEXT_MUTED};

/// Create the search bar with "Review Changes" button
pub fn search_bar(search_query: RwSignal<String>) -> impl IntoView {
    Stack::horizontal((
        // Search icon
        Label::derived(|| "⌕".to_string()).style(|s| s.color(TEXT_MUTED).font_size(16.0)),
        // Search input field
        text_input(search_query)
            .placeholder("Search settings...")
            .style(search_input_style),
        // Review Changes button
        Label::derived(|| "Review Changes".to_string())
            .style(button_secondary_style)
            .on_click_stop(|_| {
                // TODO: Open review changes dialog
            }),
    ))
    .style(search_bar_style)
}
