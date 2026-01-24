//! Modern horizontal navigation system
//!
//! Features:
//! - Primary category tabs (System, Visual, Input, Layout, Rules, Advanced)
//! - Secondary page tabs within each category
//! - Search bar with icon
//! - Smooth transitions and hover states

use iced::widget::{button, container, row, text, text_input, Id, Row};
use iced::{Alignment, Element, Length};

use crate::messages::{Message, Page, PageCategory};
use crate::theme::{fonts, nav_bar_style, nav_tab_style, search_container_style, subnav_bar_style, subnav_tab_style};

/// Stable ID for search input to maintain focus
pub fn search_input_id() -> Id {
    Id::new("search-input")
}

/// Creates the primary navigation bar with category tabs
pub fn primary_nav<'a>(
    current_page: Page,
    search_query: &'a str,
    show_search_bar: bool,
) -> Element<'a, Message> {
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

    let nav_content = if show_search_bar {
        // Search bar with stable ID
        let search = container(
            row![
                text("🔍").size(14),
                text_input("Search settings...", search_query)
                    .id(search_input_id())
                    .on_input(Message::SearchQueryChanged)
                    .padding([6, 10])
                    .width(Length::Fixed(300.0))
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        )
        .padding([6, 12])
        .style(search_container_style);

        row![tabs]
            .push(container(search).width(Length::Fill).align_x(iced::alignment::Horizontal::Right))
            .align_y(Alignment::Center)
    } else {
        // Search button (opens search modal on Ctrl+K or click)
        let search_btn = button(
            row![
                text("🔍").size(14),
                text("Search").size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
        )
        .on_press(Message::ToggleSearch)
        .padding([6, 12])
        .style(|theme: &iced::Theme, status| {
            let mut style = search_container_style(theme);
            if matches!(status, iced::widget::button::Status::Hovered) {
                style.background = Some(iced::Background::Color(iced::Color::from_rgba(0.25, 0.25, 0.25, 0.9)));
            }
            iced::widget::button::Style {
                background: style.background,
                border: style.border,
                text_color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                ..Default::default()
            }
        });

        row![tabs]
            .push(
                container(search_btn)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .padding([0, 12])
            )
            .align_y(Alignment::Center)
    };

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
        Page::Preferences,
        Page::ConfigEditor,
        Page::Backups,
    ];

    all_pages
        .into_iter()
        .filter(|page| page.category() == category)
        .collect()
}
