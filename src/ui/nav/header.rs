//! Header navigation component
//!
//! Layout: Centered title at top, primary nav tabs below

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::{Button, Label, Stack};

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::{
    nav_tab_selected_style, nav_tab_style, MANTLE, OVERLAY0, SPACING_LG, SPACING_MD, SPACING_SM,
    SURFACE0,
};

/// Create the header with title and primary nav tabs
pub fn header(nav_group: RwSignal<NavGroup>, category: RwSignal<Category>) -> impl IntoView {
    Stack::vertical((
        // App title (centered)
        Label::derived(|| "niri settings".to_string()).style(|s| {
            s.font_size(14.0)
                .color(OVERLAY0)
                .padding_top(SPACING_LG)
                .padding_bottom(SPACING_MD)
        }),
        // Primary nav tabs (centered row)
        Stack::horizontal(
            NavGroup::all()
                .iter()
                .map(|group| {
                    let group = *group;
                    let is_selected = move || nav_group.get() == group;

                    Button::new(group.label())
                        .style(move |s| {
                            if is_selected() {
                                nav_tab_selected_style(s)
                            } else {
                                nav_tab_style(s)
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
        .style(|s| s.gap(SPACING_SM).padding_bottom(SPACING_MD)),
    ))
    .style(|s| {
        s.width_full()
            .items_center()
            .background(MANTLE)
            .border_bottom(1.0)
            .border_color(SURFACE0)
    })
}
