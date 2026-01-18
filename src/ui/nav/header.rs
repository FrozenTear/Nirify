//! Header navigation component
//!
//! Crystalline Dark header with app title and primary navigation tabs
//! Uses underline indicators for selected state

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Container, Empty, Label, Stack};

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{
    header_style, nav_tab_selected_style, nav_tab_style, ACCENT, BORDER_SUBTLE, FONT_SIZE_SM,
    SPACING_LG, SPACING_MD, SPACING_SM, TEXT_GHOST,
};

/// Create the header with app title and primary navigation tabs
pub fn header(nav_group: RwSignal<NavGroup>, category: RwSignal<Category>) -> impl IntoView {
    Stack::vertical((
        // App title (subtle, centered)
        Label::derived(|| "n i r i   s e t t i n g s".to_string()).style(|s| {
            s.font_size(FONT_SIZE_SM)
                .color(TEXT_GHOST)
                .padding_bottom(SPACING_LG)
        }),
        // Primary navigation tabs row
        Stack::horizontal(
            NavGroup::all()
                .iter()
                .map(|group| {
                    let group = *group;
                    nav_tab(group, nav_group, category)
                })
                .collect::<Vec<_>>(),
        )
        .style(|s| s.gap(SPACING_SM).items_end()),
    ))
    .style(header_style)
}

/// Individual navigation tab with underline indicator
fn nav_tab(
    group: NavGroup,
    nav_group: RwSignal<NavGroup>,
    category: RwSignal<Category>,
) -> impl IntoView {
    let is_selected = move || nav_group.get() == group;

    Stack::vertical((
        // Tab label
        Label::derived(move || group.label().to_string()).style(move |s| {
            if is_selected() {
                nav_tab_selected_style(s)
            } else {
                nav_tab_style(s)
            }
        }),
        // Underline indicator
        Container::new(Empty::new()).style(move |s| {
            let base = s
                .width_full()
                .height(2.0)
                .margin_top(SPACING_SM)
                .border_radius(1.0);

            if is_selected() {
                base.background(ACCENT)
            } else {
                base.background(BORDER_SUBTLE)
            }
        }),
    ))
    .style(|s| s.padding_horiz(SPACING_MD).items_center())
    .on_click_stop(move |_| {
        nav_group.set(group);
        // Set category to first in group
        if let Some(first_cat) = group.categories().first() {
            category.set(*first_cat);
        }
    })
}
