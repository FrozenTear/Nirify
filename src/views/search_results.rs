//! Search results view
//!
//! Displays search results as content with clickable items

use iced::widget::{button, column, container, scrollable, text};
use iced::{Border, Color as IcedColor, Element, Length};

use crate::messages::Message;
use crate::search::SearchResult;

/// Creates the search results view
/// Note: Takes owned copies of data to avoid lifetime issues without memory leaks
pub fn view(results: &[SearchResult], query: &str) -> Element<'static, Message> {
    if query.trim().is_empty() || results.is_empty() {
        return container(
            text("No results found").size(16).color([0.75, 0.75, 0.75])
        )
        .padding(40)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into();
    }

    let result_count = results.len();
    // Use owned String directly - text() accepts Into<String>
    let header_text = format!(
        "Found {} result{} for \"{}\"",
        result_count,
        if result_count == 1 { "" } else { "s" },
        query
    );

    let mut content = column![
        text(header_text)
            .size(18)
            .color([0.9, 0.9, 0.9]),
        text("Click a result to navigate to that page")
            .size(13)
            .color([0.75, 0.75, 0.75]),
    ]
    .spacing(12)
    .padding(20);

    // Add search result items - clone data to owned values
    for (index, result) in results.iter().enumerate() {
        let page_title = result.page_title.clone();
        let keywords_text = format!("Keywords: {}", result.matched_keywords.join(", "));

        let result_item = button(
            column![
                text(page_title).size(18),
                text(keywords_text)
                    .size(13)
                    .color([0.75, 0.75, 0.75]),
            ]
            .spacing(6)
            .padding(16)
        )
        .on_press(Message::SearchResultSelected(index))
        .width(Length::Fill)
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();

            button::Style {
                background: Some(iced::Background::Color(
                    match status {
                        button::Status::Hovered => IcedColor::from_rgb(0.25, 0.30, 0.35),
                        button::Status::Pressed => IcedColor::from_rgb(0.20, 0.25, 0.30),
                        _ => IcedColor::from_rgb(0.18, 0.20, 0.22),
                    }
                )),
                text_color: palette.background.base.text,
                border: Border {
                    color: IcedColor::from_rgb(0.35, 0.35, 0.35),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: IcedColor::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                snap: false,
            }
        });

        content = content.push(result_item);
    }

    // Wrap in scrollable container that fills the content area
    scrollable(
        container(content)
            .padding(20)
            .width(Length::Fill)
    )
    .into()
}
