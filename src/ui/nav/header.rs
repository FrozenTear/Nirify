//! Header navigation component
//!
//! Clean, minimal header with app title and primary navigation tabs
//! Uses understated underline indicators for selected state

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Container, Empty, Label, Stack};

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{
    header_style, nav_tab_selected_style, nav_tab_style, theme, FONT_SIZE_BASE, SPACING_LG,
    SPACING_MD, SPACING_XL,
};

/// Create the header with app title and primary navigation tabs
pub fn header(nav_group: RwSignal<NavGroup>, category: RwSignal<Category>) -> impl IntoView {
    Stack::horizontal((
        // App title - refined, left-aligned
        Label::derived(|| "niri settings".to_string()).style(move |s| {
            s.font_size(FONT_SIZE_BASE)
                .color(theme().text_muted)
                .margin_right(SPACING_XL)
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
        .style(|s| s.gap(SPACING_MD).items_center()),
    ))
    .style(header_style)
}

/// Individual navigation tab with subtle underline indicator
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
        // Underline indicator - only visible when selected
        Container::new(Empty::new()).style(move |s| {
            let t = theme();
            let base = s
                .width_full()
                .height(2.0)
                .margin_top(SPACING_MD)
                .border_radius(1.0);

            if is_selected() {
                base.background(t.accent)
            } else {
                base.background(floem::peniko::Color::TRANSPARENT)
            }
        }),
    ))
    .style(|s| s.padding_horiz(SPACING_LG).items_center().cursor(floem::style::CursorStyle::Pointer))
    .on_click_stop(move |_| {
        nav_group.set(group);
        // Set category to first in group
        if let Some(first_cat) = group.categories().first() {
            category.set(*first_cat);
        }
    })
}
