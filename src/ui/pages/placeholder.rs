//! Placeholder page for unimplemented categories

use floem::prelude::*;
use floem::views::{Label, Stack};

use crate::ui::state::Category;
use crate::ui::theme::{SPACING_LG, TEXT_PRIMARY, TEXT_SECONDARY};

/// Create a placeholder page for categories not yet implemented
pub fn placeholder_page(category: Category) -> impl IntoView {
    Stack::vertical((
        Label::derived(move || category.label().to_string())
            .style(|s| s.font_size(24.0).font_bold().color(TEXT_PRIMARY)),
        Label::derived(|| "This page is not yet implemented.".to_string()).style(|s| {
            s.font_size(14.0)
                .color(TEXT_SECONDARY)
                .margin_top(SPACING_LG)
        }),
    ))
    .style(|s| s.width_full().height_full().items_center().justify_center())
}
