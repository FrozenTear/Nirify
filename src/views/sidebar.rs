//! Sidebar navigation component

use iced::widget::{button, container, scrollable, text, Column};
use iced::{Element, Length};

use crate::messages::{Message, Page, PageCategory};

/// Creates the sidebar navigation UI
pub fn view(current_page: Page) -> Element<'static, Message> {
    let mut sidebar = Column::new()
        .spacing(8)
        .padding(12)
        .width(Length::Fixed(220.0));

    // Group pages by category
    for category in &[
        PageCategory::System,
        PageCategory::Visual,
        PageCategory::Input,
        PageCategory::Layout,
        PageCategory::Rules,
        PageCategory::Advanced,
    ] {
        // Category header
        sidebar = sidebar.push(
            text(category.name())
                .size(12)
                .color([0.75, 0.75, 0.75])
        );

        // Pages in this category
        for page in pages_in_category(*category) {
            let is_active = current_page == page;

            let btn = button(text(page.name()).size(14))
                .on_press(Message::NavigateToPage(page))
                .width(Length::Fill)
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                });

            sidebar = sidebar.push(btn);
        }

        // Spacer between categories
        sidebar = sidebar.push(container(text("")).height(Length::Fixed(8.0)));
    }

    scrollable(sidebar).into()
}

/// Returns all pages in a given category
fn pages_in_category(category: PageCategory) -> Vec<Page> {
    use Page::*;

    match category {
        PageCategory::System => vec![Overview, Outputs, Miscellaneous, Startup, Environment],
        PageCategory::Visual => vec![Appearance, Behavior, Animations, Cursor],
        PageCategory::Input => vec![
            Keyboard,
            Mouse,
            Touchpad,
            Trackpoint,
            Trackball,
            Tablet,
            Touch,
            Keybindings,
        ],
        PageCategory::Layout => vec![LayoutExtras, Gestures, Workspaces],
        PageCategory::Rules => vec![WindowRules, LayerRules],
        PageCategory::Advanced => vec![Debug, SwitchEvents, RecentWindows],
    }
}
