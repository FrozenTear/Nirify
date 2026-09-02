//! Sidebar navigation component — redesigned with 7 screens + gear + search

use iced::widget::{button, column, container, row, text, text_input, Column, Space};
use iced::{Alignment, Element, Length};

use crate::messages::{Message, Screen};
use crate::theme::{fonts, search_container_style, sidebar_item_style, sidebar_style};
use crate::views::navigation::search_input_id;

/// Creates the sidebar navigation UI
pub fn view<'a>(
    current_screen: Screen,
    search_query: &'a str,
    show_search_bar: bool,
) -> Element<'a, Message> {
    let mut items = Column::new().spacing(4).padding([16, 12]);

    // App title
    items = items.push(
        column![
            text("Nirify").size(20).font(fonts::UI_FONT_SEMIBOLD),
            text("Window Manager")
                .size(11)
                .font(fonts::UI_FONT)
                .color([0.5, 0.5, 0.55]),
        ]
        .spacing(2)
        .padding([0, 8]),
    );

    items = items.push(Space::new().height(12));

    // Search: inline field (default) or a button that opens the Ctrl+K modal
    if show_search_bar {
        items = items.push(
            container(
                row![
                    text("🔍").size(13),
                    text_input("Search settings...", search_query)
                        .id(search_input_id())
                        .on_input(Message::SearchQueryChanged)
                        .on_submit(Message::SearchNavActivate)
                        .padding([6, 8])
                        .size(13)
                        .width(Length::Fill),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding([4, 6])
            .width(Length::Fill)
            .style(search_container_style),
        );
    } else {
        items = items.push(
            button(
                row![text("🔍").size(13), text("Search").size(13)]
                    .spacing(6)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ToggleSearch)
            .width(Length::Fill)
            .padding([8, 12])
            .style(sidebar_item_style(false)),
        );
    }

    items = items.push(Space::new().height(12));

    // Main screen items
    for screen in Screen::sidebar_items() {
        let is_active = current_screen == *screen;

        let btn = button(text(screen.name()).size(14).font(if is_active {
            fonts::UI_FONT_MEDIUM
        } else {
            fonts::UI_FONT
        }))
        .on_press(Message::NavigateToScreen(*screen))
        .width(Length::Fill)
        .padding([10, 16])
        .style(sidebar_item_style(is_active));

        items = items.push(btn);
    }

    // Spacer to push gear to bottom
    items = items.push(Space::new().height(Length::Fill));

    // Separator line
    items = items.push(container(Space::new().height(1)).width(Length::Fill).style(
        |theme: &iced::Theme| {
            let bg = theme.palette().background;
            container::Style {
                background: Some(iced::Background::Color(crate::theme::lighten_pub(bg, 0.10))),
                ..Default::default()
            }
        },
    ));

    items = items.push(Space::new().height(4));

    // Gear/Settings button
    let gear_active = current_screen == Screen::Gear;
    items = items.push(
        button(text("Settings").size(14).font(if gear_active {
            fonts::UI_FONT_MEDIUM
        } else {
            fonts::UI_FONT
        }))
        .on_press(Message::NavigateToScreen(Screen::Gear))
        .width(Length::Fill)
        .padding([10, 16])
        .style(sidebar_item_style(gear_active)),
    );

    container(items)
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .style(sidebar_style)
        .into()
}
