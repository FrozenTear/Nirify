//! Main application view
//!
//! Layout (from top to bottom):
//! - Header (title + primary nav tabs)
//! - Secondary nav (subcategory tabs)
//! - Search bar
//! - Content area (scrollable)
//! - Footer (status + close button)

use floem::prelude::*;
use floem::views::{dyn_view, Scroll, Stack};

use crate::ui::nav::{footer, header, search_bar, sidebar};
use crate::ui::pages::{appearance_page, placeholder_page};
use crate::ui::state::{AppState, Category};
use crate::ui::theme::{content_style, BG_DEEP};

/// Create the main application view
pub fn app_view(state: AppState) -> impl IntoView {
    let nav_group = state.nav_group;
    let category = state.category;
    let search_query = state.search_query;
    let state_for_content = state.clone();

    Stack::vertical((
        // Header (title + primary nav)
        header(nav_group, category),
        // Secondary nav (subcategory tabs)
        sidebar(nav_group, category),
        // Search bar
        search_bar(search_query),
        // Content area (scrollable)
        Scroll::new(
            dyn_view(move || {
                let cat = category.get();
                let state = state_for_content.clone();

                match cat {
                    Category::Appearance => appearance_page(state).into_any(),
                    _ => placeholder_page(cat).into_any(),
                }
            })
            .style(|s| s.width_full()),
        )
        .style(content_style),
        // Footer
        footer(),
    ))
    .style(|s| s.width_full().height_full().background(BG_DEEP))
}
