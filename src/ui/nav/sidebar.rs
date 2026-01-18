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
        .horizontal()
        .width(Size::fill())
        .padding((SPACING_MD, SPACING_2XL, SPACING_MD, SPACING_2XL))
        .background(MANTLE)
        .spacing(SPACING_SM)
        .child(category_buttons(nav_group, current_category))
}

/// Generate category buttons based on the current nav group
fn category_buttons(nav_group: NavGroup, mut current_category: State<Category>) -> impl IntoElement {
    // Build the buttons manually based on the nav group
    let categories = nav_group.categories();

    // For simplicity, just show the first few categories
    // A proper implementation would need to handle this dynamically
    rect()
        .horizontal()
        .spacing(SPACING_SM)
        .child(
            match categories.get(0) {
                Some(cat) => {
                    let cat = *cat;
                    Button::new()
                        .on_press(move |_| *current_category.write() = cat)
                        .child(cat.label())
                }
                None => Button::new().child("None"),
            }
        )
        .child(
            match categories.get(1) {
                Some(cat) => {
                    let cat = *cat;
                    Button::new()
                        .on_press(move |_| *current_category.write() = cat)
                        .child(cat.label())
                }
                None => Button::new().child(""),
            }
        )
        .child(
            match categories.get(2) {
                Some(cat) => {
                    let cat = *cat;
                    Button::new()
                        .on_press(move |_| *current_category.write() = cat)
                        .child(cat.label())
                }
                None => Button::new().child(""),
            }
        )
}
