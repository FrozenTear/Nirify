//! Header component with title and primary navigation

use freya::prelude::*;

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::*;

/// Create the header with title and primary navigation tabs
pub fn header(
    current_nav_group: State<NavGroup>,
    current_category: State<Category>,
) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .padding((SPACING_LG, SPACING_2XL, SPACING_LG, SPACING_2XL))
        .background(MANTLE)
        .child(
            // Title
            rect()
                .color(TEXT_PRIMARY)
                .font_size(FONT_SIZE_XL)
                .font_weight(FontWeight::BOLD)
                .child("Niri Settings"),
        )
        .child(
            // Spacer
            rect().width(Size::fill()),
        )
        .child(
            // Navigation tabs - simplified for now
            rect()
                .horizontal()
                .spacing(SPACING_SM)
                .child(nav_button("Appearance", NavGroup::Appearance, current_nav_group, current_category))
                .child(nav_button("Input", NavGroup::Input, current_nav_group, current_category))
                .child(nav_button("Visuals", NavGroup::Visuals, current_nav_group, current_category))
                .child(nav_button("Behavior", NavGroup::Behavior, current_nav_group, current_category))
                .child(nav_button("Rules", NavGroup::Rules, current_nav_group, current_category))
                .child(nav_button("System", NavGroup::System, current_nav_group, current_category)),
        )
}

fn nav_button(
    label: &'static str,
    group: NavGroup,
    mut current_nav_group: State<NavGroup>,
    mut current_category: State<Category>,
) -> Button {
    Button::new()
        .on_press(move |_| {
            *current_nav_group.write() = group;
            if let Some(first_cat) = group.categories().first() {
                *current_category.write() = *first_cat;
            }
        })
        .child(label)
}
