//! Main application view

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{dyn_view, Stack};

use crate::ui::nav::{header, sidebar};
use crate::ui::pages::{appearance_page, placeholder_page};
use crate::ui::state::{AppState, Category};
use crate::ui::theme::{BG_BASE, BG_DEEP};

/// Create the main application view
pub fn app_view(state: AppState) -> impl IntoView {
    let nav_group = state.nav_group;
    let category = state.category;
    let state_for_content = state.clone();

    Stack::vertical((
        // Header navigation
        header(nav_group, category),
        // Main content area
        Stack::horizontal((
            // Sidebar
            sidebar(nav_group, category),
            // Page content
            dyn_view(move || {
                let cat = category.get();
                let state = state_for_content.clone();

                match cat {
                    Category::Appearance => appearance_page(state).into_any(),
                    // All other categories show placeholder for now
                    _ => placeholder_page(cat).into_any(),
                }
            })
            .style(|s| s.flex_grow(1.0).height_full().background(BG_BASE)),
        ))
        .style(|s| s.flex_grow(1.0)),
    ))
    .style(|s| s.width_full().height_full().background(BG_DEEP))
}
