//! Secondary navigation showing subcategories
//!
//! Horizontal row of pill-style tabs for navigating within a group

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Label;

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{secondary_nav_style, secondary_tab_selected_style, secondary_tab_style};

/// Create the secondary nav showing subcategories for the current group
pub fn sidebar(nav_group: RwSignal<NavGroup>, category: RwSignal<Category>) -> impl IntoView {
    dyn_stack(
        move || nav_group.get().categories().to_vec(),
        |cat| *cat,
        move |cat| {
            let is_selected = move || category.get() == cat;

            Label::derived(move || cat.label().to_string())
                .style(move |s| {
                    if is_selected() {
                        secondary_tab_selected_style(s)
                    } else {
                        secondary_tab_style(s)
                    }
                })
                .on_click_stop(move |_| {
                    category.set(cat);
                })
        },
    )
    .style(secondary_nav_style)
}
