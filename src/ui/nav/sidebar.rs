//! Secondary navigation showing subcategories
//!
//! Horizontal row of subcategory tabs below the main header

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Button, Label, Stack};

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{
    secondary_nav_style, ACCENT, BG_DEEP, SPACING_MD, SPACING_SM, TEXT_PRIMARY, TEXT_SECONDARY,
};

/// Create the secondary nav showing subcategories for the current group
pub fn sidebar(nav_group: RwSignal<NavGroup>, category: RwSignal<Category>) -> impl IntoView {
    dyn_stack(
        move || nav_group.get().categories().to_vec(),
        |cat| *cat,
        move |cat| {
            let is_selected = move || category.get() == cat;

            Button::new(cat.label())
                .style(move |s| {
                    let base = s
                        .padding_horiz(SPACING_MD)
                        .padding_vert(SPACING_SM)
                        .border_radius(4.0);

                    if is_selected() {
                        base.color(ACCENT)
                    } else {
                        base.color(TEXT_SECONDARY)
                    }
                })
                .action(move || {
                    category.set(cat);
                })
        },
    )
    .style(secondary_nav_style)
}
