//! Sub-navigation component with centered category pills
//!
//! Secondary navigation that changes based on selected NavGroup

use freya::prelude::*;

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::*;

/// Create the sub-navigation bar with category pills
pub fn subnav(
    current_nav_group: State<NavGroup>,
    current_category: State<Category>,
) -> impl IntoElement {
    let nav_group = *current_nav_group.read();
    let active_category = *current_category.read();
    let categories = nav_group.categories();

    rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(SUBNAV_HEIGHT))
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .background(BG_DEEP)
        .child(
            // Pill container - centered
            rect()
                .content(Content::flex())
                .direction(Direction::Horizontal)
                .padding((SPACING_XS, SPACING_SM, SPACING_XS, SPACING_SM))
                .corner_radius(RADIUS_LG)
                .background(BG_SURFACE)
                .spacing(SPACING_2XS)
                .child(category_pills(categories, active_category, current_category)),
        )
}

/// Generate category pill buttons
fn category_pills(
    categories: &'static [Category],
    active_category: Category,
    current_category: State<Category>,
) -> impl IntoElement {
    // Build pills dynamically based on available categories
    let mut container = rect()
        .content(Content::flex())
        .direction(Direction::Horizontal)
        .spacing(SPACING_2XS);

    for &cat in categories {
        let is_active = cat == active_category;
        container = container.child(category_pill(cat, is_active, current_category));
    }

    container
}

/// Individual category pill button
fn category_pill(
    category: Category,
    is_active: bool,
    mut current_category: State<Category>,
) -> impl IntoElement {
    let text_color = if is_active { TEXT_BRIGHT } else { TEXT_DIM };
    let bg_color: (u8, u8, u8, u8) = if is_active {
        (ACCENT_VIVID.0, ACCENT_VIVID.1, ACCENT_VIVID.2, 255)
    } else {
        (0x00, 0x00, 0x00, 0x00)
    };

    rect()
        .content(Content::flex())
        .padding((SPACING_SM, SPACING_LG, SPACING_SM, SPACING_LG))
        .corner_radius(RADIUS_MD)
        .background(bg_color)
        .on_pointer_down(move |_| {
            *current_category.write() = category;
        })
        .child(
            label()
                .text(category.label())
                .color(text_color)
                .font_size(FONT_SIZE_SM)
                .font_weight(if is_active { FontWeight::MEDIUM } else { FontWeight::NORMAL })
                .max_lines(1),
        )
}
