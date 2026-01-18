//! Header navigation component with tab groups

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Button, Label, Stack};

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{
    ACCENT, BG_BASE, BG_DEEP, BG_ELEVATED, BORDER, SPACING_LG, SPACING_MD, SPACING_SM,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

/// Create the header navigation bar
pub fn header(
    nav_group: RwSignal<NavGroup>,
    category: RwSignal<Category>,
) -> impl IntoView {
    Stack::horizontal((
        // App title
        Label::derived(|| "niri settings".to_string())
            .style(|s| s.font_size(18.0).font_bold().color(TEXT_PRIMARY)),
        // Nav group tabs
        Stack::horizontal(
            NavGroup::all()
                .iter()
                .map(|group| {
                    let group = *group;
                    let is_selected = move || nav_group.get() == group;

                    Button::new(group.label())
                        .style(move |s| {
                            let base = s
                                .padding_horiz(SPACING_MD)
                                .padding_vert(SPACING_SM)
                                .border_radius(6.0)
                                .margin_right(SPACING_SM);

                            if is_selected() {
                                base.background(ACCENT).color(BG_DEEP)
                            } else {
                                base.background(BG_ELEVATED).color(TEXT_SECONDARY)
                            }
                        })
                        .action(move || {
                            nav_group.set(group);
                            // Set category to first in group
                            if let Some(first_cat) = group.categories().first() {
                                category.set(*first_cat);
                            }
                        })
                })
                .collect::<Vec<_>>(),
        )
        .style(|s| s.margin_left(SPACING_LG)),
    ))
    .style(|s| {
        s.width_full()
            .height(56.0)
            .padding_horiz(SPACING_LG)
            .items_center()
            .background(BG_DEEP)
            .border_bottom(1.0)
            .border_color(BORDER)
    })
}
