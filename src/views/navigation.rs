//! Modern horizontal navigation system
//!
//! Features:
//! - Primary category tabs (System, Visual, Input, Layout, Rules, Advanced)
//! - Secondary page tabs within each category
//! - Search bar with icon
//! - Smooth transitions and hover states

use iced::widget::{button, container, row, text, text_input, Row};
use iced::{Alignment, Element, Length};

use crate::messages::{Message, Page, PageCategory};
use crate::theme::{fonts, nav_bar_style, nav_tab_style, search_container_style, subnav_bar_style, subnav_tab_style};

/// Creates the primary navigation bar with category tabs
pub fn primary_nav<'a>(current_page: Page, search_query: &'a str) -> Element<'a, Message> {
    let current_category = current_page.category();

    // Category tabs
    let categories = vec![
        (PageCategory::System, "System"),
        (PageCategory::Visual, "Visual"),
        (PageCategory::Input, "Input"),
        (PageCategory::Layout, "Layout"),
        (PageCategory::Rules, "Rules"),
        (PageCategory::Advanced, "Advanced"),
    ];

    let mut tabs = Row::new().spacing(8).padding([12, 20]);

    for (category, label) in categories {
        let is_active = category == current_category;

        // Get first page in category to navigate to
        let target_page = get_first_page_in_category(category);

        let tab = button(
            text(label)
                .size(14)
                .font(if is_active { fonts::UI_FONT_SEMIBOLD } else { fonts::UI_FONT_MEDIUM })
        )
        .on_press(Message::NavigateToPage(target_page))
        .padding([8, 16])
        .style(nav_tab_style(is_active));

        tabs = tabs.push(tab);
    }

    // Search bar
    let search = container(
        row![
            text("🔍").size(14),
            text_input("Search settings...", search_query)
                .on_input(Message::SearchQueryChanged)
                .padding([6, 10])
                .width(Length::Fixed(300.0))
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    )
    .padding([6, 12])
    .style(search_container_style);

    let nav_content = row![tabs]
        .push(container(search).width(Length::Fill).align_x(iced::alignment::Horizontal::Right))
        .align_y(Alignment::Center);

    container(nav_content)
        .width(Length::Fill)
        .style(nav_bar_style)
        .into()
}

/// Creates the secondary navigation bar with page tabs
pub fn secondary_nav(current_page: Page) -> Element<'static, Message> {
    let current_category = current_page.category();
    let pages_in_category = get_pages_in_category(current_category);

    let mut tabs = Row::new().spacing(6).padding([10, 20]);

    for page in pages_in_category {
        let is_active = page == current_page;

        let tab = button(
            text(page.name())
                .size(13)
                .font(if is_active { fonts::UI_FONT_MEDIUM } else { fonts::UI_FONT })
        )
        .on_press(Message::NavigateToPage(page))
        .padding([6, 12])
        .style(subnav_tab_style(is_active));

        tabs = tabs.push(tab);
    }

    container(tabs)
        .width(Length::Fill)
        .style(subnav_bar_style)
        .into()
}

/// Helper: Get the first page in a category (for category tab navigation)
fn get_first_page_in_category(category: PageCategory) -> Page {
    match category {
        PageCategory::System => Page::Overview,
        PageCategory::Visual => Page::Appearance,
        PageCategory::Input => Page::Keyboard,
        PageCategory::Layout => Page::LayoutExtras,
        PageCategory::Rules => Page::WindowRules,
        PageCategory::Advanced => Page::Debug,
    }
}

/// Helper: Get all pages in a category
fn get_pages_in_category(category: PageCategory) -> Vec<Page> {
    let all_pages = vec![
        Page::Overview,
        Page::Appearance,
        Page::Behavior,
        Page::Keyboard,
        Page::Mouse,
        Page::Touchpad,
        Page::Trackpoint,
        Page::Trackball,
        Page::Tablet,
        Page::Touch,
        Page::Animations,
        Page::Cursor,
        Page::LayoutExtras,
        Page::Gestures,
        Page::Workspaces,
        Page::WindowRules,
        Page::LayerRules,
        Page::Keybindings,
        Page::Outputs,
        Page::Miscellaneous,
        Page::Startup,
        Page::Environment,
        Page::Debug,
        Page::SwitchEvents,
        Page::RecentWindows,
        Page::Tools,
    ];

    all_pages
        .into_iter()
        .filter(|page| page.category() == category)
        .collect()
}
