//! Sidebar component with secondary navigation (category pills)

use freya::prelude::*;

use crate::ui::state::{Category, NavGroup};
use crate::ui::theme::*;

/// Create the sidebar with subcategory pills
pub fn sidebar(
    current_nav_group: State<NavGroup>,
    current_category: State<Category>,
) -> impl IntoElement {
    let nav_group = *current_nav_group.read();

    rect()
        .direction(Direction::Horizontal)
        .width(Size::fill())
        .height(Size::px(50.0))
        .padding((SPACING_MD, SPACING_2XL, SPACING_MD, SPACING_2XL))
        .background(MANTLE)
        .spacing(SPACING_SM)
        .child(category_buttons(nav_group, current_category))
}

/// Generate category buttons based on the current nav group
fn category_buttons(nav_group: NavGroup, mut current_category: State<Category>) -> impl IntoElement {
    let categories = nav_group.categories();

    rect()
        .direction(Direction::Horizontal)
        .spacing(SPACING_SM)
        .child(
            match categories.get(0) {
                Some(cat) => {
                    let cat = *cat;
                    Button::new()
                        .on_press(move |_| *current_category.write() = cat)
                        .child(label().text(cat.label()).max_lines(1))
                }
                None => Button::new().child(label().text("None").max_lines(1)),
            }
        )
        .child(
            match categories.get(1) {
                Some(cat) => {
                    let cat = *cat;
                    Button::new()
                        .on_press(move |_| *current_category.write() = cat)
                        .child(label().text(cat.label()).max_lines(1))
                }
                None => Button::new().child(label().text("").max_lines(1)),
            }
        )
        .child(
            match categories.get(2) {
                Some(cat) => {
                    let cat = *cat;
                    Button::new()
                        .on_press(move |_| *current_category.write() = cat)
                        .child(label().text(cat.label()).max_lines(1))
                }
                None => Button::new().child(label().text("").max_lines(1)),
            }
        )
}
