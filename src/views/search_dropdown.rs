//! Search dropdown overlay
//!
//! Floating dropdown that appears below the search bar

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::messages::Message;
use crate::search::SearchResult;
use crate::theme::{fonts, search_dropdown_item_style, search_dropdown_style};

/// Maximum results to show in dropdown (shared with the modal + keyboard-nav clamp)
const MAX_DROPDOWN_RESULTS: usize = crate::search::MAX_VISIBLE_RESULTS;

/// Creates the search dropdown if there are results to show
/// Returns None if query is empty
pub fn view(
    results: &[SearchResult],
    query: &str,
    selected: usize,
) -> Option<Element<'static, Message>> {
    if query.trim().is_empty() {
        return None;
    }

    let dropdown = if results.is_empty() {
        // No results message
        container(
            text("No matching settings found")
                .size(13)
                .color([0.6, 0.6, 0.6]),
        )
        .padding([12, 16])
        .width(Length::Fixed(320.0))
        .style(search_dropdown_style)
        .into()
    } else {
        build_results_list(results, selected)
    };

    Some(dropdown)
}

/// Builds the results list
fn build_results_list(results: &[SearchResult], selected: usize) -> Element<'static, Message> {
    let total_count = results.len();
    let mut items = column![].spacing(2).padding([8, 8]);

    for (index, result) in results.iter().take(MAX_DROPDOWN_RESULTS).enumerate() {
        let setting_name = result.setting_name.clone();
        let description = result.description.clone();
        let page_name = result.destination.location_label();
        let is_selected = index == selected;

        let item = button(
            row![
                column![
                    text(setting_name).size(14).font(fonts::UI_FONT_MEDIUM),
                    text(description).size(11).color([0.6, 0.6, 0.6]),
                ]
                .spacing(2)
                .width(Length::Fill),
                column![text(page_name).size(10).color([0.5, 0.5, 0.5]),].width(Length::Shrink),
            ]
            .align_y(Alignment::Center)
            .spacing(8)
            .padding([10, 12]),
        )
        .on_press(Message::SearchResultSelected(index))
        .width(Length::Fill)
        .style(move |theme: &iced::Theme, status| {
            if is_selected {
                let palette = theme.extended_palette();
                button::Style {
                    background: Some(iced::Background::Color(palette.primary.weak.color)),
                    text_color: palette.primary.weak.text,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            } else {
                search_dropdown_item_style()(theme, status)
            }
        });

        items = items.push(item);
    }

    // Add "and X more..." if there are additional results
    if total_count > MAX_DROPDOWN_RESULTS {
        let more_count = total_count - MAX_DROPDOWN_RESULTS;
        let more_text = container(
            text(format!("and {} more...", more_count))
                .size(12)
                .color([0.5, 0.5, 0.5]),
        )
        .padding([8, 16]);
        items = items.push(more_text);
    }

    // Wrap in scrollable if many results
    let content: Element<'static, Message> = if total_count > 4 {
        scrollable(items).height(Length::Fixed(320.0)).into()
    } else {
        items.into()
    };

    container(content)
        .width(Length::Fixed(380.0))
        .style(search_dropdown_style)
        .into()
}
