//! Header component with centered title and primary navigation
//!
//! Editorial design: clean, centered layout with geometric precision

use freya::prelude::*;

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::*;

/// Create the header with centered title and primary navigation tabs
pub fn header(
    current_nav_group: State<NavGroup>,
    current_category: State<Category>,
) -> impl IntoElement {
    let active_group = *current_nav_group.read();

    rect()
        .content(Content::flex())
        .direction(Direction::Vertical)
        .width(Size::fill())
        .background(BG_DEEP)
        .child(
            // Main header area - centered content
            rect()
                .content(Content::flex())
                .direction(Direction::Vertical)
                .width(Size::fill())
                .height(Size::px(NAV_HEIGHT))
                .main_align(Alignment::Center)
                .cross_align(Alignment::Center)
                .child(
                    // Centered title
                    label()
                        .text("niri")
                        .color(TEXT_BRIGHT)
                        .font_size(FONT_SIZE_2XL)
                        .font_weight(FontWeight::LIGHT)
                        .max_lines(1),
                )
                .child(
                    // Navigation tabs - horizontally centered
                    rect()
                        .content(Content::flex())
                        .direction(Direction::Horizontal)
                        .spacing(SPACING_XS)
                        .margin((SPACING_MD, 0.0, 0.0, 0.0))
                        .child(nav_tab("Appearance", NavGroup::Appearance, active_group, current_nav_group, current_category))
                        .child(nav_tab("Input", NavGroup::Input, active_group, current_nav_group, current_category))
                        .child(nav_tab("Visuals", NavGroup::Visuals, active_group, current_nav_group, current_category))
                        .child(nav_tab("Behavior", NavGroup::Behavior, active_group, current_nav_group, current_category))
                        .child(nav_tab("Rules", NavGroup::Rules, active_group, current_nav_group, current_category))
                        .child(nav_tab("System", NavGroup::System, active_group, current_nav_group, current_category)),
                ),
        )
        .child(
            // Subtle accent line at bottom
            rect()
                .width(Size::fill())
                .height(Size::px(1.0))
                .background(BORDER_SUBTLE),
        )
}

/// Primary navigation tab with active state indicator
fn nav_tab(
    text: &'static str,
    group: NavGroup,
    active_group: NavGroup,
    mut current_nav_group: State<NavGroup>,
    mut current_category: State<Category>,
) -> impl IntoElement {
    let is_active = group == active_group;

    let (text_color, bg_color) = if is_active {
        (ACCENT_VIVID, SELECTED_BG)
    } else {
        (TEXT_DIM, (0x00, 0x00, 0x00, 0x00))
    };

    rect()
        .content(Content::flex())
        .padding((SPACING_SM, SPACING_LG, SPACING_SM, SPACING_LG))
        .corner_radius(RADIUS_MD)
        .background(bg_color)
        .on_pointer_down(move |_| {
            *current_nav_group.write() = group;
            if let Some(first_cat) = group.categories().first() {
                *current_category.write() = *first_cat;
            }
        })
        .child(
            label()
                .text(text)
                .color(text_color)
                .font_size(FONT_SIZE_SM)
                .font_weight(if is_active { FontWeight::SEMI_BOLD } else { FontWeight::NORMAL })
                .max_lines(1),
        )
}
