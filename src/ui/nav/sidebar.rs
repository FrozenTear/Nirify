//! Sidebar navigation showing categories within the current group

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Button, Label, Stack};

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{
    sidebar_style, ACCENT, BG_DEEP, BG_ELEVATED, BORDER, SPACING_MD, SPACING_SM, TEXT_PRIMARY,
    TEXT_SECONDARY,
};

/// Create the sidebar showing categories for the current nav group
pub fn sidebar(nav_group: RwSignal<NavGroup>, category: RwSignal<Category>) -> impl IntoView {
    Stack::vertical((
        // Group title
        Label::derived(move || nav_group.get().label().to_string())
            .style(|s| {
                s.font_size(14.0)
                    .font_bold()
                    .color(TEXT_SECONDARY)
                    .margin_bottom(SPACING_MD)
            }),
        // Category list (dynamic based on nav_group)
        dyn_stack(
            move || nav_group.get().categories().to_vec(),
            |cat| *cat,
            move |cat| {
                let is_selected = move || category.get() == cat;

                Button::new(cat.label())
                    .style(move |s| {
                        let base = s
                            .width_full()
                            .padding_horiz(SPACING_MD)
                            .padding_vert(SPACING_SM)
                            .border_radius(6.0)
                            .margin_bottom(SPACING_SM / 2.0)
                            .justify_start();

                        if is_selected() {
                            base.background(ACCENT).color(BG_DEEP)
                        } else {
                            base.background(BG_ELEVATED).color(TEXT_PRIMARY)
                        }
                    })
                    .action(move || {
                        category.set(cat);
                    })
            },
        )
        .style(|s| s.flex_col().width_full()),
    ))
    .style(sidebar_style)
}
